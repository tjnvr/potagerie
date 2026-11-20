# Potagerie — Use-case test matrix

The automated use cases live in `front/e2e/use-cases.spec.ts` (Playwright Test). They run the interface
against the Vite dev server with the in-memory fake of the Tauri commands (`?example`), in **Chromium and WebKit**
(the engine of the macOS app). The Rust rules are covered by `cargo test` (S03 §12 cases, edge cases, random walk).

- Locally: `make e2e`, or `pnpm --dir front e2e` (first time: `pnpm --dir front exec playwright install chromium webkit`).
- In CI: the Ubuntu job of `.github/workflows/ci.yml` runs them after the unit tests; the traces of a failed run are uploaded.
- A new use case = a row below + a `UC-xx` test with the same id (see `skills/spec-driven-delivery`, step 3 and 6).

## Matrix

| Id | Use case | Spec | Expected | Test |
|---|---|---|---|---|
| UC-01 | First launch: no plot, create it | F-TER-1 | "No plot yet" screen, then a plot with 0 / 10 beds | automated |
| UC-02 | Add a bed by clicking free grass | F-PLN-1 | Counter 8 → 9 (the first click only clears the selection) | automated |
| UC-03 | Select a bed on the plan | F-PLN-6 | Handles appear, no refusal message | automated |
| UC-04 | Navigate years: past read only, future editable | F-PLA-4 | "Read only" banner in 2025; counter in 2027 | automated |
| UC-05 | Garden opens on today | F-VIS-3 | Whatever the cursor was left on, the tab shows today | automated |
| UC-06 | Plant a suggestion from a free slot | F-PLA-1 | Slot → form → "Plant · …" adds one bar | automated |
| UC-07 | Notes: refuse empty, 80 characters max | F-VIS-5 | Save disabled when empty, input capped, note appears | automated |
| UC-08 | Catalogue: 67 crops, family filter | F-CAT-1, F-CAT-3 | Count text, filter narrows the list | automated |
| UC-09 | Switch the language, remembered | F-CPT (settings) | French labels, kept after reload | automated |
| UC-10 | Timeline export dialog | F-EXP-1 | Two date inputs (period shown), "Export as PDF" | automated |
| UC-11 | Escape closes every window | accessibility | Settings, catalogue, export, note | automated |
| UC-12 | Phone: no sideways scroll (garden, plan) | layout | `scrollWidth ≤ innerWidth` at 390 px | automated |
| UC-13 | Phone: catalogue two crops a row | layout | Two cards on the first row | automated |
| UC-14 | Edit a planting from its bar | F-PLA-1 | A click on a visible bar opens "Edit the planting" | automated |
| UC-15 | Draw a zone by dragging on free grass | F-PLN-1 | A preview follows the pointer from the first pixels; releasing creates a bed of that size | automated |
| UC-16 | Release a drawn zone near the plot edge | F-PLN-1 | A bed is created (not refused for its default size) | automated |
| UC-20 | Move / resize a bed by dragging, refusal goes back | F-PLN-6 | Position kept or restored with the reason | automated (e2e; the rules themselves are covered by `cargo test`) |
| UC-21 | Delete a bed with plantings (confirmation) | F-PLN-4 | Warning with the count, then removal | automated |
| UC-22 | Clear the plan, erase the data (double confirmation) | F-CPT-4 | Plan emptied / app data erased | automated (clear; erase is Rust-tested) |
| UC-23 | Export and import a garden file | F-CPT-3 | Round trip, "older file" warning, "to resolve" list | Rust: `garden.rs` tests; the file dialog needs the real window |
| A11Y | Axe audit (WCAG A and AA) of plan, garden, settings and catalogue | all | No serious or critical violation | automated (`front/e2e/a11y.spec.ts`) |
| UC-24 | Damaged database: Retry / settings / erase | storage | Explanation screen, then a fresh garden | Rust: `src-tauri/src/tests.rs`; screen not automated |
| UC-25 | Delete a bed that has plantings | F-PLN-7 | Warning shows the count of plantings deleted; Cancel keeps the bed | automated (e2e) |
| UC-26 | Planting form: change the start date | F-PLA-2 | The end stays within a year of the start and follows the new date | automated (e2e) |
| UC-27 | Plan: type a bed length in the panel | F-PLN-3 | No error; a refused value goes back to the real one | automated (e2e) |
| SER-01 | Catalogue sheet: no internal reference ("V1", "S02") in its footer, English and French | F-CAT-3 | The footer only says the sheet is read-only | automated (`front/e2e/series-form.spec.ts`) |
| SER-02 | Catalogue: pick a crop, then filter by family | F-CAT-3 | The sheet always belongs to a card that is shown | automated (`front/e2e/series-form.spec.ts`) |
| SER-03 | Catalogue on a phone (390 px): tap a crop card | F-CAT-3 | The sheet, under the cards, is scrolled into view | automated (`front/e2e/series-form.spec.ts`) |
| SER-04 | Planting form: clear the start date, then type another | F-PLA-2 | No invented dates (1900), Save disabled until a date is typed, dates back after | automated (`front/e2e/series-form.spec.ts`) |
| SER-05 | Planting form from a slot: the end field, the last step, the label and the slot all read the last occupied day (S03 §1) | F-PLA-2 | A slot "… → 9 May" proposes an end that reads 9 May; a typed end is stored one day later | automated (`front/e2e/series-form.spec.ts`, `front/src/lib/planting/series-form.test.ts`) |
| SER-06 | Apply a suggestion, then type an end | F-PLA-2 | "End limited to the free slot" for a proposed end, "Edited by hand" only once the user typed it | automated (`front/e2e/series-form.spec.ts`, unit) |
| SER-07 | Conflict on a single segment | F-PLA-1 | "on segment 4" (singular), "on segments 3–5" (plural), English and French | automated (unit, `series-form.test.ts`) |
| SER-08 | Catalogue count and vocabulary | F-CAT-3 | The count follows the filter (total without), singular for one, "crops" / "cultures" everywhere, no "légume" | automated (`front/e2e/series-form.spec.ts`, unit) |
| KBD-01..07 | Keyboard only: focus trap and focus return in the five windows, Enter/Space/Escape on beds, Escape in a confirmation | accessibility | Tab never leaves a window; Escape returns focus to the opener | automated (`front/e2e/keyboard.spec.ts`) |
| LAY-01..04 | Phone (360 and 390 px, EN and FR): year, speed and bed panel buttons and links, plan line without a stray bullet | accessibility | Targets of at least 40 px | automated (`front/e2e/layout.spec.ts`) |
| LAY-05 | Phone: timeline bars and free-slot chips | accessibility | At least 40 px high (invisible tap rect, the drawing is unchanged) | automated (`front/e2e/targets.spec.ts`) |
| LAY-06 | Phone: export dialog (buttons, link, date fields, checkboxes through their label) | accessibility | Every control at least 40 px | automated (`front/e2e/targets.spec.ts`) |
| LAY-07 | Phone: planting form opened from a free slot (chips, fields, select, link, segment strip) | accessibility | Every control at least 40 px | automated (`front/e2e/targets.spec.ts`) |
| LAY-08 | Phone: garden screen (time slider, annotations checkbox, header, timeline) | accessibility | Every control at least 40 px; the 3D beds and note texts are scene content, not measured | automated (`front/e2e/targets.spec.ts`) |
| MONKEY | Seeded random clicks, drags and keys | all | No page error, no console error | automated (`front/e2e/monkey.spec.ts`) |
| UC-30 | Real Tauri window, touch, installed build | all | — | **manual** (never automated yet) |

## First analysis of the navigation (17 November 2026, done once)

Map of the interface: header (Garden / Plan tabs, year buttons, Today, save state, Catalogue, Settings) →
**Garden** (isometric view, time cursor, play, timeline with free slots → planting form, bars, annotate, export) and
**Plan** (grass click or tray drag, handles, bed panel, "See the garden"); windows: Settings, Catalogue, Export, Note, Planting.
Walked with the example data at 1440 px and 390 px, in English and French, in WebKit.

Found and fixed: the catalogue footer said "6 crops in V1" (0.2.7); the catalogue was one card a row on a phone (0.2.7);
bed names were cut in the timeline (0.2.7, 0.2.9); the Garden tab kept the date the cursor was left on (0.2.6);
the header wrapped onto a third row on a phone in French (0.3.2); the export dialog announced "A4 or A3" while only A4 exists (0.3.5); emoji and symbols came out as empty boxes in the export (0.3.3).

Observations, decided:
1. The app opens on **Plan**: kept: no spec asks for another first screen and the garden is one tab away.
2. The empty "A bed?" card on desktop: kept, it holds the place of the bed panel and avoids a layout jump on selection.
3. **Free slots wholly in the past** stay offered: plantings of the past stay editable to record the history (S01 F-PLA-4, S03 §17), and a sowing proposed in the past is turned into plants (S03 §9).
4. Annotation labels over a bar label: the halo keeps both readable; no change.
5. Small garden view at 390 px: a drawing choice (P01); no change without a new design.
6. Free slots in the export preview: not a spec point (the export crate draws no slot); no change without a design decision.
7. Not testable here: the real Tauri window, touch gestures, the installed build, the file dialogs (UC-30, manual).

## Coverage of the requirements

Requirement → test, for the 29 requirements of S01 §4 listed in `POT-00_lot1-coverage.md` (reviewed 18 November 2026).
"Behaviour" means a test asserts the expected result, not only that a screen exists. Rust = `cargo test`; unit = vitest;
e2e = Playwright against the fake of the commands (`?example`), which does not run the Rust rules (those are covered by the Rust tests).

| Requirement | Behaviour asserted by | Verdict |
|---|---|---|
| F-CPT-1 Sign-in | — (removed) | n/a |
| F-CPT-2 Saving | Rust: `sqlite.rs` (versions, stale copy, reopening), `commands` tests (restart); unit `state.test.ts` ("Saving failed" keeps the change, Retry runs it again) | Covered. The header indicator itself (three states) is not asserted in a browser |
| F-CPT-3 Export / import | Rust: `garden.rs`, `garden_stress.rs`, `hostile_import.rs`, `tests.rs` (older file, invalid, incoherent import) | Covered in Rust. **Gap**: the Settings screen (file dialog, confirmation before replacing, "to resolve" list) is manual only (UC-23, UC-30) |
| F-CPT-4 Erase | Rust: `tests.rs` (damaged database erased), `store` `replace_and_clear`; e2e UC-22 (clear the plan only) | **Gap**: the double confirmation of "Erase the data" has no UI test |
| F-TER-1 Plot creation | e2e UC-01 (defaults); Rust `placement.rs` (dimensions, aisle) | **Gap**: choosing 25 / 100 cm cells or a 30 / 80 cm aisle at creation, and the "cannot be changed" warning, are not asserted |
| F-PLN-1 Beds | Rust: `eleventh_bed_refused`, `placement.rs`, `names_have_1_to_30_characters`, `length_not_a_multiple...`; e2e UC-02, UC-15, UC-16, **REQ-03** (tray greyed out at 10 with its message), **REQ-05** (rename, empty refused) | Covered. Partial requirement: the same-name warning does not exist |
| F-PLN-2 Aisles | Rust: `the_aisle_width_is_kept_between_beds`, `a_larger_aisle_can_make_the_plan_invalid`; unit `beds.test.ts` | Covered for the enforced width. Changing the aisle in the Settings has no UI test |
| F-PLN-4 Plan by season | Rust: `modification_holds_until_the_next_recorded_layout`, `past_year_is_read_only`, deletion tests, `t11`, clear tests; e2e UC-04, UC-21, UC-22, UC-25 | Covered |
| F-PLN-5 Plan summary | unit `beds.test.ts` ("summary") | **Gap**: the right-hand panel is not asserted in a browser |
| F-PLN-6 Place and size | unit `drag.test.ts`; Rust `placement.rs`, `t12`; e2e UC-03, UC-20, UC-27 | Covered for the lot 1 scope. Rotate, duplicate, Undo / Redo absent (Partial) |
| F-CAT-1 Content | Rust: `catalogue` tests, `catalogue_wide.rs`; e2e UC-08 (67 crops) | Covered |
| F-CAT-2 Crop sheet | Rust `tomato_matches_s02`, `v1_has_unique_ids_and_valid_sheets`; e2e SER-01..03 | Covered |
| F-CAT-3 Browsing | e2e UC-08, UC-13, SER-02, SER-03, SER-08; Rust `crops_in_season_follow_the_planting_window` | **Gap**: the "in season now" checkbox is not clicked in any browser test |
| F-PLA-1 Create a planting | Rust: `s03_cas` t01..t10, `edge_cases` (modes, durations copied); unit `planting.test.ts`; e2e UC-06, SER-04..06, REQ-04 | Covered. The three "How do you start?" cards and the days in pots: e2e EXP-03, EXP-04 (the transplanting-date link stays unit only) |
| F-PLA-2 Edit / delete | Rust: `dragging_shifts_the_date_and_the_manual_end`, `manual_end_is_not_recomputed`; unit `geometry.test.ts` (drag accepted / refused); e2e UC-14, UC-26, **REQ-01** (delete asks, Cancel keeps, confirm removes) | Covered. The drag of a bar in a browser is not asserted (unit only) |
| F-PLA-3 Successions | Rust: `disjoint_zones_same_period_no_conflict`, `conflict_reports_shared_segments_and_period`, `t06`; unit SER-07, `geometry.test.ts` ("in conflict") | Conflict message in the form (singular / plural of the segments, period ending on the last busy day, a start on the day the other leaves is free): e2e EXP-01, EXP-02 |
| F-PLA-4 Continuous time | Rust: `a_planting_from_the_previous_year_occupies_the_start_of_the_year`; unit `geometry.test.ts` (straddling plantings shown once); e2e UC-04, UC-05 | Covered. The diorama taking the layout of a new year when the cursor crosses 1 January has no test |
| F-PLA-5 Quantity | Rust: `quantities.rs`, `t04`, `t05`, `quantity_minimums`; unit `plants.test.ts`; e2e **REQ-04** (rows × per row = plants, follows the zone) | Covered |
| F-VIS-1 Isometric view | unit `projection.test.ts`, `plants.test.ts`, `animation.test.ts`; Rust `stages.rs`; e2e DIO-01, **REQ-06** (rotate quarter by quarter) | Zoom and pan are not implemented (coverage doc corrected to Partial). The "Free" sign and the seasonal mood have no assertion |
| F-VIS-2 Timeline | unit `geometry.test.ts` (rows, phases, stacking, zoom 3 months to 3 years, scroll); e2e UC-06, UC-14, TML-01, LAY-05 | Covered. Greyed row of a missing bed: unit only |
| F-VIS-3 Time cursor | e2e UC-05 (Next week, Today); unit `geometry.test.ts` | Covered |
| F-VIS-4 Season playback | unit `animation.test.ts` (15 s a year, ×2, reduced motion, bounce); e2e DIO-01 (plays, frame p95) | Covered. The pause button and the ×1 / ×2 buttons are only tested for their size (LAY-02) |
| F-VIS-5 Annotations | e2e UC-07, TML-01, **REQ-02** (hide, edit window, delete); unit `note.test.ts`; Rust `annotations.rs` | Covered for what exists. Double click / long press creation and drag are absent (coverage doc corrected to Partial) |
| F-VIS-6 Screen layout | e2e UC-12, UC-13, LAY-01..08, A11Y | Covered at 360 / 390 / 1440 px |
| F-VIS-7 Bed panel | e2e UC-21, UC-25, **REQ-05** (rename in place); Rust `deletion_warning_quantities_and_history`, `families_of_the_two_previous_seasons`; LAY-03 | **Gap**: the plantings and the History section of the panel are not asserted in a browser |
| F-CRE-1 Free slots | Rust: `slot_of_35_days_dropped_and_36_days_kept`, window 1 March – 15 November, `t07`; e2e UC-06, LAY-05 | Covered |
| F-CRE-2 Suggestions | Rust: `t08`, `t10`, ranking tests, `catalogue_wide.rs`, `random_walk.rs`; unit `suggestions.test.ts`; e2e SER-05, SER-06 | Covered |
| F-EXP-1 Timeline export | Rust: `timeline.rs`, `timeline_front.rs`, `tests.rs` (valid PNG and PDF); unit `exportPlan.test.ts`, `exportSvg.test.ts`; e2e UC-10, LAY-06 | Beds (select / deselect all, one bed) and options (phases, today marker, annotations) of the dialog: e2e EXP-05 to EXP-07, read on the SVG of the preview. **Gap**: writing the file is not run end to end (needs the native dialog) |
| F-EXP-2 Content | unit `exportSvg.test.ts` (phases, fonts); Rust `timeline_stress.rs` | Partial requirement (no header, thumbnail, task calendar); `tasks.rs` is tested in Rust but has no screen |

### Gaps closed in this review (REQ-01 to REQ-06, `front/e2e/requirements.spec.ts`)

| Id | Requirement | Asserts |
|---|---|---|
| REQ-01 | F-PLA-2 | Deleting a planting asks first; Cancel keeps the bar, confirming removes exactly one |
| REQ-02 | F-VIS-5 | "Show the annotations" hides and restores every note; a note opens for editing and is deleted |
| REQ-03 | F-PLN-1 | At 10 beds the four tray items are disabled and "10 beds at most per season." appears |
| REQ-04 | F-PLA-5 | The form shows "≈ n plants, r rows × p", n = r × p, the button says the same n, and a quarter zone follows |
| REQ-05 | F-VIS-7, F-PLN-1 | Renaming a bed in place updates its panel; the field is capped at 30; an empty name is refused and the old name comes back |
| REQ-06 | F-VIS-1 | "Rotate the view" moves the beds at each quarter turn and four turns come back |

Found while writing them: the fake of the commands (`front/src/lib/simulation.ts`) accepted an empty bed name where Rust refuses
(`name_invalid`); it now applies the same rule (trimmed, 1 to 30 characters). The application code was not changed.

### Gaps closed (EXP-01 to EXP-07, `front/e2e/export-form.spec.ts`)

| Id | Requirement | Asserts |
|---|---|---|
| EXP-01 | F-PLA-3 | Zone 3-5 over a tomato on 1-4 and a lettuce on 5-8: "segments 3–4" (plural) and "segment 5" (singular), period from the start to the last busy day; the button is disabled |
| EXP-02 | F-PLA-3 | The period ends on the last busy day (26 Sep, not 27); starting on the day the other leaves (27 Sep) gives no conflict |
| EXP-03 | F-PLA-1 | Tomato: under cover (sowing, transplanting +56, harvest +131, end), bought plants (planting, harvest +75), open ground (harvest +131); question and steps follow the card |
| EXP-04 | F-PLA-1 | A crop without pots started under cover asks for the days in pots and moves the transplanting; they do not leak into the ground start |
| EXP-05 | F-EXP-1 | Deselect all (button, message, export disabled), one bed (only its bars), select all |
| EXP-06 | F-EXP-1 | Phases, today marker and annotations off / on change the drawing of the preview |
| EXP-07 | F-EXP-1 | The annotations of a deselected bed leave the drawing, those of the whole garden stay |

Found while writing them: no defect in the application. The fake of the commands kept the days typed for the pots in an open-ground
start of a crop without pots, where Rust (`pot_days_for`) keeps the catalogue's value; it now applies the Rust rule (the form never
sent that case, so no test depended on it). In the English tests the engines do not abbreviate September alike ("Sep" in WebKit, "Sept" in Chromium), so the dates are compared loosely.

### Open gaps (not closed, by impact)

1. Settings screen: import with confirmation and "to resolve" list, erase with double confirmation (F-CPT-3, F-CPT-4): Rust only.
2. Plot creation options: cell size, aisle, warning (F-TER-1).
3. Bed panel: plantings and History sections (F-VIS-7); plan summary panel (F-PLN-5).
4. Catalogue "in season now" filter (F-CAT-3); pause and speed buttons of the playback (F-VIS-4).
