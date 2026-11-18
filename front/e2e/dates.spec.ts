// Dates and time zones (DAT-xx): « today » is the LOCAL date of the device, whatever
// its zone, and it moves on at the local midnight while the app stays open.
import { expect, test, type Page } from "@playwright/test";

// An instant whose UTC date is not the local date, for each zone.
const CASES = [
  { zone: "UTC", instant: "2027-06-20T10:00:00Z", day: 20 },
  { zone: "Pacific/Auckland", instant: "2027-06-20T20:00:00Z", day: 21 },
  { zone: "America/Los_Angeles", instant: "2027-06-21T03:00:00Z", day: 20 },
  { zone: "Pacific/Kiritimati", instant: "2027-06-20T12:00:00Z", day: 21 },
] as const;

const heading = (page: Page) => page.locator("h1#diorama-date");

for (const { zone, instant, day } of CASES) {
  test.describe(`in ${zone}`, () => {
    test.use({ timezoneId: zone });

    test(`DAT-01 the garden opens on the local date (${zone})`, async ({ page }) => {
      await page.clock.install({ time: new Date(instant) });
      await page.goto("/?example");
      await page.getByRole("button", { name: "Garden", exact: true }).click();
      await expect(heading(page)).toContainText(new RegExp(`\\b${day}\\b`));
      await expect(heading(page)).toContainText("2027");
    });

    test(`DAT-02 the catalogue is dated with the local date (${zone})`, async ({ page }) => {
      await page.clock.install({ time: new Date(instant) });
      await page.goto("/?example");
      await page.getByRole("button", { name: /Catalogue/ }).first().click();
      await expect(page.getByRole("dialog")).toContainText(new RegExp(`\\b${day}\\b`));
    });
  });
}

test.describe("the new year comes while the app is open", () => {
  test.use({ timezoneId: "Pacific/Auckland" });

  test("DAT-03 the seasons on offer move on at the local midnight", async ({ page }) => {
    // 2027-12-31 23:59:50 in Auckland (UTC+13 in summer).
    await page.clock.install({ time: new Date("2027-12-31T10:59:50Z") });
    await page.goto("/?example");
    const seasons = page.getByRole("group", { name: /season/i });
    await expect(seasons.getByRole("button", { name: "2026", exact: true })).toBeVisible();
    await expect(seasons.getByRole("button", { name: "2030", exact: true })).toHaveCount(0);
    await page.clock.runFor(20_000);
    await expect(seasons.getByRole("button", { name: "2030", exact: true })).toBeVisible();
    await expect(seasons.getByRole("button", { name: "2026", exact: true })).toHaveCount(0);
  });
});
