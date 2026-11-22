# T26 · Touch e2e TCH-02 and TCH-03 flaky under a full local run

- Status: Ready
- Owner: agent
- Depends on: T22
- Refs: S01 F-PLN-6 · S05 §4 · `docs/POT-00_test-matrix.md` TCH-02, TCH-03
- From: T23 (agent)

## Context

While running T23's full local gate on 2026-11-22 (`pnpm -C front e2e`, all workers, on a Mac), `front/e2e/touch.spec.ts` TCH-02 (tray item dropped after a long press) and TCH-03 (second finger during a drag) failed once on Chromium at 360 × 800. They passed alone (`--repeat-each 3`), and the next full run was green. T23 does not change the tray or the move gesture, only the resize handles, so the failures are probably about timing: the long press (450 ms) and the dispatched touches race when the machine is loaded. The CI runs 2 workers with 1 retry, which may hide it.

## Goal

The touch e2e tests pass every time, under load too, without retries.

## Acceptance

- [ ] Cause found (trace of a failing run) and written in `Log`.
- [ ] `pnpm -C front exec playwright test e2e/touch.spec.ts --repeat-each 10` green under a full parallel run, with no retry.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/e2e/touch.spec.ts`
- `front/src/lib/editor/` (only if the cause is in the app)

## Questions

## Log
