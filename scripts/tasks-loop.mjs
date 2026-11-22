#!/usr/bin/env node
// Runs the tasks continuously on this Mac (scripts/tasks.sh loop).
// Every few minutes: resume answered tasks, restart crashed agents, start the next Ready tasks.
// When Claude says its usage limit is reached, sleep until the reset (5 h if unknown), then go on.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync, appendFileSync, unlinkSync } from "node:fs";
import { join } from "node:path";
import {
  ROOT, HOME, MAX, fetchOrigin, readTasks, lastRun, limitReset, readLimit, writeLimit, clearLimit, publish,
} from "./tasks-lib.mjs";

const EVERY = Number(process.env.TASKS_LOOP_MINUTES || 5) * 60_000;
const RETRIES = Number(process.env.TASKS_LOOP_RETRIES || 2);
const MARGIN = Number(process.env.TASKS_LIMIT_MARGIN_SECONDS ?? 120) * 1000;
const PID = join(HOME, "loop.pid");
const LOG = join(HOME, "loop.log");
const RETRY_FILE = join(HOME, "retries.json");

mkdirSync(HOME, { recursive: true });
const say = (msg) => {
  const line = `${new Date().toLocaleString("sv-SE").slice(0, 16)}  ${msg}`;
  console.log(line);
  appendFileSync(LOG, line + "\n");
};

// One loop at a time. An empty or unreadable pid file means no loop (pid 0 would mean "my own group").
const otherLoop = () => {
  try {
    const pid = Number(readFileSync(PID, "utf8").trim());
    if (!Number.isInteger(pid) || pid <= 0 || pid === process.pid) return 0;
    process.kill(pid, 0);
    return pid;
  } catch { return 0; }
};
const running = otherLoop();
if (running) {
  console.error(`tasks: a loop is already running (pid ${running}), see ${LOG}`);
  process.exit(1);
}
writeFileSync(PID, String(process.pid));
const bye = () => { try { if (readFileSync(PID, "utf8").trim() === String(process.pid)) unlinkSync(PID); } catch {} process.exit(0); };
process.on("SIGINT", bye);
process.on("SIGTERM", bye);
process.on("SIGHUP", () => {}); // a closed terminal must not stop the loop

const retries = () => { try { return JSON.parse(readFileSync(RETRY_FILE, "utf8")); } catch { return {}; } };
const saveRetries = (r) => writeFileSync(RETRY_FILE, JSON.stringify(r));

const tasksSh = (...args) => {
  try { return execFileSync(join(ROOT, "scripts/tasks.sh"), args, { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim(); }
  catch (e) { return `failed: ${(e.stderr || e.message || "").trim()}`; }
};

let lastPublish = "";
function tick() {
  const now = Date.now();
  // 0. Send to main what was answered in the board (left alone 2 min) and the task-system commits.
  try {
    const out = publish({ idleMs: 120_000 });
    if (out !== "nothing to publish" && out !== lastPublish) say(`publish · ${out}`);
    lastPublish = out;
  } catch (e) { say(`publish failed: ${(e.stderr || e.message || "").toString().trim().split("\n")[0]}`); }

  const limit = readLimit();
  if (limit && now < limit.until) return limit.until - now; // still paused
  if (limit) { say(`Claude limit over, resuming ${limit.tasks?.join(", ") || "work"}`); clearLimit(); }

  fetchOrigin(true);
  let state = readTasks();

  // 1. Did an agent stop because of the Claude limit?
  const hit = [];
  let until = 0;
  for (const t of state.tasks) {
    if (!t.stalled) continue;
    const run = lastRun(t.id);
    const reset = run.ended && run.code !== 0 ? limitReset(run.text, run.at) : null;
    if (reset && reset > now) { hit.push(t.id); until = Math.max(until, reset); }
  }
  if (hit.length) {
    // Running agents will hit the same wall: let them end on their own, they are resumed after the reset.
    writeLimit({ until: until + MARGIN, since: now, tasks: hit });
    say(`Claude limit reached (${hit.join(", ")}): sleeping until ${new Date(until + MARGIN).toLocaleString("sv-SE").slice(0, 16)}`);
    return until + MARGIN - now;
  }

  // 2. Restart what stopped: limit victims, crashed agents (a few times), answered questions.
  const r = retries();
  for (const t of state.tasks) {
    if (t.alive) { delete r[t.id]; continue; } // a run in progress starts a fresh count
    if (t.owner !== "agent") continue;
    if (t.status === "Done" || (t.status === "Blocked" && !t.answered)) { delete r[t.id]; continue; }
    const victim = limit?.tasks?.includes(t.id);
    if (t.stalled && !victim) {
      if ((r[t.id] || 0) >= RETRIES) {
        if (r[t.id] === RETRIES) { say(`${t.id} keeps stopping (${RETRIES} restarts): left alone, see its log`); r[t.id] = RETRIES + 1; }
        continue;
      }
      r[t.id] = (r[t.id] || 0) + 1;
    }
    if (t.stalled || t.answered) {
      if (state.running >= MAX) break;
      say(`${t.id}: ${victim ? "resumed after the limit" : t.answered ? "question answered, resumed" : `restarted (${r[t.id]}/${RETRIES})`} · ${tasksSh("run", t.id)}`);
      state.running++;
    }
  }
  saveRetries(r);

  // 3. Fill the free slots with the next Ready tasks.
  for (let i = state.running; i < MAX; i++) {
    const out = tasksSh("next");
    if (/nothing ready|already running/.test(out)) break;
    say(`next · ${out}`);
  }
  return EVERY;
}

say(`loop started in ${ROOT} (every ${EVERY / 60_000} min, ${MAX} agents at most)`);
const step = () => {
  let wait = EVERY;
  try { wait = tick(); } catch (e) { say(`error: ${e.message}`); }
  setTimeout(step, Math.max(Math.min(30_000, EVERY), Math.min(wait, EVERY)));
};
step();
