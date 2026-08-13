# The loop of the sessions of the takeover

A design of 2026-08-13. It makes the loop of the sessions automatic.

## The problem

The takeover moves one session at a time. Each session ends with `docs/HANDOVER.md`,
and the last part of that file, `## The prompt for the next session`, holds a block
of quoted lines. The maintainer then does three things by hand, again and again:

1. They give the command `/clear` to the program of the agent.
2. They copy the quoted block out of `docs/HANDOVER.md`.
3. They put that block into the new session.

The work of the maintainer in this loop is mechanical. It holds no decision. This
design removes it.

## The idea

`docs/HANDOVER.md` is the memory of the loop. A session writes the prompt of the
next session into it, therefore the round N + 1 reads what the round N wrote. No
other state is necessary.

A `/clear` gives a session with no context. A driver outside the program of the
agent can send that same command, and it keeps its own memory of the loop. The
driver is a script. It runs in a pane of Herdr, and it uses the CLI of Herdr to
control the pane of the agent.

## The parts

### The driver

The file is `~/.local/bin/toutui-loop`. It is outside the repository: it belongs to
the machine of the maintainer, and not to the program.

The driver does this:

```
make a pane at the right of the pane of the driver
start the agent "toutui" in that pane, of the kind "claude", with
    --dangerously-skip-permissions
give the focus to the pane of the agent

for each round, to a limit of 50:
    if the file "stop" is there       -> stop
    if the file "complete" is there   -> stop, and show the reason
    read the prompt of the next session out of docs/HANDOVER.md
    send "/clear" to the agent, and wait
    send the prompt and the suffix to the agent, and wait
    if the state is "blocked"         -> stop, and show what the agent asks
```

The limit of 50 rounds is a guard against a loop that runs away. It is not a limit
of the work.

### The prompt of a round

The driver takes the lines of `docs/HANDOVER.md` that come after the heading
`## The prompt for the next session` and that start with `> `. It removes the two
characters `> ` of each line. This is the same block that the maintainer copies by
hand today.

The driver adds a suffix to that block:

> This session is one round of a loop, and no person watches the keys. Do not stop
> to ask a question: take the decision, write the reason in `docs/HANDOVER.md`, and
> go on. End the round as every session ends, with the handover and the prompt for
> the next session.
>
> **The one condition that stops the loop.** If, and only if, no item of the road
> stays open and no condition of the road stays unmeasured, write the file
> `/home/nyverino/.local/state/toutui-loop/complete`, and put the reason in it. A
> round that closed its own items and left others open must not write it.

The suffix holds the path in full. A shell does not expand the character `~` of a
prompt, therefore the agent must see the whole path.

### The state of the loop

The directory is `~/.local/state/toutui-loop/`. It is outside the repository,
therefore no file of the loop can go into a commit of the fork by an error.

| The file | Who writes it | What it does |
|---|---|---|
| `complete` | the agent | The loop stops, and it shows the reason. This is the one condition of the end. |
| `stop` | the maintainer | The loop ends the round that runs, and then it stops. |
| `driver.log` | the driver | One block for each round. |

The driver removes `complete` and `stop` at its start.

## What the maintainer sees

The tab holds two panes.

The pane of the left is the driver. It writes one block for each round: the number
of the round, the time of the start, the commit at the start, the commit at the
end, the state that Herdr gave, and the time that the round took.

The pane of the right is the program of the agent. The maintainer watches it as
they watch a session today.

The driver does not copy the text of the agent into a file. The program of the
agent draws on the alternate screen of the terminal, and `herdr agent read` cannot
give the lines that left that screen. The record of the work stays what it is
today: the commits, and `docs/HANDOVER.md`. The log of the driver is an index over
those.

## The states of Herdr

`herdr agent prompt <name> <text> --wait` waits for the first settled state of the
pane: `idle`, `done`, or `blocked`. **That call alone is not enough, and a
measurement of the live program shows why.** The wait follows the state of the
pane, and not the turn that the driver sent. The pane is settled at the moment
that a round starts, because the round before it settled the pane. The wait
therefore gives an answer at once, and the round says that it ended one second
after it started.

The measurement, with two rounds and a true program of Claude:

| The time | The event |
|---|---|
| 23:23:11 | round 1 starts |
| 23:23:12 | **round 1 says that it ends** |
| 23:23:12 | round 2 starts, and it reads `docs/HANDOVER.md` |
| 23:23:37 | **the commit of the session of round 1** |

The driver read the handover of round 2 twenty-five seconds before the session of
round 1 wrote it. **The handover is the one memory of the loop**, therefore every
round would take the words of the round before it, and the loop would run one
session again and again for ever. With a limit of 50 rounds the driver would push
50 prompts into the queue of the program in a few seconds.

**The round waits in three steps**, and each of them is measured:

1. Send with `herdr agent prompt <name> <text>`, and **with no `--wait`**.
2. `herdr agent wait <name> --until working --timeout 30000`. This is the proof
   that the turn began. It gave an answer after one second.
3. `herdr agent wait <name> --timeout <the limit of the round>` for the settled
   state.

A turn can begin and end inside those 30 seconds, and then step 2 fails for a
round that worked. `state_change_seq` of `herdr agent get` tells the two apart: a
sequence that moved is a turn that happened, and a sequence that did not move is a
prompt that the program never took.

**These three steps are for the words of the session, and not for the `/clear`.**
A `/clear` is not a turn, and a measurement of the live program shows it:

| The signal | Before the `/clear` | After it |
|---|---|---|
| `agent_status` | `idle` | `idle` |
| `state_change_seq` | 384 | **384** |
| `wait --until working` | — | it fails after 8 seconds |

The program takes the command, and it has nothing to think about.

**Both of the simple ways to send a `/clear` fail, and each one killed a run.**

| The way | What happens |
|---|---|
| `agent prompt --wait` | Herdr gives `agent_prompt_stalled`. A prompt from a state that is not `working` must make a change of the lifecycle inside **5 seconds**, and a `/clear` makes none. The round says "the agent did not take the command /clear" |
| `agent prompt`, and the words of the session at once after it | **The two prompts break into each other.** The pane held `arDo.`, `Did you mean /clear?`, and `Args from unknown skill: you see the word BANANA…`. The program was in the middle of the command when the second paste came |

The first live run passed its `/clear` twice by chance alone: round 1 sent it
while the program still started, and round 2 sent it while the program still
worked. Each of those is a change of the lifecycle, and each satisfied the wait.

**The way that works, and every step of it is measured:**

1. `herdr agent prompt <name> "/clear"`, with no `--wait` and no test of its
   status.
2. A pause of **3 seconds**.
3. The three steps above, with the words of the session.

The proof: a word went into a session, then those steps ran with a question that
asks whether that word stands anywhere earlier in the conversation. The wait for
`working` gave 0, the wait for the settled state gave 0, and **the program
answered `NO`**.

**Herdr holds no signal that says a `/clear` took effect**, therefore the driver
tests nothing there. A `/clear` that fails is silent, and the round after it
carries the context of the round before it. The measurement of the part below is
the one thing that says the command works at all.

The maintainer can still write into the pane while the driver waits, and that
exchange can settle the pane. The file `stop` is the clean way in: the maintainer
makes that file, the driver stops at the end of the round, and the pane of the
agent is theirs.

A round that gives `blocked` stops the loop. The driver shows the lines of the pane
with `herdr agent read`, and the maintainer decides.

## The guards of a long run

A run of three rounds does not reach the faults of a run of fifty. Each of these
was measured with a false `herdr`, and each one holds now.

| The condition | What the driver did | What it does |
|---|---|---|
| A session that does nothing | 12 rounds, and the status 0 | A round with no new commit **and** no change of the handover is a barren round. **Two of them in a row stop the loop.** One does not |
| `agent prompt` fails | Four "clean" rounds, and no prompt sent at all | The status of both sends is tested. A send that fails names itself in the log, and the round gives 2 |
| A second driver of a forgetful maintainer | `state_init` reset the marker of the run, therefore the first driver never saw its own `stop` or `complete`, and two drivers wrote prompts into one agent | A lock in `$STATE_DIR/lock` holds the process id, and it comes **before** `state_init`. `kill -0` says whether that process lives. A driver that loses dies and changes nothing |
| `complete` of the last round | Lost. The log said "the loop made 1 rounds, and that is its limit" | The sentinels are read again after the loop, and in the branch of a round that did not settle |
| `agent read` fails | The one path that explains a stop died before it explained | The pipeline is guarded, and a read that fails still leaves a line |
| The session ends on another branch | Nothing saw it | The log holds the branch and the count of the lines of `git status --porcelain`. A branch that changed inside a round ends the loop |
| "What did round 30 say?" | No answer. The handover is overwritten every round | The text of each round goes to `$STATE_DIR/prompt-<n>.txt` before the round |

**The lock does not hold against two drivers that start in the same instant.**
This is a tool of one person, and the lock is for the second start of a
maintainer who thinks the loop is stuck.

**A block of a prompt whose first line starts with `/` is refused.** The block
comes from a program, and a line that starts with `/` goes to another program as
a command and not as words.

## The faults, and what the driver does with each

| The fault | What the driver does |
|---|---|
| `docs/HANDOVER.md` holds no block of a prompt | Stop, and say the heading that it did not find. A round that sends an empty prompt is worse than no round. |
| A round takes more than the limit of time (4 hours) | Stop. A round that never settles holds the loop for ever. |
| The state is `blocked` | Stop, and show the lines of the pane. |
| Herdr gives `agent_prompt_stalled` | Stop. The agent did not answer the prompt at all. |
| The pane of the agent is gone | Stop. |

## The sharp edge

The suffix tells the agent not to ask a question, and the agent starts with
`--dangerously-skip-permissions`. Together, they give the judgement of the
maintainer to the program: it makes tags, and it pushes them, and no person sees
that before it happens. This is what an unattended loop is. The rules of
`docs/HANDOVER.md` already say "tag, push, and go on", therefore the loop does not
change the way of working — but it removes the last person who could stop a round.

The file `stop` and the limit of 50 rounds are the answer to that, and they are the
only answer that this design holds.

## The measurement of `/clear`

A driver can send the characters `/clear`, and a program can take them as the
words of a prompt and not as a command. No test of a shell can tell the two apart:
it needs the true program in a true terminal.

The measurement asks each round to count the turns of the user that it can see
before its own, and to write that count to a file. Two rounds ran, and each of
them wrote **"I see 0 turns before this one"**. **The command `/clear` works**,
and a session of the loop starts with nothing behind it.

## What this design does not hold

- No test of the driver. It is a script of the machine of the maintainer, and the
  measurement of it is a run of two rounds.
- No record of the text of the agent. See the part above of the alternate screen.
- No round that runs at the same time as another round. The loop is one agent, and
  one round at a time.
