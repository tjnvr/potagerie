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
