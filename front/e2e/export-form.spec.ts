// Planting form (conflict message, the three « How do you start? » cards) and export
// dialog (beds and options), in English and in French, desktop
// (docs/POT-00_test-matrix.md, EXP-xx). The fake of the commands answers; the Rust rules
// are covered by `cargo test` (S03 §2, §5).
import { expect, test, type Locator, type Page } from "@playwright/test";

const LANGS = [
  {
    locale: "en-US",
    tag: "en-GB",
    garden: "Garden",
    tomatoes: "The tomatoes",
    roots: "Roots",
    tomato: "Tomato",
    lettuce: "Lettuce",
    carrot: "Carrot",
    bed: "Bed",
    segment: (n: number) => `Segment ${n}`,
    conflict1: (crop: string, n: number, from: string, to: string) => `Conflict with ${crop} on segment ${n}, from ${from} to ${to}.`,
    conflictN: (crop: string, a: number, b: number, from: string, to: string) => `Conflict with ${crop} on segments ${a}–${b}, from ${from} to ${to}.`,
    indoor: "I sow under cover",
    plants: "I buy plants",
    ground: "I sow in the open ground",
    sowIndoor: "Sowing under cover",
    transplanting: "Transplanting",
    planting: "Planting",
    sowing: "Sowing",
    harvest: "First harvest",
    qSow: "When do you sow?",
    qPlant: "When do you plant?",
    pots: "How many days in pots?",
  },
  {
    locale: "fr-FR",
    tag: "fr-FR",
    garden: "Potager",
    tomatoes: "Les tomates",
    roots: "Racines",
    tomato: "Tomate",
    lettuce: "Laitue",
    carrot: "Carotte",
    bed: "Planche",
    segment: (n: number) => `Tronçon ${n}`,
    conflict1: (crop: string, n: number, from: string, to: string) => `Conflit avec ${crop} sur le tronçon ${n}, du ${from} au ${to}.`,
    conflictN: (crop: string, a: number, b: number, from: string, to: string) => `Conflit avec ${crop} sur les tronçons ${a}–${b}, du ${from} au ${to}.`,
    indoor: "Je sème à l’abri",
    plants: "J’achète des plants",
    ground: "Je sème en pleine terre",
    sowIndoor: "Semis à l’abri",
    transplanting: "Repiquage",
    planting: "Plantation",
    sowing: "Semis",
    harvest: "1re récolte",
    qSow: "Quand sèmes-tu ?",
    qPlant: "Quand plantes-tu ?",
    pots: "Combien de jours en godets ?",
  },
];

const MONTHS = {
  en: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
  fr: ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."],
};

/** The text as a regular expression, September being « Sep » or « Sept » according to the engine. */
const loose = (text: string) => new RegExp(`^${text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&").replace(/Sep(?!t)/g, "Sept?")}$`);

const addDays = (iso: string, n: number) => {
  const d = new Date(`${iso}T12:00:00Z`);
  d.setUTCDate(d.getUTCDate() + n);
  return d.toISOString().slice(0, 10);
};

async function openForm(page: Page, garden: string): Promise<Locator> {
  await page.goto("/?example");
  await page.getByRole("button", { name: garden, exact: true }).click();
  const slot = page.locator("[data-slot]").first();
  await slot.scrollIntoViewIfNeeded();
  await slot.click({ force: true });
  const dialog = page.getByRole("dialog");
  await expect(dialog.locator("ol.steps")).toBeVisible();
  return dialog;
}

async function setDate(dialog: Locator, iso: string) {
  const field = dialog.locator("label.date input");
  await field.fill(iso);
  await field.dispatchEvent("change");
}

for (const v of LANGS) {
  test.describe(v.locale, () => {
    test.use({ locale: v.locale });
    // Fixed month names: the engines (Node, WebKit, Chromium) do not abbreviate September alike.
    const day = (iso: string) => `${Number(iso.slice(8, 10))} ${MONTHS[v.tag === "fr-FR" ? "fr" : "en"][Number(iso.slice(5, 7)) - 1]}`;
    const dayRe = (iso: string) => new RegExp(`${loose(day(iso)).source.slice(0, -1)}( \\d{4})?$`);
    const crop = (dialog: Locator, name: string) => dialog.locator("button.crop").filter({ has: dialog.page().getByText(name, { exact: true }) });

    test(`EXP-01 the conflict message names the segments (singular, plural) and the last busy day (${v.locale})`, async ({ page }) => {
      const dialog = await openForm(page, v.garden);
      await dialog.getByLabel(v.bed, { exact: true }).selectOption({ label: v.tomatoes });
      const Y = Number((await dialog.locator("label.date input").inputValue()).slice(0, 4));
      // The tomatoes bed (8 segments): tomato 1-4 and lettuce 5-8, from the example.
      await crop(dialog, v.lettuce).click();
      await dialog.locator("label.mode", { hasText: v.plants }).click();
      await setDate(dialog, `${Y}-07-01`);
      // Zone 3-5: two conflicts, one on two segments (the tomato), one on a single segment (the lettuce).
      await dialog.getByRole("button", { name: v.segment(3), exact: true }).click();
      await dialog.getByRole("button", { name: v.segment(5), exact: true }).click();
      const problems = dialog.locator("ul.problems li");
      // Planted lettuce: 1 July + 50 + 20 days = 9 September (exclusive); the tomato leaves on 27 September.
      await expect(problems).toHaveText([
        loose(v.conflictN(v.tomato, 3, 4, day(`${Y}-07-01`), day(`${Y}-09-08`))),
        loose(v.conflict1(v.lettuce, 5, day(`${Y}-07-01`), day(`${Y}-07-18`))),
      ]);
      await expect(dialog.locator("button.validate")).toBeDisabled();
    });

    test(`EXP-02 the period ends on the last busy day, and a start on the day the other leaves is free (${v.locale})`, async ({ page }) => {
      const dialog = await openForm(page, v.garden);
      await dialog.getByLabel(v.bed, { exact: true }).selectOption({ label: v.tomatoes });
      const Y = Number((await dialog.locator("label.date input").inputValue()).slice(0, 4));
      await crop(dialog, v.lettuce).click();
      await dialog.locator("label.mode", { hasText: v.plants }).click();
      await dialog.getByRole("button", { name: v.segment(1), exact: true }).click();
      await dialog.getByRole("button", { name: v.segment(2), exact: true }).click();
      // The tomato occupies [5 May + 56 + 75 + 70 days[ = until 27 September (exclusive).
      await setDate(dialog, `${Y}-09-20`);
      const problems = dialog.locator("ul.problems li");
      await expect(problems).toHaveCount(1);
      await expect(problems.first()).toContainText(day(`${Y}-09-26`));
      await expect(problems.first()).not.toContainText(day(`${Y}-09-27`));
      await expect(problems.first()).toContainText(v.locale === "fr-FR" ? "tronçons 1–2" : "segments 1–2");
      // On the day the tomato leaves, the zone is free: no message, the button works.
      await setDate(dialog, `${Y}-09-27`);
      await expect(dialog.locator("ul.problems")).toHaveCount(0);
      await expect(dialog.locator("button.validate")).toBeEnabled();
    });

    test(`EXP-03 the three cards give three sets of dates (${v.locale})`, async ({ page }) => {
      const dialog = await openForm(page, v.garden);
      await crop(dialog, v.tomato).click();
      const start = "2027-03-01";
      await setDate(dialog, start);
      const steps = dialog.locator("ol.steps li");
      const step = (label: string) => dialog.locator("ol.steps li").filter({ hasText: label }).locator("strong");
      const last = dialog.locator(".end input");
      // Tomato: 56 days in pots, 75 days to the first harvest, 70 days of harvest.
      await dialog.locator("label.mode", { hasText: v.indoor }).click();
      await expect(dialog.getByRole("heading", { name: v.qSow })).toBeVisible();
      await expect(steps).toHaveCount(4);
      await expect(step(v.sowIndoor)).toHaveText(dayRe(start));
      await expect(step(v.transplanting)).toHaveText(dayRe(addDays(start, 56)));
      await expect(step(v.harvest)).toHaveText(dayRe(addDays(start, 131)));
      await expect(last).toHaveValue(addDays(start, 200));
      // Bought plants: no sowing, no pots; the date becomes the planting date.
      await dialog.locator("label.mode", { hasText: v.plants }).click();
      await expect(dialog.getByRole("heading", { name: v.qPlant })).toBeVisible();
      await expect(steps).toHaveCount(3);
      await expect(step(v.planting)).toHaveText(dayRe(start));
      await expect(step(v.harvest)).toHaveText(dayRe(addDays(start, 75)));
      await expect(last).toHaveValue(addDays(start, 144));
      // Open ground: sown on the date, the pot time is spent in the ground before the harvest.
      await dialog.locator("label.mode", { hasText: v.ground }).click();
      await expect(dialog.getByRole("heading", { name: v.qSow })).toBeVisible();
      await expect(steps).toHaveCount(3);
      await expect(step(v.sowing)).toHaveText(dayRe(start));
      await expect(step(v.harvest)).toHaveText(dayRe(addDays(start, 131)));
      await expect(last).toHaveValue(addDays(start, 200));
      // The chosen card is the only one checked.
      await expect(dialog.locator("label.mode.active")).toHaveCount(1);
      await expect(dialog.locator("label.mode.active")).toContainText(v.ground);
    });

    test(`EXP-04 a crop without pots asks for the days in pots when started under cover (${v.locale})`, async ({ page }) => {
      const dialog = await openForm(page, v.garden);
      await crop(dialog, v.carrot).click();
      await setDate(dialog, "2027-04-01");
      await dialog.locator("label.mode", { hasText: v.indoor }).click();
      const pots = dialog.locator("label.field input[type=number]");
      await expect(dialog.getByText(v.pots)).toBeVisible();
      await pots.fill("10");
      await pots.dispatchEvent("change");
      await expect(dialog.locator("ol.steps li").filter({ hasText: v.transplanting }).locator("strong")).toHaveText(dayRe("2027-04-11"));
      await pots.fill("20");
      await pots.dispatchEvent("change");
      await expect(dialog.locator("ol.steps li").filter({ hasText: v.transplanting }).locator("strong")).toHaveText(dayRe("2027-04-21"));
      // Back to the ground: the pot field is gone, sowing and harvest only.
      await dialog.locator("label.mode", { hasText: v.ground }).click();
      await expect(dialog.getByText(v.pots)).toHaveCount(0);
      await expect(dialog.locator("ol.steps li").filter({ hasText: v.transplanting })).toHaveCount(0);
      // The days typed for the pots do not leak into the ground start: carrot, 100 days to the harvest.
      await expect(dialog.locator("ol.steps li").filter({ hasText: v.harvest }).locator("strong")).toHaveText(dayRe("2027-07-10"));
    });
  });
}

// ---------- Export dialog ----------

const EXPORT = [
  { locale: "en-US", openName: /Export the timeline/, none: "Deselect all", all: "Select all", choose: "Choose at least one bed.", pdf: "Export as PDF", phases: "Phases (under cover, growth, harvest)", marker: "\"Today\" marker", notes: "Show the annotations", garden: "Garden", holidays: "Holidays", mulching: "Mulching" },
  { locale: "fr-FR", openName: /Exporter la frise/, none: "Tout désélectionner", all: "Tout sélectionner", choose: "Choisis au moins une planche.", pdf: "Exporter en PDF", phases: "Phases (abri, croissance, récolte)", marker: "Repère « aujourd’hui »", notes: "Afficher les annotations", garden: "Potager", holidays: "Holidays", mulching: "Paillage" },
];

async function openExport(page: Page, v: (typeof EXPORT)[number]): Promise<Locator> {
  await page.goto("/?example");
  await page.getByRole("button", { name: v.garden, exact: true }).click();
  await page.getByRole("button", { name: v.openName }).click();
  const dialog = page.getByRole("dialog", { name: v.openName });
  await expect(dialog).toBeVisible();
  // The preview is the very SVG that would be exported.
  await expect(dialog.locator(".feuille img")).toBeVisible();
  return dialog;
}

/** The text of the SVG the preview shows. */
async function previewSvg(dialog: Locator): Promise<string> {
  const src = await dialog.locator(".feuille img").getAttribute("src");
  return decodeURIComponent((src ?? "").replace(/^data:image\/svg\+xml;charset=utf-8,/, ""));
}

for (const v of EXPORT) {
  test.describe(`export ${v.locale}`, () => {
    test.use({ locale: v.locale });

    test(`EXP-05 beds: deselect all, pick one, select all (${v.locale})`, async ({ page }) => {
      const dialog = await openExport(page, v);
      const boxes = dialog.locator(".puces input[type=checkbox]");
      const total = await boxes.count();
      expect(total).toBeGreaterThanOrEqual(8);
      for (let i = 0; i < total; i++) await expect(boxes.nth(i)).toBeChecked();
      const labelsAll = (await previewSvg(dialog)).match(/clip-path="url\(#timeline-bar-/g)?.length ?? 0;
      expect(labelsAll).toBeGreaterThan(0);

      await dialog.getByRole("button", { name: v.none, exact: true }).click();
      for (let i = 0; i < total; i++) await expect(boxes.nth(i)).not.toBeChecked();
      await expect(dialog.getByText(v.choose)).toBeVisible();
      await expect(dialog.getByRole("button", { name: v.pdf })).toBeDisabled();
      await expect(dialog.locator(".feuille img")).toHaveCount(0);
      // The link now offers to select everything.
      await expect(dialog.getByRole("button", { name: v.all, exact: true })).toBeVisible();

      // One bed: the export is possible and the drawing holds only that bed's bars.
      await dialog.locator(".puce").filter({ hasText: /Roots|Racines/ }).locator("input").check();
      await expect(dialog.getByRole("button", { name: v.pdf })).toBeEnabled();
      await expect(dialog.locator(".feuille img")).toBeVisible();
      const one = await previewSvg(dialog);
      const oneBars = one.match(/clip-path="url\(#timeline-bar-/g)?.length ?? 0;
      expect(oneBars).toBeGreaterThan(0);
      expect(oneBars).toBeLessThan(labelsAll);
      expect(one).toContain(v.locale === "fr-FR" ? "Racines" : "Roots");
      expect(one).not.toContain(v.locale === "fr-FR" ? "Les tomates" : "The tomatoes");

      await dialog.getByRole("button", { name: v.all, exact: true }).click();
      for (let i = 0; i < total; i++) await expect(boxes.nth(i)).toBeChecked();
      await expect.poll(async () => (await previewSvg(dialog)).match(/clip-path="url\(#timeline-bar-/g)?.length ?? 0).toBe(labelsAll);
      await expect(dialog.getByRole("button", { name: v.none, exact: true })).toBeVisible();
    });

    test(`EXP-06 options: phases, today marker, annotations change the drawing (${v.locale})`, async ({ page }) => {
      const dialog = await openExport(page, v);
      const phases = dialog.getByLabel(v.phases);
      const marker = dialog.getByLabel(v.marker);
      const notes = dialog.getByLabel(v.notes);
      await expect(phases).toBeChecked();
      await expect(marker).toBeChecked();
      await expect(notes).toBeChecked();

      const accentLine = (svg: string) => (svg.match(/stroke-dasharray="3 3"/g)?.length ?? 0);
      const count = (svg: string, re: RegExp) => svg.match(re)?.length ?? 0;
      const withPhases = (svg: string) => count(svg, /fill="url\(#timeline-(pots|growth)-/g);
      const noteGroups = (svg: string) => [v.holidays, v.mulching].filter((t) => svg.includes(t)).length;

      const base = await previewSvg(dialog);
      expect(withPhases(base)).toBeGreaterThan(0);
      expect(noteGroups(base)).toBeGreaterThan(0);
      const dashes = accentLine(base);
      expect(dashes).toBeGreaterThan(0);

      // Phases off: the bars are plain, the pots and growth patterns are no longer used.
      await phases.uncheck();
      await expect.poll(async () => withPhases(await previewSvg(dialog))).toBe(0);
      await phases.check();
      await expect.poll(async () => withPhases(await previewSvg(dialog))).toBeGreaterThan(0);

      // Today marker off: one dashed line fewer.
      await marker.uncheck();
      await expect.poll(async () => accentLine(await previewSvg(dialog))).toBe(dashes - 1);
      await marker.check();
      await expect.poll(async () => accentLine(await previewSvg(dialog))).toBe(dashes);

      // Annotations off: no note, no period; back on, they return.
      await notes.uncheck();
      await expect.poll(async () => noteGroups(await previewSvg(dialog))).toBe(0);
      expect(await previewSvg(dialog)).not.toContain(v.mulching);
      await notes.check();
      await expect.poll(async () => noteGroups(await previewSvg(dialog))).toBe(noteGroups(base));
    });

    test(`EXP-07 the annotations of a deselected bed leave the drawing, those of the whole garden stay (${v.locale})`, async ({ page }) => {
      const dialog = await openExport(page, v);
      await dialog.getByRole("button", { name: v.none, exact: true }).click();
      await dialog.locator(".puce").filter({ hasText: /Roots|Racines/ }).locator("input").check();
      await expect(dialog.locator(".feuille img")).toBeVisible();
      const svg = await previewSvg(dialog);
      // « Holidays » and « Last frosts? » belong to no bed; « Mulching » is the tomatoes bed's.
      expect(svg).toContain(v.holidays);
      expect(svg).not.toContain(v.mulching);
    });
  });
}
