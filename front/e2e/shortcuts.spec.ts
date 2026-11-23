// Keyboard shortcuts of the plan editor (T25, S01 F-PLN-6, docs/POT-00_test-matrix.md
// UC-38 to UC-42). Each one stands for a button or a gesture. The platform is set
// in the page (`navigator.platform`), so both the macOS and the other modifier are
// tested whatever machine runs the suite.
import { expect, test, type Page } from "@playwright/test";

const PLATFORMS = [
  { name: "macOS", platform: "MacIntel", undo: "Meta+z", redo: "Meta+Shift+z", foreign: "Control+z" },
  { name: "Windows", platform: "Win32", undo: "Control+z", redo: "Control+y", foreign: "Meta+z" },
] as const;

async function openPlan(page: Page, platform = "Win32") {
  await page.addInitScript((p) => {
    Object.defineProperty(Navigator.prototype, "platform", { get: () => p });
  }, platform);
  await page.goto("/?example");
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await expect(page.locator(".counter")).toContainText("8 / 10");
  await page.locator("svg.plan").scrollIntoViewIfNeeded();
}

/** The group of the bed named `name` (its accessible name starts with it). */
const bed = (page: Page, name: string) => page.getByRole("button", { name: new RegExp(`^${name}, `) });

/** The drawn rectangle of a bed, in centimetres. */
async function geo(page: Page, name: string) {
  const rect = bed(page, name).locator("rect").first();
  const n = async (a: string) => Number(await rect.getAttribute(a));
  return { x: await n("x"), y: await n("y"), width: await n("width"), height: await n("height") };
}

/** Where the point (x, y) of the plan, in centimetres, is on the screen. */
async function toScreen(page: Page, p: { x: number; y: number }) {
  return page.locator("svg.plan").evaluate((svg: SVGSVGElement, p) => {
    const m = svg.getScreenCTM()!;
    return { x: m.a * p.x + m.c * p.y + m.e, y: m.b * p.x + m.d * p.y + m.f };
  }, p);
}

/** With a bed selected, a click on the grass clears the selection (with none, it
 *  would add a bed) and leaves the focus on the page. */
async function clickGrass(page: Page) {
  await expect(page.locator('g.bed[aria-pressed="true"]')).toHaveCount(1);
  const p = await toScreen(page, { x: 1000, y: 600 });
  await page.mouse.click(p.x, p.y);
  await expect(page.getByRole("heading", { name: "Plan summary" })).toBeVisible();
}

async function select(page: Page, name: string) {
  await bed(page, name).click();
  await expect(bed(page, name)).toHaveAttribute("aria-pressed", "true");
}

const undoButton = (page: Page) => page.getByRole("button", { name: "Undo the last action" });

test("UC-38 plan: Escape during a drag or a resize puts the bed back, otherwise deselects", async ({ page }) => {
  await openPlan(page);
  const before = await geo(page, "Leek square");

  // A move, cancelled before the release: the bed is back, the release sends nothing.
  const from = await toScreen(page, { x: before.x + 50, y: before.y + 50 });
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(from.x + 120, from.y + 60, { steps: 10 });
  await expect.poll(() => geo(page, "Leek square")).not.toEqual(before);
  await page.keyboard.press("Escape");
  await expect.poll(() => geo(page, "Leek square")).toEqual(before);
  await expect(page.locator("rect.origin")).toHaveCount(0);
  await page.mouse.move(from.x + 160, from.y + 80, { steps: 4 });
  await page.mouse.up();
  await expect.poll(() => geo(page, "Leek square")).toEqual(before);
  await expect(undoButton(page)).toBeDisabled();
  // The bed stays selected: the gesture is dropped, not the selection.
  await expect(bed(page, "Leek square")).toHaveAttribute("aria-pressed", "true");

  // A resize from the south-east handle, cancelled the same way.
  const handle = await toScreen(page, { x: before.x + before.width, y: before.y + before.height });
  await page.mouse.move(handle.x, handle.y);
  await page.mouse.down();
  await page.mouse.move(handle.x + 80, handle.y + 30, { steps: 10 });
  await expect.poll(() => geo(page, "Leek square")).not.toEqual(before);
  await page.keyboard.press("Escape");
  await expect.poll(() => geo(page, "Leek square")).toEqual(before);
  await page.mouse.up();
  await expect.poll(() => geo(page, "Leek square")).toEqual(before);
  await expect(undoButton(page)).toBeDisabled();

  // With no gesture, Escape deselects.
  await page.keyboard.press("Escape");
  await expect(bed(page, "Leek square")).toHaveAttribute("aria-pressed", "false");
  await expect(page.getByRole("heading", { name: "Plan summary" })).toBeVisible();
});

test("UC-39 plan: Delete and Backspace delete the selected bed after the same confirmation", async ({ page }) => {
  await openPlan(page);
  const counter = page.locator(".counter");
  await select(page, "Leek square");
  await page.keyboard.press("Delete");
  const confirm = page.getByRole("button", { name: "Delete the bed" });
  await expect(confirm).toBeVisible();
  // Escape closes the confirmation only: nothing deleted, the bed still selected.
  await page.keyboard.press("Escape");
  await expect(confirm).toHaveCount(0);
  await expect(counter).toContainText("8 / 10");
  await expect(bed(page, "Leek square")).toHaveAttribute("aria-pressed", "true");

  // In the name field, Backspace erases a letter and nothing else.
  const name = page.getByLabel("Name", { exact: true });
  await name.focus();
  await page.keyboard.press("End");
  await page.keyboard.press("Backspace");
  await expect(name).toHaveValue("Leek squar");
  await expect(confirm).toHaveCount(0);
  await name.fill("Leek square");
  await name.blur();

  await page.keyboard.press("Backspace");
  await confirm.click();
  await expect(counter).toContainText("7 / 10");
  await expect(bed(page, "Leek square")).toHaveCount(0);

  // With nothing selected, the key does nothing.
  await page.keyboard.press("Delete");
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("UC-40 plan: the arrows move the selected bed by a cell, Shift by a metre, with the refusals of a drag", async ({ page }) => {
  await openPlan(page);
  await select(page, "Leek square");
  const start = await geo(page, "Leek square");
  await page.keyboard.press("ArrowDown");
  await expect.poll(() => geo(page, "Leek square")).toEqual({ ...start, y: start.y + 50 });
  await page.keyboard.press("Shift+ArrowDown");
  await expect.poll(() => geo(page, "Leek square")).toEqual({ ...start, y: start.y + 150 });
  // One more cell would leave the plot: refused, pale red, the bed stays.
  await page.keyboard.press("ArrowDown");
  await expect(bed(page, "Leek square")).toHaveClass(/refuse/);
  await expect(page.getByRole("alert")).toBeVisible();
  await expect.poll(() => geo(page, "Leek square")).toEqual({ ...start, y: start.y + 150 });

  // Each move is one step of Undo.
  await undoButton(page).click();
  await expect.poll(() => geo(page, "Leek square")).toEqual({ ...start, y: start.y + 50 });
  await undoButton(page).click();
  await expect.poll(() => geo(page, "Leek square")).toEqual(start);
  await expect(undoButton(page)).toBeDisabled();

  // Onto another bed: refused, the bed stays.
  await select(page, "The tomatoes");
  const tomatoes = await geo(page, "The tomatoes");
  await page.keyboard.press("Shift+ArrowUp");
  await expect(bed(page, "The tomatoes")).toHaveClass(/refuse/);
  await expect.poll(() => geo(page, "The tomatoes")).toEqual(tomatoes);
  await expect(undoButton(page)).toBeDisabled();

  // Closer than the aisle: the bubble of a drop, « Go back » puts it back.
  await select(page, "Leek square");
  await page.keyboard.press("ArrowRight");
  await expect(page.getByRole("button", { name: "Go back" })).toBeVisible();
  await page.getByRole("button", { name: "Go back" }).click();
  await expect.poll(() => geo(page, "Leek square")).toEqual(start);
  // Escape on the bubble is « Go back » too, and only that: the bed stays selected.
  await page.keyboard.press("ArrowRight");
  await expect(page.getByRole("button", { name: "Go back" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("button", { name: "Go back" })).toHaveCount(0);
  await expect.poll(() => geo(page, "Leek square")).toEqual(start);
  await expect(bed(page, "Leek square")).toHaveAttribute("aria-pressed", "true");

  // Left of the plot: refused without a call.
  await select(page, "Back of the garden");
  const back = await geo(page, "Back of the garden");
  await page.keyboard.press("Shift+ArrowLeft");
  await expect(bed(page, "Back of the garden")).toHaveClass(/refuse/);
  await expect.poll(() => geo(page, "Back of the garden")).toEqual(back);

  // Nothing selected: the arrows do nothing to the beds.
  await clickGrass(page);
  await page.keyboard.press("ArrowDown");
  await expect.poll(() => geo(page, "Back of the garden")).toEqual(back);
});

test("UC-40 plan: the arrows are ignored in a field and in a past season", async ({ page }) => {
  await openPlan(page);
  await select(page, "Leek square");
  const start = await geo(page, "Leek square");
  await page.getByLabel("Name", { exact: true }).focus();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Delete");
  await expect.poll(() => geo(page, "Leek square")).toEqual(start);
  await expect(page.getByRole("dialog")).toHaveCount(0);

  await page.getByRole("button", { name: "2025", exact: true }).click();
  await expect(page.getByText(/read only/i).first()).toBeVisible();
  const first = page.locator("g.bed").first();
  await first.click();
  await expect(first).toHaveAttribute("aria-pressed", "true");
  const x = await first.locator("rect").first().getAttribute("x");
  const y = await first.locator("rect").first().getAttribute("y");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("Delete");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(first.locator("rect").first()).toHaveAttribute("x", x!);
  await expect(first.locator("rect").first()).toHaveAttribute("y", y!);
});

test("UC-41 plan: Tab and Shift+Tab select the next and the previous bed, then leave the plan", async ({ page }) => {
  await openPlan(page);
  // Nothing selected, the focus on the page (not on the Plan tab just clicked).
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  const beds = page.locator("g.bed");
  const count = await beds.count();
  await page.keyboard.press("Tab");
  await expect(beds.nth(0)).toHaveAttribute("aria-pressed", "true");
  await expect(beds.nth(0)).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(beds.nth(1)).toHaveAttribute("aria-pressed", "true");
  await expect(beds.nth(0)).toHaveAttribute("aria-pressed", "false");
  await expect(beds.nth(1)).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(beds.nth(0)).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("heading", { name: (await beds.nth(0).locator("text").textContent())! })).toBeVisible();

  // Before the first one, Shift+Tab leaves the plan; the selection stays.
  await page.keyboard.press("Shift+Tab");
  await expect(beds.nth(0)).not.toBeFocused();
  await expect(beds.nth(0)).toHaveAttribute("aria-pressed", "true");

  // From the last one, Tab leaves the plan too: the focus is never trapped.
  await clickGrass(page);
  await page.keyboard.press("Shift+Tab");
  await expect(beds.nth(count - 1)).toHaveAttribute("aria-pressed", "true");
  await expect(beds.nth(count - 1)).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(beds.nth(count - 1)).not.toBeFocused();
  await expect(page.locator("svg.plan :focus")).toHaveCount(0);
});

for (const p of PLATFORMS) {
  test(`UC-37 plan (${p.name}): the platform's modifier undoes and redoes, the other does nothing, never while typing`, async ({ page }) => {
    await openPlan(page, p.platform);
    await select(page, "Leek square");
    const name = page.getByLabel("Name", { exact: true });
    // Set by script, not typed: on a macOS host, Playwright plays Cmd+Z as the
    // browser's own undo of the last typing, even outside the field.
    await name.evaluate((input: HTMLInputElement) => {
      input.value = "Herbs";
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    });
    const renamed = page.getByRole("heading", { name: "Herbs", exact: true });
    const original = page.getByRole("heading", { name: "Leek square", exact: true });
    await expect(renamed).toBeVisible();

    // In the field, the keys belong to the field.
    await name.focus();
    await page.keyboard.press(p.undo);
    await expect(renamed).toBeVisible();
    await name.blur();

    // The modifier of the other platform does nothing.
    await page.keyboard.press(p.foreign);
    await expect(renamed).toBeVisible();

    await page.keyboard.press(p.undo);
    await expect(original).toBeVisible();
    await page.keyboard.press(p.redo);
    await expect(renamed).toBeVisible();
    // Nothing left to redo: nothing happens, nothing is said.
    await page.keyboard.press(p.redo);
    await expect(renamed).toBeVisible();
    await expect(page.getByRole("alert")).toHaveCount(0);
  });
}

const TOOLTIPS = [
  {
    locale: "en-US",
    platform: "MacIntel",
    undo: "Undo the last action (⌘Z)",
    redo: "Redo (⇧⌘Z)",
    remove: "Delete (⌫)",
    close: "Deselect (Esc)",
    plan: "Plan",
  },
  {
    locale: "en-US",
    platform: "Win32",
    undo: "Undo the last action (Ctrl+Z)",
    redo: "Redo (Ctrl+Y)",
    remove: "Delete (Delete)",
    close: "Deselect (Esc)",
    plan: "Plan",
  },
  {
    locale: "fr-FR",
    platform: "MacIntel",
    undo: "Annuler la dernière action (⌘Z)",
    redo: "Rétablir (⇧⌘Z)",
    remove: "Supprimer (⌫)",
    close: "Désélectionner (Échap)",
    plan: "Plan",
  },
  {
    locale: "fr-FR",
    platform: "Linux x86_64",
    undo: "Annuler la dernière action (Ctrl+Z)",
    redo: "Rétablir (Ctrl+Y)",
    remove: "Supprimer (Suppr)",
    close: "Désélectionner (Échap)",
    plan: "Plan",
  },
];

for (const t of TOOLTIPS) {
  test.describe(`${t.locale}, ${t.platform}`, () => {
    test.use({ locale: t.locale });
    test(`UC-42 plan: the tooltips of the buttons show their shortcut (${t.locale}, ${t.platform})`, async ({ page }) => {
      await page.addInitScript((p) => {
        Object.defineProperty(Navigator.prototype, "platform", { get: () => p });
      }, t.platform);
      await page.goto("/?example");
      await page.getByRole("button", { name: t.plan, exact: true }).click();
      await expect(page.locator(`button[title="${t.undo}"]`)).toHaveCount(1);
      await expect(page.locator(`button[title="${t.redo}"]`)).toHaveCount(1);
      await page.locator("g.bed").first().click();
      // The panel's Delete and the one of the bar under the bed (T27).
      await expect(page.locator(`button[title="${t.remove}"]`)).toHaveCount(2);
      await expect(page.locator(`button[title="${t.close}"]`)).toHaveCount(1);
    });
  });
}
