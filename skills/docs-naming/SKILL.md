---
name: docs-naming
description: Name, version and file Potagerie specs and designs (POT-TNN convention). Use when creating, renaming or versioning a document in docs/.
---

# Docs naming

**Format**: `POT-<T><NN>_<topic>_v<X.Y>`
- `T`: `S` for a spec (`.md` file), `D` for a design (folder).
- `NN`: 2-digit number per type, never reused.
- `topic`: lowercase, no accents, words separated by hyphens.
- `v0.Y` is a draft, `v1.0` is validated; X changes when the scope changes.

**Layout**
- Everything lives under `docs/`: `specs/`, `design/`, and the index `POT-00_index.md`.
- One current version per document; the previous one moves unchanged to the sibling `archives/` folder.
- The `Version · Date · Auteur` header matches the name. For a design, it sits in its `README.md`.
- Update the index on every creation or new version.

**Designs**
- A folder holds `README.md`, `project/` (copy of the canvas) and `exports/`.
- Artboards: `<E05|DV01>_<screen>_<desktop|mobile>.dc.html`; the entry artboard stays `Main.dc.html`.
- The canvas title starts with the code ("POT-D01 · …").
