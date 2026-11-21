import { expect, test } from "vitest";
import type { YearView } from "./bindings";
import { app, type Result } from "./state.svelte";

const view = { year: 2026, version: 2 } as YearView;

test("a write failure keeps the change retryable, then « Retry » runs it again", async () => {
  let calls = 0;
  const action = async (): Promise<Result<YearView>> => {
    calls++;
    return calls === 1
      ? { status: "error", error: { code: "saving", detail: "disk full" } }
      : { status: "ok", data: view };
  };
  expect(await app.modify(action)).toBe(false);
  expect(app.saving).toBe("error");
  expect(app.message).toBeNull();

  await app.retry();
  expect(calls).toBe(2);
  expect(app.saving).toBe("saved");
  expect(app.view).toEqual(view);
});

test("other refusals leave the indicator alone and show a message", async () => {
  const refusal = async (): Promise<Result<YearView>> => ({
    status: "error",
    error: { code: "year_past", detail: "" },
  });
  expect(await app.modify(refusal)).toBe(false);
  expect(app.saving).toBe("saved");
  expect(app.message?.tone).toBe("error");
  expect(app.message?.text).toBe("Past seasons can no longer be changed.");
});

test("a first load that fails is kept for the shell, not left as an endless « Loading… »", async () => {
  app.view = null;
  app.loadFailure = null;
  // No Tauri here: the command fails like a damaged data file would.
  await app.load();
  expect(app.view).toBeNull();
  expect(app.loadFailure).not.toBeNull();
});

test("« Move » from the garden opens the plan with the bed waiting to be selected, once (F-VIS-7)", () => {
  app.tab = "garden";
  const year = app.year;
  app.moveBed("bed-1");
  expect(app.tab).toBe("plan");
  expect(app.year).toBe(year);
  expect(app.takeBedToSelect()).toBe("bed-1");
  expect(app.takeBedToSelect()).toBeNull();
});
