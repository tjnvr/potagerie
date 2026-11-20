---
name: run-task
description: Run one Potagerie task (docs/tasks/T<NN>_*.md) end to end, alone, and push it to main. Use when asked to run, resume or take a task.
---

# Run a task

You run unattended. Nobody answers questions in the chat: they go in the task file.

## Setup
- You are in the task's worktree, on branch `task/T<NN>-<topic>`. Never check out `main`.
- Read the task file, then every spec section in `Refs`. Read `skills/spec-driven-delivery` and `skills/docs-git-flow`: they apply in full, except the branch name and the push target below.
- Resuming? Read `Questions` (answers) and `Log` first, and continue from there.

## Start
Set `- Status: Running`, commit `docs(tasks): start T<NN>`.

## Work
- Follow `spec-driven-delivery` steps 1 to 6: spec first if a rule changes, failing test first, then code.
- Stay inside `Scope`. Needing another path is a question.
- Found something outside the task (bug, missing rule, follow-up)? Do not do it: create a task with skill `new-task` (`From: T<NN> (agent)`), commit it on this branch, mention it in `Log`.

## Block instead of guessing
Unclear or contradictory spec, missing decision, a path outside `Scope`, a gate you cannot make green after three honest attempts:
1. Add a numbered question under `Questions`: the problem, the options, your recommendation.
2. Set `- Status: Blocked`, add a `Log` line, commit, stop. Do not push.

## Ship
1. Full local gate green (`spec-driven-delivery` step 5).
2. `git fetch origin && git rebase origin/main`. If the rebase brought commits, run the full gate again. If an id you added is now taken on `origin/main` (a task `T<NN>`, a use case `UC-xx` or another test id in `POT-00_test-matrix.md`), renumber yours to the next free one, everywhere you used it.
3. Version bump per `spec-driven-delivery` step 7, computed from `origin/main`.
4. Set `- Status: Done`, fill `Log` (in English): date, what changed, version, spec versions, what was checked, what was NOT.
5. `git push origin HEAD:main`. Rejected: back to 2, at most 3 times, then Blocked.
6. Only after the push succeeded: tag `vX.Y.Z` on `HEAD`, push the tag, check CI and release (steps 7 and 8). Red CI: fix forward in this same task.

## Rules
- Commit messages cite the task: `feat(front): aisle bubble (S01 F-PLN-2, T01)`.
- No `Co-Authored-By`, no "generated with", anywhere.
- Never edit another existing task file, never delete branches or worktrees, never force-push.
- Another agent may be running at the same time on another task: pick new ids (`UC-xx`, task numbers) from `origin/main` and expect to renumber at step 2.
- Report in French at the end of your output: done or blocked, and why.
