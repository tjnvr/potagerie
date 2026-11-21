---
name: run-task
description: Run one Potagerie task (docs/tasks/T<NN>_*.md) end to end, alone, and push it to main. Use when asked to run, resume or take a task.
---

# Run a task

You run unattended. Tim only decides **functional scope** (what the user can see or do); every other decision is yours.

## Setup
- You are in the task's worktree, on branch `task/T<NN>-<topic>`. Never check out `main`.
- Read the task file, then every spec section in `Refs`. Read `skills/spec-driven-delivery` and `skills/docs-git-flow`: they apply in full, except the branch name and the push target below.
- Resuming? Read `Questions` (answers), `Decisions` and `Log` first, and continue from there.

## Start
Set `- Status: Running`, commit `docs(tasks): start T<NN>`.

## Work
- Follow `spec-driven-delivery` steps 1 to 6: spec first if a rule changes, failing test first, then code.
- `Scope` is a starting point, not a fence: touch whatever the goal needs and list the extra paths in `Log`.
- Found something outside the task (bug, missing rule, follow-up)? Do not do it: create a task with skill `new-task` (`From: T<NN> (agent)`), commit it on this branch, mention it in `Log`.

## Decide, do not ask

**Yours, never a question.** Files and paths; the design of commands, data and code; test ids; how S03 / S05 must change so the app does what S01 says; a contradiction between S01 and S03 / S05 (S01 wins); edge cases; a requirement S01 describes that a spec deferred (a `Ready` task is the go-ahead). Choose: rules in Rust, smallest change, consistent with the existing code. Write each choice in `Log` under `Decisions`, one line: the choice and why.

**Tim's, a question.** Only when the app would behave differently from S01 / D01 / D02: a behaviour added, removed or changed, a requirement dropped or moved to another lot, a text whose meaning changes, or S01 silent or ambiguous where the readings give a visibly different app.
1. Do first everything that does not depend on the answer; commit it (do not push).
2. Add one question under `Questions`, starting `[functional]`, in plain words (no code, file or command names): what the user would see with each option (2 or 3), and your recommendation.
3. Set `- Status: Blocked`, add a `Log` line, commit, stop.

**Technical dead end** (a gate still red after three honest attempts, a push rejected three times, a missing tool): not a question for Tim. Write what you tried in `Log` under `Stuck`, set `- Status: Blocked` with no new question, commit, stop. The board shows it as stuck; the next run picks it up from your notes.

## Ship
1. Full local gate green (`spec-driven-delivery` step 5).
2. `git fetch origin && git rebase origin/main`. If the rebase brought commits, run the full gate again. If an id you added is now taken on `origin/main` (a task `T<NN>`, a use case `UC-xx` or another test id in `POT-00_test-matrix.md`), renumber yours to the next free one, everywhere you used it.
3. Version bump per `spec-driven-delivery` step 7, computed from `origin/main`.
4. Set `- Status: Done`, fill `Log` (in English): date, what changed, version, spec versions, what was checked, what was NOT.
5. `git push origin HEAD:main`. Rejected: back to 2, at most 3 times, then stuck (see above).
6. Only after the push succeeded: tag `vX.Y.Z` on `HEAD`, push the tag, check CI and release (steps 7 and 8). Red CI: fix forward in this same task.

## Rules
- Commit messages cite the task: `feat(front): aisle bubble (S01 F-PLN-2, T01)`.
- No `Co-Authored-By`, no "generated with", anywhere.
- Never edit another existing task file, never delete branches or worktrees, never force-push.
- Never end your turn to wait for something (CI, a build, a timer): nobody wakes you up. Wait in the foreground, e.g. `gh run watch <id> --exit-status`, with a timeout.
- Write a line in `Log` (and commit) at each milestone: tests written, code passing, pushed for CI, CI result. If you are stopped, the next run starts from there.
- A branch you push to run CI (`task/T<NN>-…`) is yours to delete on `origin` once the task is done.
- Another agent may be running at the same time on another task: pick new ids (`UC-xx`, task numbers) from `origin/main` and expect to renumber at step 2.
- Report in French at the end of your output: done, blocked on a functional question, or stuck; and why.
