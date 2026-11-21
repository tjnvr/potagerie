# T24 · Plan editor: duplicate, copy and paste a bed

- Status: Ready
- Owner: agent
- Depends on: T08
- Refs: S01 F-PLN-6 (duplicate), F-PLN-1 (10 beds, names) · S03 placement · D01 E03
- From: Tim (chat)

## Context

Tim, 2026-11-21: wants keyboard actions in the plan editor, among them copy and paste. Duplicating a bed was in T07 with the rotation; T07 now keeps the rotation only (Tim said later), and duplicate comes here.

Behaviour, as decided with Tim: "Duplicate" in the bar under the selected bed and in its panel; Cmd+C / Ctrl+C copies the selected bed, Cmd+V / Ctrl+V pastes a copy, Cmd+D / Ctrl+D duplicates. The copy has the same type and size, no plantings, and a default name ("Bed C"). It lands on the first free spot next to the original (then anywhere on the plot), keeping the aisle; if there is none, or at 10 beds, nothing is added and a message says why. The copy is selected. Pasting works across years (copy in 2027, paste in 2028). Shortcuts do nothing while typing in a field. Paste and duplicate are undoable (T08).

## Goal

A bed can be duplicated with a button or the keyboard, and copied and pasted, landing on a free spot.

## Acceptance

- [ ] Rust: a command (or the existing create) places the copy on the first free spot by a rule written in S03 with §12 cases (next to the original, then scan; aisle kept; refused at 10 beds).
- [ ] Front: Duplicate button in the bar and the panel; Cmd/Ctrl+C, V, D; the copy is selected; messages in en and fr.
- [ ] e2e tests: duplicate by button and by shortcut, copy in one year and paste in another, paste refused at 10 beds, shortcut ignored in the name field.
- [ ] Matrix rows, ids free on `origin/main`.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `crates/plan/`
- `src-tauri/src/`
- `front/src/lib/`
- `front/messages/`
- `front/e2e/`
- `docs/specs/`
- `docs/POT-00_*`

## Questions

## Log
