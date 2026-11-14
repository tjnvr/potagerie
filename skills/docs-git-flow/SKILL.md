---
name: docs-git-flow
description: Prepare git commits for Potagerie specs and designs (branches, messages, pre-commit checks). Use before committing anything in docs/.
---

# Docs git flow

Claude never credits itself as co-author: no `Co-Authored-By` trailer, no "generated with" line, in commits, tags or PRs.

**Branch**: `docs/<code>-<topic>` (e.g. `docs/S02-catalogue`). Small fixes go straight to `main`.

**Message**: `<type>(<code>): <summary>`
- Types: `spec`, `design`, `docs` (index, layout), `fix`.
- Summary in English, imperative mood, under 72 chars, starting with the version when it changes.
- e.g. `spec(S01): v0.9 — add continuous timeline` · `design(D01): v1.0 — validate key screens`

**Rules**
- One commit per document version, containing the new file, the previous one moved to `archives/`, and the updated index.
- Add files by explicit path, never `git add -A`.

**Check before proposing**
1. One current version per document; older ones in `archives/`.
2. Header (version, status) matches the file name.
3. `docs/POT-00_index.md` is up to date.
4. Design folder is in sync with the canvas.
5. Nothing stray (`.DS_Store`, `_to_delete/`).
