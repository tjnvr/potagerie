// A minimal Chrome DevTools Protocol client, for the screenshots and the frame
// rate measurement. No dependency: Node 22+ has a global WebSocket and fetch.
// Starts its own headless Chrome with a throw-away profile; it never touches
// the desktop or the user's browser.
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CHEMINS = [
  process.env.CHROME,
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/usr/bin/google-chrome",
  "/usr/bin/chromium",
  "C:/Program Files/Google/Chrome/Application/chrome.exe",
].filter(Boolean);

export async function lancerChrome(port = 9333) {
  const profil = mkdtempSync(join(tmpdir(), "potagerie-chrome-"));
  let processus = null;
  for (const chemin of CHEMINS) {
    try {
      processus = spawn(
        chemin,
        [
          "--headless=new",
          `--remote-debugging-port=${port}`,
          `--user-data-dir=${profil}`,
          "--no-first-run",
          "--disable-extensions",
          "--hide-scrollbars",
          ...(process.env.CHROME_FLAGS ? process.env.CHROME_FLAGS.split(" ") : []),
          "about:blank",
        ],
        { stdio: "ignore" },
      );
      await new Promise((ok, ko) => {
        processus.once("spawn", ok);
        processus.once("error", ko);
      });
      break;
    } catch {
      processus = null;
    }
  }
  if (!processus) throw new Error("Chrome not found: set CHROME to its path");

  let cible = null;
  for (let i = 0; i < 50 && !cible; i++) {
    try {
      const reponse = await fetch(`http://127.0.0.1:${port}/json/list`);
      cible = (await reponse.json()).find((c) => c.type === "page");
    } catch {
      await new Promise((r) => setTimeout(r, 100));
    }
  }
  if (!cible) throw new Error("Chrome did not answer on the debugging port");

  const socket = new WebSocket(cible.webSocketDebuggerUrl);
  await new Promise((ok, ko) => {
    socket.onopen = ok;
    socket.onerror = ko;
  });
  let n = 0;
  const attente = new Map();
  socket.onmessage = (e) => {
    const m = JSON.parse(e.data);
    if (m.id && attente.has(m.id)) {
      const { ok, ko } = attente.get(m.id);
      attente.delete(m.id);
      if (m.error) ko(new Error(JSON.stringify(m.error)));
      else ok(m.result);
    }
  };
  const envoyer = (methode, params = {}) =>
    new Promise((ok, ko) => {
      const id = ++n;
      attente.set(id, { ok, ko });
      socket.send(JSON.stringify({ id, method: methode, params }));
    });

  return {
    envoyer,
    /** Evaluates an expression in the page and returns its value. */
    async evaluer(expression) {
      const r = await envoyer("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
      if (r.exceptionDetails) throw new Error(JSON.stringify(r.exceptionDetails));
      return r.result.value;
    },
    async fermer() {
      socket.close();
      processus.kill();
      await new Promise((r) => setTimeout(r, 300));
      rmSync(profil, { recursive: true, force: true });
    },
  };
}

export const pause = (ms) => new Promise((r) => setTimeout(r, ms));
