#!/usr/bin/env python3
"""A program whose terminal goes away with no SIGHUP. See T-271.

`tmux kill-session` is the SIGHUP of a terminal that goes away (the trap 103),
and the default action of that signal stops the program. Therefore that road
measures nothing of the program itself: the kernel does the work.

**A program does not always get that signal.** The kernel sends SIGHUP to the
foreground process group of the terminal alone, therefore a program that the
user put in the background, a program of `nohup`, and a program of a unit of
systemd whose terminal goes away each stay. This harness gives that condition:
it sets the disposition of SIGHUP to `SIG_IGN`, and an ignored disposition
stays over `exec` (POSIX). It then starts the program of its command line.

    tmux new-session -d -s check -x 160 -y 45 \\
        "env XDG_CONFIG_HOME=... python3 docs/harness/the_terminal_of_the_program_goes_away.py \\
         ./target/debug/toutui"
    tmux kill-session -t check
    # The program of the pane stays. Read its state with `ps`.

The harness changes no byte of the program and no row of the sandbox.
"""

import os
import signal
import sys


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2

    # An ignored disposition stays over `exec`. A handler does not.
    signal.signal(signal.SIGHUP, signal.SIG_IGN)

    os.execvp(sys.argv[1], sys.argv[1:])


if __name__ == "__main__":
    sys.exit(main())
