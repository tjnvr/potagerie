# POT-D01 · Key screens V1 — v1.2

> Version: 1.2 (draft) · Date: 2026-11-15 · Author: Tim
> Canvas: https://claude.ai/artifact/DoHRUtcujzwPF6bfjdQtSN
> Specs: [S01 v1.6](../../specs/POT-S01_functional-spec_v1.7.md) · [S02 v0.4](../../specs/POT-S02_crop-catalogue_v0.4.md) · [S03 v0.13](../../specs/POT-S03_rules-and-calculations_v0.14.md)
> v1.2 is the English translation of v1.1: texts, files and generator are in English. The layout does not change.

## Changes since v1.0 (kept from v1.1)

- No more account: the avatar of the header is removed.
- No more sunlight, obstacles or decor: the editor (E03) no longer shows the sunlight map, the apple tree, the water tank, the compost bin or the compass. Its tray only offers Bed, Square, Mound and Planter, and the summary shows the number of beds.
- No more frost: the frost periods are removed from the timeline, its legend and the visual direction.
- Only the 6 reference crops of V1 (S02): tomato, lettuce, carrot, bean, zucchini, leek. The dates of the timeline and of the panel follow the rules of S03, and the quantities are recomputed.
- The phone layout (E05, E07) serves lot 2.

| File | Screen |
|---|---|
| `project/canvas.json` | Canvas index (positions, titles, notes) |
| `project/Main.dc.html` | E05 · Garden — computer (entry sheet of the canvas) |
| `project/E05_garden_mobile.dc.html` | E05 · Garden — phone |
| `project/E07_planting_mobile.dc.html` | E07 · New planting — phone |
| `project/DV01_visual-direction.dc.html` | DV01 · Visual direction |
| `project/E03_plan-editor_desktop.dc.html` | E03 · Plan editor — computer |
| `gen.py`, `plants2.py` | Generator of the sheets and crop drawings (`python3 gen.py`) |
| `exports/` | PNG and PDF exported from the canvas |

The online canvas is the reference. This folder is its versioned copy: after touch-ups in the canvas, you resynchronise then commit.
`gen.py` regenerates everything from scratch and overwrites the touch-ups made by hand in the canvas.

## Flow

The sheets are linked together: the Play button of a sheet lets you follow the links.

| From | To | Trigger |
|---|---|---|
| ① E03 Plan editor | ② E05 Garden (computer) | "Garden" tab or "See the garden" button |
| ② E05 Garden (computer) | ① E03 Plan editor | "Plan" tab |
| ② E05 Garden (computer) | ③ E07 New planting | "Plant here", or a click on a free slot of the timeline |
| ② E05 Garden (phone) | ③ E07 New planting | "+" button of the bed |
| ③ E07 New planting | ② E05 Garden (phone) | "Plant" or the cross |
