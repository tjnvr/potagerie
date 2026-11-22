# Potagerie — Documentation index

## Layout

```
docs/
├── POT-00_index.md                         ← this file
├── POT-00_lot1-coverage.md                 ← state of the F-… requirements at lot 1
├── POT-00_review-2026-11.md                ← edge cases and bugs found in a review
├── POT-00_test-matrix.md                   ← use cases and their automated tests (e2e), first navigation analysis
├── tasks/                                  ← work handed to agents, one file per task (see tasks/README.md)
├── specs/
│   ├── POT-S01_functional-spec_v1.11.md     ← current versions
│   ├── POT-S02_crop-catalogue_v0.5.md
│   ├── POT-S03_rules-and-calculations_v0.20.md
│   ├── POT-S05_technical-spec_v0.24.md
│   └── archives/                           ← previous versions (the ones before v1.6 / v0.4 / v0.11 / v0.12 are in French)
├── design/
│   ├── POT-D01_key-screens_v1.2/           ← current version (one folder per canvas)
│   ├── POT-D02_additional-screens_v0.2/
│   └── archives/                           ← previous versions
└── prototypes/
    ├── POT-P01_iso-rendering_v0.2/         ← one folder per prototype
    └── archives/
```

## Naming convention (the same for specs, designs and prototypes)

```
POT-<T><NN>_<subject>_v<X.Y>      (.md for a spec, folder for a design or a prototype)
```

| Element | Rule | Example |
|---|---|---|
| `POT` | Prefix of the Potagerie project | `POT` |
| `<T>` | Type: `S` spec, `D` design, `P` prototype | `S`, `D`, `P` |
| `<NN>` | 2-digit number, specific to each type, assigned once and for all, never reused | `01` |
| `<subject>` | Lowercase English, no accents, words separated by hyphens | `functional-spec`, `key-screens` |
| `v<X.Y>` | Version: `v0.Y` draft, `v1.0` validated. Later changes, even of scope, advance Y (`v1.1`). X only moves to 2 once a first version of the app has shipped (lot 1 released) | `v0.5`, `v1.1` |

Common rules:
- **A single current version** of each document in `specs/`, `design/` or `prototypes/`. At each new version, the previous one is moved as is into the neighbouring `archives/` folder.
- **Header**: each spec, and the `README.md` of each design or prototype folder, starts with `Version · Date · Author`, consistent with the name.
- **Index**: this file is updated at each creation, renaming or new version.
- **Tasks**: `tasks/T<NN>_<topic>.md`, `NN` never reused, no version in the name; the task's `Log` keeps its history. Run and followed with `scripts/tasks.sh`.
- **Language**: everything is written in English (documents, code, comments, commit messages). The archives keep the language they were written in.
- The title of the canvas on claude.ai takes up the code: "POT-D01 · Key screens V1".

Specific to designs:
- Content of a folder: `README.md`, `project/` (exact copy of the files of the canvas), `exports/` (PNG, PDF), and an optional generator.
- Sheets: `<screen-code>_<screen>_<medium>.dc.html`, with the screen code of the spec (`E05`, `E07`) or `DV<NN>` for the visual direction, and `desktop` or `mobile` as the medium. Exception: the entry sheet of the canvas is always called `Main.dc.html`, because the tool requires it.
- The exports carry the name of their sheet: `exports/E05_garden_mobile.png`.

Specific to prototypes:
- Throwaway, standalone code (opens without installation), that answers **one** precise question.
- Exception: when the question is about the stack itself (for example Tauri + Svelte), the prototype may require a build. Its `README.md` then gives the install and launch commands, and a `.gitignore` excludes the dependencies and the build output.
- The `README.md` contains: the question, the protocol, the results and the decision taken.

## Documents

| Code | Document | Version | Status | Link |
|---|---|---|---|---|
| S01 | V1 functional spec (installed app, no account, desktop, mobile and sync lots) | 1.11 | Draft | [specs/POT-S01_functional-spec_v1.11.md](specs/POT-S01_functional-spec_v1.11.md) |
| S02 | Crop catalogue (67 annual crops and the first red fruits, 6 reference) | 0.5 | Draft | [specs/POT-S02_crop-catalogue_v0.5.md](specs/POT-S02_crop-catalogue_v0.5.md) |
| S03 | Rules and calculations (dates, stages, conflicts, quantities, layouts, slots, suggestions, tasks, validation after a merge; plot and placement of the beds; sunlight in an annex) | 0.20 | Draft | [specs/POT-S03_rules-and-calculations_v0.20.md](specs/POT-S03_rules-and-calculations_v0.20.md) |
| S05 | Technical spec (Rust + Svelte, desktop then mobile Tauri app, serverless CRDT sync) | 0.24 | Draft | [specs/POT-S05_technical-spec_v0.24.md](specs/POT-S05_technical-spec_v0.24.md) |
| P01 | Isometric rendering prototype (SVG or Canvas?) | 0.2 | Draft | [prototypes/POT-P01_iso-rendering_v0.2/](prototypes/POT-P01_iso-rendering_v0.2/README.md) · [page](https://claude.ai/artifact/PBrzt3qta6sHKisnhfZWMv) |
| D01 | Key V1 screens (E03 editor, E05 computer and phone, E07, visual direction) | 1.2 | Draft | [design/POT-D01_key-screens_v1.2/](design/POT-D01_key-screens_v1.2/README.md) · [canvas](https://claude.ai/artifact/DoHRUtcujzwPF6bfjdQtSN) (older French version, not maintained) |
| D02 | Additional V1 screens (E02, E06, E07 computer, E09, E10 computer and phone, E11, cross-cutting states) | 0.2 | Draft | [design/POT-D02_additional-screens_v0.2/](design/POT-D02_additional-screens_v0.2/README.md) |

## Reserved codes

| Code | Planned document |
|---|---|
| S04 | Visual direction |

D02 was reserved for "Plan editor and export"; the editor was finally covered by D01 (E03), and D02 became the remaining additional screens — see its Decisions section.
