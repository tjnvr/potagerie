# Potagerie: Rust workspace (crates/, src-tauri/) and Svelte interface (front/).
# Needs: Rust stable, the Tauri CLI (cargo install tauri-cli --version "^2"),
# Node.js 22+ and pnpm.

.PHONY: install dev build test check e2e e2e-app fmt bindings

install:
	pnpm --dir front install --frozen-lockfile

# Opens the app; the interface reloads on save. Regenerates bindings.ts.
dev: install
	cargo tauri dev

# Potagerie.app / .dmg on macOS, .exe / .msi on Windows, in target/release/bundle.
build: install
	cargo tauri build

# src-tauri embeds front/dist at compile time: build the interface first (as the CI does).
test: install
	pnpm --dir front build
	cargo test --workspace
	pnpm --dir front test

# Formatting, lints and type checks of the CI, without the tests (make test, make e2e) or the bundles.
check: install
	cargo fmt --all --check
	pnpm --dir front build
	cargo clippy --workspace --all-targets -- -D warnings
	pnpm --dir front check

fmt:
	cargo fmt --all

# Rewrites front/src/lib/bindings.ts from the Rust commands.
bindings:
	UPDATE_BINDINGS=1 cargo test -p potagerie bindings_are_up_to_date

# Use cases of the interface in Chromium and WebKit (first time: pnpm --dir front exec playwright install chromium webkit).
e2e: install
	pnpm --dir front e2e

# Smoke tests of the real app (APP-xx in docs/POT-00_test-matrix.md) through tauri-driver,
# on Linux or Windows only (tauri-driver does not support macOS). Needs tauri-driver
# (cargo install tauri-driver --locked) and the native driver of the webview: WebKitWebDriver
# on Linux (package webkit2gtk-driver), on Windows the msedgedriver of the WebView2 version,
# its path in NATIVE_DRIVER. Uses a temporary data folder, never your potagerie.db.
e2e-app: install
	cargo tauri build --no-bundle
	node --test e2e-app/smoke.test.mjs
