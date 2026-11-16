# Potagerie — Review of edge cases and missed bugs (15 November 2026)

Scope: the code of lot 1 after the move to English (`crates/`, `src-tauri/`, `front/`). Method: new tests that feed the code odd inputs, a catalogue-wide check, a scan for the side effects of the mechanical renaming, and a read of the riskiest paths. Everything marked **fixed** has a test.

## Fixed

| # | Problem | Effect | Fix |
|---|---|---|---|
| 1 | A plan from a file could hold durations of millions of years, or layouts 4 billion years apart | The year view looped for ever or overflowed. An import stores the document, so the app would hang at each start | `crates/plan/src/range.rs`: years 1900–2200, durations up to 3,650 days, checked when a planting is entered, a layout is set and a file is imported (error `out_of_range`). Tests: `plan/tests/extremes.rs` |
| 2 | The renaming turned `fenetre` into `window` and `mesure` into `measure` | A local `window` hid the browser one; `{ ...measure }` spread a function instead of the result (the measures were lost) | Renamed the variables; the spread uses the result |
| 3 | `troncons` and `phases` collapsed into one `segments` key in the timeline bars | The second key silently replaced the first: bars had no phases | The zone is now `zone` |
| 4 | The word-by-word renaming hit library and language names | `sans-serif`, serde's `de` module, `delete`/`export`/`import`/`class`/`new`/`with` as variable names | All repaired; the full test suite and the type check pass |
| 5 | The first SQLite migration had been edited in place | A 0.2.0 database would have failed at the first write | Migration 1 is back as released; migration 2 renames the column; `plan::upgrade_legacy_json` upgrades the French documents (tests, including a real 0.2 database) |
| 6 | The layout tag value became `shown`, the task type stayed `planter` | Wrong wire names | `present` and `plant` |
| 7 | The front tests depended on the language of the machine | They would fail on a French Mac | The tests pin the English messages |

## Fixed after the review (one patch each)

- **0.2.1** — Garlic and spring onion are now suggested in the slots that reach 15 November (S03 v0.12, §9). Test: `a_winter_crop_may_run_past_the_end_of_the_search_window`.
- **0.2.2** — Ties between suggestions are sorted with the crop name shown in the interface language, not the English one (S03 v0.13): Rust also sends the crops tied with the third, the front sorts them again (`front/src/lib/suggestions.ts`).
- **0.2.3** — A damaged database no longer stops the app at launch, and no longer leaves "Loading…" for ever: commands answer `base_unreadable`, the shell shows an explanation with Retry and a way to the settings, and "Erase the app data" removes the damaged file and starts again. Tests in `src-tauri/src/tests.rs` (`a_damaged_database_does_not_stop_the_app_and_can_be_erased`).
- **0.2.4** — The migration is now tested on a real database written by the `v0.2.0` tag (`crates/store/tests/fixtures/potagerie-v0.2.0.db`, test `the_real_0_2_0_database_is_upgraded_and_saved_again`): it opens, reads, passes `check_structure` and is saved again. Still worth one manual look with the installed app.
- **0.2.5** — A winter suggestion (garlic, spring onion) is no longer offered when its bed is already taken next spring: it would have been refused at creation. Test: `a_winter_suggestion_is_not_proposed_when_next_spring_is_taken`.
- **0.3.2** — A database written by a newer app version is reported `base_too_recent` and never erased (it was shown as damaged and offered for erasure). `delete_bed` and `clear_plan` refuse an absurd year (`out_of_range`). The document language follows the interface, catalogue sorts follow it too, long / emoji / double-space bed names are handled in the timeline, and the header no longer wraps onto a third row at 390 px in French (S05 v0.15).
- **Tests added**: a random walk of 25 seeds × 150 operations keeps the plan sound (the only expected exception is a bed shortened under a planting, S03 §17), and every suggestion of every free slot, in 12 random states of the plan, can be created without refusal.
- **0.3.3** — The PNG / PDF export no longer shows empty boxes for emoji or symbols (`≈`, `⅓`, `✓`) in bed names and notes: the characters the embedded fonts lack are removed from the exported file (S05 v0.16 §16). Accented French text was already drawn correctly.

## Known problems, not fixed

1. **The canvases on claude.ai** (D01, D02) are still the French ones: the files in the repository are the reference, the canvases have to be re-imported by hand.
2. **Not covered**: the real Tauri window (WebKit, WebView2) and touch were never exercised by these checks, only headless WebKit and Chrome; Undo / Redo is not built (F-PLN-6); the PDF has no header, plan thumbnail or task calendar yet (S01 F-EXP-2).

## What was checked and found sound

- Extreme years, dates (0001, 9999, 29 February), odd JSON, and idempotence of the legacy upgrade.
- All 67 crops over about 1,600 slots each: every suggestion makes a valid planting, ends inside its slot and never conflicts in an empty bed.
- Date arithmetic saturates instead of panicking (`days.rs`).
- Error codes, message keys and `bindings.ts` agree (a missing translation breaks the type check).
