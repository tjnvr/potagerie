# T12 · Frame lot 2 (mobile)

- Status: Draft
- Owner: agent
- Depends on: —
- Refs: S01 (lots) · S05 §9 lot 2, §10 (iOS account) · D01 (phone layouts)
- From: chat

## Context

Created on 2026-11-20. Lot 2 (S05 §9): the Tauri app on iOS and Android, the phone layout of D01, transfer between devices by JSON export and import. S05 §10 leaves one choice open: a paid Apple Developer account (99 $ a year) to install for good on an iPhone, or a free account (reinstall from Xcode every 7 days), or Android only.

Reworked on 2026-11-21 (chat): first owned by Tim. The framing itself (spec updates, cutting into tasks) is agent work; only the start and the phones are Tim's call.

## Goal

Lot 2 is cut into agent tasks, ready to run, for the phones Tim chose.

## Acceptance

- [ ] S05 (and S01 if needed) updated for the chosen phones, per `spec-driven-delivery`.
- [ ] Lot 2 written as tasks (`new-task`), small slices with `Depends on`, `Ready` when nothing functional is left open.
- [ ] `POT-00_lot1-coverage.md` or a new coverage doc for lot 2, per the spec rules.

## Scope

- `docs/`

## Questions

1. [functional] Start lot 2 (the app on your phone) now, and on which phones?
   (a) Android only: free, the app is installed from a file.
   (b) Android and iPhone, with a paid Apple account (99 $ a year): the iPhone app stays installed.
   (c) Android and iPhone with a free Apple account: the iPhone app must be reinstalled from the Mac every 7 days.
   (d) Not now.
   Recommendation: (a) first; iPhone later if you want it.

## Log
