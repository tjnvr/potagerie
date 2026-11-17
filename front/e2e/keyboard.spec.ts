// Keyboard-only use (docs/POT-00_test-matrix.md, KBD-xx): focus stays in a window,
// goes back to the control that opened it, and bed groups answer Enter, Space and Escape.
import { expect, test, type Page } from "@playwright/test";

async function open(page: Page) {
  await page.goto("/?example");
  await expect(page.getByRole("button", { name: "Plan", exact: true })).toBeVisible();
}

const inDialog = (page: Page) =>
  page.evaluate(() => !!document.activeElement?.closest('[role="dialog"], dialog'));

/** Opens a window from the keyboard (focus on its button, then Enter). */
async function openWindow(page: Page, name: string | RegExp) {
  const button = page.getByRole("button", { name }).first();
  await button.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toBeVisible();
  return button;
}

for (const [id, name] of [
  ["KBD-01 settings", /Settings/],
  ["KBD-02 catalogue", /Catalogue/],
] as const) {
  test(`${id}: Tab and Shift+Tab never leave the window, Escape gives the focus back`, async ({ page }) => {
    await open(page);
    const opener = await openWindow(page, name);
    for (let i = 0; i < 80; i++) {
      await page.keyboard.press("Tab");
      expect(await inDialog(page), `Tab ${i}`).toBe(true);
    }
    for (let i = 0; i < 80; i++) {
      await page.keyboard.press("Shift+Tab");
      expect(await inDialog(page), `Shift+Tab ${i}`).toBe(true);
    }
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(opener).toBeFocused();
  });
}

test("KBD-03 export: the window traps the focus and gives it back", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Garden", exact: true }).click();
  const opener = await openWindow(page, /Export the timeline/);
  for (let i = 0; i < 40; i++) {
    await page.keyboard.press("Tab");
    expect(await inDialog(page), `Tab ${i}`).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(opener).toBeFocused();
});

test("KBD-04 note: the window traps the focus and gives it back", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Garden", exact: true }).click();
  const opener = await openWindow(page, /Annotate/);
  for (let i = 0; i < 30; i++) {
    await page.keyboard.press("Tab");
    expect(await inDialog(page), `Tab ${i}`).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(opener).toBeFocused();
});

test("KBD-05 planting form: opened from a free slot with Enter, the focus stays inside", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Garden", exact: true }).click();
  const slot = page.locator("[data-slot]").first();
  await slot.scrollIntoViewIfNeeded();
  await slot.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toBeVisible();
  for (let i = 0; i < 40; i++) {
    await page.keyboard.press("Tab");
    expect(await inDialog(page), `Tab ${i}`).toBe(true);
  }
  await page.keyboard.press("Shift+Tab");
  expect(await inDialog(page)).toBe(true);
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("KBD-06 plan: Enter and Space select a bed, Escape clears the selection and keeps the focus", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  const bed = page.locator("g.bed").first();
  await bed.focus();
  await page.keyboard.press("Enter");
  await expect(bed).toHaveAttribute("aria-pressed", "true");
  await page.keyboard.press("Escape");
  await expect(bed).toHaveAttribute("aria-pressed", "false");
  await page.keyboard.press("Space");
  await expect(bed).toHaveAttribute("aria-pressed", "true");
  await expect(bed).toBeFocused();
});

test("KBD-07 settings: Escape in a confirmation only closes the confirmation", async ({ page }) => {
  await open(page);
  await openWindow(page, /Settings/);
  await page.getByRole("button", { name: "Clear", exact: true }).focus();
  await page.keyboard.press("Enter");
  await expect(page.getByText(/Clear the plan from/)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByText(/Clear the plan from/)).toHaveCount(0);
  await expect(page.getByRole("dialog", { name: "Settings" })).toBeVisible();
});
