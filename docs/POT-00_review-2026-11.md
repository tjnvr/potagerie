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

## Known problems, not fixed

1. **An unreadable database shows "Saving failed" and an endless "Loading…"**, with a Retry that cannot help. The Settings screen still opens, so "Erase the app data" recovers, but nothing says so. A dedicated "data unreadable" state is missing.
2. **The version 0.2.0 release was built before the renaming.** It is the last version that writes French names. The first release with English names must be tested by hand on a real 0.2.0 database (the migration is only tested on a database built by the tests).
3. **The canvases on claude.ai** (D01, D02) are still the French ones: the files in the repository are the reference, the canvases have to be re-imported by hand.
4. **Not covered**: the real Tauri window (WebKit, WebView2) and touch were never exercised by these checks, only headless WebKit and Chrome; Undo / Redo is not built (F-PLN-6); the PDF has no header, plan thumbnail or task calendar yet (S01 F-EXP-2).

## What was checked and found sound

- Extreme years, dates (0001, 9999, 29 February), odd JSON, and idempotence of the legacy upgrade.
- All 67 crops over about 1,600 slots each: every suggestion makes a valid planting, ends inside its slot and never conflicts in an empty bed.
- Date arithmetic saturates instead of panicking (`days.rs`).
- Error codes, message keys and `bindings.ts` agree (a missing translation breaks the type check).
