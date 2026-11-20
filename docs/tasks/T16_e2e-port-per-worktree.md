# T16 · One e2e server per task worktree

- Status: Ready
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

- [ ] Run without `E2E_PORT`, the e2e suite never reuses a server started by another worktree: either the port is derived from the worktree (e.g. a hash of the `front/` path) or `reuseExistingServer` is only on when asked for explicitly.
- [ ] `E2E_PORT` still overrides the port; CI behaves as before.
- [ ] Two worktrees running `pnpm -C front e2e` at the same time are both green (say how it was checked in the Log).
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/playwright.config.ts`
- `scripts/tasks.sh`

## Questions

## Log
