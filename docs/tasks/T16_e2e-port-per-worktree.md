# T16 · One e2e server per task worktree

- Status: Done
- Owner: agent
- Depends on: —
- Refs: `skills/spec-driven-delivery` step 5 (local gate)
- From: T02 (agent)

## Context

While running T02 (2026-11-20), `pnpm -C front e2e` failed in bulk (230 failed, then 87 failed on a second try) while other task worktrees (T01, T03) were running at the same time. Every failing test, including the new UC-28 and the unrelated SMOKE, A11Y and UC-13, passed when run alone, and the full suite was green with `--workers=2`.

The likely cause is `front/playwright.config.ts`: every worktree uses the same port (`E2E_PORT`, default 5320) and `reuseExistingServer: !process.env.CI`. A worktree can then test another worktree's Vite server (other code: UC-28 fails without the zoom), or lose it halfway through when that worktree's run ends. The default worker count on a machine already running other suites adds load on top of that.

In an unattended agent session, prefixing the command with `E2E_PORT=…` may need a permission the agent cannot get, so the fix must work with the plain `pnpm -C front e2e`.

## Goal

Several task worktrees can run the local gate at the same time without testing each other's server.

## Acceptance

- [x] Run without `E2E_PORT`, the e2e suite never reuses a server started by another worktree: either the port is derived from the worktree (e.g. a hash of the `front/` path) or `reuseExistingServer` is only on when asked for explicitly.
- [x] `E2E_PORT` still overrides the port; CI behaves as before.
- [x] Two worktrees running `pnpm -C front e2e` at the same time are both green (say how it was checked in the Log).
- [x] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/playwright.config.ts`
- `scripts/tasks.sh`

## Questions

## Log

- 2026-11-21 · Done, v0.7.1 (patch: dev tooling only; no spec change, no spec version).
  - Changed: `front/playwright.config.ts`. Without `E2E_PORT`, the local port is derived from the path of `front/` (sha1, 5400–7399), so each checkout gets its own; the CI keeps 5320; `E2E_PORT` still wins. `reuseExistingServer` is now opt-in (`E2E_REUSE=1`, never in CI), so a server already on the port fails the run loudly (`--strictPort`) instead of being tested. `scripts/tasks.sh` needed no change.
  - Checked: full local gate green (fmt, clippy, cargo test, front check, test 215, build, e2e 319). Two checkouts at the same time (this worktree and a `git clone --shared` in `/tmp/t16-second`, both running `pnpm -C front e2e`): both servers started side by side (they would have failed to start on a shared port); round 1 had 319 passed there, 318 + 1 failed here (SET-13 fr-FR, WebKit, a timeout under the load of two suites; that spec then passed 150/150 alone with `--repeat-each=5`); round 2 had both green, 319 passed each.
  - NOT checked: `E2E_PORT=… pnpm -C front e2e` (the env prefix needs a permission this session does not have; the code path is a plain `??`); `E2E_REUSE=1`; the CI run is checked after the push. The default local worker count was not changed: with two suites at once a WebKit test may still time out under load.
