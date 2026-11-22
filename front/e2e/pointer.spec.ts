// Pointer precision of the plan editor (T21, docs/POT-00_test-matrix.md PTR-01 to PTR-04): the bed
// lands on the cells under the pointer, whatever the window, at a screen density of 2. The pointer
// is placed from the browser's own mapping of the drawing (`getScreenCTM`), never from the code
// under test; positions are read in centimetres from the SVG.
import { expect, test, type Page } from "@playwright/test";

type Cm = { x: number; y: number };

// The `?example` plot: 1100 × 650 cm, cells of 50 cm, aisle 50 cm. The corner from
// x 700 and y 390 down to the bottom right is free.
const WINDOWS = [
  { name: "1440 × 900", width: 1440, height: 900 },
  // The plan is at most 70 % of the window high: here the element is wider than its
  // drawing, which is centred with empty bands on the sides (the cause found by T21).
  { name: "1440 × 640, low window", width: 1440, height: 640 },
  { name: "360 × 800, phone", width: 360, height: 800 },
];

/** Where the point (x, y) of the plan, in centimetres, is on the screen. */
async function toScreen(page: Page, p: Cm): Promise<Cm> {
  return page.locator("svg.plan").evaluate((svg: SVGSVGElement, p) => {
    const m = svg.getScreenCTM()!;
    return { x: m.a * p.x + m.c * p.y + m.e, y: m.b * p.x + m.d * p.y + m.f };
  }, p);
}

/** The drawn rectangle of the bed named `name`, in centimetres. */
function bed(page: Page, name: string) {
  return page.locator("g.bed", { has: page.locator("text", { hasText: name }) }).locator("rect").first();
}

async function cm(rect: ReturnType<typeof bed>): Promise<Cm> {
  return { x: Number(await rect.getAttribute("x")), y: Number(await rect.getAttribute("y")) };
}

async function openPlan(page: Page) {
  await page.goto("/?example");
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await expect(page.locator(".counter")).toContainText("8 / 10");
  await page.locator("svg.plan").scrollIntoViewIfNeeded();
}

/** Press at `from`, move to `to` in small steps; `release` only after `during` has run. */
async function drag(page: Page, from: Cm, to: Cm, during: () => Promise<void>) {
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(to.x, to.y, { steps: 12 });
  await during();
  await page.mouse.up();
}

/** Screen pixels per centimetre of the plan. */
async function pxPerCm(page: Page): Promise<number> {
  return page.locator("svg.plan").evaluate((svg: SVGSVGElement) => svg.getScreenCTM()!.a);
}

async function clickAt(page: Page, p: Cm) {
  const s = await toScreen(page, p);
  await page.mouse.click(s.x, s.y);
}

/** The right and bottom edges of `rect` are under the pointer `at`: the bottom one
 *  within a pixel (to the centimetre), the right one within half a cell (cell by cell). */
async function edgesUnderPointer(page: Page, rect: ReturnType<typeof bed>, at: Cm) {
  const n = async (a: string) => Number(await rect.getAttribute(a));
  const corner = await toScreen(page, { x: (await n("x")) + (await n("width")), y: (await n("y")) + (await n("height")) });
  expect(Math.abs(corner.y - at.y)).toBeLessThanOrEqual(1);
  expect(Math.abs(corner.x - at.x)).toBeLessThanOrEqual(25 * (await pxPerCm(page)) + 1);
}

/** The panel shows the size drawn on the plan, in metres. */
async function panelMatches(page: Page, rect: ReturnType<typeof bed>) {
  const inputs = page.locator(".dimensions input");
  await expect(inputs.nth(0)).toHaveValue(String(Number(await rect.getAttribute("width")) / 100));
  await expect(inputs.nth(1)).toHaveValue(String(Number(await rect.getAttribute("height")) / 100));
}

for (const w of WINDOWS) {
  test.describe(`${w.name}, density 2`, () => {
    test.use({ viewport: { width: w.width, height: w.height }, deviceScaleFactor: 2 });

    test("PTR-01 an item of the tray lands centred on the cells under the pointer", async ({ page }) => {
      await openPlan(page);
      const item = page.locator("button.element").filter({ hasText: "Bed" }).first();
      // On a phone the tray is under the plan: scroll so that both are on screen.
      await page.evaluate(() => {
        const plan = document.querySelector("svg.plan")!.getBoundingClientRect();
        window.scrollBy(0, plan.top - 8);
      });
      await expect(item).toBeInViewport();
      const box = (await item.boundingBox())!;
      // A 300 × 120 cm bed held by its centre: the pointer at (910, 518) puts its top
      // left at (760, 458), on the cell (750, 450).
      const target = await toScreen(page, { x: 910, y: 518 });
      const ghost = page.locator("rect.ghost");
      await drag(page, { x: box.x + box.width / 2, y: box.y + box.height / 2 }, target, async () => {
        await expect(ghost).toHaveAttribute("x", "750");
        await expect(ghost).toHaveAttribute("y", "450");
      });
      await expect(page.locator(".counter")).toContainText("9 / 10");
      await expect(page.locator('g.bed > rect:first-child[x="750"][y="450"][width="300"]')).toHaveCount(1);
    });

    test("PTR-02 a bed dragged by its middle stays under the pointer, the preview is where it lands", async ({ page }) => {
      await openPlan(page);
      const rect = bed(page, "Salad square"); // 100 × 100 at (550, 390)
      expect(await cm(rect)).toEqual({ x: 550, y: 390 });
      // Held 50 cm in, it goes 300 cm right and 110 cm down, 10 cm off the grid.
      const from = await toScreen(page, { x: 600, y: 440 });
      const to = await toScreen(page, { x: 910, y: 550 });
      await drag(page, from, to, async () => {
        expect(await cm(rect)).toEqual({ x: 850, y: 500 });
      });
      await expect(page.getByRole("alert")).toHaveCount(0);
      await expect.poll(() => cm(rect)).toEqual({ x: 850, y: 500 });
    });

    test("PTR-03 a bed dragged by a corner stays under the pointer, the preview is where it lands", async ({ page }) => {
      await openPlan(page);
      const rect = bed(page, "Leek square"); // 100 × 100 at (400, 390)
      expect(await cm(rect)).toEqual({ x: 400, y: 390 });
      // Held 15 cm inside its bottom right corner.
      const from = await toScreen(page, { x: 485, y: 475 });
      const to = await toScreen(page, { x: 892, y: 593 });
      await drag(page, from, to, async () => {
        expect(await cm(rect)).toEqual({ x: 800, y: 500 });
      });
      await expect(page.getByRole("alert")).toHaveCount(0);
      await expect.poll(() => cm(rect)).toEqual({ x: 800, y: 500 });
    });

    test("PTR-05 a corner handle pressed on its dot: the edges end under the pointer, the panel shows that size", async ({ page }) => {
      await openPlan(page);
      const rect = bed(page, "Salad square"); // 100 × 100 at (550, 390): right 650, bottom 490
      await clickAt(page, { x: 600, y: 440 });
      const from = await toScreen(page, { x: 650, y: 490 });
      // Right to 787 cm: 4.74 cells, so 5 (right edge 800); down to 571 cm, to the centimetre.
      const to = await toScreen(page, { x: 787, y: 571 });
      await drag(page, from, to, async () => {
        expect(await rect.getAttribute("width")).toBe("250");
        await edgesUnderPointer(page, rect, to);
      });
      await expect.poll(() => rect.getAttribute("width")).toBe("250");
      await edgesUnderPointer(page, rect, to);
      await panelMatches(page, rect);
    });

    test("PTR-06 a handle pressed off its dot: the edge moves with the pointer, without a jump", async ({ page }) => {
      await openPlan(page);
      const rect = bed(page, "Salad square");
      await clickAt(page, { x: 600, y: 440 });
      // Pressed in the hit circle of the south-east handle, half a radius right of and
      // below its centre: away from the dot, which is at or inside the centre.
      const zone = page.locator('circle.handle-zone[data-handle="se"]');
      const r = Number(await zone.getAttribute("r"));
      const press = { x: Number(await zone.getAttribute("cx")) + r / 2, y: Number(await zone.getAttribute("cy")) + r / 2 };
      const from = await toScreen(page, press);
      const cmPerPx = 1 / (await pxPerCm(page));
      await page.mouse.move(from.x, from.y);
      await page.mouse.down();
      // A first move of 8 px down: the bottom edge moves 8 px too, it does not jump to the pointer.
      await page.mouse.move(from.x, from.y + 8, { steps: 4 });
      expect(await rect.getAttribute("width")).toBe("100");
      expect(Math.abs(Number(await rect.getAttribute("height")) - (100 + 8 * cmPerPx))).toBeLessThanOrEqual(1 + cmPerPx);
      // Then 140 cm right and 60 cm down from the press: 240 cm long, so 5 cells
      // (250 cm), and 60 cm wider.
      const to = await toScreen(page, { x: press.x + 140, y: press.y + 60 });
      await page.mouse.move(to.x, to.y, { steps: 12 });
      await page.mouse.up();
      await expect.poll(() => rect.getAttribute("width")).toBe("250");
      expect(Math.abs(Number(await rect.getAttribute("height")) - 160)).toBeLessThanOrEqual(1 + cmPerPx);
      await panelMatches(page, rect);
    });

    test("PTR-04 a zone drawn on the grass covers the cells from the press to the release", async ({ page }) => {
      await openPlan(page);
      const from = await toScreen(page, { x: 760, y: 410 });
      const to = await toScreen(page, { x: 890, y: 540 });
      const ghost = page.locator("rect.ghost");
      await drag(page, from, to, async () => {
        await expect(ghost).toHaveAttribute("x", "750");
        await expect(ghost).toHaveAttribute("y", "400");
        await expect(ghost).toHaveAttribute("width", "150");
        await expect(ghost).toHaveAttribute("height", "150");
      });
      await expect(page.locator(".counter")).toContainText("9 / 10");
      await expect(
        page.locator('g.bed > rect:first-child[x="750"][y="400"][width="150"][height="150"]'),
      ).toHaveCount(1);
    });
  });
}

// With a finger (Chromium touch events): the handles are targets of 40 px at least,
// they leave the middle of a small bed free to move it, and they follow the finger
// at once, with no long press.
test.describe("360 × 800, phone, touch, density 2", () => {
  test.use({ viewport: { width: 360, height: 800 }, deviceScaleFactor: 2, hasTouch: true });

  test("PTR-07 a finger resizes with a handle: 40 px targets, the edges end under the finger, the panel shows that size", async ({ page, browserName }) => {
    test.skip(browserName !== "chromium", "touch events are sent through the Chromium protocol");
    await openPlan(page);
    const cdp = await page.context().newCDPSession(page);
    const touch = (type: string, p?: Cm) =>
      cdp.send("Input.dispatchTouchEvent", { type, touchPoints: p ? [{ x: p.x, y: p.y }] : [] });
    const rect = bed(page, "Salad square"); // 100 × 100 at (550, 390)
    const centre = await toScreen(page, { x: 600, y: 440 });
    await touch("touchStart", centre);
    await touch("touchEnd");
    await expect(page.locator("circle.handle-zone")).not.toHaveCount(0);

    const zones = await page.locator("circle.handle-zone").evaluateAll((els) =>
      els.map((e) => {
        const b = e.getBoundingClientRect();
        return { w: b.width, h: b.height };
      }),
    );
    for (const z of zones) {
      expect(z.w).toBeGreaterThanOrEqual(40);
      expect(z.h).toBeGreaterThanOrEqual(40);
    }
    // The middle of the bed is still the bed, not a handle.
    const hit = await page.evaluate(
      ([x, y]) => document.elementFromPoint(x, y)?.closest("g.bed, circle.handle-zone")?.tagName,
      [centre.x, centre.y],
    );
    expect(hit).toBe("g");

    // Dragged at once (no long press) from the south-east dot to (787, 571).
    const from = await toScreen(page, { x: 650, y: 490 });
    const to = await toScreen(page, { x: 787, y: 571 });
    // A finger lands on the handle, not on the bed under it (Chromium moves a touch to
    // the nearest element that listens for it).
    await touch("touchStart", from);
    for (let i = 1; i <= 12; i++)
      await touch("touchMove", { x: from.x + ((to.x - from.x) * i) / 12, y: from.y + ((to.y - from.y) * i) / 12 });
    expect(await rect.getAttribute("width")).toBe("250");
    await edgesUnderPointer(page, rect, to);
    await touch("touchEnd");
    await expect.poll(() => rect.getAttribute("width")).toBe("250");
    await edgesUnderPointer(page, rect, to);
    await panelMatches(page, rect);
  });
});
