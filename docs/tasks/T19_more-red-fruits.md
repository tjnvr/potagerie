# T19 · The other red fruits, and drawings of their own

- Status: Ready
- Owner: agent
- Depends on: T18
- Refs: S02 §3 ter · S03 §2 bis, §3 · S01 F-CAT-1, F-VIS-1
- From: T18 (agent)

## Context

T18 split Tim's request (red fruits in the catalogue, answer (a): planted once, kept year after year) in two slices. T18 shipped the model (S03 §2 bis: a planting over several years, a harvest each year, bare in winter, never suggested) with raspberry and strawberry. Left for this task:

1. **The five other red fruits Tim kept** (he left none out of the list): redcurrant, blackcurrant, gooseberry, thornless blackberry, blueberry. Add them to `crates/catalogue/crops.json` and to S02 §3 ter (new version), with family (Grossulariaceae for the currants and gooseberry, Rosaceae for the blackberry, Ericaceae for the blueberry), spacing, planting-out window, harvest after, harvest from, harvest length, lifespan (1 to 20 years, S03 §17), frost; names in `front/messages/en.json` and `fr.json` (`crop_<id>`, `family_<family>`). The question said "blueberry (in a planter: it needs acid soil)": T18's reading is a line on the catalogue sheet saying so, not a refusal on other beds (a refusal would be a new behaviour S01 does not describe).
2. **Drawings of their own** for the seven red fruits in `front/scripts/plant_art.py`, in the style of the others, with **stage 0 drawn bare** (canes or a low brown rosette): S03 §3 shows a perennial in winter with its stage 0 (`dormant` → `plant-<crop>-sowing`, `front/src/lib/diorama/animation.ts`). Then add them to `DRAWN` in `front/src/lib/appearances.ts` (raspberry and strawberry borrow the tomato and the lettuce since T18) and regenerate `front/src/lib/diorama/symboles.ts` with `python3 front/scripts/crop_symbols.py`. T18 could not run Python in its session, so this step was not done there. See also T17 on look-alike icons.

The catalogue count (UC-08, "69 crops") and `NEVER_SUGGESTED` in `crates/plan/tests/catalogue_wide.rs` grow with the new crops.

## Goal

The user finds all seven red fruits in the catalogue, each with its own drawing, bare in winter in the garden.

## Acceptance

- [ ] S02 new version with the five crops in §3 ter; `crops.json`, `red_fruits_match_s02` extended.
- [ ] Seven red fruit drawings generated in `symboles.ts`, stage 0 bare; screenshots of the catalogue and of the garden in January and July under `docs/tasks/T19/`.
- [ ] UC-08 count updated; UC-34 asserts the raspberry's own drawing (`plant-raspberry-…`).
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `crates/catalogue/`, `front/scripts/plant_art.py`, `front/src/lib/appearances.ts`, `front/src/lib/diorama/symboles.ts`, `front/messages/`, `front/e2e/`
- `docs/specs/POT-S02_*`

## Questions

## Log
