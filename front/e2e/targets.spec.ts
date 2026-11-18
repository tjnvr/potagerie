// Touch targets on a phone (360 and 390 px, English and French): every interactive
// control the user can tap is at least 40 px high and wide (docs/POT-00_test-matrix.md, LAY-xx).
import { expect, test, type Page } from "@playwright/test";

const PHONES = [
  { w: 360, h: 740, locale: "fr-FR", garden: "Potager" },
  { w: 390, h: 844, locale: "en-US", garden: "Garden" },
];

const SCAN_SELECTOR = "button, a[href], input, select, textarea, label, [role=button], [data-bar], [data-slot]";

/** Interactive targets under `root` smaller than 40 px, as readable strings. A checkbox or
 * radio is measured through its label; a label without a control inside is ignored. */
async function smallTargets(page: Page, root: string, skip = ""): Promise<string[]> {
  return page.locator(root).first().evaluate((el, [selector, skip]) => {
    const out: string[] = [];
    const seen = new Set<Element>();
    for (const e of Array.from(el.querySelectorAll(selector))) {
      let t: Element = e;
      if (e instanceof HTMLInputElement && ["checkbox", "radio"].includes(e.type)) t = e.closest("label") ?? e;
      if (e instanceof HTMLLabelElement && !e.querySelector("input[type=checkbox], input[type=radio]")) continue;
      if ((skip && e.matches(skip)) || e.closest("[hidden], [aria-hidden=true]") || seen.has(t)) continue;
      seen.add(t);
      const r = t.getBoundingClientRect();
      if (r.width === 0 || r.height === 0) continue;
      const cs = getComputedStyle(t);
      if (cs.visibility === "hidden" || cs.display === "none") continue;
      if (Math.round(r.height) < 40 || Math.round(r.width) < 40)
        out.push(`${t.tagName.toLowerCase()}.${(t.getAttribute("class") ?? "").trim().replace(/\s+/g, ".")} "${(t.textContent ?? "").trim().slice(0, 20)}" ${Math.round(r.width)}x${Math.round(r.height)}`);
    }
    return out;
  }, [SCAN_SELECTOR, skip]);
}

for (const v of PHONES) {
  test.describe(`${v.w}px ${v.locale}`, () => {
    test.use({ viewport: { width: v.w, height: v.h }, locale: v.locale, hasTouch: true });

    test(`LAY-05 timeline bars and free-slot chips are at least 40 px high (${v.w})`, async ({ page }) => {
      await page.goto("/?example");
      await page.getByRole("button", { name: v.garden, exact: true }).click();
      await page.locator("[data-bar]").first().waitFor();
      for (const sel of ["[data-bar]", "[data-slot]"]) {
        const boxes = await page.locator(sel).evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().height)));
        expect(boxes.length, sel).toBeGreaterThan(0);
        for (const h of boxes) expect(h, sel).toBeGreaterThanOrEqual(40);
      }
    });

    test(`LAY-06 the export dialog controls are at least 40 px (${v.w})`, async ({ page }) => {
      await page.goto("/?example");
      await page.getByRole("button", { name: v.garden, exact: true }).click();
      await page.getByRole("button", { name: /Export the timeline|Exporter la frise/ }).click();
      await expect(page.getByRole("dialog")).toBeVisible();
      expect(await smallTargets(page, "[role=dialog]")).toEqual([]);
    });

    test(`LAY-07 the planting form controls are at least 40 px (${v.w})`, async ({ page }) => {
      await page.goto("/?example");
      await page.getByRole("button", { name: v.garden, exact: true }).click();
      const slot = page.locator("[data-slot]").first();
      await slot.scrollIntoViewIfNeeded();
      await slot.click({ force: true });
      await expect(page.getByRole("dialog")).toBeVisible();
      await expect(page.locator("ol.steps")).toBeVisible();
      expect(await smallTargets(page, "[role=dialog]")).toEqual([]);
    });

    test(`LAY-08 the garden screen controls are at least 40 px (${v.w})`, async ({ page }) => {
      await page.goto("/?example");
      await page.getByRole("button", { name: v.garden, exact: true }).click();
      await page.locator("[data-bar]").first().waitFor();
      // Not covered: the beds of the 3D scene and the text of the notes are scene content, not controls.
      expect(await smallTargets(page, "body", "g.bed, g.note")).toEqual([]);
    });
  });
}
