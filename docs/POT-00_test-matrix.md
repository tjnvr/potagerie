# Potagerie — Use-case test matrix

The automated use cases live in `front/e2e/use-cases.spec.ts` (Playwright Test). They run the interface
against the Vite dev server with the in-memory fake of the Tauri commands (`?example`), in **Chromium and WebKit**
(the engine of the macOS app). The Rust rules are covered by `cargo test` (S03 §12 cases, edge cases, random walk).

- Locally: `make e2e`, or `pnpm --dir front e2e` (first time: `pnpm --dir front exec playwright install chromium webkit`).
- In CI: the Ubuntu job of `.github/workflows/ci.yml` runs them after the unit tests; the traces of a failed run are uploaded.
- A new use case = a row below + a `UC-xx` test with the same id (see `skills/spec-driven-delivery`, step 3 and 6).

The smoke tests of the **real app** (`APP-xx`, `e2e-app/smoke.test.mjs`) drive the built binary through `tauri-driver`
(WebDriver), with the real Rust commands and SQLite, in **WebView2 (Windows) and WebKitGTK (Linux)**. They use a fresh
temporary data folder, and the "Open" and "Save as" windows answer one file of it (S05 §3): the native windows themselves are
not driven. `tauri-driver` does not support macOS (WKWebView); touch is not driven either.

- Locally, on Linux or Windows: `make e2e-app` (needs `tauri-driver` and the native driver, see the `Makefile`).
- In CI: the `app` job of `.github/workflows/ci.yml`, on `ubuntu-24.04` and `windows-latest`; screenshots of failed steps are uploaded.

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
| UC-08 | Catalogue: 69 crops, family filter | F-CAT-1, F-CAT-3 | Count text, filter narrows the list | automated |
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
| UC-23 | Export and import a garden file | F-CPT-3 | Round trip, "older file" warning, "to resolve" list | Rust: `garden.rs` tests; real app: APP-04, APP-06 (the native file window answered by the test stand-in) |
| A11Y | Axe audit (WCAG A and AA) of plan, garden, settings and catalogue | all | No serious or critical violation | automated (`front/e2e/a11y.spec.ts`) |
| UC-24 | Damaged database: Retry / settings / erase | storage | Explanation screen, then a fresh garden | Rust: `src-tauri/src/tests.rs`; screen not automated |
| UC-25 | Delete a bed that has plantings | F-PLN-7 | Warning shows the count of plantings deleted; Cancel keeps the bed | automated (e2e) |
| UC-26 | Planting form: change the start date | F-PLA-2 | The end stays within a year of the start and follows the new date | automated (e2e) |
| UC-27 | Plan: type a bed length in the panel | F-PLN-3 | No error; a refused value goes back to the real one | automated (e2e) |
| UC-28 | Zoom and pan the garden view | F-VIS-1 | The wheel zooms around the pointer; dragging the background pans; zoomed all the way out, the wheel scrolls the page; a bed still opens on a click | automated (e2e) |
| UC-29 | Timeline: double click (computer) or long press (touch) on a row | F-VIS-5 | "New note" opens on that bed, on that day; saved, the note stands there | automated (e2e; the long press through Chromium touch events only) |
| UC-31 | Timeline: drag a note or a period | F-VIS-5 | A note takes the new date and the bed of the row it is dropped on; a period dragged into a past year is refused with the reason and goes back | automated (e2e) |
| UC-32 | "Move" in the bed panel of the garden | F-VIS-7 | The Plan tab opens in the same year with the bed selected (handles, panel); in a past season the action is disabled and says why | automated (e2e) |
| UC-33 | Drop, move or resize a bed closer than the aisle | F-PLN-2 | A bubble asks "Keep here" or "Go back": "Go back" puts the bed back, "Keep here" keeps it there without an error; an overlap is still refused, without a question | automated (e2e; the rules by `cargo test`, S03 T14) |
| UC-34 | Plant a raspberry in November, look at the next summer and winter | F-CAT-1, F-PLA-1, F-VIS-1, F-VIS-2 | The end proposed is ten years on; in July 2027 the plants are in fruit, in January bare (the smallest drawing) and still on the bed; one bar | automated (e2e; the rules by `cargo test`, S03 T15 to T19) |
| UC-35 | Plan: rename a bed with the name of another bed, case aside | F-PLN-1 | Accepted without an error; the panel of both beds says another bed of the season has the same name; renamed apart, the warning goes | automated (e2e) |
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
| APP-01 | Real app, first launch: no plot, create it | F-TER-1 | "No plot yet", then "0 / 10"; `potagerie.db` is in the temporary data folder | automated (`e2e-app/`, WebView2 and WebKitGTK) |
| APP-02 | Real app: a click on free grass adds a bed | F-PLN-1 | Counter 0 → 1 | automated (`e2e-app/`) |
| APP-03 | Real app: plant from a free slot | F-PLA-1 | Slot (keyboard Enter) → form → "Plant · …" adds one bar | automated (`e2e-app/`) |
| APP-04 | Real app: export the garden | F-CPT-3 | "Garden exported.", the JSON file is written | automated (`e2e-app/`) |
| APP-05 | Real app: restart | storage | The bed and the planting are still there | automated (`e2e-app/`) |
| APP-06 | Real app: clear the plan, then import the file | F-PLN-4, F-CPT-3 | "Plan cleared.", then "Replace anyway" (older file), "Garden imported.", the bed and the bar are back | automated (`e2e-app/`) |
| UC-30 | macOS window (WKWebView), touch, installed build, native file windows | all | — | **manual** (Windows and Linux windows: APP-01 to APP-06) |

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
| F-CPT-3 Export / import | Rust: `garden.rs`, `garden_stress.rs`, `hostile_import.rs`, `tests.rs` (older file, invalid, incoherent import) | Covered in Rust. Settings screen: e2e **SET-02..05** (confirmation with summary, cautious button for an older file, issue list, "to resolve" band, unreadable / too recent / file refusals). Real app: export, then import after clearing, in WebView2 and WebKitGTK (APP-04, APP-06). The native file window itself is not driven (UC-30 manual) |
| F-CPT-4 Erase | Rust: `tests.rs` (damaged database erased), `store` `replace_and_clear`; e2e UC-22, **SET-06** (clear the plan), **SET-07** (erase: each of the two steps can be cancelled, the second erases) | Covered |
| F-TER-1 Plot creation | e2e UC-01 (defaults), **SET-09..13** (defaults, "cannot be changed" warning, cell size in the preview, side under 1 m refused, 25 / 100 cm cells and 30 / 80 cm aisle kept); Rust `placement.rs` (dimensions, aisle) | Covered |
| F-PLN-1 Beds | Rust: `eleventh_bed_refused`, `placement.rs`, `names_have_1_to_30_characters`, `length_not_a_multiple...`; e2e UC-02, UC-15, UC-16, **REQ-03** (tray greyed out at 10 with its message), **REQ-05** (rename, empty refused), **UC-35** (same name flagged); Rust `t20_two_beds_with_the_same_name_are_flagged`, `two_beds_with_the_same_name_are_flagged_not_refused` | Covered |
| F-PLN-2 Aisles | Rust: `the_aisle_width_is_asked_for_between_beds`, `a_pair_made_too_close_in_a_following_year_is_asked_for`, `keep_close_never_lets_an_overlap_or_a_bed_outside_the_plot_through`, `t14_keep_a_bed_closer_than_the_aisle`, `a_larger_aisle_can_make_the_plan_invalid`, `tests.rs` `a_narrow_aisle_is_kept_only_when_asked`; unit `beds.test.ts`; e2e **UC-33** ("Keep here" / "Go back") | Covered. Changing the aisle in the Settings: e2e **SET-01** (30 cm kept, 80 cm refused with the reason) |
| F-PLN-4 Plan by season | Rust: `modification_holds_until_the_next_recorded_layout`, `past_year_is_read_only`, deletion tests, `t11`, clear tests; e2e UC-04, UC-21, UC-22, UC-25 | Covered |
| F-PLN-5 Plan summary | unit `beds.test.ts` ("summary") | Covered. e2e **PAN-01** (plot, workable, aisles, bed count; follows an added bed; gives way to the bed panel) |
| F-PLN-6 Place and size | unit `drag.test.ts`; Rust `placement.rs`, `t12`; e2e UC-03, UC-20, UC-27 | Covered for the lot 1 scope. Rotate, duplicate, Undo / Redo absent (Partial) |
| F-CAT-1 Content | Rust: `catalogue` tests (`red_fruits_match_s02`), `catalogue_wide.rs`; e2e UC-08 (69 crops) | Covered. Red fruits (v1.8): raspberry and strawberry; the other five come with T19 |
| F-CAT-2 Crop sheet | Rust `tomato_matches_s02`, `red_fruits_match_s02`, `v1_has_unique_ids_and_valid_sheets`; e2e SER-01..03 | Covered. The lifespan and yearly harvest lines of a red fruit sheet have no e2e assertion |
| F-CAT-3 Browsing | e2e UC-08, UC-13, SER-02, SER-03, SER-08; Rust `crops_in_season_follow_the_planting_window` | Covered. e2e **PAN-07**, **PAN-08** ("in season now" keeps the crops of the window, combines with the family, badge, count) |
| F-PLA-1 Create a planting | Rust: `s03_cas` t01..t10, `edge_cases` (modes, durations copied); unit `planting.test.ts`; e2e UC-06, SER-04..06, REQ-04 | Covered. The three "How do you start?" cards and the days in pots: e2e EXP-03, EXP-04 (the transplanting-date link stays unit only) |
| F-PLA-2 Edit / delete | Rust: `dragging_shifts_the_date_and_the_manual_end`, `manual_end_is_not_recomputed`; unit `geometry.test.ts` (drag accepted / refused); e2e UC-14, UC-26, **REQ-01** (delete asks, Cancel keeps, confirm removes) | Covered. The drag of a bar in a browser is not asserted (unit only) |
| F-PLA-3 Successions | Rust: `disjoint_zones_same_period_no_conflict`, `conflict_reports_shared_segments_and_period`, `t06`; unit SER-07, `geometry.test.ts` ("in conflict") | Conflict message in the form (singular / plural of the segments, period ending on the last busy day, a start on the day the other leaves is free): e2e EXP-01, EXP-02 |
| F-PLA-4 Continuous time | Rust: `a_planting_from_the_previous_year_occupies_the_start_of_the_year`; unit `geometry.test.ts` (straddling plantings shown once); e2e UC-04, UC-05 | Covered. The diorama taking the layout of a new year when the cursor crosses 1 January has no test |
| F-PLA-5 Quantity | Rust: `quantities.rs`, `t04`, `t05`, `quantity_minimums`; unit `plants.test.ts`; e2e **REQ-04** (rows × per row = plants, follows the zone) | Covered |
| F-VIS-1 Isometric view | unit `projection.test.ts`, `plants.test.ts`, `animation.test.ts`; Rust `stages.rs`, `perennials.rs` (bare in winter); e2e UC-34; unit `camera.test.ts` (bounds, zoom around a point, pan kept inside the frame, wheel steps); e2e DIO-01, DIO-02 (zoomed playback), **REQ-06** (rotate quarter by quarter), UC-28 (wheel zoom, background drag) | Pinch (touch) and the Safari trackpad gesture are not automated. The "Free" sign and the seasonal mood have no assertion |
| F-VIS-2 Timeline | unit `geometry.test.ts` (rows, phases, a harvest each year for a perennial, stacking, zoom 3 months to 3 years, scroll); e2e UC-06, UC-14, TML-01, LAY-05 | Covered. Greyed row of a missing bed: unit only |
| F-VIS-3 Time cursor | e2e UC-05 (Next week, Today); unit `geometry.test.ts` | Covered |
| F-VIS-4 Season playback | unit `animation.test.ts` (15 s a year, ×2, reduced motion, bounce); e2e DIO-01, DIO-02 (plays, frame p95, at full view and zoomed in) | Covered. e2e **PAN-09** (play, pause, resume), **PAN-10** (×1 default, ×2 exclusive and about twice as fast); LAY-02 for the size |
| F-VIS-5 Annotations | e2e UC-07, TML-01, **REQ-02** (hide, edit window, delete), **UC-29** (double click / long press creation), **UC-31** (drag, refusal goes back); unit `note.test.ts` (moved input), `geometry.test.ts` (row under the pointer); Rust `annotations.rs` | Covered. The long press on a real phone is not driven (UC-30 manual) |
| F-VIS-6 Screen layout | e2e UC-12, UC-13, LAY-01..08, A11Y | Covered at 360 / 390 / 1440 px |
| F-VIS-7 Bed panel | e2e UC-21, UC-25, **UC-32** ("Move"), **REQ-05** (rename in place); Rust `deletion_warning_quantities_and_history`, `families_of_the_two_previous_seasons`; LAY-03 | Covered. e2e **PAN-02..06** (plantings of the year, empty bed, History folded / open / empty, past planting read only, "Plant here", same panel from the garden) |
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

### Gaps closed (SET-01 to SET-13, `front/e2e/settings-plot.spec.ts`, English and French, Chromium and WebKit)

SET-01 aisle; SET-02..05 import; SET-06 clear the plan; SET-07 erase with double confirmation; SET-08 language (the page reloads, the choice is kept); SET-09..13 plot creation.
The fake of the commands now follows Rust on `create_plot` (plot once, cell 25 / 50 / 100, sides 1 to 100 m and at least one cell, aisle 30 / 50 / 80) and `modify_plot` (`Plan::set_plot`: valid plot, cell kept, refused when the beds no longer fit the aisle).
The `?example` plan had "Leek square" 30 cm from "Shed side" with a 50 cm aisle, which Rust refuses; it moved 20 cm down.

### Open gaps (not closed, by impact)

1. ~~Settings screen~~ closed by SET-01..08 (`front/e2e/settings-plot.spec.ts`).
2. ~~Plot creation options~~ closed by SET-09..13.
3. The conflict message and the three "How do you start?" cards in a browser (F-PLA-1, F-PLA-3).
4. ~~Bed panel and plan summary panel~~ and 5. ~~catalogue "in season now" filter, pause and speed buttons~~: closed by PAN-01 to PAN-10 (below).
6. Export dialog: bed selection and options (F-EXP-1).

### Gaps 4 and 5 closed (PAN-01 to PAN-10, `front/e2e/panels-catalogue.spec.ts`)

Chromium and WebKit, English and French, with `?date=2027-06-20` so the example data does not depend on the day.

| Id | Requirement | Asserts |
|---|---|---|
| PAN-01 | F-PLN-5 | Summary: 71.5 / 24 / 47.5 m² and "8 / 10"; an added square gives 25 / 46.5 m² and "9 / 10"; a selected bed replaces the summary |
| PAN-02 | F-VIS-7 | The panel of "Roots" lists its three plantings of the year with segments and plant count |
| PAN-03 | F-VIS-7 | A bed without planting says so; History is folded, opens, says it is empty, folds again |
| PAN-04 | F-VIS-7 | History lists the planting of the past season (2026), with no action button |
| PAN-05 | F-PLA-1 | "Plant here" opens "New planting" on the selected bed |
| PAN-06 | F-VIS-7 | The same panel opens from the garden, History included |
| PAN-07 | F-CAT-3 | The "in season now" switch keeps exactly the crops of the window (computed from `crops.json` with Rust's rule); badge and count follow |
| PAN-08 | F-CAT-3 | The season filter combines with a family; another date (10 January) gives the list of that date |
| PAN-09 | F-VIS-4 | Play, pause (date frozen), resume |
| PAN-10 | F-VIS-4 | ×1 by default, ×2 exclusive and at least 1.4 times faster over one second |

The fake of the commands already followed Rust on the cases tested (`crops_of_season` copies `plan::in_window`); neither the fake nor the application was changed.


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
