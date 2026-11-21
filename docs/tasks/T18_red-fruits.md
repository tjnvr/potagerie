# T18 · Red fruits in the catalogue: raspberry, strawberry and others

- Status: Running
- Owner: agent
- Depends on: —
- Refs: S02 §1, §3 bis, §4 · S01 F-PLA (plantings), F-VIS-1 (garden view), F-VIS-2 (timeline), "Wider catalogue" (after V1) · S03 §2 (dates), §5 (conflicts), §7 (free slots)
- From: Tim (chat)

## Context

Tim asked on 2026-11-21 to add the red fruits to the catalogue: raspberry, strawberry, and the like.

The catalogue holds 67 annual crops (S02 v0.4). S02 §1 and S01 put perennials out of V1 on purpose: "growing them over several years does not fit the schema". A planting today lives within one season (sowing or planting out, growth, harvest, end), and a bed segment is free again after it. A raspberry cane stays about ten years and gives fruit every summer; a strawberry bed is usually kept three years. So the task is not only data: the model needs a planting that spans several years, and the timeline, the garden view, conflicts, free slots and the export must show it.

Tim asking for it is the go-ahead to bring this part of the "wider catalogue" forward. What is left for him is only how it looks and behaves (question 1). Everything technical is the agent's: the model change, S02 / S03 / S05 versions, the data of each crop, the drawings (in the style of the existing ones; see also T17 on look-alike icons).

## Goal

The user can plant red fruit bushes and strawberries on a bed, see them in the garden and the timeline year after year, and plan the free space around them.

## Acceptance

- [ ] S01 / S02 / S03 / S05 updated per `spec-driven-delivery`, with §12 cases for a planting over several years (conflict with an annual crop in year 2, free slot that ignores the perennial's segments, ending a perennial).
- [ ] The crops chosen in question 1 in `crops.json`, with family, spacing, planting window, harvest period each year, lifespan, frost behaviour; their drawings in the garden and the catalogue.
- [ ] Rust tests for the rules, `UC-xx` e2e tests for planting a raspberry and seeing it the next summer.
- [ ] If the work is too big for one slice, the agent splits it into linked tasks (model first, then the crops) and ships the first.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `crates/catalogue/`, `crates/plan/`, `src-tauri/src/`, `front/src/lib/`, `front/scripts/`, `front/messages/`, `front/e2e/`
- `docs/specs/`, `docs/POT-00_*`

## Questions

1. [functional] How should a raspberry or strawberry planting behave, and which red fruits do you want?
   Behaviour:
   (a) Planted once, it stays on its part of the bed year after year until you end it (or until its usual lifespan: about 10 years for raspberry, 3 for strawberry, then the app suggests renewing it). The timeline shows one long bar with the harvest highlighted each summer; the garden shows the bushes every year, bare in winter, in fruit in summer. Nothing else can be planted on its segments meanwhile.
   (b) Treated like the vegetables: one planting per year, to be re-entered each year. Simpler, but a raspberry would vanish from the garden every winter.
   Crops: raspberry, strawberry, redcurrant, blackcurrant, gooseberry, thornless blackberry, blueberry (in a planter: it needs acid soil). Say which to leave out.
   Recommendation: (a), with the whole list.
   Answer: a

## Log
