// User tests of the planting form and of the catalogue, desktop and phone, in
// English and in French (docs/POT-00_test-matrix.md, SER-xx). One test per defect found.
import { expect, test, type Page } from "@playwright/test";

const LANGS = [
  { locale: "en-US", garden: "Garden", catalogue: "Catalogue", tomato: "Tomato" },
  { locale: "fr-FR", garden: "Potager", catalogue: "Catalogue", tomato: "Tomate" },
];

async function openCatalogue(page: Page, catalogue: string) {
  await page.goto("/?example");
  await page.getByRole("button", { name: catalogue, exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
}

for (const v of LANGS) {
  test.describe(v.locale, () => {
    test.use({ locale: v.locale });

    test(`SER-01 the catalogue sheet footer has no internal reference (${v.locale})`, async ({ page }) => {
      await openCatalogue(page, v.catalogue);
      const note = await page.locator(".fiche .lock").innerText();
      expect(note).not.toMatch(/V\d|\bS\d\d\b|schema|schéma/i);
    });

    test(`SER-02 the catalogue sheet follows the filter (${v.locale})`, async ({ page }) => {
      await openCatalogue(page, v.catalogue);
      await page.locator("button.carte").filter({ has: page.getByText(v.tomato, { exact: true }) }).click();
      await expect(page.locator(".fiche h2")).toHaveText(v.tomato);
      // A family that does not hold the tomato: its sheet is no longer shown.
      const families = page.locator(".filters button.filter");
      for (let i = 1; i < (await families.count()); i++) {
        await families.nth(i).click();
        const cards = await page.locator("button.carte strong").allInnerTexts();
        expect(cards.length).toBeGreaterThan(0);
        await expect(page.locator(".fiche h2")).toHaveText(cards.includes(v.tomato) ? v.tomato : cards[0]!);
      }
    });

    test(`SER-04 the planting form with a cleared start date shows no invented date (${v.locale})`, async ({ page }) => {
      await page.goto("/?example");
      await page.getByRole("button", { name: v.garden, exact: true }).click();
      const slot = page.locator("[data-slot]").first();
      await slot.scrollIntoViewIfNeeded();
      await slot.click({ force: true });
      const dialog = page.getByRole("dialog");
      const start = dialog.locator("input[type=date]").first();
      await expect(dialog.locator("ol.steps")).toBeVisible();
      await start.fill("");
      await start.dispatchEvent("change");
      await expect(dialog.locator("button.validate")).toBeDisabled();
      await expect(dialog.locator("ol.steps")).toHaveCount(0);
      await start.fill("2026-06-10");
      await start.dispatchEvent("change");
      await expect(dialog.locator("ol.steps")).toContainText("10");
      await expect(dialog.locator("button.validate")).toBeEnabled();
    });

    test(`SER-05/06 the end is the last free day, and its label says where it comes from (${v.locale})`, async ({ page }) => {
      await page.goto("/?example");
      await page.getByRole("button", { name: v.garden, exact: true }).click();
      const dialog = page.getByRole("dialog");
      const day = (iso: string) =>
        new Date(`${iso}T12:00:00`).toLocaleDateString(v.locale === "fr-FR" ? "fr-FR" : "en-GB", { day: "numeric", month: "short" });
      let found = false;
      const count = await page.locator("[data-slot]").count();
      for (let i = 0; i < count && !found; i++) {
        await page.goto("/?example");
        await page.getByRole("button", { name: v.garden, exact: true }).click();
        const slot = page.locator("[data-slot]").nth(i);
        await slot.scrollIntoViewIfNeeded();
        await slot.click({ force: true });
        const buttons = dialog.locator(".slot button");
        for (let j = 0; j < (await buttons.count()) && !found; j++) {
          await buttons.nth(j).click();
          await expect(dialog.locator("ol.steps")).toBeVisible();
          const manual = dialog.locator(".end .manual");
          // Only a suggestion cut short by the slot has an end of its own.
          await page.evaluate(() => Promise.all(document.getAnimations().map((a) => a.finished.catch(() => null))).then(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))));
          if ((await manual.count()) === 0) continue;
          found = true;
          // Not « by hand »: the user has touched nothing.
          await expect(manual).toHaveText(v.locale === "fr-FR" ? /créneau libre/ : /free slot/i);
          // The end field, the label, the last step and the slot all read the same last day.
          const last = day(await dialog.locator(".end input").inputValue());
          await expect(dialog.locator(".slot-text")).toContainText(`→ ${last}`);
          await expect(manual).toContainText(last);
          await expect(dialog.locator("ol.steps li").last().locator("strong")).toContainText(last);
          // Typed by the user: now it is by hand.
          await dialog.locator(".end input").fill("2027-04-20");
          await dialog.locator(".end input").dispatchEvent("change");
          await expect(manual).toHaveText(v.locale === "fr-FR" ? /à la main/ : /by hand/);
          await expect(manual).toContainText(day("2027-04-20"));
          await expect(dialog.locator(".end input")).toHaveValue("2027-04-20");
        }
      }
      expect(found).toBe(true);
    });

    test(`SER-08 the catalogue count follows the filter (${v.locale})`, async ({ page }) => {
      await openCatalogue(page, v.catalogue);
      const note = page.locator("p.note");
      const total = await page.locator("button.carte").count();
      await expect(note).toContainText(`${total} `);
      const families = page.locator(".filters button.filter");
      await families.nth(1).click();
      const shown = await page.locator("button.carte").count();
      expect(shown).toBeLessThan(total);
      await expect(note).toContainText(`${shown} `);
      await expect(note).not.toContainText(`${total} `);
      await expect(note).toContainText(v.locale === "fr-FR" ? "culture" : "crop");
    });
  });
}

test.describe("phone", () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  test("SER-03 a tap on a crop card brings its sheet into view (phone)", async ({ page }) => {
    await openCatalogue(page, "Catalogue");
    await page.locator("button.carte").nth(1).click();
    const box = async () => page.locator("aside.fiche").evaluate((e) => {
      const r = e.getBoundingClientRect();
      return { top: r.top, bottom: r.bottom, height: innerHeight };
    });
    // Under the 69 cards: without the scroll, the sheet stays off screen.
    await expect.poll(async () => (await box()).top).toBeLessThan(400);
    const r = await box();
    expect(r.top).toBeGreaterThanOrEqual(-1);
    expect(r.bottom).toBeLessThanOrEqual(r.height + 1);
  });
});
