// Accessibility audit (axe) of the main screens, desktop and phone. Serious and
// critical violations fail the test; the lower levels are only reported.
import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const SCREENS: [string, (p: import("@playwright/test").Page) => Promise<void>][] = [
  ["plan", async (p) => { await p.getByRole("button", { name: "Plan", exact: true }).click(); }],
  ["garden", async (p) => { await p.getByRole("button", { name: "Garden", exact: true }).click(); }],
  ["settings", async (p) => { await p.getByRole("button", { name: "Settings" }).first().click(); }],
  ["catalogue", async (p) => { await p.getByRole("button", { name: /Catalogue/ }).first().click(); }],
];

for (const [name, go] of SCREENS) {
  test(`A11Y ${name}`, async ({ page }) => {
    await page.goto("/?example");
    await expect(page.getByRole("button", { name: "Plan", exact: true })).toBeVisible();
    await go(page);
    await page.waitForTimeout(600);
    const { violations } = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa"]).analyze();
    for (const v of violations) for (const n of v.nodes.slice(0, 20)) console.log(`   ${(n.any[0]?.data as { fgColor?: string; bgColor?: string; contrastRatio?: number } | undefined)?.fgColor} on ${(n.any[0]?.data as { bgColor?: string } | undefined)?.bgColor} ${(n.any[0]?.data as { contrastRatio?: number } | undefined)?.contrastRatio} ${n.target.join(' ')}`);
    for (const v of violations) console.log(`[${name}] ${v.impact} ${v.id}: ${v.nodes.length} nodes — ${v.help}`);
    expect(violations.filter((v) => v.impact === "critical" || v.impact === "serious")).toEqual([]);
  });
}
