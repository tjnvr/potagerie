---
name: docs-naming
description: Name, version and file Potagerie specs and designs (POT-TNN convention). Use when creating, renaming or versioning a document in docs/.
---

# Docs naming

**Format**: `POT-<T><NN>_<topic>_v<X.Y>`
- `T`: `S` spec (`.md` file), `D` design (folder), `P` prototype (folder).
- `NN`: 2-digit number per type, never reused.
- `topic`: lowercase, no accents, words separated by hyphens.
- `v0.Y` is a draft, `v1.0` is validated. Later changes, scope included, bump Y (`v1.1`, `v1.2`).
- X moves to `v2.0` only once a first version of the app has shipped (lot 1 released).

**Layout**
- Everything lives under `docs/`: `specs/`, `design/`, `prototypes/`, and the index `POT-00_index.md`.
- One current version per document; the previous one moves unchanged to the sibling `archives/` folder.
- The `Version · Date · Auteur` header matches the name. For a design or prototype, it sits in its `README.md`.
- Update the index on every creation or new version.

**Designs**
- A folder holds `README.md`, `project/` (copy of the canvas) and `exports/`.
- Artboards: `<E05|DV01>_<screen>_<desktop|mobile>.dc.html`; the entry artboard stays `Main.dc.html`.
- The canvas title starts with the code ("POT-D01 · …").

**Prototypes**
- Throwaway, self-contained code answering **one** question.
- `README.md` holds the question, protocol, results and decision.
