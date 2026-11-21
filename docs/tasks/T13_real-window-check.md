# T13 · Test the real app windows in CI (tauri-driver)

- Status: Running
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

### Decisions
- Tests in a new `e2e-app/` folder, Node's built-in test runner and a small WebDriver client (`webdriver.mjs`) over `fetch`: no new dependency, nothing shared with the Playwright fake of `front/e2e/`.
- Data folder and file windows from the environment (`POTAGERIE_DATA_DIR`, `POTAGERIE_DIALOG_FILE`, `src-tauri/src/smoke.rs`): the release binary is the one tested, no test-only build feature; without the variables the app behaves as before.
- The file windows are replaced by a stand-in plugin named `dialog` with the same `open` / `save` commands: WebDriver cannot drive native windows, and the interface and the `main` capability stay untouched.
- On Windows the window passes wry's WebView2 arguments plus those msedgedriver sets in `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`: wry's own list otherwise hides the driver's debugging port and the session never attaches.
- The `app` job runs on every push, in parallel with the others: Windows takes about 11 min, Linux about 3.5 min, the Playwright e2e job about 10 min, so the CI wall time grows by about one minute.
- Ids `APP-01` to `APP-06` in the test matrix; `UC-30` (manual) narrowed to the macOS window, touch, the installed build and the native file windows.
- S05 v0.21 (first written as v0.19, then v0.20): T18 then T06 took those versions on `main` meanwhile; the branch was replayed on `origin/main` each time and its spec change rewritten on top of the current S05.

### Progress
- 2026-11-21: smoke tests and Rust support written, CI made green on `task/T13-real-window-check-ci` (run 44916368333, both `app` jobs green) after fixes for the W3C element key, icon buttons by `aria-label`, the WebView2 arguments and a Plant click while the form opens.
- 2026-11-21: replayed on `origin/main` (v0.9.0, T18), S05 bumped to v0.20; full local gate green (fmt, clippy, cargo test, front check, test, build, e2e 323 passed).
- 2026-11-21: CI green on the replayed commit (run 44945943201, branch `task/T13-real-window-check-ci2`, all six jobs).
- 2026-11-21: `main` moved again (T06, v0.10.0, S05 v0.20): replayed once more, S05 bumped to v0.21.
- Extra paths outside `Scope`: `.gitignore` (`e2e-app/results/`), `docs/specs/` (S05 v0.21 and the cross-references in S01 v1.9, S03 v0.20), `docs/POT-00_index.md`.
