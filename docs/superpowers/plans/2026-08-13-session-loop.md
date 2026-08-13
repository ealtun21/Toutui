# The loop of the sessions: the plan of the work

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make a driver that runs one session of the takeover after another, with
no person at the keys: it reads the prompt out of `docs/HANDOVER.md`, it sends
`/clear` and that prompt to a program of the agent in a pane of Herdr, and it waits
for the settled state of that pane.

**Architecture:** One script of `bash`. It holds four parts that a test can measure
one at a time: the extraction of the prompt out of the handover, the controls of the
loop (the sentinels and the log), the round, and the start of the agent. The script
calls Herdr through the name in the variable `HERDR_BIN`, therefore a test gives it
a false `herdr` that writes down every call and answers with the JSON of its choice.

**Tech Stack:** `bash`, `awk`, `jq`, `git`, and the CLI of Herdr. No new program of
the machine: each of these is there today.

**The design:** `docs/superpowers/specs/2026-08-13-session-loop-design.md`

## Global Constraints

- The driver is **outside the repository**. Its home is
  `~/.local/share/toutui-loop/`, and `~/.local/bin/toutui-loop` is a symbolic link
  to the script in that directory. No file of this plan goes into `src/` or into
  `tests/` of Toutui. The two documents of `docs/superpowers/` are the exception:
  they are the design and this plan.
- **The script removes no file.** The sentinels of the loop are older or newer than
  the marker of the run, and the driver reads their time. It does not delete them.
- The shell of the maintainer is `fish`. **The script is `bash`**, and its first
  line is `#!/usr/bin/env bash`.
- The state of the loop lives in `~/.local/state/toutui-loop/`, and the variable
  `TOUTUI_LOOP_STATE` gives a different directory to a test.
- The path of the file `complete` goes into the prompt **in full**. A shell does not
  expand the character `~` of a prompt.
- All prose of a file of this repository is in ASD-STE100 simplified technical
  English.
- The tests are `bash`, and they need no program that the machine does not hold.
- Every step of a test says the command and the answer that the command must give.

---

### Task 1: The directory, the script, and the extraction of the prompt

**Files:**
- Create: `~/.local/share/toutui-loop/toutui-loop`
- Create: `~/.local/share/toutui-loop/test.sh`
- Create: `~/.local/share/toutui-loop/fixtures/handover-simple.md`

**Interfaces:**
- Consumes: nothing.
- Produces: `extract_prompt <file>` writes the lines of the block of the prompt to
  the standard output, with the two characters `> ` removed of each line.
  `require_prompt <file>` does the same, and it stops the program with a message
  when the file is not there or the block is empty. The variable `PROMPT_HEADING`
  holds `## The prompt for the next session`. The script can be read with `source`:
  it calls `main` only when a shell runs it directly.

- [ ] **Step 1: Make the directory, and the fixture of the test**

```bash
mkdir -p ~/.local/share/toutui-loop/fixtures
```

Write `~/.local/share/toutui-loop/fixtures/handover-simple.md`:

```markdown
# A handover of a test

Prose that is not the prompt.

## What is open

> This quote comes before the heading, and it is not the prompt.

## The prompt for the next session

Prose that stands between the heading and the block.

> the first line
>
> the second line

## A heading after the block

> This quote comes after the block, and it is not the prompt.
```

- [ ] **Step 2: Write the failing test**

Write `~/.local/share/toutui-loop/test.sh`:

```bash
#!/usr/bin/env bash
# The measurement of toutui-loop. Run it with: bash ~/.local/share/toutui-loop/test.sh
set -uo pipefail

DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FAILS=0

check() { # name expected actual
  if [ "$2" = "$3" ]; then
    printf 'ok   %s\n' "$1"
  else
    printf 'FAIL %s\n  it must give: %q\n  it gave:      %q\n' "$1" "$2" "$3"
    FAILS=$((FAILS + 1))
  fi
}

# shellcheck source=./toutui-loop
source "$DIR/toutui-loop"

# --- Task 1: the extraction of the prompt ---

check "the block of the prompt, and nothing else" \
  "$(printf 'the first line\n\nthe second line')" \
  "$(extract_prompt "$DIR/fixtures/handover-simple.md")"

check "the handover of Toutui gives a block that starts with its first word" \
  "Continue" \
  "$(extract_prompt /home/nyverino/Documents/Toutui/docs/HANDOVER.md | head -1 | cut -d' ' -f1)"

printf '\n%s\n' "$FAILS of the measurements failed."
[ "$FAILS" -eq 0 ]
```

- [ ] **Step 3: Run the test, and see it fail**

Run: `bash ~/.local/share/toutui-loop/test.sh`
Expected: it fails, because the file `toutui-loop` is not there
(`No such file or directory`).

- [ ] **Step 4: Write the script**

Write `~/.local/share/toutui-loop/toutui-loop`:

```bash
#!/usr/bin/env bash
# toutui-loop — one session of the takeover after another, with no person at the keys.
# The design: docs/superpowers/specs/2026-08-13-session-loop-design.md of the repository.
set -euo pipefail

PROMPT_HEADING='## The prompt for the next session'

# extract_prompt <file> — the lines of the block of the prompt, without the "> ".
extract_prompt() {
  awk -v heading="$PROMPT_HEADING" '
    index($0, heading) == 1 { found = 1; next }
    !found                  { next }
    /^## /                  { exit }
    /^> /                   { print substr($0, 3); next }
    $0 == ">"               { print ""; next }
  ' "$1"
}

# require_prompt <file> — the same, and it stops the program when the block is empty.
require_prompt() {
  local file=$1 text
  [ -f "$file" ] || die "there is no handover at $file"
  text=$(extract_prompt "$file")
  [ -n "${text//[[:space:]]/}" ] ||
    die "the file $file holds no block of a prompt under the heading \"$PROMPT_HEADING\""
  printf '%s\n' "$text"
}

die() {
  printf 'toutui-loop: %s\n' "$*" >&2
  exit 1
}

main() {
  die "the loop is not built yet"
}

# The test reads this file with "source". Only a direct run calls main.
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  main "$@"
fi
```

- [ ] **Step 5: Run the test, and see it pass**

Run: `bash ~/.local/share/toutui-loop/test.sh`
Expected: two lines that start with `ok`, and `0 of the measurements failed.`

- [ ] **Step 6: Commit the plan of this task**

The script is outside the repository, therefore there is nothing of it to commit.
Mark the steps of this task, and go to Task 2.

---

### Task 2: The controls of the loop, with no removal of a file

**Files:**
- Modify: `~/.local/share/toutui-loop/toutui-loop`
- Modify: `~/.local/share/toutui-loop/test.sh`

**Interfaces:**
- Consumes: `die` of Task 1.
- Produces: `STATE_DIR` (the directory of the state, and `TOUTUI_LOOP_STATE`
  overrides it), `state_init` (it makes that directory and it writes the marker
  `run-started`), `sentinel_fresh <name>` (it gives 0 when the file of that name is
  there **and** it is newer than the marker of this run), and `log <words>` (it
  writes one line with the time to the standard output and to `driver.log`).

- [ ] **Step 1: Write the failing test**

Add to `~/.local/share/toutui-loop/test.sh`, before the last three lines:

```bash
# --- Task 2: the controls of the loop ---

TMP=$(mktemp -d)
export TOUTUI_LOOP_STATE="$TMP/state"
STATE_DIR="$TOUTUI_LOOP_STATE"

mkdir -p "$STATE_DIR"
touch "$STATE_DIR/stop"          # a sentinel of a run that is over
sleep 0.01
state_init                       # the marker of this run comes after it

check "a sentinel of an older run does not stop this one" \
  "old" \
  "$(if sentinel_fresh stop; then echo fresh; else echo old; fi)"

sleep 0.01
touch "$STATE_DIR/stop"          # the maintainer writes it now

check "a sentinel of this run stops it" \
  "fresh" \
  "$(if sentinel_fresh stop; then echo fresh; else echo old; fi)"

check "a sentinel that no one wrote is not there" \
  "old" \
  "$(if sentinel_fresh complete; then echo fresh; else echo old; fi)"

log "a line of the measurement" > /dev/null

check "the log holds the line" \
  "1" \
  "$(grep -c 'a line of the measurement' "$STATE_DIR/driver.log")"

check "state_init keeps the file of an older run" \
  "yes" \
  "$(if [ -f "$STATE_DIR/stop" ]; then echo yes; else echo no; fi)"
```

- [ ] **Step 2: Run the test, and see it fail**

Run: `bash ~/.local/share/toutui-loop/test.sh`
Expected: `state_init: command not found`, and the count of the failures is more
than 0.

- [ ] **Step 3: Write the controls**

In `~/.local/share/toutui-loop/toutui-loop`, put this after `die`:

```bash
STATE_DIR="${TOUTUI_LOOP_STATE:-$HOME/.local/state/toutui-loop}"

# state_init — the directory of the state, and the marker of the time of this run.
# It removes nothing: a sentinel of an older run is older than this marker, and
# sentinel_fresh reads that time.
state_init() {
  mkdir -p "$STATE_DIR"
  : > "$STATE_DIR/run-started"
}

# sentinel_fresh <name> — 0 when the file is there and it belongs to this run.
sentinel_fresh() {
  local file="$STATE_DIR/$1"
  [ -f "$file" ] && [ "$file" -nt "$STATE_DIR/run-started" ]
}

log() {
  printf '%s %s\n' "$(date -Is)" "$*" | tee -a "$STATE_DIR/driver.log"
}
```

- [ ] **Step 4: Run the test, and see it pass**

Run: `bash ~/.local/share/toutui-loop/test.sh`
Expected: seven lines that start with `ok`, and `0 of the measurements failed.`

---

### Task 3: The round, and a false Herdr that measures it

**Files:**
- Modify: `~/.local/share/toutui-loop/toutui-loop`
- Modify: `~/.local/share/toutui-loop/test.sh`
- Create: `~/.local/share/toutui-loop/fixtures/fake-herdr`

**Interfaces:**
- Consumes: `log` and `STATE_DIR` of Task 2.
- Produces: `HERDR` (the name of the program of Herdr, and `HERDR_BIN` overrides
  it), `AGENT_NAME`, `REPO`, `CLEAR_TIMEOUT_MS`, `ROUND_TIMEOUT_MS`,
  `agent_status` (the word of the state of the agent), `suffix` (the lines that the
  driver adds to the prompt of a round), and `run_round <number> <text>`. The
  answer of `run_round` is 0 for a round that ended, 1 for a round that did not
  settle, and 2 for a round that is `blocked`.

- [ ] **Step 1: Write the false Herdr**

Write `~/.local/share/toutui-loop/fixtures/fake-herdr`:

```bash
#!/usr/bin/env bash
# A false Herdr for the measurement. It writes down every call, and it answers
# with the state of the variable FAKE_HERDR_STATUS.
printf '%s\n' "$*" >> "${FAKE_HERDR_CALLS:?the test must give FAKE_HERDR_CALLS}"

case "${1:-} ${2:-}" in
  "agent get")
    printf '{"result":{"agent":{"agent_status":"%s"}}}\n' "${FAKE_HERDR_STATUS:-idle}"
    ;;
  "agent prompt")
    exit "${FAKE_HERDR_PROMPT_RC:-0}"
    ;;
  *)
    printf '{"result":{}}\n'
    ;;
esac
```

Then: `chmod +x ~/.local/share/toutui-loop/fixtures/fake-herdr`

- [ ] **Step 2: Write the failing test**

Add to `~/.local/share/toutui-loop/test.sh`, before the last three lines:

```bash
# --- Task 3: the round ---

export FAKE_HERDR_CALLS="$TMP/calls"
HERDR="$DIR/fixtures/fake-herdr"
REPO="$TMP/repo"
mkdir -p "$REPO"
git -C "$REPO" init -q
git -C "$REPO" commit -q --allow-empty -m "the first commit"

: > "$FAKE_HERDR_CALLS"
FAKE_HERDR_STATUS=idle run_round 1 "the words of the round" > /dev/null
check "the round sends /clear first" \
  "agent prompt toutui /clear --wait --timeout $CLEAR_TIMEOUT_MS" \
  "$(head -1 "$FAKE_HERDR_CALLS")"

check "the round sends the words after it" \
  "yes" \
  "$(if grep -q 'the words of the round' "$FAKE_HERDR_CALLS"; then echo yes; else echo no; fi)"

: > "$FAKE_HERDR_CALLS"
rc=0; FAKE_HERDR_STATUS=blocked run_round 2 "the words" > /dev/null || rc=$?
check "a round that is blocked gives 2" "2" "$rc"

: > "$FAKE_HERDR_CALLS"
rc=0; FAKE_HERDR_PROMPT_RC=1 run_round 3 "the words" > /dev/null || rc=$?
check "a round that does not settle gives 1" "1" "$rc"

check "the suffix holds the path of the file complete, in full" \
  "yes" \
  "$(if suffix | grep -q "$STATE_DIR/complete"; then echo yes; else echo no; fi)"

check "the suffix holds no character ~" \
  "0" \
  "$(suffix | grep -c '~')"
```

- [ ] **Step 3: Run the test, and see it fail**

Run: `bash ~/.local/share/toutui-loop/test.sh`
Expected: `run_round: command not found`, and the count of the failures is more
than 0.

- [ ] **Step 4: Write the round**

In `~/.local/share/toutui-loop/toutui-loop`, put this after `log`:

```bash
REPO="${TOUTUI_LOOP_REPO:-/home/nyverino/Documents/Toutui}"
HANDOVER="${TOUTUI_LOOP_HANDOVER:-}"     # empty: $REPO/docs/HANDOVER.md
HERDR="${HERDR_BIN:-herdr}"
AGENT_NAME="${TOUTUI_LOOP_AGENT:-toutui}"
CLEAR_TIMEOUT_MS=120000
ROUND_TIMEOUT_MS=14400000                # four hours

# suffix — the lines that the driver adds to the prompt of every round.
suffix() {
  cat <<EOF

---

This session is one round of a loop, and no person watches the keys. Do not stop
to ask a question: take the decision, write the reason in \`docs/HANDOVER.md\`,
and go on. End the round as every session ends, with the handover and the prompt
for the next session.

**The one condition that stops the loop.** If, and only if, no item of the road
stays open and no condition of the road stays unmeasured, write the file
\`$STATE_DIR/complete\`, and put the reason in it. A round that closed its own
items and left others open must not write it.
EOF
}

agent_status() {
  "$HERDR" agent get "$AGENT_NAME" | jq -r '.result.agent.agent_status'
}

# run_round <number> <text> — 0 for a round that ended, 1 for a round that did not
# settle, 2 for a round that is blocked.
run_round() {
  local n=$1 text=$2 before after state started elapsed
  started=$(date +%s)
  before=$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo none)
  log "round $n: it starts, and the commit is $before"

  if ! "$HERDR" agent prompt "$AGENT_NAME" "/clear" \
       --wait --timeout "$CLEAR_TIMEOUT_MS" > /dev/null; then
    log "round $n: the agent did not take the command /clear"
    return 1
  fi

  if ! "$HERDR" agent prompt "$AGENT_NAME" "$text" \
       --wait --timeout "$ROUND_TIMEOUT_MS" > /dev/null; then
    log "round $n: the agent did not settle inside the limit of time"
    return 1
  fi

  state=$(agent_status)
  after=$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo none)
  elapsed=$(( $(date +%s) - started ))
  log "round $n: it ends with the state $state after $((elapsed / 60)) minutes, and the commit went $before -> $after"
  [ "$state" != blocked ] || return 2
  return 0
}
```

- [ ] **Step 5: Run the test, and see it pass**

Run: `bash ~/.local/share/toutui-loop/test.sh`
Expected: thirteen lines that start with `ok`, and `0 of the measurements failed.`

---

### Task 4: The start of the agent, the loop, and the options

**Files:**
- Modify: `~/.local/share/toutui-loop/toutui-loop`
- Create: the symbolic link `~/.local/bin/toutui-loop`

**Interfaces:**
- Consumes: everything of Task 1 to Task 3.
- Produces: `MAX_ROUNDS`, `DRY_RUN`, `parse_args`, `bootstrap_agent`, and a `main`
  that runs the loop. The options are `--repo PATH`, `--handover PATH`,
  `--state-dir PATH`, `--agent NAME`, `--max-rounds N`, `--timeout-ms MS`, and
  `--dry-run`.

- [ ] **Step 1: Write the start, the loop, and the options**

In `~/.local/share/toutui-loop/toutui-loop`, put this after `run_round`, and
**replace** the `main` of Task 1:

```bash
MAX_ROUNDS=50
DRY_RUN=0

parse_args() {
  while [ $# -gt 0 ]; do
    case $1 in
      --repo)       REPO=$2; shift 2 ;;
      --handover)   HANDOVER=$2; shift 2 ;;
      --state-dir)  STATE_DIR=$2; shift 2 ;;
      --agent)      AGENT_NAME=$2; shift 2 ;;
      --max-rounds) MAX_ROUNDS=$2; shift 2 ;;
      --timeout-ms) ROUND_TIMEOUT_MS=$2; shift 2 ;;
      --dry-run)    DRY_RUN=1; shift ;;
      -h|--help)
        cat <<'EOF'
toutui-loop — one session of the takeover after another.

  --repo PATH        the repository (docs/HANDOVER.md is inside it)
  --handover PATH    a different handover
  --state-dir PATH   the directory of the state
  --agent NAME       the name of the agent in Herdr (toutui)
  --max-rounds N     the guard against a loop that runs away (50)
  --timeout-ms MS    the limit of time of one round (four hours)
  --dry-run          show the prompt of the first round, and stop

The maintainer stops the loop with: touch ~/.local/state/toutui-loop/stop
EOF
        exit 0 ;;
      *) die "the option $1 is not one of mine. Use --help." ;;
    esac
  done
  [ -n "$HANDOVER" ] || HANDOVER="$REPO/docs/HANDOVER.md"
}

bootstrap_agent() {
  local pane
  [ "${HERDR_ENV:-}" = 1 ] || die "toutui-loop runs inside a pane of Herdr"
  pane=$("$HERDR" pane split --current --direction right --cwd "$REPO" --no-focus |
         jq -r '.result.pane.pane_id')
  [ -n "$pane" ] && [ "$pane" != null ] || die "Herdr gave no pane"
  log "the pane of the agent is $pane"
  "$HERDR" agent start "$AGENT_NAME" --kind claude --pane "$pane" \
    -- --dangerously-skip-permissions > /dev/null
  "$HERDR" agent focus "$AGENT_NAME" > /dev/null
  log "the agent $AGENT_NAME is ready, and the loop starts"
}

main() {
  parse_args "$@"

  if [ "$DRY_RUN" = 1 ]; then
    printf '%s\n' "$(require_prompt "$HANDOVER")$(suffix)"
    return 0
  fi

  state_init
  bootstrap_agent

  local n=1 text rc
  while [ "$n" -le "$MAX_ROUNDS" ]; do
    if sentinel_fresh stop; then
      log "the file stop is there, and the loop ends"
      return 0
    fi
    if sentinel_fresh complete; then
      log "the agent says that the work is complete, and the reason is:"
      cat "$STATE_DIR/complete" | tee -a "$STATE_DIR/driver.log"
      return 0
    fi

    text=$(require_prompt "$HANDOVER")
    rc=0
    run_round "$n" "$text$(suffix)" || rc=$?

    case $rc in
      0) ;;
      2)
        log "the agent is blocked, and the lines of its pane are:"
        "$HERDR" agent read "$AGENT_NAME" --source detection --lines 40 |
          tee -a "$STATE_DIR/driver.log"
        return 1 ;;
      *) return 1 ;;
    esac

    n=$((n + 1))
  done

  log "the loop made $MAX_ROUNDS rounds, and that is its limit"
}
```

- [ ] **Step 2: Make the script one that a shell can run, and make the link**

```bash
chmod +x ~/.local/share/toutui-loop/toutui-loop
mkdir -p ~/.local/bin
ln -sfn ~/.local/share/toutui-loop/toutui-loop ~/.local/bin/toutui-loop
```

- [ ] **Step 3: Run the tests of Task 1 to Task 3 again**

Run: `bash ~/.local/share/toutui-loop/test.sh`
Expected: thirteen lines that start with `ok`, and `0 of the measurements failed.`
The new code must break none of them.

- [ ] **Step 4: Measure the prompt of the first round, with no agent at all**

Run: `toutui-loop --dry-run | head -3`
Expected: the first line is `Continue the Toutui takeover. Repo:
`/home/nyverino/Documents/Toutui``

Run: `toutui-loop --dry-run | tail -8`
Expected: the lines of the suffix, and one of them holds
`/home/nyverino/.local/state/toutui-loop/complete`.

Run: `toutui-loop --dry-run | grep -c '^> '`
Expected: `0`. The characters `> ` of the quote are gone.

Run: `toutui-loop --handover /dev/null --dry-run`
Expected: it fails, and it says `holds no block of a prompt under the heading`.

---

### Task 5: The measurement of two rounds, and the proof that `/clear` clears

**Files:**
- Create: `~/.local/share/toutui-loop/measurement/` (a repository of a test)

This task runs the driver against a repository that is **not** Toutui. A round of
the real takeover makes tags and it pushes them; a fault of the driver must not
find that. The measurement gives the answer to three questions: does a round end,
does the driver read the new prompt that the round wrote, and **does `/clear`
clear?**

- [ ] **Step 1: Make the repository of the measurement**

```bash
mkdir -p ~/.local/share/toutui-loop/measurement/docs
cd ~/.local/share/toutui-loop/measurement
git init -q
git commit -q --allow-empty -m "the first commit"
```

- [ ] **Step 2: Write the handover of the measurement**

Write `~/.local/share/toutui-loop/measurement/docs/HANDOVER.md`:

```markdown
# The handover of the measurement

## The prompt for the next session

> This is a measurement of a loop, and it is not the work of a program.
>
> 1. Count the turns of the user that you can see in this conversation, before
>    this one. Add one line to `log.txt` of this repository: `round <the number
>    of the lines of log.txt, plus one>, and I see <the count> turns before this
>    one`.
> 2. Write this file again, `docs/HANDOVER.md`. Keep the heading
>    `## The prompt for the next session` and the block of the quote of this
>    prompt, word for word.
> 3. Commit the two files with the message `the round of the measurement`.
```

Then: `git -C ~/.local/share/toutui-loop/measurement add -A && git -C ~/.local/share/toutui-loop/measurement commit -q -m "the handover of the measurement"`

- [ ] **Step 3: Run two rounds**

```bash
toutui-loop \
  --repo ~/.local/share/toutui-loop/measurement \
  --state-dir ~/.local/share/toutui-loop/measurement-state \
  --agent loopcheck \
  --max-rounds 2
```

Expected: a new pane opens at the right with a program of Claude in it, the driver
writes `round 1: it starts`, and the loop ends with
`the loop made 2 rounds, and that is its limit`.

- [ ] **Step 4: Read the answer of the three questions**

Run: `cat ~/.local/share/toutui-loop/measurement/log.txt`

Expected: **two lines**, and **each of them says that the agent sees 0 or 1 turns
before its own**. A line that says 2 or more is the proof that `/clear` did not
clear, and the loop is not correct: stop, and read the pane of the agent with
`herdr agent read loopcheck --source recent-unwrapped --lines 60`. The first thing
to measure is whether the text `/clear` went into the program as a command or as
the words of a prompt.

Run: `git -C ~/.local/share/toutui-loop/measurement log --oneline`
Expected: two commits that say `the round of the measurement`.

Run: `cat ~/.local/share/toutui-loop/measurement-state/driver.log`
Expected: one block for each round, and the commit of the start of round 2 is the
commit of the end of round 1.

- [ ] **Step 5: Measure the file `stop`**

Run the driver again with `--max-rounds 50`, and while round 1 works:

```bash
touch ~/.local/share/toutui-loop/measurement-state/stop
```

Expected: the driver ends the round that works, it writes `the file stop is there,
and the loop ends`, and it stops. It does not start round 2.

- [ ] **Step 6: Measure the file `complete`**

Run the driver again. While round 1 works:

```bash
echo "the measurement of the file complete" > \
  ~/.local/share/toutui-loop/measurement-state/complete
```

Expected: the driver ends the round, it writes `the agent says that the work is
complete`, it shows the line of the file, and it stops.

- [ ] **Step 7: Measure that an old sentinel does not stop a new run**

The files `stop` and `complete` of Step 5 and Step 6 are still there. Run the
driver again with `--max-rounds 1`.

Expected: it makes round 1. It does not stop at its start. This is the proof of
the rule that the driver removes no file.

---

### Task 6: The pointer of the handover

**Files:**
- Modify: `/home/nyverino/Documents/Toutui/docs/HANDOVER.md`

A session of the loop must know that it is in a loop, and where the driver is. The
suffix says the first part. This task says the second, for a person who reads the
handover.

- [ ] **Step 1: Add the lines to the part `## The rules that do not change`**

Add this to the end of that part of `docs/HANDOVER.md`:

```markdown
**The sessions can run in a loop.** The driver is `~/.local/bin/toutui-loop`,
outside this repository, and its design is
`docs/superpowers/specs/2026-08-13-session-loop-design.md`. It reads the block of
the quote of `## The prompt for the next session`, therefore **that block must stay
the last part of this file, and every line of it must start with `> `**. A session
that writes a handover with no such block stops the loop.
```

- [ ] **Step 2: Measure that the driver still reads the block**

Run: `toutui-loop --dry-run | head -1`
Expected: `Continue the Toutui takeover. Repo: `/home/nyverino/Documents/Toutui``

The new lines stand in `## The rules that do not change`, and that part comes
before the part of the prompt. The extraction must not change.

- [ ] **Step 3: Commit**

```bash
cd /home/nyverino/Documents/Toutui
git add docs/HANDOVER.md
git commit -m "docs(harness): the handover says that a loop can read its prompt

The driver toutui-loop takes the block of the quote of the last part of this
file and gives it to a session with no context. The rule of that block is now
in the part of the rules that do not change.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

## How the maintainer starts the loop

In a pane of Herdr, inside the repository:

```bash
toutui-loop
```

To watch: the pane of the left is the driver, and the pane of the right is the
program of the agent.

To stop it: `touch ~/.local/state/toutui-loop/stop`
