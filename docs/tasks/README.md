# Potagerie — Tasks

The memory of the work, one file per task. The specs say **what** the app does; a task holds **what is left to do and why**, so no context is lost between sessions. Anyone writes tasks: Tim, an agent running a task, a chat session (skill `new-task`).

- Requirements view: [`POT-00_lot1-coverage.md`](../POT-00_lot1-coverage.md) (done / partial / postponed).
- Work view: this folder. A task closes a "Partial", fixes a known problem, or delivers part of a lot.

## A task file

`docs/tasks/T<NN>_<topic>.md`, from [`_template.md`](_template.md). `NN` is a number, at least two digits (`T07`, `T42`, `T130`), never reused.

```
# T07 · Rotate and duplicate a bed

- Status: Ready
- Owner: agent
- Depends on: T04
- Refs: S01 F-PLN-6 · S03 §4
- From: T04 (agent)
```

| Field | Values |
|---|---|
| Status | `Draft` · `Ready` · `Running` · `Blocked` · `Done` |
| Owner | `agent` (an agent may take it) or `Tim` (needs a human: a real device, a canvas, a product choice) |
| Depends on | Task ids that must be `Done` first, or `—` |
| Refs | The spec items the task implements |
| From | Who created it and in which context: `Tim`, `T04 (agent)`, `chat` |

Sections: **Context** (why the task exists: what was seen, where, what was tried), **Goal** (one sentence), **Acceptance** (checkable: S03 §12 cases, `UC-xx` rows, commands), **Scope** (where the work is expected; a starting point), **Questions** (`[functional]` only), **Log** (with `Decisions`).

## Statuses

```
Draft ───────▶ Ready ──(agent)──▶ Running ──▶ Done        pushed to main, Log filled
                                       │
                                       ├──▶ Blocked, [functional] question ──(Tim answers)──▶ Running
                                       └──▶ Blocked, stuck (technical) ──(run again)──▶ Running
```

- **Draft**: waits for a functional decision from Tim (one `[functional]` question). Nothing else keeps a task in Draft.
- **Ready**: an agent can do it alone (the specs answer everything) once its dependencies are `Done`. Nothing runs on its own: a task starts when it is launched (`run`, `next`).
- **Running** and **Blocked** live on the branch `task/T<NN>-<topic>`, in the task's worktree. `main` only ever sees `Draft`, `Ready` and `Done`.
- **Done**: the slice passed the full local gate and was pushed to `main` by the agent (skill `run-task`). Tim reads the Log afterwards.

## Who decides what

Tim decides **functional scope only**: what the user can see or do, and when (which lot). Everything else (code, design, files, specs S03 / S05 written to match S01, tests, edge cases) is decided by the agents, written in the task's `Log` under `Decisions`, and never asked. A technical dead end is not a question either: the task is marked stuck and the next run picks it up.

## Who does what

- **Agents running a task** never widen it: what they find outside its scope (a bug, a missing rule, a follow-up) becomes a new task, pushed with their work (skill `new-task`).
- **Chat sessions** turn a discussion into tasks before it ends, so the next session starts from the file, not from memory.
- **Tim** launches tasks and answers `[functional]` questions in the board. An answer to a blocked task is committed on its branch and can resume it; an answer to a `Draft` can set it `Ready`, and **Publish** pushes the task files to `main`.

## Commands

| Command | Role |
|---|---|
| `scripts/tasks.sh ui` | Local board on http://localhost:4321: columns by status, dependency order, task details and agent log, Start / Resume buttons, answers to the questions |
| `scripts/tasks.sh` or `board` | Tasks by status, open questions of the blocked ones, worktrees left over |
| `scripts/tasks.sh run T07` | Starts (or resumes) an agent on T07, in the background, in its own worktree |
| `scripts/tasks.sh next` | Starts the first `Ready` agent task whose dependencies are `Done`, if fewer than 2 are running |
| `scripts/tasks.sh log T07` | Follows the agent's output |
| `scripts/tasks.sh clean` | Removes the worktrees and branches of `Done` tasks |
| `scripts/tasks.sh loop` | Runs the tasks continuously on this Mac (see below) |

Worktrees and logs are in `../potagerie-tasks/` (outside the repository). `TASKS_MAX` sets the number of tasks running at once (2 by default).

## Running continuously

`scripts/tasks.sh loop` keeps the Mac awake (`caffeinate`) and, every 5 minutes:

1. **Claude limit**: if an agent stopped on a Claude usage limit, the loop sleeps until the reset time given in the message (5 hours when there is none, plus 2 minutes), then resumes the tasks it cut. The board shows "Claude limit · resumes HH:MM".
2. **Restarts**: an agent that stopped for another reason is restarted, twice at most; a task you answered in the board is resumed.
3. **Next**: free slots (2) take the next `Ready` tasks.

Stuck tasks (technical dead end) and tasks waiting for a `[functional]` answer are left alone. One loop at a time; its journal is `../potagerie-tasks/loop.log`. Leave it in a terminal tab, or in the background:

```
nohup scripts/tasks.sh loop > /dev/null 2>&1 &
```

Settings: `TASKS_MAX` (agents at once, 2), `TASKS_LOOP_MINUTES` (5), `TASKS_LIMIT_HOURS` (5), `TASKS_LOOP_RETRIES` (2).
