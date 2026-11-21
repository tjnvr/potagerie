# T13 · Test the real app windows in CI (tauri-driver)

- Status: Ready
- Owner: agent
- Depends on: —
- Refs: POT-00_review-2026-11 (known problem 2) · S05 §3 · POT-00_test-matrix.md
- From: chat

## Context

Created on 2026-11-20 from the review's known problem 2: the real Tauri windows (WebKit, WebView2) and touch were never exercised; the e2e tests run the interface in Playwright against the fake commands (`?example`), not the built app.

Reworked on 2026-11-21 (chat): first owned by Tim as a manual check. Most of it can be automated: `tauri-driver` drives the built app through WebDriver on Windows (WebView2, with `msedgedriver`) and Linux (WebKitGTK, with `WebKitWebDriver`). It does not support macOS (WKWebView): there, the Playwright WebKit e2e stay the closest check, plus an occasional manual look by Tim (see `docs/tasks/README.md`). Touch belongs to lot 2 (mobile).

## Goal

Every push runs a few use cases in the real built app, with the real Rust commands and SQLite, on Windows and Linux.

## Acceptance

- [ ] A CI job builds the app and runs smoke tests through `tauri-driver` on `windows-latest` (WebView2) and on Ubuntu (WebKitGTK): first launch and plot creation, add a bed, create a planting from a free slot, export then import a garden file (UC-01, UC-02, UC-06, UC-23), the app restarted once to check the data is kept.
- [ ] The tests run against a temporary data folder, never the developer's own `potagerie.db`.
- [ ] Rows in `POT-00_test-matrix.md` (ids free on `origin/main`), and the review's known problem 2 updated: what is now covered, what is not (macOS window, touch).
- [ ] A `make` target runs the same tests locally on Linux or Windows; documented in the README.
- [ ] CI time stays reasonable: the job may run on `main` and tags only if it is slow (say so in the Log).
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5), and the new job green on the pushed commit.

## Scope

- `.github/workflows/ci.yml`
- `front/e2e/` (or a new `e2e-app/` folder)
- `src-tauri/` (only what the tests need, e.g. a data-folder override for tests)
- `Makefile`, `README.md`
- `docs/POT-00_*`

## Questions

## Log
