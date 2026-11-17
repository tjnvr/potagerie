// Layout checks on a phone (360 and 390 px wide), in English and in French: touch
// targets of at least 40 px and no wrapped decoration. Found by a visual review of
// every screen; one assertion per fix.
import { expect, test, type Page } from "@playwright/test";

const PHONES = [
  { w: 360, h: 740, locale: "fr-FR", garden: "Potager" },
  { w: 390, h: 844, locale: "en-US", garden: "Garden" },
];

async function size(page: Page, selector: string) {
  await page.locator(selector).first().waitFor();
  const boxes = await page.locator(selector).evaluateAll((els) =>
    els.map((e) => {
      const r = e.getBoundingClientRect();
      return { w: Math.round(r.width), h: Math.round(r.height) };
    }),
  );
  expect(boxes.length).toBeGreaterThan(0);
  return boxes;
}

for (const v of PHONES) {
  test.describe(`${v.w}px ${v.locale}`, () => {
    test.use({ viewport: { width: v.w, height: v.h }, locale: v.locale, hasTouch: true });

    test(`LAY-01 the year buttons are at least 40 px (${v.w})`, async ({ page }) => {
      await page.goto("/?example");
      for (const b of await size(page, ".season")) {
        expect(b.h).toBeGreaterThanOrEqual(40);
        expect(b.w).toBeGreaterThanOrEqual(40);
      }
    });

    test(`LAY-02 the speed buttons are at least 40 px (${v.w})`, async ({ page }) => {
      await page.goto("/?example");
      await page.getByRole("button", { name: v.garden, exact: true }).click();
      for (const b of await size(page, ".speed button")) expect(b.h).toBeGreaterThanOrEqual(40);
    });

    test(`LAY-03 the bed panel buttons and links are at least 40 px (${v.w})`, async ({ page }) => {
      await page.goto("/?example");
      const grass = (await page.locator("rect.grass").boundingBox())!;
      const x = grass.x + grass.width * 0.2;
      const y = grass.y + grass.height * 0.2;
      await page.mouse.click(x, y);
      await page.mouse.click(x, y);
      await page.locator(".link").first().waitFor();
      for (const sel of [".link", "button.fallback", "button.rond"]) {
        for (const b of await size(page, sel)) expect(b.h).toBeGreaterThanOrEqual(40);
      }
    });

    test(`LAY-04 the plan scope does not start a line with a bullet (${v.w})`, async ({ page }) => {
      await page.goto("/?example");
      const scope = page.locator(".bandeau .scope");
      await expect(scope).toBeVisible();
      await expect(scope.locator(".dot")).toBeHidden();
    });
  });
}
