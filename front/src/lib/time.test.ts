// @vitest-environment happy-dom
// « Today » while the app stays open past midnight, and the places that must
// read the LOCAL date, never the UTC one (`toISOString().slice(0, 10)`).
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";
import { msUntilMidnight } from "./dates";
import { today } from "./time";

afterEach(() => {
  vi.useRealTimers();
});

describe("today follows the local midnight", () => {
  it("msUntilMidnight counts to the next local midnight", () => {
    expect(msUntilMidnight(new Date(2027, 11, 31, 23, 59, 59, 0))).toBe(1000);
    expect(msUntilMidnight(new Date(2028, 1, 28, 0, 0, 0, 0))).toBe(86_400_000);
    expect(msUntilMidnight(new Date(2028, 1, 29, 12, 0, 0, 0))).toBe(12 * 3_600_000);
  });

  it("the store changes when midnight passes, over the new year and 29 February", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2027, 11, 31, 23, 59, 30));
    const seen: string[] = [];
    const stop = today.subscribe((d) => seen.push(d));
    expect(get(today)).toBe("2027-12-31");
    vi.advanceTimersByTime(31_000);
    expect(get(today)).toBe("2028-01-01");
    vi.setSystemTime(new Date(2028, 1, 28, 23, 59, 59));
    document.dispatchEvent(new Event("visibilitychange"));
    expect(get(today)).toBe("2028-02-28");
    vi.advanceTimersByTime(2_000);
    expect(get(today)).toBe("2028-02-29");
    stop();
    expect(seen).toContain("2028-01-01");
  });

  it("is refreshed when the window comes back (the machine slept past midnight)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2027, 5, 20, 22, 0));
    const stop = today.subscribe(() => {});
    expect(get(today)).toBe("2027-06-20");
    vi.setSystemTime(new Date(2027, 5, 22, 8, 0));
    document.dispatchEvent(new Event("visibilitychange"));
    expect(get(today)).toBe("2027-06-22");
    stop();
  });
});

function sources(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return name === "paraglide" ? [] : sources(path);
    return /\.(ts|svelte)$/.test(name) && !/\.test\.ts$/.test(name) ? [path] : [];
  });
}

describe("no UTC date taken for the local one", () => {
  it("nothing reads « today » as new Date().toISOString().slice(0, 10)", () => {
    const bad = sources("src").filter((f) => /new Date\(\)\.toISOString\(\)\.slice\(0, ?10\)/.test(readFileSync(f, "utf8")));
    expect(bad).toEqual([]);
  });
});
