// Touch in the plan editor (T22, docs/POT-00_test-matrix.md TCH-01 to TCH-05): after the long
// press the bed follows the finger at each move and lands under it, the page neither scrolls nor
// zooms meanwhile; a swipe on the grass still scrolls, a tap still selects. The moves are real
// touches sent to the browser (Chromium's `Input.dispatchTouchEvent`), so `touch-action`, the
// scrolling and the pointer events are the browser's own; Playwright can only tap in WebKit.
import { expect, test, type CDPSession, type Locator, type Page } from "@playwright/test";

type Cm = { x: number; y: number };
type Finger = { x: number; y: number; id?: number };

// The `?example` plot: 1100 × 650 cm, cells of 50 cm. The corner from x 700 and
// y 390 down to the bottom right is free.
const SCREENS = [
  { name: "phone 360 × 800", width: 360, height: 800, isMobile: true },
  { name: "touch screen 1440 × 900", width: 1440, height: 900, isMobile: false },
];

/** Longer than the long press of the editor (450 ms). */
const LONG_PRESS_MS = 600;

async function toScreen(page: Page, p: Cm): Promise<Cm> {
  return page.locator("svg.plan").evaluate((svg: SVGSVGElement, p) => {
    const m = svg.getScreenCTM()!;
    return { x: m.a * p.x + m.c * p.y + m.e, y: m.b * p.x + m.d * p.y + m.f };
  }, p);
}

function bed(page: Page, name: string) {
  return page.locator("g.bed", { has: page.locator("text", { hasText: name }) });
}

async function cm(page: Page, name: string): Promise<Cm> {
  const rect = bed(page, name).locator("rect").first();
  return { x: Number(await rect.getAttribute("x")), y: Number(await rect.getAttribute("y")) };
}

/** Where the page is scrolled and how much it is zoomed. */
function viewport(page: Page) {
  return page.evaluate(() => ({ scrollY: Math.round(window.scrollY), scale: window.visualViewport!.scale }));
}

/** Plan and tray both on screen, the page still able to scroll both ways. */
async function openPlan(page: Page) {
  await page.goto("/?example");
  await page.getByRole("button", { name: "Plan", exact: true }).click();
  await expect(page.locator(".counter")).toContainText("8 / 10");
  await page.evaluate(() => {
    const plan = document.querySelector("svg.plan")!.getBoundingClientRect();
    window.scrollBy(0, plan.top - 8);
  });
}

function touch(cdp: CDPSession, type: "touchStart" | "touchMove" | "touchEnd", fingers: Finger[]) {
  return cdp.send("Input.dispatchTouchEvent", { type, touchPoints: fingers });
}

/** Whether the context menu that a long press sends (Windows, Android) would open on
 *  `target`: the browser does not send it for a touch it emulates, so it is sent here. */
function opensMenu(target: Locator): Promise<boolean> {
  return target.evaluate((el) => {
    const e = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
    el.dispatchEvent(e);
    return !e.defaultPrevented;
  });
}

/** The point `i / n` of the way from `a` to `b`. */
function along(a: Cm, b: Cm, i: number, n: number): Cm {
  return { x: a.x + ((b.x - a.x) * i) / n, y: a.y + ((b.y - a.y) * i) / n };
}

for (const s of SCREENS) {
  test.describe(s.name, () => {
    test.use({
      viewport: { width: s.width, height: s.height },
      deviceScaleFactor: 2,
      hasTouch: true,
      isMobile: s.isMobile,
    });

    test.describe("moves", () => {
      test.skip(({ browserName }) => browserName !== "chromium", "Playwright drives touch moves only in Chromium");

      test("TCH-01 after a long press the bed follows each move of the finger and lands under it", async ({ page }) => {
        await openPlan(page);
        const cdp = await page.context().newCDPSession(page);
        expect(await cm(page, "Salad square")).toEqual({ x: 550, y: 390 }); // 100 × 100
        const before = await viewport(page);
        // Held 50 cm in, carried 300 cm right and 110 cm down.
        const grab: Cm = { x: 600, y: 440 };
        const drop: Cm = { x: 910, y: 550 };
        const from = await toScreen(page, grab);
        const to = await toScreen(page, drop);
        await touch(cdp, "touchStart", [from]);
        await page.waitForTimeout(LONG_PRESS_MS);
        await expect(bed(page, "Salad square")).toHaveClass(/armed/);
        const n = 10;
        for (let i = 1; i <= n; i++) {
          await touch(cdp, "touchMove", [along(from, to, i, n)]);
          // The point held stays under the finger, to half a cell (the grid snaps).
          const finger = along(grab, drop, i, n);
          await expect
            .poll(async () => {
              const p = await cm(page, "Salad square");
              return Math.max(Math.abs(p.x + 50 - finger.x), Math.abs(p.y + 50 - finger.y));
            }, { timeout: 500 })
            .toBeLessThanOrEqual(25);
        }
        expect(await viewport(page)).toEqual(before);
        await touch(cdp, "touchEnd", []);
        await expect(page.getByRole("alert")).toHaveCount(0);
        await expect.poll(() => cm(page, "Salad square")).toEqual({ x: 850, y: 500 });
        expect(await viewport(page)).toEqual(before);
      });

      test("TCH-02 an item of the tray, after a long press, is dropped under the finger", async ({ page }) => {
        await openPlan(page);
        const cdp = await page.context().newCDPSession(page);
        const item = page.locator("button.element").filter({ hasText: "Bed" }).first();
        await expect(item).toBeInViewport();
        const box = (await item.boundingBox())!;
        const before = await viewport(page);
        const from = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
        // A 300 × 120 cm bed held by its centre: the finger at (910, 518) puts it on (750, 450).
        const to = await toScreen(page, { x: 910, y: 518 });
        const ghost = page.locator("rect.ghost");
        await touch(cdp, "touchStart", [from]);
        await page.waitForTimeout(LONG_PRESS_MS);
        expect(await opensMenu(item)).toBe(false);
        const n = 10;
        for (let i = 1; i <= n; i++) await touch(cdp, "touchMove", [along(from, to, i, n)]);
        await expect(ghost).toHaveAttribute("x", "750");
        await expect(ghost).toHaveAttribute("y", "450");
        expect(await viewport(page)).toEqual(before);
        await touch(cdp, "touchEnd", []);
        await expect(page.locator(".counter")).toContainText("9 / 10");
        await expect(page.locator('g.bed > rect:first-child[x="750"][y="450"][width="300"]')).toHaveCount(1);
        expect(await viewport(page)).toEqual(before);
      });

      test("TCH-03 a second finger neither stops nor takes the bed, a long press opens no menu", async ({ page }) => {
        await openPlan(page);
        const cdp = await page.context().newCDPSession(page);
        const before = await viewport(page);
        const from = await toScreen(page, { x: 600, y: 440 });
        const to = await toScreen(page, { x: 910, y: 550 });
        // The second finger presses the Leek square (100 × 100 at 400, 390), then moves.
        const leek = await toScreen(page, { x: 450, y: 440 });
        await touch(cdp, "touchStart", [{ ...from, id: 1 }]);
        await page.waitForTimeout(LONG_PRESS_MS);
        expect(await opensMenu(bed(page, "Salad square"))).toBe(false);
        const n = 10;
        for (let i = 1; i <= n / 2; i++) await touch(cdp, "touchMove", [{ ...along(from, to, i, n), id: 1 }]);
        await touch(cdp, "touchStart", [{ ...along(from, to, n / 2, n), id: 1 }, { ...leek, id: 2 }]);
        await page.waitForTimeout(LONG_PRESS_MS);
        for (let i = n / 2 + 1; i <= n; i++) {
          await touch(cdp, "touchMove", [
            { ...along(from, to, i, n), id: 1 },
            { x: leek.x - 4 * i, y: leek.y - 4 * i, id: 2 },
          ]);
        }
        await expect.poll(() => cm(page, "Salad square")).toEqual({ x: 850, y: 500 });
        await touch(cdp, "touchEnd", [{ ...along(from, to, n, n), id: 1 }]);
        await touch(cdp, "touchEnd", []);
        await expect(page.getByRole("alert")).toHaveCount(0);
        await expect.poll(() => cm(page, "Salad square")).toEqual({ x: 850, y: 500 });
        expect(await cm(page, "Leek square")).toEqual({ x: 400, y: 390 });
        expect(await viewport(page)).toEqual(before);
      });

      test("TCH-04 a swipe on the grass, or on a bed without the long press, scrolls the page", async ({ page }) => {
        test.skip(!s.isMobile, "the touch screen window does not scroll");
        await openPlan(page);
        const cdp = await page.context().newCDPSession(page);
        for (const start of [{ x: 900, y: 150 }, { x: 600, y: 440 }]) {
          const before = await viewport(page);
          const from = await toScreen(page, start);
          await touch(cdp, "touchStart", [from]);
          // The plan is scrolled to the bottom of the page: the finger goes down, the page up.
          for (let i = 1; i <= 10; i++) await touch(cdp, "touchMove", [{ x: from.x, y: from.y + 15 * i }]);
          await touch(cdp, "touchEnd", []);
          await expect.poll(async () => (await viewport(page)).scrollY).toBeLessThan(before.scrollY - 50);
          expect(await cm(page, "Salad square")).toEqual({ x: 550, y: 390 });
          await expect(page.locator(".counter")).toContainText("8 / 10");
          await page.evaluate(() => {
            const plan = document.querySelector("svg.plan")!.getBoundingClientRect();
            window.scrollBy(0, plan.top - 8);
          });
        }
      });
    });

    test("TCH-05 a short tap selects a bed, a tap on the grass deselects it", async ({ page }) => {
      await openPlan(page);
      const salad = bed(page, "Salad square");
      await expect(salad).toHaveAttribute("aria-pressed", "false");
      const p = await toScreen(page, { x: 600, y: 440 });
      await page.touchscreen.tap(p.x, p.y);
      await expect(salad).toHaveAttribute("aria-pressed", "true");
      expect(await cm(page, "Salad square")).toEqual({ x: 550, y: 390 });
      const grass = await toScreen(page, { x: 900, y: 150 });
      await page.touchscreen.tap(grass.x, grass.y);
      await expect(salad).toHaveAttribute("aria-pressed", "false");
      await expect(page.locator(".counter")).toContainText("8 / 10");
    });
  });
}
