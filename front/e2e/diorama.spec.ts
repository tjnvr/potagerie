// The diorama under load: a dense garden (`/?example&dense`: 10 beds fully planted)
// opens, plays the whole season and keeps its frame times reasonable.
// Frame times are measured in the page with requestAnimationFrame, Chromium only
// (WebKit timings in CI are too noisy to assert on).
import { expect, test } from "@playwright/test";

const P95_LIMIT_MS = 50;

test("DIO-01 the dense garden opens, plays the season and keeps frame p95 under 50 ms", async ({ page, browserName }) => {
  test.skip(browserName !== "chromium", "frame times are only asserted in Chromium");
  const problems: string[] = [];
  page.on("pageerror", (e) => problems.push(`pageerror: ${e.message}`));
  await page.goto("/?example&dense&fps");
  await page.getByRole("button", { name: "Garden", exact: true }).click();
  await expect(page.getByRole("button", { name: "Play the season" })).toBeVisible();
  await page.evaluate(() => {
    const w = window as unknown as { __frames: number[] };
    w.__frames = [];
    let last = performance.now();
    const tick = (t: number) => {
      w.__frames.push(t - last);
      last = t;
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
  await page.getByRole("button", { name: "×1", exact: true }).click();
  await page.getByRole("button", { name: "Play the season" }).click();
  await expect(page.getByRole("button", { name: "Pause" })).toBeVisible();
  await page.waitForFunction(
    () => (window as unknown as { __potagerieMeasures?: unknown[] }).__potagerieMeasures?.length,
    undefined,
    { timeout: 90_000 },
  );
  const { frames, plants } = await page.evaluate(() => {
    const w = window as unknown as { __frames: number[]; __potagerieMeasures: { plants: number }[] };
    return { frames: w.__frames.slice(10), plants: w.__potagerieMeasures[0]!.plants };
  });
  const sorted = [...frames].sort((a, b) => a - b);
  const q = (p: number) => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))]!;
  console.log(
    `DIO-01 frames=${sorted.length} plants=${plants} median=${q(0.5).toFixed(1)} p95=${q(0.95).toFixed(1)} max=${sorted.at(-1)!.toFixed(1)}`,
  );
  expect(plants).toBeGreaterThan(100);
  expect(sorted.length).toBeGreaterThan(30);
  expect(q(0.95)).toBeLessThan(P95_LIMIT_MS);
  expect(problems).toEqual([]);
});
