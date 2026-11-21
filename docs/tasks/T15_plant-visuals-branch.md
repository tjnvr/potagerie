# T15 · Land or drop the branch feat/plant-visuals

- Status: Ready
- Owner: agent
- Depends on: —
- Refs: S01 §5 (visual direction) · D01 (DV01) · S02 · git branch `feat/plant-visuals`
- From: chat

## Context

Created on 2026-11-20: `feat/plant-visuals` (3 commits, 16 November, never merged) redraws 14 crops in a faceted style, makes the catalogue icons reuse the diorama drawings and points the look-alikes to the new crops (`front/scripts/plant_art.py`, `crop_symbols.py`, `appearances.ts`, `diorama/symboles.ts`, `PlantIcon.svelte`). `main` has moved a lot since (v0.7.x).

Reworked on 2026-11-21 (chat): first owned by Tim as "decide". The agent does the work and the comparison; Tim only decides if the look departs from the visual direction.

## Goal

The branch's drawings are either in `main` (if they fit the visual direction) or dropped, and the branch is gone.

## Acceptance

- [ ] The branch is rebased (or its changes re-applied) on `origin/main`; the full gate passes.
- [ ] Before / after screenshots of the garden and the catalogue (1440 px), saved under `docs/tasks/T15/` and linked in the Log.
- [ ] Judged against S01 §5 and DV01 ("soft isometric, close to low-poly, simple rounded shapes, no realism"): if it fits, ship it; if it does not, one `[functional]` question with the screenshots (keep the new drawings, keep the old ones, or keep only some).
- [ ] Once landed or dropped: the branch `feat/plant-visuals` is deleted, locally and on `origin` (allowed for this task only).

## Scope

- `front/scripts/`, `front/src/lib/appearances.ts`, `front/src/lib/diorama/`, `front/src/lib/planting/`
- `docs/tasks/T15/` (screenshots)

## Questions

## Log
