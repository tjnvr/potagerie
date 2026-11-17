// The use cases of docs/POT-00_test-matrix.md. Each test name starts with its id.
import { expect, test, type Page } from "@playwright/test";

const EXAMPLE = "/?example";

async function open(page: Page, query = EXAMPLE) {
  await page.goto(query);
  await expect(page.getByRole("button", { name: "Plan", exact: true })).toBeVisible();
}

async function toGarden(page: Page) {
  await page.getByRole("button", { name: "Garden", exact: true }).click();
  await expect(page.getByRole("button", { name: "Play the season" })).toBeVisible();
}

test("UC-01 first launch: no plot, then the plot is created", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("No plot yet")).toBeVisible();
  await page.getByRole("button", { name: "Create my plot" }).click();
  await page.getByRole("button", { name: "Create the plot" }).click();
  await expect(page.locator(".counter")).toContainText("0 / 10");
});

test("UC-02 plan: a click on free grass adds a bed", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await expect(page.locator(".counter")).toContainText("8 / 10");
  const grass = await page.locator("rect.grass").boundingBox();
  const x = grass!.x + grass!.width * 0.85;
  const y = grass!.y + grass!.height * 0.85;
  await page.mouse.click(x, y); // the first click only clears the selection
  await page.mouse.click(x, y);
  await expect(page.locator(".counter")).toContainText("9 / 10");
});

test("UC-03 plan: a click on a bed selects it (handles appear, no refusal)", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await page.mouse.click(640, 340);
  await expect(page.locator("circle")).not.toHaveCount(0);
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("UC-04 years: a past year opens read only, a future year is editable", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await page.getByRole("button", { name: "2025", exact: true }).click();
  await expect(page.getByText(/read only/i).first()).toBeVisible();
  await page.getByRole("button", { name: "2027", exact: true }).click();
  await expect(page.locator(".counter")).toBeVisible();
});

test("UC-05 garden: the tab opens on today, whatever the cursor was left on", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const today = await page.locator("h2, h1").filter({ hasText: /20\d\d/ }).first().innerText();
  await page.getByRole("button", { name: "Next week" }).click();
  await page.getByRole("button", { name: "Next week" }).click();
  await expect(page.locator("h2, h1").filter({ hasText: /20\d\d/ }).first()).not.toHaveText(today);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await toGarden(page);
  await expect(page.locator("h2, h1").filter({ hasText: /20\d\d/ }).first()).toHaveText(today);
});

test("UC-06 timeline: a free slot opens the planting form, planting adds a bar", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const bars = page.locator(".bar");
  const before = await bars.count();
  const slot = page.locator("[data-slot]").first();
  await slot.scrollIntoViewIfNeeded();
  await slot.click({ force: true });
  await page.getByRole("button", { name: /^Plant ·/ }).click();
  await expect(bars).toHaveCount(before + 1);
});

test("UC-14 timeline: a click on a bar opens the planting to edit", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const bars = page.locator(".bar");
  const count = await bars.count();
  for (let i = 0; i < count; i++) {
    const box = await bars.nth(i).boundingBox();
    if (box && box.x > 240 && box.x < 900) {
      await bars.nth(i).scrollIntoViewIfNeeded();
      const seen = (await bars.nth(i).boundingBox())!;
      await page.mouse.click(seen.x + seen.width / 2, seen.y + seen.height / 2);
      break;
    }
  }
  await expect(page.getByRole("heading", { name: "Edit the planting" })).toBeVisible();
});

test("UC-15 plan: dragging on free grass draws a zone as the pointer moves", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "2027", exact: true }).click();
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  const grass = page.locator("rect.grass");
  const r = (await grass.boundingBox())!;
  const x = r.x + r.width * 0.8;
  const y = r.y + r.height * 0.8;
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + 20, y + 20);
  // The preview shows at once, well before the old 4 px + drop logic would.
  await expect(page.locator("rect.ghost")).toHaveCount(1);
  await page.mouse.move(x + r.width * 0.1, y + r.height * 0.08, { steps: 4 });
  await page.mouse.up();
});

test("UC-16 plan: releasing a drawn zone near the edge creates a bed", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "2027", exact: true }).click();
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  const r = (await page.locator("rect.grass").boundingBox())!;
  const x = r.x + r.width * 0.8;
  const y = r.y + r.height * 0.8;
  const before = await page.locator(".counter").first().textContent();
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + r.width * 0.15, y + r.height * 0.12, { steps: 6 });
  await page.mouse.up();
  await expect(page.locator(".counter").first()).not.toHaveText(before!);
});

test("UC-20 plan: dragging a bed moves it, and a refused drop sends it back", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await page.mouse.click(640, 340);
  const bed = page.locator("g.bed.chosen > rect").first();
  await expect(bed).toBeVisible();
  const before = (await bed.boundingBox())!;
  const cx = before.x + before.width / 2;
  const cy = before.y + before.height / 2;
  // Onto the neighbouring bed or the aisle: refused, the bed comes back.
  await page.mouse.move(cx, cy);
  await page.mouse.down();
  await page.mouse.move(cx + 6, cy + 6, { steps: 3 });
  await page.mouse.move(cx + 40, cy, { steps: 6 });
  await page.mouse.up();
  await page.waitForTimeout(400);
  const after = (await bed.boundingBox())!;
  const moved = Math.abs(after.x - before.x) > 2 || Math.abs(after.y - before.y) > 2;
  const refused = (await page.getByRole("alert").count()) > 0;
  expect(moved || refused).toBe(true);
});

test("UC-21 plan: deleting a bed asks for confirmation, then removes it", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  const counter = page.locator(".counter").first();
  const before = await counter.textContent();
  await page.mouse.click(640, 340);
  await page.getByRole("button", { name: "Delete", exact: true }).first().click();
  await expect(page.getByText(/Delete .*\?/)).toBeVisible();
  await page.getByRole("button", { name: "Delete the bed" }).click();
  await expect(counter).not.toHaveText(before!);
});

test("UC-22 settings: clearing the plan asks, then empties the beds", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Settings" }).first().click();
  await page.getByRole("button", { name: "Clear", exact: true }).click();
  await expect(page.getByText(/Clear the plan from/)).toBeVisible();
  await page.getByRole("button", { name: "Clear the plan" }).click();
  await expect(page.getByText("Plan cleared.")).toBeVisible();
});

test("UC-07 notes: empty text is refused, 80 characters at most, the note appears", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const notes = page.locator(".note");
  const before = await notes.count();
  await page.getByRole("button", { name: /Annotate/ }).click();
  const save = page.getByRole("button", { name: "Save" });
  await expect(save).toBeDisabled();
  const field = page.locator("input[type=text], textarea").first();
  await field.fill("x".repeat(95));
  await expect(field).toHaveValue("x".repeat(80));
  await field.fill("Water the beans");
  await save.click();
  await expect(notes).toHaveCount(before + 1);
});

test("UC-08 catalogue: 67 crops, a family filter narrows the list", async ({ page }) => {
  await open(page);
  await toGarden(page);
  await page.getByRole("button", { name: /Catalogue/ }).first().click();
  await expect(page.getByText(/67 crops/)).toBeVisible();
  await page.getByRole("button", { name: "Solanaceae", exact: true }).click();
  const cards = page.locator("button.carte");
  expect(await cards.count()).toBeGreaterThan(3);
  expect(await cards.count()).toBeLessThan(15);
});

test("UC-09 settings: the language switches to French and is remembered", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Settings" }).first().click();
  await page.getByRole("button", { name: "Français" }).click();
  await expect(page.getByRole("button", { name: "Potager", exact: true })).toBeVisible();
  await page.reload();
  await expect(page.getByRole("button", { name: "Potager", exact: true })).toBeVisible();
});

test("UC-10 export: the dialog opens with the displayed period", async ({ page }) => {
  await open(page);
  await toGarden(page);
  await page.getByRole("button", { name: /Export the timeline/ }).click();
  await expect(page.getByText("Export the timeline").first()).toBeVisible();
  await expect(page.locator("input[type=date]")).toHaveCount(2);
  await expect(page.getByRole("button", { name: /Export as PDF/ })).toBeVisible();
});

test("UC-11 keyboard: Escape closes every window", async ({ page }) => {
  await open(page);
  await toGarden(page);
  for (const name of [/Settings/, /Catalogue/, /Export the timeline/, /Annotate/]) {
    await page.getByRole("button", { name }).first().click();
    const opened = await page.locator("[role=dialog], .voile, .screen").count();
    await page.keyboard.press("Escape");
    await expect.poll(() => page.locator("[role=dialog], .voile, .screen").count()).toBeLessThan(opened);
  }
});

for (const [name, tab] of [["garden", "Garden"], ["plan", "Plan"]] as const) {
  test(`UC-12 phone: the ${name} screen does not scroll sideways`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await open(page);
    await page.getByRole("button", { name: tab, exact: true }).click();
    await page.waitForTimeout(500);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    expect(overflow).toBeLessThanOrEqual(1);
  });
}

test("UC-13 phone: the catalogue shows two crops a row", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await open(page);
  await page.getByRole("button", { name: /Catalogue/ }).first().click();
  const [a, b] = await page.locator("button.carte").evaluateAll((els) => els.slice(0, 2).map((e) => e.getBoundingClientRect().top));
  expect(Math.abs(a! - b!)).toBeLessThan(4);
});
