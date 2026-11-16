---
name: spec-driven-delivery
description: Deliver a change to Potagerie spec first, then code, tests, functional check, tag and release. Use for any feature, fix or rule change.
---

# Spec-driven delivery

The specs in `docs/specs/` are the source of truth. Code follows them, never the reverse.
One change = one slice, shipped on its own. Small slices, one by one.

## Versioning (semver)

- **patch** (`0.2.x`): a fix, a test, a doc or site change that ships with the app.
- **minor** (`0.x.0`): a new feature or a visible behaviour change.
- **major**: only the human decides and asks for it. Never bump a major on your own. The specs
  move to v2.0 with the first `v1.0.0` of the app, not before.

## 1. Frame
- Name the requirement (S01 `F-…`), the rule (S03 §) and the contract (S05 §).
- Unclear or contradictory spec: ask before coding. Decisions go in the spec, not in the chat.

## 2. Spec first
- Edit the spec that owns the rule and bump its version: archive the old file in `archives/`,
  update the index `docs/POT-00_index.md` and every cross-reference.
- A rule change adds or updates a case in S03 §12.
- Update `docs/POT-00_lot1-coverage.md` (done / partial / postponed).

## 3. Test first (Rust, then front)
- Write the failing test from the spec: `crates/*/tests/`, then `front/src/**/*.test.ts`.
- A rule or an edge case gets a test with the odd input (extremes, other year, other language).
- A change of the stored format needs a migration and a test on a real old database.

## 4. Implement
- The Rust core (`crates/`) owns the rules, `src-tauri` exposes commands, `front/` only displays.
- New command or error code: regenerate `bindings.ts`
  (`UPDATE_BINDINGS=1 cargo test -p potagerie bindings`) and add the message to
  `front/messages/en.json` AND `fr.json`.
- Everything in English (code, comments, commits, docs); UI texts only in `messages/`.

## 5. Local gate: nothing is pushed to `main` unless ALL of it passes
`cargo fmt --all --check` · `cargo clippy --workspace --all-targets -- -D warnings` ·
`cargo test --workspace` · `pnpm -C front check` · `pnpm -C front test` · `pnpm -C front build` · `pnpm -C front e2e`.
Same list as `.github/workflows/ci.yml`. One failure = fix first, then run the whole list again.

## 6. Functional check
- Run the app (`make dev`), or the front with `?example` in headless WebKit: walk the user path
  and look at the result (screenshots at 1440 and 360 px, in English and French).
- A new user-visible behaviour gets a row in `docs/POT-00_test-matrix.md` and a `UC-xx` test in `front/e2e/` (run by `pnpm -C front e2e` and by the CI).
- Say what was NOT verified (real Tauri window, touch, installed build).

## 7. Commit and ship
- Conventional commits: `feat|fix|test|docs|spec|build|ci(scope): subject`. No co-author line.
- Push to `main` only after the local gate (step 5) is fully green. No PR.
- A release bumps the 3 versions (`src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`,
  `front/package.json`) and `Cargo.lock` per the semver rules above, in a commit
  `build(app): bump version to X.Y.Z`; then tag `vX.Y.Z` and push `main` and the tag together
  (the release workflow refuses a tag that does not match the versions).
- After the push, check the CI of `main` and of the tag; if red, fix forward.

## 8. Release and landing page
- Check the assets (`gh release view vX.Y.Z`). The release workflow deletes the older pre-releases of the same major by itself (tags stay): only the latest one remains.
- A change the user can see also updates `site/`: texts in `index.html` and its French map,
  screenshots in `site/img/` and `site/img/fr/`. The Pages workflow deploys on push.

## 9. Close
- Move the item from "Known problems" to "Fixed" in the review doc, if there is one.
- Report in French: what changed, which spec versions, what was checked, what was not.
