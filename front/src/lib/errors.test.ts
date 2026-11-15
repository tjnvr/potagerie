import { expect, test } from "vitest";
import { estFailureSaving, translateError } from "./errors";

test("an error code is translated, the technical detail is not shown", () => {
  const text = translateError({ code: "year_past", detail: "year 2026 is in the past" });
  expect(text).toBe("Past seasons can no longer be changed.");
});

test("an unknown code falls back to a generic message", () => {
  // A code from a newer Rust than these bindings.
  const text = translateError({ code: "new_code" as never, detail: "" });
  expect(text).toBe("An unexpected error occurred.");
});

test("only write failures can be retried", () => {
  expect(estFailureSaving({ code: "saving", detail: "" })).toBe(true);
  expect(estFailureSaving({ code: "conflict", detail: "" })).toBe(false);
});
