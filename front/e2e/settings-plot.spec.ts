// Tests of the Settings screen (import with confirmation and "to resolve" list, clear the plan,
// erase the data, language, aisle) and of the plot creation options (cell size, aisle, warning).
// Gaps 1 and 2 of docs/POT-00_test-matrix.md. Ids: SET-xx. Texts come from the message files,
// so the same tests run in English and in French.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test, type Page } from "@playwright/test";

type Messages = Record<string, string>;
const MESSAGES: Record<string, Messages> = {
  en: JSON.parse(readFileSync(join(process.cwd(), "messages", "en.json"), "utf8")),
  fr: JSON.parse(readFileSync(join(process.cwd(), "messages", "fr.json"), "utf8")),
};

const VARIANTS = [
  { locale: "en-US", lang: "en" },
  { locale: "fr-FR", lang: "fr" },
] as const;

for (const v of VARIANTS) {
  const M = MESSAGES[v.lang]!;
  /** A message of the interface, its {placeholders} replaced. */
  const t = (key: string, params: Record<string, string | number> = {}) => {
    const text = M[key];
    if (text === undefined) throw new Error(`unknown message ${key}`);
    return text.replace(/\{(\w+)\}/g, (_, name: string) => String(params[name]));
  };

  test.describe(`settings and plot creation (${v.locale})`, () => {
    test.use({ locale: v.locale });

    async function openSettings(page: Page, query = "example") {
      await page.goto(`/?${query}`);
      await expect(page.locator(".counter").first()).toBeVisible().catch(() => {});
      await page.getByRole("button", { name: t("settings"), exact: true }).first().click();
      await expect(page.getByRole("dialog", { name: t("settings") })).toBeVisible();
      return page.getByRole("dialog", { name: t("settings") });
    }

    test(`SET-01 the aisle: 30 cm is kept, 80 cm is refused with the reason and the aisle stays (${v.locale})`, async ({ page }) => {
      const panel = await openSettings(page);
      const group = panel.getByRole("group", { name: t("settings_aisle") });
      await expect(group.getByRole("button", { name: t("cm", { value: 50 }), exact: true })).toHaveAttribute("aria-pressed", "true");
      await group.getByRole("button", { name: t("cm", { value: 30 }), exact: true }).click();
      await expect(group.getByRole("button", { name: t("cm", { value: 30 }), exact: true })).toHaveAttribute("aria-pressed", "true");
      await expect(group.getByRole("button", { name: t("cm", { value: 50 }), exact: true })).toHaveAttribute("aria-pressed", "false");
      // The example has beds 50 cm apart (S03 §15): 80 cm would break them, Rust refuses the change.
      await group.getByRole("button", { name: t("cm", { value: 50 }), exact: true }).click();
      await group.getByRole("button", { name: t("cm", { value: 80 }), exact: true }).click();
      await expect(page.getByRole("alert").filter({ hasText: t("error_aisle_too_narrow") })).toBeVisible();
      await expect(group.getByRole("button", { name: t("cm", { value: 50 }), exact: true })).toHaveAttribute("aria-pressed", "true");
      await expect(group.getByRole("button", { name: t("cm", { value: 80 }), exact: true })).toHaveAttribute("aria-pressed", "false");
    });

    test(`SET-02 import of an older file: a confirmation with the summary, Cancel changes nothing, Replace imports (${v.locale})`, async ({ page }) => {
      const panel = await openSettings(page, "example&import=old");
      await panel.getByRole("button", { name: t("import"), exact: true }).click();
      const card = panel.getByRole("group", { name: t("import_older") });
      await expect(card).toBeVisible();
      await expect(card).toContainText(/\b8\b/);
      await expect(card).toContainText(/\b12\b/);
      await expect(card).toContainText(t("import_replaces_all"));
      await expect(card).toContainText(/2026/);
      // The file is older: the local garden's date is shown and the button is the cautious one.
      await expect(card.getByRole("button", { name: t("replace_anyway") })).toBeVisible();
      await card.getByRole("button", { name: t("cancel") }).click();
      await expect(card).toHaveCount(0);
      await expect(page.getByRole("alert").filter({ hasText: t("import_ok") })).toHaveCount(0);
      await panel.getByRole("button", { name: t("import"), exact: true }).click();
      await panel.getByRole("button", { name: t("replace_anyway") }).click();
      await expect(page.getByRole("alert").filter({ hasText: t("import_ok") })).toBeVisible();
      await expect(card).toHaveCount(0);
    });

    test(`SET-03 import of a recent, coherent file: a plain confirmation, no issue list (${v.locale})`, async ({ page }) => {
      const panel = await openSettings(page, "example&import=new");
      await panel.getByRole("button", { name: t("import"), exact: true }).click();
      const card = panel.getByRole("group", { name: t("import_confirm") });
      await expect(card).toBeVisible();
      await expect(card.getByRole("button", { name: t("replace"), exact: true })).toBeVisible();
      await expect(card.getByRole("button", { name: t("replace_anyway") })).toHaveCount(0);
      await expect(card.locator("ul")).toHaveCount(0);
      await card.getByRole("button", { name: t("replace"), exact: true }).click();
      await expect(page.getByRole("alert").filter({ hasText: t("import_ok") })).toBeVisible();
    });

    test(`SET-04 import with issues: the list is shown before replacing, then the "to resolve" band keeps it (${v.locale})`, async ({ page }) => {
      const panel = await openSettings(page, "example&import=incoherent");
      await panel.getByRole("button", { name: t("import"), exact: true }).click();
      const card = panel.getByRole("group", { name: t("import_confirm") });
      await expect(card).toBeVisible();
      await expect(card).toContainText(t("import_incoherences_title", { count: 4 }));
      await expect(card.locator("li")).toHaveCount(4);
      await expect(card).toContainText(t("import_incoherences_text"));
      await card.getByRole("button", { name: t("replace_anyway") }).click();
      await expect(page.getByRole("alert").filter({ hasText: t("import_ok") })).toBeVisible();
      await page.getByRole("button", { name: t("close"), exact: true }).first().click();
      await expect(panel).toHaveCount(0);
      const band = page.getByRole("region", { name: t("resolve_title", { count: 4 }) });
      await expect(band).toBeVisible();
      await expect(band.locator("li")).toHaveCount(0);
      await band.getByRole("button", { name: t("resolve_show") }).click();
      await expect(band.locator("li")).toHaveCount(4);
      await expect(band).toContainText(t("resolve_help"));
      await band.getByRole("button", { name: t("resolve_hide") }).click();
      await expect(band.locator("li")).toHaveCount(0);
    });

    for (const [name, title, text] of [
      ["unreadable", "state_import_impossible", "state_import_unreadable"],
      ["recent", "state_import_impossible", "state_import_too_recent"],
      ["file", "state_file_title", "state_file_text"],
    ] as const) {
      test(`SET-05 import refused (${name}): the reason stays in the panel and the garden is untouched (${v.locale})`, async ({ page }) => {
        const panel = await openSettings(page, `example&import=${name}`);
        await panel.getByRole("button", { name: t("import"), exact: true }).click();
        const alert = panel.getByRole("alert");
        await expect(alert).toContainText(t(title));
        await expect(alert).toContainText(t(text));
        await expect(panel.getByRole("group", { name: t("import_confirm") })).toHaveCount(0);
        await expect(panel.getByRole("button", { name: t("replace_anyway") })).toHaveCount(0);
        // A new choice of file clears the old failure.
        await panel.getByRole("button", { name: t("import"), exact: true }).click();
        await expect(alert).toHaveCount(1);
      });
    }

    test(`SET-06 clearing the plan: Cancel keeps the beds, confirming empties them (${v.locale})`, async ({ page }) => {
      const panel = await openSettings(page);
      await panel.getByRole("button", { name: t("settings_clear"), exact: true }).click();
      const year = new Date().getFullYear();
      await expect(page.getByText(t("settings_clear_confirm", { year }))).toBeVisible();
      await expect(page.getByText(/8/).filter({ hasText: String(year) }).first()).toBeVisible();
      await expect(page.getByText(t("settings_clear_kept"))).toBeVisible();
      await page.getByRole("button", { name: t("cancel") }).last().click();
      await expect(page.getByText(t("settings_clear_confirm", { year }))).toHaveCount(0);
      await expect(page.getByRole("alert").filter({ hasText: t("settings_clear_ok") })).toHaveCount(0);
      await panel.getByRole("button", { name: t("settings_clear"), exact: true }).click();
      await page.getByRole("button", { name: t("settings_clear_button") }).click();
      await expect(page.getByRole("alert").filter({ hasText: t("settings_clear_ok") })).toBeVisible();
      // Nothing left to clear: the button is disabled and says why.
      await expect(panel.getByRole("button", { name: t("settings_clear"), exact: true })).toBeDisabled();
      await expect(panel.getByText(t("settings_clear_nothing"))).toBeVisible();
      await page.getByRole("button", { name: t("close"), exact: true }).first().click();
      await page.getByRole("button", { name: "Plan", exact: true }).first().click();
      await expect(page.locator(".counter").first()).toContainText("0 / 10");
    });

    test(`SET-07 erasing the data asks twice; each step can be cancelled; the second confirms (${v.locale})`, async ({ page }) => {
      const panel = await openSettings(page);
      const erase = panel.getByRole("button", { name: t("erase"), exact: true });
      // Step 1, cancelled.
      await erase.click();
      await expect(page.getByText(t("erase_confirm_1"))).toBeVisible();
      await expect(page.getByText(t("erase_confirm_1_text"))).toBeVisible();
      await page.getByRole("button", { name: t("cancel") }).last().click();
      await expect(page.getByText(t("erase_confirm_1"))).toHaveCount(0);
      // Step 2, cancelled: nothing is erased, and the first question comes back from the start.
      await erase.click();
      await page.getByRole("button", { name: t("continue_button") }).click();
      await expect(page.getByText(t("erase_confirm_1"))).toHaveCount(0);
      await expect(page.getByText(t("erase_confirm_2"))).toBeVisible();
      await expect(page.getByText(t("erase_confirm_2_text"))).toBeVisible();
      await page.getByRole("button", { name: t("cancel") }).last().click();
      await expect(page.getByText(t("erase_confirm_2"))).toHaveCount(0);
      await expect(page.getByRole("alert").filter({ hasText: t("erase_ok") })).toHaveCount(0);
      await expect(panel.getByRole("group", { name: t("settings_aisle") })).toBeVisible();
      // Both confirmations: the app goes back to "no plot yet".
      await erase.click();
      await page.getByRole("button", { name: t("continue_button") }).click();
      await page.getByRole("button", { name: t("erase_permanently") }).click();
      await expect(page.getByRole("alert").filter({ hasText: t("erase_ok") })).toBeVisible();
      await expect(page.getByText(t("plot_absent"))).toBeVisible();
    });

    test(`SET-08 the language: choosing the other one translates the screen and marks it pressed (${v.locale})`, async ({ page }) => {
      const panel = await openSettings(page);
      const other = v.lang === "en" ? "fr" : "en";
      const names = { en: "English", fr: "Français" };
      const group = panel.getByRole("group", { name: t("settings_language") });
      await expect(group.getByRole("button", { name: names[v.lang] })).toHaveAttribute("aria-pressed", "true");
      await expect(group.getByRole("button", { name: names[other] })).toHaveAttribute("aria-pressed", "false");
      await group.getByRole("button", { name: names[other] }).click();
      const O = MESSAGES[other]!;
      // The page reloads in the new language (the fake garden restarts, the panel is closed).
      await expect(page.locator("html")).toHaveAttribute("lang", other);
      await page.getByRole("button", { name: O.settings!, exact: true }).first().click();
      await expect(page.getByRole("heading", { name: O.settings!, exact: true })).toBeVisible();
      const again = page.getByRole("group", { name: O.settings_language! });
      await expect(again.getByRole("button", { name: names[other] })).toHaveAttribute("aria-pressed", "true");
      // And back.
      await again.getByRole("button", { name: names[v.lang] }).click();
      await expect(page.locator("html")).toHaveAttribute("lang", v.lang);
      await page.getByRole("button", { name: M.settings!, exact: true }).first().click();
      await expect(page.getByRole("heading", { name: M.settings!, exact: true })).toBeVisible();
    });

    // ---- Plot creation (F-TER-1) ----

    async function openForm(page: Page) {
      await page.goto("/");
      await expect(page.getByText(t("plot_absent"))).toBeVisible();
      await page.getByRole("button", { name: t("plot_create") }).click();
      await expect(page.getByRole("heading", { name: t("plot_title") })).toBeVisible();
    }

    test(`SET-09 plot creation: defaults, the warning that the cell cannot change, the aisle help (${v.locale})`, async ({ page }) => {
      await openForm(page);
      const cell = page.getByRole("group", { name: t("plot_case") });
      const aisle = page.getByRole("group", { name: t("plot_aisle") });
      await expect(cell.getByRole("radio")).toHaveCount(3);
      await expect(aisle.getByRole("radio")).toHaveCount(3);
      await expect(cell.getByRole("radio", { name: t("cm", { value: 50 }) })).toBeChecked();
      await expect(aisle.getByRole("radio", { name: t("cm", { value: 50 }) })).toBeChecked();
      await expect(cell).toContainText(t("plot_case_lock"));
      await expect(aisle).toContainText(t("plot_aisle_help"));
    });

    test(`SET-10 plot creation: the cell size changes the preview (columns × rows) (${v.locale})`, async ({ page }) => {
      await openForm(page);
      const cell = page.getByRole("group", { name: t("plot_case") });
      const summary = (n: number, c: number, r: number) => {
        const f = (x: number) => x.toLocaleString(v.locale, { maximumFractionDigits: 2 });
        return t("plot_preview_summary", { length: f(9), width: f(6.5), case: n, columns: c, rows: r });
      };
      await expect(page.getByText(summary(50, 18, 13))).toBeVisible();
      await cell.getByText(t("cm", { value: 25 }), { exact: true }).click();
      await expect(page.getByText(summary(25, 36, 26))).toBeVisible();
      await cell.getByText(t("cm", { value: 100 }), { exact: true }).click();
      await expect(page.getByText(summary(100, 9, 6))).toBeVisible();
    });

    test(`SET-11 plot creation: a side under 1 m is refused with the reason and the button is disabled (${v.locale})`, async ({ page }) => {
      await openForm(page);
      const create = page.getByRole("button", { name: t("plot_validate") });
      await expect(create).toBeEnabled();
      await page.getByLabel(t("plot_length")).fill("0.5");
      await expect(page.getByText(t("plot_dimensions_invalid"))).toBeVisible();
      await expect(create).toBeDisabled();
      await page.getByLabel(t("plot_length")).fill("12");
      await expect(page.getByText(t("plot_dimensions_invalid"))).toHaveCount(0);
      await expect(create).toBeEnabled();
    });

    test(`SET-12 plot creation: 100 cm cells and an 80 cm aisle are kept; the Settings show the aisle (${v.locale})`, async ({ page }) => {
      await openForm(page);
      await page.getByRole("group", { name: t("plot_case") }).getByText(t("cm", { value: 100 }), { exact: true }).click();
      await page.getByRole("group", { name: t("plot_aisle") }).getByText(t("cm", { value: 80 }), { exact: true }).click();
      await page.getByRole("button", { name: t("plot_validate") }).click();
      await expect(page.locator(".counter").first()).toContainText("0 / 10");
      await page.getByRole("button", { name: t("settings"), exact: true }).first().click();
      const aisle = page.getByRole("group", { name: t("settings_aisle") });
      await expect(aisle.getByRole("button", { name: t("cm", { value: 80 }), exact: true })).toHaveAttribute("aria-pressed", "true");
      await expect(aisle.getByRole("button", { name: t("cm", { value: 50 }), exact: true })).toHaveAttribute("aria-pressed", "false");
    });

    test(`SET-13 plot creation: 25 cm cells and a 30 cm aisle are kept, Cancel goes back to the empty state (${v.locale})`, async ({ page }) => {
      await openForm(page);
      await page.getByRole("button", { name: t("cancel") }).click();
      await expect(page.getByText(t("plot_absent"))).toBeVisible();
      await page.getByRole("button", { name: t("plot_create") }).click();
      await page.getByRole("group", { name: t("plot_case") }).getByText(t("cm", { value: 25 }), { exact: true }).click();
      await page.getByRole("group", { name: t("plot_aisle") }).getByText(t("cm", { value: 30 }), { exact: true }).click();
      await page.getByRole("button", { name: t("plot_validate") }).click();
      await expect(page.locator(".counter").first()).toContainText("0 / 10");
      await page.getByRole("button", { name: t("settings"), exact: true }).first().click();
      await expect(
        page.getByRole("group", { name: t("settings_aisle") }).getByRole("button", { name: t("cm", { value: 30 }), exact: true }),
      ).toHaveAttribute("aria-pressed", "true");
    });
  });
}
