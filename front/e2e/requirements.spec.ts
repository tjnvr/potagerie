// Tests written for the gaps found in the requirement → test table of
// docs/POT-00_test-matrix.md ("Coverage of the requirements"). Ids: REQ-xx.
import { expect, test, type Page } from "@playwright/test";

async function open(page: Page) {
  await page.goto("/?example");
  await expect(page.getByRole("button", { name: "Plan", exact: true })).toBeVisible();
}

async function toGarden(page: Page) {
  await page.getByRole("button", { name: "Garden", exact: true }).click();
  await expect(page.getByRole("button", { name: "Play the season" })).toBeVisible();
}

/** Opens the edit window of the first bar that is visible in the timeline. */
async function openFirstBar(page: Page) {
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
}

test("REQ-01 F-PLA-2: deleting a planting asks first; Cancel keeps it, confirming removes it", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const bars = page.locator(".bar");
  const before = await bars.count();
  await openFirstBar(page);
  await page.getByRole("button", { name: "Delete the planting" }).click();
  await expect(page.getByText("Delete this planting?")).toBeVisible();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(page.getByText("Delete this planting?")).toHaveCount(0);
  await expect(bars).toHaveCount(before);
  await page.getByRole("button", { name: "Delete the planting" }).click();
  await page.getByRole("button", { name: "Delete the planting" }).last().click();
  await expect(page.getByRole("heading", { name: "Edit the planting" })).toHaveCount(0);
  await expect(bars).toHaveCount(before - 1);
});

test("REQ-02 F-VIS-5: the annotations can be hidden, then one is deleted", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const notes = page.locator(".note");
  const before = await notes.count();
  expect(before).toBeGreaterThan(0);
  const toggle = page.getByLabel("Show the annotations");
  await toggle.uncheck();
  await expect(notes).toHaveCount(0);
  await toggle.check();
  await expect(notes).toHaveCount(before);
  await notes.first().scrollIntoViewIfNeeded();
  await notes.first().click({ force: true });
  await expect(page.getByRole("heading", { name: /^Edit the (note|period)$/ })).toBeVisible();
  await page.getByRole("button", { name: /^Delete/ }).first().click();
  const last = page.getByRole("button", { name: /^Delete/ }).last();
  if ((await page.getByRole("heading", { name: /^Edit the (note|period)$/ }).count()) > 0) await last.click();
  await expect(notes).toHaveCount(before - 1);
});

test("REQ-03 F-PLN-1: at 10 beds the tray is greyed out and says why", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  const counter = page.locator(".counter").first();
  const items = page.locator(".element");
  await expect(counter).toContainText("8 / 10");
  await expect(items.first()).toBeEnabled();
  await expect(items).toHaveCount(4);
  for (let i = 0; i < 2; i++) {
    await items.nth(1).focus(); // the square: small enough to find a free spot
    await page.keyboard.press("Enter");
    await expect(counter).toContainText(`${9 + i} / 10`);
  }
  for (let i = 0; i < 4; i++) await expect(items.nth(i)).toBeDisabled();
  await expect(page.getByText("10 beds at most per season.")).toBeVisible();
});

test("REQ-04 F-PLA-5: the planting form shows the estimated quantity, which follows the zone", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const slot = page.locator("[data-slot]").first();
  await slot.scrollIntoViewIfNeeded();
  await slot.click({ force: true });
  const dialog = page.getByRole("dialog");
  const quantity = dialog.getByText(/≈ \d+ plants, \d+ rows? × \d+/);
  await expect(quantity).toBeVisible();
  const read = async () => {
    const found = /≈ (\d+) plants, (\d+) rows? × (\d+)/.exec((await quantity.textContent())!)!;
    return { plants: Number(found[1]), rows: Number(found[2]), perRow: Number(found[3]) };
  };
  const all = await read();
  expect(all.plants).toBe(all.rows * all.perRow);
  expect(all.rows).toBeGreaterThanOrEqual(1);
  expect(all.perRow).toBeGreaterThanOrEqual(1);
  await expect(dialog.getByRole("button", { name: /^Plant ·/ })).toContainText(`≈ ${all.plants} plants`);
  const quarter = dialog.getByRole("button", { name: "¼" });
  if (await quarter.isEnabled()) {
    await quarter.click();
    await expect(async () => {
      const q = await read();
      expect(q.plants).toBe(q.rows * q.perRow);
      expect(q.perRow).toBeLessThanOrEqual(all.perRow);
      await expect(dialog.getByRole("button", { name: /^Plant ·/ })).toContainText(`≈ ${q.plants} plants`);
    }).toPass();
  }
});

test("REQ-05 F-VIS-7: a bed is renamed in place; an empty name is refused and restored", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await page.mouse.click(640, 340);
  const name = page.getByLabel("Name", { exact: true });
  await expect(name).toBeVisible();
  expect(await name.inputValue()).not.toBe("Shed side");
  await expect(name).toHaveAttribute("maxlength", "30");
  await name.fill("Shed side");
  await name.dispatchEvent("change");
  await expect(page.getByRole("heading", { name: "Shed side" })).toBeVisible();
  await name.fill("");
  await name.dispatchEvent("change");
  await expect(name).toHaveValue("Shed side");
});

test("REQ-06 F-VIS-1: the rotate button turns the garden a quarter at a time, four turns come back", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const bed = page.getByRole("button", { name: /Plantings in the ground: \d+\. Open the details/ }).first();
  const here = async () => {
    const b = (await bed.boundingBox())!;
    return `${Math.round(b.x)},${Math.round(b.y)}`;
  };
  const start = await here();
  const seen = new Set([start]);
  for (let i = 0; i < 3; i++) {
    await page.getByRole("button", { name: "Rotate the view" }).click();
    seen.add(await here());
  }
  expect(seen.size).toBeGreaterThan(1);
  await page.getByRole("button", { name: "Rotate the view" }).click();
  expect(await here()).toBe(start);
});
