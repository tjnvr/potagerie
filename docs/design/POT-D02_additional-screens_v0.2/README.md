# POT-D02 · Additional screens V1 — v0.2

> Version: 0.2 (draft) · Date: 2026-11-15 · Author: Tim
> Specs: [S01 v1.6](../../specs/POT-S01_functional-spec_v1.7.md) · [S02 v0.4](../../specs/POT-S02_crop-catalogue_v0.4.md) · [S03 v0.18](../../specs/POT-S03_rules-and-calculations_v0.18.md)
> Follows [D01 v1.2](../POT-D01_key-screens_v1.2/README.md), which already covers E03, E05 and E07 (phone), and the visual direction (DV01).
> v0.2 is the English translation of v0.1: texts, files and generator are in English. The layout does not change.

As Tim was not available during this lot, the choices below were settled alone; they are listed in "Decisions" further down.

## Content

All the lot 1 screens that D01 does not cover yet, in a computer version, plus one phone sheet to show the adaptation of the layout (E10):

| File | Screen |
|---|---|
| `project/canvas.json` | Canvas index (positions, titles, notes) |
| `project/Main.dc.html` | E02 · Plot creation — computer (entry sheet of the canvas) |
| `project/E06_bed-panel_desktop.dc.html` | E06 · Bed panel — computer |
| `project/E07_planting_desktop.dc.html` | E07 · New planting — computer |
| `project/E09_catalogue_desktop.dc.html` | E09 · Catalogue — computer |
| `project/E10_settings_desktop.dc.html` | E10 · Settings — computer (panel) |
| `project/E10_settings_mobile.dc.html` | E10 · Settings — phone (full screen) |
| `project/E11_timeline-export_desktop.dc.html` | E11 · Timeline export — computer |
| `project/ET01_states_desktop.dc.html` | ET01 · Cross-cutting states — computer (4 states on one sheet) |
| `gen.py` | Generator of the sheets (`python3 gen.py`) |
| `plants2.py` | Crop drawings — **copied from D01**, see "Reuse" |
| `exports/` | One PNG per sheet, captured in headless Chrome (see "Visual check") |

Once published, the online canvas is the reference; this folder is its versioned copy. `gen.py` regenerates everything from scratch and overwrites the touch-ups made by hand in the canvas.

## Reuse of D01

`plants2.py` (low-poly drawings of the crops at their 4 stages, used by `crop_icon()`) is **copied as is** from `POT-D01_key-screens_v1.2/plants2.py` — it was not rewritten by hand. The palette, the icons, the fonts (Fredoka/Figtree) and the small interface components (chip, button, card…) follow the same visual direction as D01 (DV01), but are redefined in the `gen.py` of D02: they are small functions specific to each generator, not a common library to factor out for now.

A single file of D01 was modified, minimally, to close the loop of the flow: the "Edit" button of the simplified panel of the Garden screen (`POT-D01.../Main.dc.html`) becomes a link to the full bed panel of D02 (see "Flow").

## Flow

The sheets are linked together (Play button), including to D01. The complete flow 6.1 (S01, §6.1):

| From | To | Trigger |
|---|---|---|
| ① E02 Plot creation (D02) | E03 Plan editor (D01) | "Create the plot" |
| E03 Plan editor (D01) | E05 Garden (D01) | "See the garden" (already in D01) |
| E05 Garden (D01) | ④ E06 Bed panel (D02) | "Edit", in the simplified panel (link added, see "Reuse") |
| ④ E06 Bed panel (D02) | ⑤ E07 New planting (D02) | "Plant here" |
| ④ E06 Bed panel (D02) | E03 Plan editor (D01) | "Move" |
| ⑤ E07 New planting (D02) | E05 Garden (D01) | "Plant …" or the cross |

Side screens, reachable from the settings or catalogue icon of the header (D02):

| From | To | Trigger |
|---|---|---|
| Any D02 sheet | E09 Catalogue | book icon of the header |
| Any D02 sheet | E10 Settings | settings icon of the header |
| E10 Settings (computer or phone) | E05 Garden / E05 phone (D01) | cross or back arrow |

ET01 (cross-cutting states) is not part of the flow: like DV01 in D01, it is a reference sheet, with no outgoing link.

## Example data

The garden takes over that of D01 (same names, same 6 reference crops of S02), extended to **8 beds out of 10** to reflect the "Beds 8 / 10" counter already shown in the editor of D01 (see "Decisions"). 2027 season, real today = 14 November 2026.

All the dates and quantities shown were **computed by a Python script** that reimplements the rules of S03 (§2 dates, §4 and §6 segments/quantities, §9 suggestions), then checked against the test cases of S03 §12 (T1, T3, T4, T5, T7, T8, T9, T11 reproduced identically) before being copied into `gen.py`. This script is a checking script, outside the versioned folder (like the test cases of S03, which remain the only source of truth):

- **E06** (panel of "Roots") and its deletion warning reproduce **T11** as is (carrot kept, leek deleted because it straddles 2028).
- **E07** (new planting on "End of the aisle") reproduces **T13** as is (slot of 65 d, proposed end shortened to 6 Oct instead of 11 Oct, saved as edited by hand).
- The free slot of "Back of the garden" (mentioned in the context of E06/E05) takes up **T7**.
- The other plantings (tomato/lettuce of "The tomatoes", zucchini, bean, leeks of the planters) were computed with the same formulas and checked by the same script, but do not match a named test case.

## Visual check

Each sheet was captured with headless Chrome, at the size of its `$preview` (`chrome --headless --disable-gpu --screenshot=... --window-size=W,H file.dc.html`), to check that no text overflows or overlaps and that everything stays readable. These captures are the PNGs of `exports/`. An overlap error was found this way on E06 (the deletion confirmation box overflowed onto the neighbouring panel) and fixed before the final export.

## Decisions

In the absence of Tim, the following choices were settled and are to confirm with him:

| Subject | Decision | Why |
|---|---|---|
| Aisle width at plot creation (E02) | Added to F-TER-1 of S01 (v1.3): 30/50/80 cm, 50 cm by default, editable afterwards | The brief asked for the aisle on E02, but S01 v1.2 only placed it in the settings (E10) with no starting value; see S01 §11 |
| "Saving failed" state (ET01) | Added to F-CPT-2 of S01 (v1.3), with a "Retry" button | S05 §7 already foresees that a write may fail, but no state showed it; see S01 §11 |
| History in the bed panel (E06) | Added to F-VIS-7 of S01 (v1.3): plantings of past years, read-only, collapsible, plus the "Move" action | The table of screens (S01 §6) already promised "history" and "actions", richer than what F-VIS-7 detailed; see S01 §11 |
| 8th bed of the example ("End of the aisle", mound) | Added to the example garden of D01 to reach 8 / 10 | The tray of the editor (D01, E03) already shows "Beds 8 / 10", while the example only had 7 beds; D02 fixes this inconsistency by adding the 8th. It also serves as the blank bed for the demonstration of E07 (free slot, T13) |
| Highlighting the shortened end (E07) | An "Edited by hand" badge next to the end date, plus a "Recompute" link toward the normal end | S01 F-PLA-2 only says the value stays editable; no mockup showed how it is displayed |
| Move conflict message (ET01) | "Position refused. “X” would overlap “Y”. The bed goes back to its place." | F-PLN-6 describes the behaviour (pale red, back to its place) but not the text shown to the user |
| Desktop panel vs full-screen phone (E10) | Same content, shown as a panel on a computer (over a greyed background) and as a full-screen page on a phone (back arrow instead of a cross) | Demonstrates F-VIS-6 (adaptive layout from lot 1) on a concrete screen, as requested |
| Catalogue (E09): "in season" crops | Demonstration dated 20 April 2027 (lettuce and carrot in season that day) rather than at the real date | On 14 November 2026 (real date), none of the 6 reference crops is in its planting-out window: the demonstration would have been empty |
| Export preview (E11) | Simplified timeline (without the "sowing under cover" phase, sub-rows only if there are several plantings per bed) rather than a complete reproduction of the timeline of E05 | It is a preview in a settings panel, not the main screen; the full timeline is already that of D01 (E05) |
| Link D01 → D02 | The "Edit" button of the simplified panel of D01 (`Main.dc.html`) becomes a link to E06 (D02); no other file of D01 is modified | Needed to close the loop of the requested flow 6.1 (E02 → E05), without touching the already validated content of D01 |
| Reuse of D01 | `plants2.py` copied as is; palette/icons redefined in the `gen.py` of D02 | Avoids redrawing the crops by hand (explicit request); the palette is too small and too coupled to each screen to justify a shared library for now |
