// Gaps 4 and 5 of docs/POT-00_test-matrix.md ("Coverage of the requirements"):
// the bed panel (plantings, History, "Plant here"), the plan summary panel, the
// "in season now" filter of the catalogue and the pause / speed buttons of the
// season playback. Ids: PAN-xx. Each test runs in English and in French.
// `?date=2027-06-20` pins "today" so the fake's example data is the same every day.
import { readFileSync } from "node:fs";
import { expect, test, type Page } from "@playwright/test";

type Crop = { id: string; family: string; planting_out: { start: string; end: string } };
const CROPS: Crop[] = JSON.parse(readFileSync(new URL("../../crates/catalogue/crops.json", import.meta.url), "utf8"));

/** Rust's `plan::in_window`: the window that started last year or this year contains the date. */
function inSeason(date: string): string[] {
  const [y, mo, d] = date.split("-").map(Number) as [number, number, number];
  const at = Date.UTC(y, mo - 1, d);
  return CROPS.filter((c) => {
    const [ms, ds] = c.planting_out.start.split("-").map(Number) as [number, number];
    const [me, de] = c.planting_out.end.split("-").map(Number) as [number, number];
    return [y - 1, y].some((a) => Date.UTC(a, ms - 1, ds) <= at && at <= Date.UTC(a + (ms > me ? 1 : 0), me - 1, de));
  }).map((c) => c.id);
}

const LANGS = {
  en: {
    locale: "en-US",
    plan: "Plan",
    garden: "Garden",
    summary: "Plan summary",
    selected: "Selected bed",
    deselect: "Deselect",
    history: "History · read only",
    historyEmpty: "No planting from past seasons.",
    noPlanting: "No planting this season.",
    plantHere: "Plant here",
    newPlanting: "New planting",
    play: "Play the season",
    pause: "Pause the season",
    speed: "Playback speed",
    cursor: "Garden date",
    roots: "Roots",
    mound: "End of the aisle",
    carrot: "Carrot",
    lettuce: "Lettuce",
    leek: "Leek",
    segments: /Segments 1–3 · ≈ \d+ plants \(\d+ rows × \d+\)/,
    none: "No crop matches.",
    count: (n: number) => (n === 1 ? "1 crop." : `${n} crops.`),
    toggle: /^In season now/,
    family: "Asteraceae",
  },
  fr: {
    locale: "fr-FR",
    plan: "Plan",
    garden: "Potager",
    summary: "Bilan du plan",
    selected: "Planche sélectionnée",
    deselect: "Désélectionner",
    history: "Historique · lecture seule",
    historyEmpty: "Aucune série des saisons passées.",
    noPlanting: "Aucune série cette saison.",
    plantHere: "Planter ici",
    newPlanting: "Nouvelle série",
    play: "Jouer la saison",
    pause: "Mettre la saison en pause",
    speed: "Vitesse de lecture",
    cursor: "Date du potager",
    roots: "Racines",
    mound: "Bout de l’allée",
    carrot: "Carotte",
    lettuce: "Laitue",
    leek: "Poireau",
    segments: /Tronçons 1–3 · ≈ \d+ pieds \(\d+ rangs × \d+\)/,
    none: "Aucune culture ne correspond.",
    count: (n: number) => (n === 1 ? "1 culture." : `${n} cultures.`),
    toggle: /^De saison maintenant/,
    family: "Astéracées",
  },
};

async function open(page: Page, l: (typeof LANGS)["en"], query = "date=2027-06-20") {
  await page.goto(`/?example&${query}`);
  await expect(page.getByRole("button", { name: l.plan, exact: true })).toBeVisible();
}

for (const [code, l] of Object.entries(LANGS)) {
  test.describe(`panels and catalogue (${code})`, () => {
    test.use({ locale: l.locale });

    async function selectBed(page: Page, name: string) {
      await page.getByRole("button", { name: l.plan, exact: true }).click();
      await page.getByRole("button", { name: new RegExp(`^${name}, `) }).click();
      const panel = page.getByRole("complementary", { name: l.selected });
      await expect(panel).toBeVisible();
      return panel;
    }

    test(`PAN-01 F-PLN-5 (${code}): the summary shows surfaces and bed count, follows an added bed and gives way to the bed panel`, async ({ page }) => {
      await open(page, l);
      await page.getByRole("button", { name: l.plan, exact: true }).click();
      const summary = page.locator("aside.summary");
      await expect(summary.getByRole("heading", { name: l.summary })).toBeVisible();
      // 11 × 6.5 m plot = 71.5 m², 8 beds = 24 m², the rest is aisles.
      const fmt = (n: number) =>
        new RegExp(`^${n.toLocaleString(l.locale, { maximumFractionDigits: 1 }).replace(/\s/g, "\\s")}\\s?m²$`);
      const values = summary.locator("dd");
      await expect(values.nth(0)).toHaveText(fmt(71.5));
      await expect(values.nth(1)).toHaveText(fmt(24));
      await expect(values.nth(2)).toHaveText(fmt(47.5));
      await expect(values.nth(3)).toHaveText("8 / 10");
      // A square of 1 m² is added: 25 m² workable, 46.5 m² of aisles, 9 beds.
      await page.locator(".element").nth(1).focus();
      await page.keyboard.press("Enter");
      await expect(page.getByRole("complementary", { name: l.selected })).toBeVisible();
      await expect(summary).toHaveCount(0);
      await page.getByRole("button", { name: l.deselect }).click();
      await expect(values.nth(3)).toHaveText("9 / 10");
      await expect(values.nth(1)).toHaveText(fmt(25));
      await expect(values.nth(2)).toHaveText(fmt(46.5));
    });

    test(`PAN-02 F-VIS-7 (${code}): the bed panel lists the plantings of the year, with segments and plant count`, async ({ page }) => {
      await open(page, l);
      const panel = await selectBed(page, l.roots);
      const cards = panel.locator(".planting");
      await expect(cards).toHaveCount(3); // carrot, lettuce, leek of 2027; the 2026 carrot is History
      await expect(cards.filter({ hasText: l.carrot })).toHaveCount(1);
      await expect(cards.filter({ hasText: l.lettuce })).toHaveCount(1);
      await expect(cards.filter({ hasText: l.leek })).toHaveCount(1);
      await expect(cards.filter({ hasText: l.carrot })).toContainText(l.segments);
      await expect(cards.filter({ hasText: l.leek })).toContainText(/4–6/);
    });

    test(`PAN-03 F-VIS-7 (${code}): a bed without planting says so; History is folded, opens, and says it is empty`, async ({ page }) => {
      await open(page, l);
      const panel = await selectBed(page, l.mound);
      await expect(panel.getByText(l.noPlanting)).toBeVisible();
      const toggle = panel.getByRole("button", { name: l.history });
      await expect(toggle).toHaveAttribute("aria-expanded", "false");
      await expect(panel.getByText(l.historyEmpty)).toHaveCount(0);
      await toggle.click();
      await expect(toggle).toHaveAttribute("aria-expanded", "true");
      await expect(panel.getByText(l.historyEmpty)).toBeVisible();
      await toggle.click();
      await expect(panel.getByText(l.historyEmpty)).toHaveCount(0);
    });

    test(`PAN-04 F-VIS-7 (${code}): History lists the planting of the past season, read only`, async ({ page }) => {
      await open(page, l);
      const panel = await selectBed(page, l.roots);
      const before = await panel.locator(".planting").count();
      await panel.getByRole("button", { name: l.history }).click();
      const cards = panel.locator(".planting");
      await expect(cards).toHaveCount(before + 1);
      const past = cards.last();
      await expect(past).toContainText(l.carrot);
      await expect(past).toContainText(/2026/);
      await expect(past).toContainText(/≈ \d+/);
      await expect(past.getByRole("button")).toHaveCount(0);
    });

    test(`PAN-05 F-PLA-1 (${code}): "Plant here" opens the new planting form on this bed`, async ({ page }) => {
      await open(page, l);
      const panel = await selectBed(page, l.mound);
      await panel.getByRole("button", { name: l.plantHere }).click();
      const dialog = page.getByRole("dialog");
      await expect(dialog.getByRole("heading", { name: l.newPlanting })).toBeVisible();
      await expect(dialog).toContainText(l.mound);
    });

    test(`PAN-06 F-VIS-7 (${code}): the same panel opens from the garden, and its History opens too`, async ({ page }) => {
      await open(page, l);
      await page.getByRole("button", { name: l.garden, exact: true }).click();
      await expect(page.getByRole("button", { name: l.play })).toBeVisible();
      await page.getByRole("button", { name: new RegExp(`^${l.roots}, `) }).first().click();
      const panel = page.getByRole("complementary", { name: l.selected });
      await expect(panel).toBeVisible();
      await expect(panel.locator(".planting")).toHaveCount(3);
      await panel.getByRole("button", { name: l.history }).click();
      await expect(panel.locator(".planting")).toHaveCount(4);
    });

    async function openCatalogue(page: Page) {
      await page.getByRole("button", { name: l.garden, exact: true }).click();
      await page.getByRole("button", { name: /Catalogue/ }).first().click();
      await expect(page.locator("button.carte").first()).toBeVisible();
    }

    test(`PAN-07 F-CAT-3 (${code}): the "in season now" filter keeps exactly the crops the window allows`, async ({ page }) => {
      await open(page, l);
      await openCatalogue(page);
      const cards = page.locator("button.carte");
      const total = CROPS.length;
      await expect(cards).toHaveCount(total);
      const expected = inSeason("2027-06-20");
      expect(expected.length).toBeGreaterThan(0);
      expect(expected.length).toBeLessThan(total);
      const toggle = page.getByRole("switch", { name: l.toggle });
      await expect(toggle).not.toBeChecked();
      // The badge is shown on the cards of the season even before filtering.
      await expect(page.locator("button.carte .badge")).toHaveCount(expected.length);
      await page.locator("label.bascule").click();
      await expect(toggle).toBeChecked();
      await expect(cards).toHaveCount(expected.length);
      await expect(page.getByText(l.count(expected.length))).toBeVisible();
      await expect(page.locator("button.carte .badge")).toHaveCount(expected.length);
      await page.locator("label.bascule").click();
      await expect(toggle).not.toBeChecked();
      await expect(cards).toHaveCount(total);
    });

    test(`PAN-08 F-CAT-3 (${code}): the season filter combines with the family filter; another date gives another list`, async ({ page }) => {
      await open(page, l);
      await openCatalogue(page);
      await page.locator("label.bascule").click();
      await expect(page.getByRole("switch", { name: l.toggle })).toBeChecked();
      const cards = page.locator("button.carte");
      const seasonal = await cards.count();
      await page.getByRole("button", { name: l.family, exact: true }).click();
      const wanted = CROPS.filter((c) => c.family === "Asteraceae" && inSeason("2027-06-20").includes(c.id)).length;
      expect(wanted).toBeGreaterThan(0);
      expect(wanted).toBeLessThan(seasonal);
      await expect(cards).toHaveCount(wanted);
      // Mid-winter: the list is the one of that date, empty or not.
      await open(page, l, "date=2027-01-10");
      await openCatalogue(page);
      await page.locator("label.bascule").click();
      await expect(page.getByRole("switch", { name: l.toggle })).toBeChecked();
      const winter = inSeason("2027-01-10");
      if (winter.length === 0) await expect(page.getByText(l.none)).toBeVisible();
      else await expect(cards).toHaveCount(winter.length);
      expect(winter.length).not.toBe(seasonal);
    });

    test(`PAN-09 F-VIS-4 (${code}): play, pause and resume move the garden date only while playing`, async ({ page }) => {
      await open(page, l);
      await page.getByRole("button", { name: l.garden, exact: true }).click();
      const slider = page.getByRole("slider", { name: l.cursor });
      const start = Number(await slider.inputValue());
      await page.getByRole("button", { name: l.play }).click();
      const pause = page.getByRole("button", { name: l.pause });
      await expect(pause).toHaveAttribute("aria-pressed", "true");
      await expect.poll(async () => Number(await slider.inputValue())).toBeGreaterThan(start);
      await pause.click();
      const play = page.getByRole("button", { name: l.play });
      await expect(play).toHaveAttribute("aria-pressed", "false");
      const frozen = Number(await slider.inputValue());
      await page.waitForTimeout(600);
      expect(Number(await slider.inputValue())).toBe(frozen);
      await play.click();
      await expect.poll(async () => Number(await slider.inputValue())).toBeGreaterThan(frozen);
    });

    test(`PAN-10 F-VIS-4 (${code}): ×1 is the default, ×2 is exclusive and plays about twice as fast`, async ({ page }) => {
      await open(page, l);
      await page.getByRole("button", { name: l.garden, exact: true }).click();
      const group = page.getByRole("group", { name: l.speed });
      const x1 = group.getByRole("button", { name: "×1" });
      const x2 = group.getByRole("button", { name: "×2" });
      await expect(x1).toHaveAttribute("aria-pressed", "true");
      await expect(x2).toHaveAttribute("aria-pressed", "false");
      const slider = page.getByRole("slider", { name: l.cursor });

      /** Days travelled during one second of playback at the chosen speed. */
      async function travelled(): Promise<number> {
        const a = Number(await slider.inputValue());
        await page.getByRole("button", { name: l.play }).click();
        await page.waitForTimeout(1000);
        await page.getByRole("button", { name: l.pause }).click();
        return Number(await slider.inputValue()) - a;
      }
      const slow = await travelled();
      await x2.click();
      await expect(x2).toHaveAttribute("aria-pressed", "true");
      await expect(x1).toHaveAttribute("aria-pressed", "false");
      const fast = await travelled();
      expect(slow).toBeGreaterThan(3);
      expect(fast).toBeGreaterThan(slow * 1.4);
      await x1.click();
      await expect(x1).toHaveAttribute("aria-pressed", "true");
    });
  });
}
