# T20 · Suggest renewing a red fruit at the end of its lifespan

- Status: Draft
- Owner: agent
- Depends on: T18
- Refs: S01 F-CAT-1, F-VIS-7 · S03 §2 bis
- From: T18 (agent)

## Context

Tim's answer to T18 (option (a)) says a red fruit stays on its bed "until you end it (or until its usual lifespan: about 10 years for raspberry, 3 for strawberry, then the app suggests renewing it)". T18 shipped the lifespan: the planting ends by default `lifespan_years` after its planting out (S03 §2 bis), and its segments are free afterwards. Nothing tells the user yet that the lifespan is reached: the bar just ends. How the app suggests it is something the user sees, and S01 does not describe it.

## Goal

When a red fruit reaches the end of its usual lifespan, the app tells the user and lets them renew it in one step.

## Acceptance

- [ ] The option chosen below, in S01 (new version) and in the app.
- [ ] A `UC-xx` e2e test: a strawberry planted in 2027 shows the suggestion in 2030, and renewing it creates the new planting on the same segments.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/editor/PanelBed.svelte`, `front/src/lib/diorama/PanelDiorama.svelte`, `front/src/lib/planting/`, `front/messages/`, `front/e2e/`
- `docs/specs/POT-S01_*`

## Questions

1. [functional] How should the app suggest renewing a raspberry or a strawberry bed at the end of its lifespan?
   (a) In the bed panel, during its last year: "Strawberry · 3 years on this bed: time to renew it", with a "Renew" button that opens the planting form already filled in (same crop, same segments, starting the day it ends). Nothing changes on its own.
   (b) The same, and also a small mark at the end of its bar in the timeline.
   (c) Only the mark in the timeline, no button: the user plants again as usual.
   Recommendation: (a), the panel already lists the plantings of the bed and "Plant here".

## Log
