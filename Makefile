# Potagerie: Rust workspace (crates/, src-tauri/) and Svelte interface (front/).
# Needs: Rust stable, the Tauri CLI (cargo install tauri-cli --version "^2"),
# Node.js 22+ and pnpm.

.PHONY: install dev build test check e2e fmt bindings

install:
	pnpm --dir front install --frozen-lockfile

# Opens the app; the interface reloads on save. Regenerates bindings.ts.
dev: install
	cargo tauri dev

# Potagerie.app / .dmg on macOS, .exe / .msi on Windows, in target/release/bundle.
build: install
	cargo tauri build

test: install
	cargo test --workspace
	pnpm --dir front test

# What the CI checks, without building the bundles.
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
