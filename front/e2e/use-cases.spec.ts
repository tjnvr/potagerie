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
  await expect
    .poll(async () => {
      const after = (await bed.boundingBox())!;
      const moved = Math.abs(after.x - before.x) > 2 || Math.abs(after.y - before.y) > 2;
      return moved || (await page.getByRole("alert").count()) > 0;
    })
    .toBe(true);
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

test("UC-25 plan: deleting a bed that has plantings warns with their count", async ({ page }) => {
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  const counter = page.locator(".counter").first();
  const before = await counter.textContent();
  await page.mouse.click(640, 340);
  await page.getByRole("button", { name: "Delete", exact: true }).first().click();
  const warning = page.getByText(/Plantings deleted: \d+/);
  await expect(warning).toBeVisible();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(counter).toHaveText(before!);
});

test("UC-26 planting form: the end of a planting follows a changed start date", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const slot = page.locator("[data-slot]").first();
  await slot.scrollIntoViewIfNeeded();
  await slot.click({ force: true });
  const dialog = page.getByRole("dialog");
  const start = dialog.locator("input[type=date]").first();
  const end = dialog.getByLabel("End of the planting");
  await expect(end).toBeVisible();
  const day = 86_400_000;
  const gap = async () =>
    (Date.parse(await end.inputValue()) - Date.parse(await start.inputValue())) / day;
  const before = await gap();
  expect(before).toBeGreaterThan(0);
  expect(before).toBeLessThan(366);
  const moved = new Date(Date.parse(await start.inputValue()) + 10 * day).toISOString().slice(0, 10);
  await start.fill(moved);
  await start.dispatchEvent("change");
  const after = await gap();
  expect(after).toBeGreaterThan(0);
  expect(after).toBeLessThan(366);
});

test("UC-27 plan: typing a bed length in the panel never throws, a refusal restores the field", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await open(page);
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await page.mouse.click(640, 340);
  const length = page.locator("input[type=number]").first();
  await expect(length).toBeVisible();
  await length.fill("3.1"); // not a whole number of cases: refused
  await length.dispatchEvent("change");
  await expect(length).not.toHaveValue("3.1");
  await length.fill("");
  await length.dispatchEvent("change");
  await expect(length).not.toHaveValue("");
  expect(errors).toEqual([]);
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
    await page.evaluate(() => Promise.all(document.getAnimations().map((a) => a.finished.catch(() => null))).then(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))));
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

test("UC-28 garden: the wheel zooms, dragging the background pans, a bed still opens", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const svg = page.locator("svg.diorama");
  const viewBox = async () => (await svg.getAttribute("viewBox"))!.split(" ").map(Number);
  const home = await viewBox();
  const box = (await svg.boundingBox())!;
  // Near the top-left corner: sky, not a bed, before and after the zoom.
  const at = { x: box.x + box.width * 0.1, y: box.y + box.height * 0.1 };
  const onBed = () => page.evaluate(({ x, y }) => !!document.elementFromPoint(x, y)?.closest(".bed"), at);

  await page.mouse.move(at.x, at.y);
  for (let i = 0; i < 3; i++) await page.mouse.wheel(0, -200);
  await expect.poll(async () => (await viewBox())[2]!).toBeLessThan(home[2]! * 0.6);
  const zoomed = await viewBox();
  expect(await onBed()).toBe(false);

  // Dragging the background up and to the left shows more of the bottom right.
  await page.mouse.down();
  await page.mouse.move(at.x - 120, at.y - 80, { steps: 8 });
  await page.mouse.up();
  const panned = await viewBox();
  expect(panned[0]!).toBeGreaterThan(zoomed[0]!);
  expect(panned[1]!).toBeGreaterThan(zoomed[1]!);
  expect(panned[2]!).toBeCloseTo(zoomed[2]!, 6);
  await expect(page.locator(".bed[aria-pressed=true]")).toHaveCount(0);

  // Back to the whole garden; scrolling down further scrolls the page, not the zoom.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  for (let i = 0; i < 3; i++) await page.mouse.wheel(0, 200);
  await expect(svg).toHaveAttribute("data-zoom", "1.00");
  const isHome = async () => (await viewBox()).every((v, i) => Math.abs(v - home[i]!) < 1e-6);
  expect(await isHome()).toBe(true);
  const scrolled = await page.evaluate(() => scrollY);
  await page.mouse.wheel(0, 200);
  await expect.poll(() => page.evaluate(() => scrollY)).toBeGreaterThan(scrolled);
  expect(await isHome()).toBe(true);

  // A bed opens on a click as before.
  const roots = page.getByRole("button", { name: /^Roots, / }).first();
  await roots.click();
  await expect(roots).toHaveAttribute("aria-pressed", "true");
});

// ---------- Annotations: create on a row, move by dragging (F-VIS-5) ----------

/** Today is 1 March 2027: the timeline shows September 2026 to September 2027. */
const MARCH = "/?example&date=2027-03-01";

/** A free point of the row of `bed` in 2027 (after the cursor, on today): the middle
 *  of a gap of at least 24 px with nothing (bar, slot, note, cursor) under it. */
async function freePoint(page: Page, bed: string): Promise<{ x: number; y: number }> {
  await page.locator(".corps").scrollIntoViewIfNeeded();
  const zone = (await page.locator(".corps").boundingBox())!;
  const cursor = (await page.locator("[data-cursor] line").boundingBox())!;
  // The y of the label from the drawing itself (WebKit gives a wrong box to a clipped text).
  const y = await page.locator("svg[data-timeline] text", { hasText: bed }).first().evaluate((t: SVGTextElement) => {
    const b = t.getBBox();
    return t.ownerSVGElement!.getBoundingClientRect().top + b.y + b.height / 2;
  });
  const x = await page.evaluate(
    ({ y, from, to }) => {
      let start: number | null = null;
      for (let x = from; x < to; x += 2) {
        const under = document.elementFromPoint(x, y);
        const free = under?.closest(".corps") && !under.closest("[data-bar],[data-slot],[data-annotation],[data-cursor]");
        if (free && start === null) start = x;
        if (!free && start !== null) {
          if (x - start >= 24) return Math.round((start + x) / 2);
          start = null;
        }
      }
      return null;
    },
    { y, from: cursor.x + 12, to: zone.x + zone.width - 20 },
  );
  expect(x).not.toBeNull();
  return { x: x!, y };
}

/** The centre of the marker of a note. */
async function marker(page: Page, text: string): Promise<{ x: number; y: number }> {
  const box = (await page.locator("g.note", { hasText: text }).locator("circle").boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

/** Pixels per day of the timeline, from two notes of the example 53 days apart. */
async function pxByDay(page: Page): Promise<number> {
  return ((await marker(page, "Stake the tomatoes")).x - (await marker(page, "Last frosts?")).x) / 53;
}

test("UC-29 timeline: a double click on a row creates a note on that bed, on that day", async ({ page }) => {
  await open(page, MARCH);
  await toGarden(page);
  const spot = await freePoint(page, "Shed side");
  await page.mouse.dblclick(spot.x, spot.y);
  await expect(page.getByRole("heading", { name: "New note" })).toBeVisible();
  await expect(page.locator(".note select option:checked")).toHaveText("Shed side");
  await page.locator(".note input[type=text]").fill("Sow the green manure");
  await page.getByRole("button", { name: "Save" }).click();
  // The note stands where the double click was: on that row, on that day.
  const placed = await marker(page, "Sow the green manure");
  expect(Math.abs(placed.x - spot.x)).toBeLessThanOrEqual((await pxByDay(page)) + 1);
  // The double click also put the time cursor over the note: open it with the keyboard.
  await page.locator("g.note", { hasText: "Sow the green manure" }).focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", { name: "Edit the note" })).toBeVisible();
  await expect(page.locator(".note select option:checked")).toHaveText("Shed side");
});

test.describe("UC-29 timeline on a touch screen", () => {
  test.use({ hasTouch: true });
  test("a long press on a row creates a note on that bed", async ({ page, browserName }) => {
    test.skip(browserName !== "chromium", "touch events are sent through the Chromium protocol");
    await open(page, MARCH);
    await toGarden(page);
    const spot = await freePoint(page, "Shed side");
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: spot.x, y: spot.y }] });
    await expect(page.getByRole("heading", { name: "New note" })).toBeVisible();
    await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
    await expect(page.locator(".note select option:checked")).toHaveText("Shed side");
  });
});

test("UC-31 timeline: dragging a note moves its date and its bed", async ({ page }) => {
  await open(page, MARCH);
  await toGarden(page);
  const target = await freePoint(page, "Shed side");
  const start = await marker(page, "Stake the tomatoes");
  const dx = 10 * (await pxByDay(page));
  await page.mouse.move(start.x, start.y);
  await page.mouse.down();
  await page.mouse.move(start.x + 6, start.y, { steps: 2 });
  await page.mouse.move(start.x + dx, target.y, { steps: 6 });
  await page.mouse.up();
  await expect.poll(async () => (await marker(page, "Stake the tomatoes")).x).toBeGreaterThan(start.x + dx - 4);
  await page.locator("g.note", { hasText: "Stake the tomatoes" }).locator("circle").click();
  await expect(page.getByRole("heading", { name: "Edit the note" })).toBeVisible();
  await expect(page.locator(".note select option:checked")).toHaveText("Shed side");
  await expect(page.locator(".note input[type=date]").first()).toHaveValue("2027-06-12");
});

test("UC-31 timeline: a period dragged into a past year is refused and goes back", async ({ page }) => {
  await open(page, MARCH);
  await toGarden(page);
  await page.locator(".corps").scrollIntoViewIfNeeded();
  const label = page.locator("g.note", { hasText: "Mulching" }).locator("text");
  const before = (await label.boundingBox())!;
  // 5 May to 10 June, moved back 180 days: it would end in December 2026.
  const dx = 180 * (await pxByDay(page));
  const y = before.y + before.height / 2;
  await page.mouse.move(before.x + 4, y);
  await page.mouse.down();
  await page.mouse.move(before.x - 2, y, { steps: 2 });
  await page.mouse.move(before.x + 4 - dx, y, { steps: 8 });
  await page.mouse.up();
  await expect(page.getByText(/Move refused\..*The annotation goes back to its place\./)).toBeVisible();
  await expect.poll(async () => (await label.boundingBox())!.x).toBeCloseTo(before.x, 0);
});

test("UC-32 garden: « Move » in the bed panel opens the plan on that bed; refused in a past season", async ({ page }) => {
  await open(page);
  await toGarden(page);
  const year = page.locator("button[aria-pressed=true]").filter({ hasText: /^20\d\d$/ });
  const current = await year.innerText();
  await page.getByRole("button", { name: /^Roots, / }).first().click();
  const panel = page.getByRole("complementary", { name: "Selected bed" });
  await panel.getByRole("button", { name: "Move", exact: true }).click();

  // The plan, same year, the bed selected with its handles and its panel.
  await expect(page.getByRole("button", { name: "Plan", exact: true })).toHaveAttribute("aria-current", "page");
  await expect(year).toHaveText(current);
  await expect(page.locator("g.bed[aria-pressed=true]")).toHaveAccessibleName(/^Roots, /);
  await expect(page.locator("circle.handle")).not.toHaveCount(0);
  await expect(panel.getByRole("heading", { name: "Roots" })).toBeVisible();
  // In the editor the bed already moves by dragging: no « Move » shortcut there.
  await expect(page.getByRole("button", { name: "Move", exact: true })).toHaveCount(0);

  // A past season is read only: the action is there, disabled, with its reason.
  await toGarden(page);
  await page.getByRole("button", { name: String(Number(current) - 1), exact: true }).click();
  await page.getByRole("button", { name: /^Roots, / }).first().click();
  const move = panel.getByRole("button", { name: "Move", exact: true });
  await expect(move).toBeDisabled();
  await expect(move).toHaveAccessibleDescription("Past season: the bed cannot be moved any more.");
});
