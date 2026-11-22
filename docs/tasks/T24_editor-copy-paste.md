# T24 · Plan editor: duplicate, copy and paste a bed

- Status: Running
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

### Decisions
- One command, `paste_bed(year, name, original)`, for Duplicate and Paste: Duplicate is a paste of the bed as it is now; Rust does the whole search, the interface computes no spot (rules in Rust).
- The spot rule (S03 v0.21 §15): right, below, left, above the original, one aisle away and aligned; then the plot row by row on the cell grid; a spot must keep the aisle with every bed in every checked year, so a copy never asks "Keep here". `too_many_beds` is checked before the spot, so at 10 beds the message is about the 10 beds.
- "Next to the original" in another year means next to where the bed was when copied (the copy keeps its geometry): the bed may have moved, or not exist, in the year of the paste.
- The copied bed lives in the memory of the interface (`app.copiedBed`), not in the system clipboard: the system clipboard holds text, and a field keeps its own Cmd+C / Cmd+V. A message says the bed is copied, since a copy shows nothing else.
- Cmd+C with no selection and Cmd+V with nothing copied are left to the system; Cmd+D is always taken in the editor (a browser bookmark otherwise). In a past year, a bed can be copied but nothing is pasted (no button there either).
- The bar under the selected bed holds Duplicate only; Delete in the bar is T27 (S01 has it, not in this task), Rotate is after lot 1 (T07). It goes above the bed near the bottom of the plan, and hides while a bed is in hand.
- S01 v1.11 writes down the behaviour Tim decided (copy, paste across years, shortcuts) in F-PLN-6, and moves duplicate out of "after lot 1"; S03 v0.21 the spot rule and case T21; S05 v0.24 the command, `no_free_spot` and the copied bed (section 20).

### Progress
- 2026-11-22: tests written (S03 T21, placement extremes, IPC `a_pasted_bed_lands_next_to_the_original_and_is_undone`, unit `shortcuts`, `cmToPixels`, e2e UC-38 to UC-41), code passing. Extra paths: `src-tauri/capabilities/main.json` (permission of the new command), `front/src/lib/shell/Icon.svelte` (copy icon of D01), `docs/design/*/README.md` and `README.md` (spec links). Follow-up created: T27 (Delete in the bar).
