#!/usr/bin/env bash
# Board and launcher for the tasks of docs/tasks/ (see docs/tasks/README.md).
# Works with the bash 3.2 of macOS.
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
DIR=docs/tasks
BASE=${TASKS_BASE:-origin/main}
HOME_DIR=${TASKS_HOME:-"$(cd "$ROOT/.." && pwd)/potagerie-tasks"}
MAX=${TASKS_MAX:-2}
STATUSES="Draft Ready Running Blocked Done"

die() { echo "tasks: $*" >&2; exit 1; }

# An agent is alive while its pid file names a live process.
pid_file() { echo "$HOME_DIR/pids/$1.pid"; }
alive() { local p; p=$(pid_file "$1"); [ -f "$p" ] && kill -0 "$(cat "$p")" 2>/dev/null; }

# Value of "- <Field>: <value>" in the text on stdin.
field() { sed -n "s/^- $1: *//p" | head -n 1 | sed 's/[[:space:]]*$//'; }

# Task files known to the base branch (tasks are created on main only).
files() {
  git ls-tree --name-only "$BASE" "$DIR/" | grep -E '/T[0-9]+_[^/]*\.md$' \
    | awk '{ n = $0; sub(/.*\/T/, "", n); sub(/_.*/, "", n); print n "\t" $0 }' | sort -n | cut -f2 || true
}

id_of() { local b=${1##*/}; echo "${b%%_*}"; }
file_of() { files | grep "/$1_" | head -n 1; }
branch_of() { git for-each-ref --format='%(refname:short)' "refs/heads/task/$1-*" | head -n 1; }

# Text of a task: its branch version when the task has a branch, else the base one.
text() {
  local id=$1 f=$2 b
  b=$(branch_of "$id")
  if [ -n "$b" ] && git cat-file -e "$b:$f" 2>/dev/null; then git show "$b:$f"; else git show "$BASE:$f"; fi
}

status_of() { local f; f=$(file_of "$1"); [ -n "$f" ] || die "unknown task $1"; text "$1" "$f" | field Status; }

deps_done() {
  local deps d
  deps=$(git show "$BASE:$(file_of "$1")" | field "Depends on")
  for d in $(echo "$deps" | tr ',' ' '); do
    case "$d" in —|-|"") continue ;; esac
    [ "$(git show "$BASE:$(file_of "$d")" 2>/dev/null | field Status)" = "Done" ] || return 1
  done
}

running_count() {
  local n=0 f
  for f in "$HOME_DIR"/pids/*.pid; do
    [ -f "$f" ] && kill -0 "$(cat "$f")" 2>/dev/null && n=$((n + 1))
  done
  echo "$n"
}

board() {
  git fetch -q origin 2>/dev/null || echo "(offline: board from the last fetch)"
  local tmp s f id t
  tmp=$(mktemp)
  for f in $(files); do
    id=$(id_of "$f")
    t=$(text "$id" "$f")
    printf '%s\t%s\t%s\t%s\t%s\n' "$(echo "$t" | field Status)" "$id" \
      "$(echo "$t" | sed -n 's/^# T[0-9]* · //p' | head -n 1)" \
      "$(echo "$t" | field Owner)" "$(echo "$t" | field 'Depends on')" >> "$tmp"
  done
  for s in $STATUSES; do
    grep -q "^$s	" "$tmp" || continue
    echo "== $s"
    grep "^$s	" "$tmp" | while IFS="	" read -r _ id title owner deps; do
      line="  $id  $title"
      [ "$owner" = "Tim" ] && line="$line  [Tim]"
      case "$deps" in —|-|"") ;; *) line="$line  (after $deps)" ;; esac
      [ "$s" = "Running" ] && ! alive "$id" && line="$line  ** stalled: no agent running, use run $id **"
      echo "$line"
      if [ "$s" = "Blocked" ]; then
        text "$id" "$(file_of "$id")" | awk '/^## Questions/{on=1;next} /^## /{on=0} on && NF && !/^<!--/{print "      " $0}'
      fi
    done
  done
  [ -s "$tmp" ] || echo "No task on $BASE yet: commit and push docs/tasks/ (scripts/tasks.sh ui also shows the local ones)."
  rm -f "$tmp"
  local legacy
  legacy=$(git worktree list --porcelain | grep -c '^worktree .*/\.claude/worktrees/' || true)
  [ "$legacy" -gt 0 ] && echo "($legacy old agent worktrees in .claude/worktrees: git worktree prune once they are gone)"
  return 0
}

run() {
  local id=$1 f st b wt topic log
  git fetch -q origin || true
  alive "$id" && die "$id is already running (pid $(cat "$(pid_file "$id")")): see scripts/tasks.sh log $id"
  f=$(file_of "$id"); [ -n "$f" ] || die "$id is not on $BASE: commit and push it first"
  st=$(status_of "$id")
  case "$st" in
    Ready) deps_done "$id" || die "$id waits for its dependencies" ;;
    Blocked|Running) ;;
    *) die "$id is $st" ;;
  esac
  [ "$(git show "$BASE:$f" | field Owner)" = "agent" ] || die "$id is for Tim"
  b=$(branch_of "$id")
  topic=$(basename "$f" .md); topic=${topic#*_}
  wt="$HOME_DIR/$id"
  mkdir -p "$HOME_DIR/logs"
  if [ ! -d "$wt" ]; then
    if [ -n "$b" ]; then git worktree add -q "$wt" "$b"
    else git worktree add -q -b "task/$id-$topic" "$wt" "$BASE"; fi
  fi
  log="$HOME_DIR/logs/$id.log"
  mkdir -p "$HOME_DIR/pids"
  echo "=== $(date '+%Y-%m-%d %H:%M') run $id" >> "$log"
  export TASKS_CLAUDE_ARGS="${TASKS_CLAUDE_ARGS:-}"
  cd "$wt"
  # The wrapper writes the exit code in the log and frees the lock, even when claude fails.
  nohup bash -c '
    # shellcheck disable=SC2086
    claude -p "$1" --permission-mode acceptEdits \
      --allowedTools "Read" "Edit" "Write" "Glob" "Grep" \
        "Bash(cargo:*)" "Bash(pnpm:*)" "Bash(make:*)" "Bash(git:*)" "Bash(gh:*)" \
        "Bash(ls:*)" "Bash(cat:*)" "Bash(grep:*)" "Bash(find:*)" "Bash(sed:*)" "Bash(mkdir:*)" \
      $TASKS_CLAUDE_ARGS >> "$2" 2>&1 < /dev/null
    rc=$?; printf "\n=== exit %s %s\n" "$rc" "$(date "+%Y-%m-%d %H:%M")" >> "$2"
    rm -f "$3"' _ "Run task $id ($f). Follow $ROOT/skills/run-task/SKILL.md: the latest version, from the main checkout, even if your branch holds an older one." "$log" "$(pid_file "$id")" > /dev/null 2>&1 &
  echo $! > "$(pid_file "$id")"
  cd "$ROOT"
  echo "$id started in $wt (log: scripts/tasks.sh log $id)"
}

next() {
  git fetch -q origin || true
  [ "$(running_count)" -lt "$MAX" ] || { echo "$MAX tasks already running"; return 0; }
  local f id
  for f in $(files); do
    id=$(id_of "$f")
    [ -z "$(branch_of "$id")" ] || continue
    [ "$(git show "$BASE:$f" | field Status)" = "Ready" ] || continue
    [ "$(git show "$BASE:$f" | field Owner)" = "agent" ] || continue
    deps_done "$id" || continue
    run "$id"; return 0
  done
  echo "nothing ready"
}

clean() {
  git fetch -q origin || true
  local f id b
  for f in $(files); do
    id=$(id_of "$f"); b=$(branch_of "$id")
    [ -n "$b" ] || continue
    [ "$(git show "$BASE:$f" | field Status)" = "Done" ] || continue
    [ -d "$HOME_DIR/$id" ] && git worktree remove "$HOME_DIR/$id"
    git branch -D "$b" >/dev/null && echo "removed $b"
  done
  git worktree prune
}

case "${1:-board}" in
  board) board ;;
  run) [ -n "${2:-}" ] || die "usage: tasks.sh run T<NN>"; run "$2" ;;
  next) next ;;
  log) [ -n "${2:-}" ] || die "usage: tasks.sh log T<NN>"; tail -n 50 -f "$HOME_DIR/logs/$2.log" ;;
  clean) clean ;;
  ui) exec node "$ROOT/scripts/tasks-ui.mjs" ;;
  loop)
    # caffeinate keeps the Mac awake while the loop runs (macOS only).
    if command -v caffeinate > /dev/null; then exec caffeinate -is node "$ROOT/scripts/tasks-loop.mjs"; fi
    exec node "$ROOT/scripts/tasks-loop.mjs" ;;
  *) die "usage: tasks.sh [board | run T<NN> | next | log T<NN> | clean | ui | loop]" ;;
esac
