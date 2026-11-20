---
name: new-task
description: Write a Potagerie task in docs/tasks/ so the work and its context survive the session. Use when something should be done later, when a discussion ends with work left, or when an agent finds work outside its task.
---

# New task

1. Id: highest `T<NN>` in `docs/tasks/` on `origin/main` (and on your branch) + 1. Never reuse one.
2. Copy `docs/tasks/_template.md` to `docs/tasks/T<NN>_<topic>.md` (lowercase English, hyphens).
3. Fill it, in English:
   - `From`: `Tim`, `T<NN> (agent)` or `chat`.
   - `Context`: what was seen, where (file, screen, spec §), what was tried or decided. Write for someone who missed the conversation: this is the point of the file.
   - `Goal`: one sentence. `Refs`: the S01 / S03 / S05 items.
   - `Acceptance`: checkable lines (S03 §12 case, `UC-xx`, command). `Scope`: the paths.
4. Status:
   - `Ready` only if the specs answer every question and the acceptance is checkable by an agent alone.
   - Otherwise `Draft`, with the missing decisions as numbered `Questions` (options, recommendation).
5. One task = one slice that ships alone. Bigger: several tasks linked by `Depends on`.
6. Check `docs/tasks/` first: if a task already covers it, add to its `Context` instead (agents: only if it is not running; else create a new task that refers to it).
