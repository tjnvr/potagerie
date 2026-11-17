// Timeline checks found by user-testing the Garden tab. Each test name starts with its id.
import { expect, test } from "@playwright/test";

test("TML-01 two close notes of the same row never overprint their labels", async ({ page }) => {
  await page.goto("/?example");
  await page.getByRole("button", { name: "Garden", exact: true }).click();
  const mulching = page.locator("g.note", { hasText: "Mulching" });
  const stake = page.locator("g.note", { hasText: "Stake the tomatoes" });
  await mulching.scrollIntoViewIfNeeded();
  // Mulching is a period (no circle): its label box starts where the text starts.
  const m = await mulching.locator("svg").boundingBox();
  const s = await stake.locator("circle").boundingBox();
  expect(m && s).toBeTruthy();
  const gap = s!.x - m!.x;
  expect(gap).toBeGreaterThan(0);
  // The label of the first note is clipped by its own box, which ends before the next note.
  const room = Number(await mulching.locator("svg").getAttribute("width"));
  expect(room).toBeLessThan(gap);
});
