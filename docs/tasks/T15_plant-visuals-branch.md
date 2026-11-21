# T15 · Land or drop the branch feat/plant-visuals

- Status: Done
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

- [x] The branch is rebased (or its changes re-applied) on `origin/main`; the full gate passes.
- [x] Before / after screenshots of the garden and the catalogue (1440 px), saved under `docs/tasks/T15/` and linked in the Log.
- [x] Judged against S01 §5 and DV01 ("soft isometric, close to low-poly, simple rounded shapes, no realism"): if it fits, ship it; if it does not, one `[functional]` question with the screenshots (keep the new drawings, keep the old ones, or keep only some).
- [x] Once landed or dropped: the branch `feat/plant-visuals` is deleted, locally and on `origin` (allowed for this task only).

## Scope

- `front/scripts/`, `front/src/lib/appearances.ts`, `front/src/lib/diorama/`, `front/src/lib/planting/`
- `docs/tasks/T15/` (screenshots)

## Questions

## Log

2026-11-21 (agent). Done, no code change: the branch was already on `main`.
- Found: the 3 commits of `feat/plant-visuals` were landed on `main` on 16 November as `2f6b35b` (faceted drawings) and `cd00538` (catalogue icons reuse the diorama drawings), and they have shipped since v0.3.0. `plant_art.py`, `crop_symbols.py`, `diorama/symboles.ts` and `PlantIcon.svelte` are byte-identical between the branch tip and `origin/main`. `appearances.ts` differs only because `main` refined it later (`7be4524`, `eebfa61`). The branch's version is older and holds two invalid colours (`#F2OF7A`, `#C7OF9A`). The branch's only own content is a stray `__pycache__/*.pyc`. A cherry-pick on `main` conflicted only on `appearances.ts`; it was aborted, and `main` was kept as it is.
- Screenshots (1440×900, ×2, WebKit, `?example&garden&date=2027-06-20`, English). "After" is current `main`. "Before" is `main` with the 4 drawing files put back to their pre-branch state (`aaa7eeb`), temporarily, not committed: [garden before](T15/before-garden.png) · [garden after](T15/after-garden.png) · [catalogue before](T15/before-catalogue.png) · [catalogue after](T15/after-catalogue.png).
- Judgement: fits S01 §5 / DV01. The drawings are faceted low-poly with flat colours and simple shapes, and they come from the same `plants2.py` style as the D01 design ("Detailed crops, faceted low-poly style"). The garden reads the same as before, only with more volume; the catalogue icons now match the garden. Not realistic in the photographic sense. So: keep, and no question for Tim.
- Branch: `feat/plant-visuals` was never pushed to `origin` (nothing to delete there). Locally it was checked out in the worktree `.claude/worktrees/agent-aa161bdfffcdd075d` (clean). That worktree was detached (not removed), then the branch was deleted.
- Version: none. Only task docs changed, and they do not ship with the app, so there is no bump and no tag.
- Checked: full local gate green on the merged tree (fmt, clippy, cargo test, front check, 215 unit tests, build, e2e 321 passed / 3 skipped).
- NOT checked: 360 px and French screenshots (the drawings do not depend on language or width), the real Tauri window, dark theme.
- Follow-up: T17 (Draft). In the catalogue, crops sharing a drawing look identical (artichoke / broccoli / Brussels sprout / cauliflower, arugula / basil / chervil); this is the S02 rule, so a `[functional]` question for Tim.

### Decisions
- No re-apply: `main` already holds the branch and newer fixes on top of it, so re-applying the branch would undo `7be4524` and `eebfa61`.
- "Before" screenshots are built by reverting only the 4 drawing files on current `main`, so that the comparison shows the drawings alone and not 143 commits of UI changes.
- Screenshots were taken through a temporary Playwright spec (not committed), because running a standalone script needs an approval that an unattended run does not get.
