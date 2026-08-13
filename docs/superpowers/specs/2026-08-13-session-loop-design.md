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
pane: `idle`, `done`, or `blocked`. The driver uses that call for the `/clear` and
for the prompt of the round.

The wait follows the state of the pane, and not the turn that the driver sent.
Therefore, if the maintainer writes into the pane of the agent while the driver
waits, the driver can see the settled state of that exchange and start the next
round too early. The file `stop` is the clean way in: the maintainer makes that
file, the driver stops at the end of the round, and the pane of the agent is
theirs.

A round that gives `blocked` stops the loop. The driver shows the lines of the pane
with `herdr agent read`, and the maintainer decides.

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

## What this design does not hold

- No test of the driver. It is a script of the machine of the maintainer, and the
  measurement of it is a run of two rounds.
- No record of the text of the agent. See the part above of the alternate screen.
- No round that runs at the same time as another round. The loop is one agent, and
  one round at a time.
