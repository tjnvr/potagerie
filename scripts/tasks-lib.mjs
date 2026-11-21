// Shared by the board (tasks-ui.mjs) and the loop (tasks-loop.mjs): reads docs/tasks/ and the agents' state.
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

export const ROOT = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
export const DIR = "docs/tasks";
export const BASE = process.env.TASKS_BASE || "origin/main";
export const HOME = process.env.TASKS_HOME || join(resolve(ROOT, ".."), "potagerie-tasks");
export const MAX = Number(process.env.TASKS_MAX || 2);
export const PORT = Number(process.env.TASKS_PORT || 4321);
export const ORDER = ["Draft", "Ready", "Running", "Blocked", "Done"];

export const git = (...args) => {
  try { return execFileSync("git", args, { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }); }
  catch { return null; }
};

let lastFetch = 0;
export function fetchOrigin(force = false) {
  if (!force && Date.now() - lastFetch < 60_000) return;
  git("fetch", "-q", "origin");
  lastFetch = Date.now();
}

// An agent is alive while its pid file names a live process (written by tasks.sh run).
export function alive(id) {
  const p = join(HOME, "pids", `${id}.pid`);
  if (!existsSync(p)) return false;
  try { process.kill(Number(readFileSync(p, "utf8").trim()), 0); return true; } catch { return false; }
}

export const field = (text, name) => (text.match(new RegExp(`^- ${name}: *(.*)$`, "m"))?.[1] ?? "").trim();
const list = (v) => v.split(",").map((s) => s.trim()).filter((s) => /^T\d+$/.test(s));

export function readTasks() {
  const isTask = (f) => /^T\d+_[^/]*\.md$/.test(f);
  const remote = (git("ls-tree", "--name-only", BASE, `${DIR}/`) || "")
    .split("\n").map((f) => f.split("/").pop()).filter(isTask);
  const local = existsSync(join(ROOT, DIR)) ? readdirSync(join(ROOT, DIR)).filter(isTask) : [];
  const names = [...new Set([...remote, ...local])];
  const branches = (git("for-each-ref", "--format=%(refname:short)", "refs/heads/task/") || "")
    .split("\n").filter(Boolean);
  const tasks = names.map((name) => {
    const file = `${DIR}/${name}`;
    const id = name.split("_")[0];
    const branch = branches.find((b) => b.startsWith(`task/${id}-`)) || null;
    const base = remote.includes(name) ? git("show", `${BASE}:${file}`) : null;
    const disk = local.includes(name) ? readFileSync(join(ROOT, file), "utf8") : null;
    const onBranch = branch ? git("show", `${branch}:${file}`) : null;
    const head = git("show", `HEAD:${file}`);
    const edited = disk !== null && disk !== head; // uncommitted change on this Mac
    // Branch first (Running, Blocked); then origin when an agent finished it (Done);
    // then your uncommitted edit; then origin (a local main behind origin is just stale).
    const text = onBranch || (base && field(base, "Status") === "Done" ? base : edited ? disk : base || disk) || "";
    const log = join(HOME, "logs", `${id}.log`);
    return {
      id, file, branch, text,
      pushed: base !== null,
      edited,
      num: Number(id.slice(1)),
      title: (text.match(/^# T\d+ · (.*)$/m)?.[1] ?? id).trim(),
      status: field(text, "Status") || "Draft",
      owner: field(text, "Owner") || "agent",
      deps: list(field(text, "Depends on")),
      refs: field(text, "Refs"),
      from: field(text, "From"),
      hasLog: existsSync(log),
      alive: alive(id),
      questions: questionsOf(text),
    };
  }).sort((a, b) => a.num - b.num);

  const byId = Object.fromEntries(tasks.map((t) => [t.id, t]));
  for (const t of tasks) {
    t.waitsFor = t.deps.filter((d) => byId[d]?.status !== "Done");
    t.unblocks = tasks.filter((o) => o.deps.includes(t.id)).map((o) => o.id);
    // An agent started (its branch exists) but no longer runs, and the task is neither blocked nor done.
    t.stalled = !t.alive && !!t.branch && (t.status === "Running" || t.status === "Ready");
    t.openQuestions = t.questions.filter((q) => q.functional && !q.answer).length;
    t.stuck = t.status === "Blocked" && !t.alive && t.openQuestions === 0 && /^#+\s*Stuck|^\s*-?\s*Stuck\b/m.test(t.text.split(/^## Log\s*$/m)[1] || "");
    t.answered = t.status === "Blocked" && !t.alive && t.openQuestions === 0 && !t.stuck;
    t.needsTim = t.openQuestions > 0 || (t.owner === "Tim" && t.status !== "Done");
    t.runnable = !t.alive && t.pushed && t.owner === "agent" && (
      (t.status === "Ready" && t.waitsFor.length === 0) || t.status === "Blocked" || t.status === "Running");
  }
  // Depth in the dependency graph, for the order view.
  const depth = (t, seen = new Set()) => {
    if (t.depth !== undefined) return t.depth;
    if (seen.has(t.id)) return 0;
    seen.add(t.id);
    t.depth = t.deps.length ? 1 + Math.max(...t.deps.map((d) => (byId[d] ? depth(byId[d], seen) : 0))) : 0;
    return t.depth;
  };
  tasks.forEach((t) => depth(t));
  // What `next` would start, in order.
  const queue = tasks.filter((t) => t.pushed && t.status === "Ready" && t.owner === "agent" && !t.branch && t.waitsFor.length === 0)
    .map((t) => t.id);
  const unpushed = tasks.filter((t) => !t.pushed).length;
  const running = tasks.filter((t) => t.alive).length;
  return { tasks, queue, running, unpushed, max: MAX, base: BASE, fetchedAt: lastFetch, loop: loopState(), order: ORDER };
}

// Writes "Answer: …" under question n of the Questions section (replaces a previous answer).
export function setAnswer(text, n, answer) {
  const lines = text.split("\n");
  const qs = lines.findIndex((l) => /^## Questions\s*$/.test(l));
  if (qs < 0) throw new Error("no Questions section");
  let qe = lines.findIndex((l, i) => i > qs && /^## /.test(l));
  if (qe < 0) qe = lines.length;
  const start = lines.findIndex((l, i) => i > qs && i < qe && new RegExp(`^\\s*${n}\\.\\s`).test(l));
  if (start < 0) throw new Error(`no question ${n}`);
  let end = lines.findIndex((l, i) => i > start && i < qe && /^\s*\d+\.\s/.test(l));
  if (end < 0) end = qe;
  const block = lines.slice(start, end).filter((l, i) => i === 0 || !/^\s*Answer:/.test(l));
  while (block.length > 1 && block[block.length - 1].trim() === "") block.pop();
  block.push(`   Answer: ${answer.replace(/\s*\n\s*/g, " ").trim()}`, "");
  return [...lines.slice(0, start), ...block, ...lines.slice(end)].join("\n");
}

export function answerTask(id, n, answer, { ready = false } = {}) {
  const t = readTasks().tasks.find((x) => x.id === id);
  if (!t) throw new Error(`unknown task ${id}`);
  if (t.branch) {
    // Blocked or running: the answer goes on the task branch, in its worktree, committed.
    const wt = join(HOME, id);
    if (!existsSync(wt)) throw new Error(`${id} has a branch but no worktree in ${HOME}`);
    const p = join(wt, t.file);
    writeFileSync(p, setAnswer(readFileSync(p, "utf8"), n, answer));
    execFileSync("git", ["-C", wt, "add", t.file]);
    execFileSync("git", ["-C", wt, "commit", "-q", "-m", `docs(tasks): answer ${id} question ${n}`]);
    return `answered on ${t.branch}`;
  }
  // Not started: the answer goes in your working tree, for you to commit.
  const p = join(ROOT, t.file);
  if (!existsSync(p)) throw new Error(`${t.file} is not on your disk`);
  let text = setAnswer(readFileSync(p, "utf8"), n, answer);
  if (ready) text = text.replace(/^- Status: Draft\s*$/m, "- Status: Ready");
  writeFileSync(p, text);
  return `answered in ${t.file}${ready ? ", set Ready" : ""}: Publish to send it to main`;
}


// Questions of a task: numbered items of its Questions section. Untagged ones count as functional.
export function questionsOf(text) {
  const sec = (text.split(/^## Questions\s*$/m)[1] || "").split(/^## /m)[0].replace(/<!--[\s\S]*?-->/g, "");
  const qs = []; let cur = null;
  for (const l of sec.split("\n")) {
    const m = l.match(/^\s*(\d+)\.\s+(.*)/);
    if (m) { cur = { n: +m[1], text: m[2], answer: "", functional: !/^\**\[technical\]/i.test(m[2]) }; qs.push(cur); continue; }
    if (!cur) continue;
    const a = l.match(/^\s*Answer:\s*(.*)/);
    if (a) cur.answer = a[1]; else if (l.trim()) cur.text += "\n" + l.trim();
  }
  return qs;
}

// The output of the last run of a task: what follows its last "=== … run Txx" line.
export function lastRun(id) {
  const p = join(HOME, "logs", `${id}.log`);
  if (!existsSync(p)) return { text: "", ended: false, code: null, at: 0 };
  const log = readFileSync(p, "utf8");
  const at = statSync(p).mtimeMs;
  const i = log.lastIndexOf(`run ${id}\n`);
  const text = i < 0 ? log : log.slice(i);
  const m = text.match(/^=== exit (\d+)/m);
  return { text, ended: !!m, code: m ? Number(m[1]) : null, at };
}

// Claude usage limits. Returns the time (ms) when Claude can be used again, or null.
// `now` is when the message was written: a reset already past means the limit is over.
const LIMIT = /usage limit|limit reached|hit your limit|limit will reset|out of (?:extra )?usage|credit balance is too low|rate[_ ]limit/i;
export function limitReset(text, now = Date.now()) {
  if (!LIMIT.test(text)) return null;
  const epoch = text.match(/\|(\d{10})\b/);
  if (epoch) return Number(epoch[1]) * 1000;
  const at = text.match(/resets?\s+(?:at\s+)?(\d{1,2})(?::(\d{2}))?\s*(am|pm)?/i);
  if (at) {
    let h = Number(at[1]) % 12 + (at[3] && at[3].toLowerCase() === "pm" ? 12 : 0);
    if (!at[3]) h = Number(at[1]);
    const d = new Date(now); d.setHours(h, Number(at[2] || 0), 0, 0);
    if (d.getTime() <= now) d.setDate(d.getDate() + 1);
    return d.getTime();
  }
  return now + Number(process.env.TASKS_LIMIT_HOURS || 5) * 3600_000;
}

const LIMIT_FILE = join(HOME, "limit.json");
export function readLimit() { try { return JSON.parse(readFileSync(LIMIT_FILE, "utf8")); } catch { return null; } }
export function writeLimit(v) { mkdirSync(HOME, { recursive: true }); writeFileSync(LIMIT_FILE, JSON.stringify(v, null, 2)); }
export function clearLimit() { try { unlinkSync(LIMIT_FILE); } catch {} }

export function loopState() {
  const p = join(HOME, "loop.pid");
  let running = false;
  try { process.kill(Number(readFileSync(p, "utf8")), 0); running = true; } catch {}
  return { running, limit: readLimit() };
}

// Commits the task files changed on disk and pushes them to main.
export function publish() {
  const changed = (git("status", "--porcelain", "--", DIR) || "").split("\n").filter(Boolean)
    .map((l) => l.slice(3)).filter((f) => /\.md$/.test(f));
  if (!changed.length) return "nothing to publish";
  const run = (...a) => execFileSync("git", a, { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  run("add", "--", ...changed);
  run("commit", "-q", "-m", `docs(tasks): update ${changed.map((f) => f.split("/").pop().split("_")[0]).join(", ")} from the board`, "--", ...changed);
  run("pull", "-q", "--rebase", "--autostash", "origin", "main");
  run("push", "-q", "origin", "HEAD:main");
  lastFetch = 0;
  return `published ${changed.length} task file(s)`;
}
