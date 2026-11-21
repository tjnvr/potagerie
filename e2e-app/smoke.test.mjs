// Smoke tests of the real app (docs/POT-00_test-matrix.md, APP-xx): the built
// binary, its webview (WebView2 on Windows, WebKitGTK on Linux), the Rust
// commands and SQLite, driven through tauri-driver. Each test name starts with
// its id. The tests run in order and share the app; `make e2e-app` runs them.
//
// The app gets a fresh temporary data folder (POTAGERIE_DATA_DIR), never the
// real potagerie.db, and its "Open" and "Save as" windows answer one file of
// that folder (POTAGERIE_DIALOG_FILE): WebDriver cannot drive native windows.
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { Session, until } from "./webdriver.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, "..");
const WINDOWS = process.platform === "win32";
const APP = process.env.POTAGERIE_APP ?? join(ROOT, "target", "release", WINDOWS ? "potagerie.exe" : "potagerie");
const DRIVER = process.env.TAURI_DRIVER ?? "tauri-driver";
const PORT = 4444;
const BASE = `http://127.0.0.1:${PORT}`;
const RESULTS = join(HERE, "results");
const ENTER = "";

const DATA = mkdtempSync(join(tmpdir(), "potagerie-smoke-"));
const FILE = join(DATA, "garden.json");

const button = (label) => `//button[normalize-space()='${label}']`;
const inSettings = (xpath) => `//*[@role='dialog' and .//*[@id='settings-title']]${xpath}`;

let driver;
/** @type {Session} */
let app;

/** Starts the app and waits for its interface (the first screen or the tabs). */
async function launch() {
  app = await Session.start(BASE, APP);
  try {
    await app.find("//button", 60_000);
  } catch (e) {
    await keep("launch");
    throw e;
  }
}

async function counter() {
  return app.text(await app.find("//*[contains(@class,'counter')]"));
}

async function bars() {
  return (await app.findAll("//*[contains(concat(' ', @class, ' '), ' bar ')]")).length;
}

async function toPlan() {
  await app.click(button("Plan"));
  await app.find("//*[contains(@class,'counter')]");
}

async function toGarden() {
  await app.click(button("Garden"));
  await app.find(button("Play the season"));
}

async function openSettings() {
  await app.click("//button[@aria-label='Settings']");
  await app.find(inSettings(button("Export")));
}

async function closeSettings() {
  await app.click(inSettings("//button[@aria-label='Close']"));
}

/** Keeps a screenshot, the page source and the address of the page in e2e-app/results/. */
async function keep(id) {
  mkdirSync(RESULTS, { recursive: true });
  const base = join(RESULTS, id);
  await app?.screenshot().then((png) => writeFileSync(`${base}.png`, png)).catch(() => {});
  await app?.source().then((html) => writeFileSync(`${base}.html`, html)).catch(() => {});
  await app
    ?.cmd("GET", "/url")
    .then((url) => console.log(`${id}: page ${url}`))
    .catch(() => {});
  // Temporary: which ways of finding an element work in this webview.
  for (const [what, call] of [
    ["script", () => app.execute("return document.querySelectorAll('button').length")],
    ["css", () => app.cmd("POST", "/elements", { using: "css selector", value: "button" })],
    ["xpath", () => app.cmd("POST", "/elements", { using: "xpath", value: "//button" })],
    ["tag", () => app.cmd("POST", "/elements", { using: "tag name", value: "button" })],
  ]) {
    await call()
      .then((v) => console.log(`${id}: ${what} -> ${JSON.stringify(v).slice(0, 300)}`))
      .catch((e) => console.log(`${id}: ${what} failed: ${e.message}`));
  }
}

/** Runs a step; on failure keeps what the window shows. */
function step(name, fn) {
  it(name, async () => {
    try {
      await fn();
    } catch (e) {
      await keep(name.split(" ")[0]);
      throw e;
    }
  });
}

describe("the real app", { concurrency: false }, () => {
  before(async () => {
    assert.ok(existsSync(APP), `build the app first: ${APP} is missing (make e2e-app builds it)`);
    driver = spawn(DRIVER, process.env.NATIVE_DRIVER ? ["--native-driver", process.env.NATIVE_DRIVER] : [], {
      stdio: "inherit",
      env: {
        ...process.env,
        POTAGERIE_DATA_DIR: DATA,
        POTAGERIE_DIALOG_FILE: FILE,
        LANG: "en_US.UTF-8",
        // WebKitGTK without a GPU (Xvfb): software rendering, no DMA-BUF.
        ...(WINDOWS ? {} : { WEBKIT_DISABLE_COMPOSITING_MODE: "1", WEBKIT_DISABLE_DMABUF_RENDERER: "1" }),
      },
    });
    await until(() => fetch(`${BASE}/status`).then((r) => r.ok), "tauri-driver", 30_000);
    await launch();
  });

  after(async () => {
    await app?.quit().catch(() => {});
    driver?.kill();
  });

  step("APP-01 first launch: no plot, then the plot is created, in the test data folder", async () => {
    await app.waitText("No plot yet");
    await app.click(button("Create my plot"));
    await app.click(button("Create the plot"));
    await until(async () => (await counter()).includes("0 / 10"), "an empty plot");
    assert.ok(existsSync(join(DATA, "potagerie.db")), `no potagerie.db in ${DATA}`);
  });

  step("APP-02 plan: a click on free grass adds a bed", async () => {
    const grass = await app.find("//*[local-name()='rect' and contains(@class,'grass')]");
    // The first click may only clear the selection (UC-02): click again if needed.
    for (let i = 0; i < 2 && !(await counter()).includes("1 / 10"); i++) {
      await app.clickAt(grass);
      await until(async () => (await counter()).includes("1 / 10"), "one bed", 5_000).catch(() => {});
    }
    assert.match(await counter(), /1 \/ 10/);
  });

  step("APP-03 timeline: a free slot opens the planting form, planting adds a bar", async () => {
    await toGarden();
    assert.equal(await bars(), 0);
    await app.find("//*[@data-slot]");
    const focused = await app.execute(`
      const slot = document.querySelector("[data-slot]");
      slot.scrollIntoView({ block: "center", inline: "center" });
      slot.focus();
      return document.activeElement === slot;`);
    assert.ok(focused, "the free slot cannot take the focus");
    await app.key(ENTER);
    await app.click("//button[starts-with(normalize-space(),'Plant ·')]");
    await until(async () => (await bars()) === 1, "one bar");
  });

  step("APP-04 settings: the garden is exported to a JSON file", async () => {
    await openSettings();
    await app.click(inSettings(button("Export")));
    await app.waitText("Garden exported.");
    const file = JSON.parse(readFileSync(FILE, "utf8"));
    assert.equal(typeof file, "object");
    await closeSettings();
  });

  step("APP-05 restart: the plot, the bed and the planting are kept", async () => {
    await app.quit();
    app = undefined;
    await launch();
    await toPlan();
    assert.match(await counter(), /1 \/ 10/);
    await toGarden();
    await until(async () => (await bars()) === 1, "the planting after the restart");
  });

  step("APP-06 settings: clearing the plan, then importing the file brings the garden back", async () => {
    await openSettings();
    await app.click(inSettings(button("Clear")));
    await app.click(button("Clear the plan"));
    await app.waitText("Plan cleared.");
    await app.click(inSettings(button("Import")));
    // The file is older than the cleared plan: "Replace anyway".
    await app.click(inSettings("//button[starts-with(normalize-space(),'Replace')]"));
    await app.waitText("Garden imported.");
    await closeSettings();
    await toPlan();
    await until(async () => (await counter()).includes("1 / 10"), "the imported bed");
    await toGarden();
    await until(async () => (await bars()) === 1, "the imported planting");
  });
});
