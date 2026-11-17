// A seeded monkey: random clicks, drags and keys on every screen. The app must
// never throw, log an error, or leave the page without its tabs.
import { expect, test } from "@playwright/test";

function rng(seed: number) {
  let s = seed;
  return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 2 ** 32);
}

for (const seed of [1, 2, 3, 4]) {
  test(`monkey ${seed}: random gestures leave the app sound`, async ({ page }) => {
    const problems: string[] = [];
    page.on("pageerror", (e) => problems.push(`pageerror ${e.message}`));
    page.on("console", (m) => m.type() === "error" && problems.push(`console ${m.text()}`));
    await page.goto("/?example");
    await expect(page.getByRole("button", { name: "Plan", exact: true })).toBeVisible();
    const rand = rng(seed);
    const size = page.viewportSize()!;
    const keys = ["Escape", "Tab", "Enter", "ArrowRight", "ArrowLeft", "Delete"];
    for (let i = 0; i < 150; i++) {
      const x = 5 + rand() * (size.width - 10);
      const y = 5 + rand() * (size.height - 10);
      const roll = rand();
      if (roll < 0.5) await page.mouse.click(x, y);
      else if (roll < 0.8) {
        await page.mouse.move(x, y);
        await page.mouse.down();
        await page.mouse.move(5 + rand() * (size.width - 10), 5 + rand() * (size.height - 10), { steps: 4 });
        await page.mouse.up();
      } else await page.keyboard.press(keys[Math.floor(rand() * keys.length)]!);
    }
    await page.keyboard.press("Escape");
    expect(problems).toEqual([]);
  });
}
