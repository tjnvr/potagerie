// Accessibility audit (axe, WCAG A and AA) of the main screens, on a desktop and on
// a phone in French. Serious and critical violations fail the test.
import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";

type Labels = { plan: string; garden: string; settings: RegExp; catalogue: RegExp; exportTl: RegExp };
const EN: Labels = { plan: "Plan", garden: "Garden", settings: /Settings/, catalogue: /Catalogue/, exportTl: /Export the timeline/ };
const FR: Labels = { plan: "Plan", garden: "Potager", settings: /Réglages/, catalogue: /Catalogue/, exportTl: /Exporter la frise/ };

const SCREENS: Record<string, (p: Page, l: Labels) => Promise<void>> = {
  plan: async (p, l) => { await p.getByRole("button", { name: l.plan, exact: true }).click(); },
  garden: async (p, l) => { await p.getByRole("button", { name: l.garden, exact: true }).click(); },
  settings: async (p, l) => { await p.getByRole("button", { name: l.settings }).first().click(); },
  catalogue: async (p, l) => { await p.getByRole("button", { name: l.catalogue }).first().click(); },
  export: async (p, l) => {
    await p.getByRole("button", { name: l.garden, exact: true }).click();
    await p.getByRole("button", { name: l.exportTl }).click();
  },
  "bed panel": async (p, l) => {
    await p.getByRole("button", { name: l.plan, exact: true }).click();
    await p.mouse.click(640, 340);
    await p.locator("input[type=number]").first().waitFor();
  },
  planting: async (p, l) => {
    await p.getByRole("button", { name: l.garden, exact: true }).click();
    const slot = p.locator("[data-slot]").first();
    await slot.scrollIntoViewIfNeeded();
    await slot.click({ force: true });
  },
};

const VIEWPORTS = [
  { name: "desktop", size: { width: 1440, height: 900 }, locale: "en-US", labels: EN, screens: Object.keys(SCREENS) },
  { name: "phone-fr", size: { width: 390, height: 844 }, locale: "fr-FR", labels: FR, screens: ["plan", "garden", "catalogue", "planting"] },
];

for (const v of VIEWPORTS) {
  for (const name of v.screens) {
    test(`A11Y ${name} (${v.name})`, async ({ browser }) => {
      const context = await browser.newContext({ viewport: v.size, locale: v.locale });
      const page = await context.newPage();
      await page.goto("/?example");
      await expect(page.getByRole("button", { name: "Plan", exact: true })).toBeVisible();
      await SCREENS[name](page, v.labels);
      await page.evaluate(() => Promise.all(document.getAnimations().map((a) => a.finished.catch(() => null))).then(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))));
      const { violations } = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa"]).analyze();
      for (const x of violations) console.log(`[${name} ${v.name}] ${x.impact} ${x.id}: ${x.nodes.length} nodes — ${x.help} — ${x.nodes[0]?.target.join(" ")}`);
      await context.close();
      expect(violations.filter((x) => x.impact === "critical" || x.impact === "serious")).toEqual([]);
    });
  }
}
