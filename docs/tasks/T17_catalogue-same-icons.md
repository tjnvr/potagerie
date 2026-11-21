# T17 · Crops that look the same in the catalogue

- Status: Draft
- Owner: agent
- Depends on: —
- Refs: S02 (look-alike drawings, `appearances.ts`) · S01 §5 (visual direction)
- From: T15 (agent)

## Context

Seen on 2026-11-21 while comparing the plant drawings for T15 (screenshot `docs/tasks/T15/after-catalogue.png`, 1440 px, `?example`).

S02 says that a crop without its own drawing reuses the drawing of a close crop, in the diorama and in the icons (`front/src/lib/appearances.ts`); only the timeline gives it its own colour. In the catalogue grid this makes several neighbouring cards look identical: artichoke, broccoli, Brussels sprout and cauliflower share the same dark spiky rosette (the cauliflower shows no white head); arugula, basil and chervil share the same green ball. The crop colour stored in `appearances.ts` is not used by the catalogue icon.

This is the rule as written, not a bug, so changing it is Tim's call.

## Goal

In the catalogue, crops that share a drawing can still be told apart at a glance.

## Acceptance

- [ ] The option chosen under `Questions` is applied to the catalogue icons (and the diorama, if the answer says so).
- [ ] S02 updated (new version) if the rule changes.
- [ ] Before / after screenshots of the catalogue at 1440 px under `docs/tasks/T17/`.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/appearances.ts`, `front/src/lib/planting/PlantIcon.svelte`, `front/src/lib/diorama/symboles.ts`, `front/scripts/plant_art.py`
- `docs/specs/POT-S02_crop-catalogue_v0.4.md` (new version if the rule changes)

## Questions

- [functional] In the catalogue, several crops show exactly the same small drawing (for example artichoke, broccoli, Brussels sprout and cauliflower). What should the user see?
  1. Keep it as it is: close crops share one drawing; the name under the card tells them apart.
  2. Same shared drawing, tinted with each crop's own colour (a white-ish head for the cauliflower, a pale leaf for the chervil…), in the catalogue and in the garden.
  3. Draw a dedicated picture for the crops grown most often (cauliflower, artichoke, basil…), the others keep sharing.
  Recommendation: 2, cheap and consistent with the timeline colours; 3 later if some crops still look wrong.

## Log
