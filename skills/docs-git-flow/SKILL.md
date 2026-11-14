---
name: docs-git-flow
description: Prepare git commits for Potagerie specs, designs, prototypes and application code (branches, messages, pre-commit checks). Use before committing anything in the repo.
---

# Docs git flow

Claude never credits itself as co-author: no `Co-Authored-By` trailer, no "generated with" line, in commits, tags or PRs.

**Branch**: `docs/<code>-<topic>` (e.g. `docs/S02-catalogue`). Small fixes go straight to `main`.

**Message**: `<type>(<code>): <summary>`
- Types: `spec`, `design`, `proto`, `docs` (index, layout), `fix`.
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

## Application code

Applies to everything outside `docs/` and `skills/` (`Cargo.toml`, `crates/`, `src-tauri/`, `front/`, `.github/`).

**Branch**: `feat/<scope>-<topic>` for a feature, `fix/<scope>-<topic>` for a bug fix (e.g. `feat/plan-dates`). Never commit code straight to `main`.

**Message**: `<type>(<scope>): <summary>`
- Types: `feat`, `fix`, `test`, `refactor`, `build` (Cargo, pnpm, Tauri config), `ci`.
- Scope: the crate name (`plan`, `catalogue`, `store`, `export`), `app` for `src-tauri/`, `front` for `front/`, `ci` for workflows.
- Summary in English, imperative mood, under 72 chars, citing the spec section when it implements one.
- e.g. `feat(plan): compute series dates (S03 §2)` · `test(plan): add S03 cases T1 to T13`

**Check before proposing**
1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.
2. When `front/` changed: `pnpm check` and `pnpm test` pass.
3. Nothing generated or stray: no `target/`, `node_modules/`, `dist/`, `.DS_Store`. `bindings.ts` is committed only when it is regenerated from Rust.
4. Spec tests are copied from the spec, never adjusted to match the code. A disagreement means a new spec version.
