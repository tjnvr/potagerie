// No script error, no console error and no failed request while visiting every screen.
import { expect, test } from "@playwright/test";

test("SMOKE every screen opens without console errors or failed requests", async ({ page }) => {
  const problems: string[] = [];
  page.on("pageerror", (e) => problems.push(`pageerror: ${e.message}`));
  page.on("console", (m) => {
    if (m.type() === "error") problems.push(`console: ${m.text()}`);
  });
  page.on("requestfailed", (r) => problems.push(`request failed: ${r.url()}`));
  page.on("response", (r) => {
    if (r.status() >= 400) problems.push(`HTTP ${r.status()}: ${r.url()}`);
  });
  await page.goto("/?example");
  for (const tab of ["Plan", "Garden"]) {
    await page.getByRole("button", { name: tab, exact: true }).click();
    await page.waitForTimeout(400);
  }
  for (const label of [/Settings/, /Catalogue/, /Export the timeline/, /Annotate/]) {
    await page.getByRole("button", { name: label }).first().click();
    await page.waitForTimeout(400);
    await page.keyboard.press("Escape");
  }
  await page.getByRole("button", { name: "2027", exact: true }).click();
  await page.waitForTimeout(400);
  expect(problems).toEqual([]);
});
