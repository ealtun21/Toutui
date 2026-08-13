#!/usr/bin/env python3
"""The covers of the Kitty protocol, as the terminal of the user holds them.

**A screen of tmux says nothing of a cover.** `capture-pane` gives the letters
of the screen, and the harness of `drive.sh` therefore measures the covers of
blocks of Unicode only: the program inside tmux asks the terminal nothing (see
`asks_the_terminal` of `src/ui/cover.rs`). The terminal of a user answers, and
it then draws the covers with the graphics protocol of kitty.

This file reads that protocol. The program draws a cover with the **unicode
placeholders** of that protocol: every cell of the picture holds the character
U+10EEEE, and **the colour of the letter of that cell holds the identity of the
picture**. Therefore the letters of the screen of kitty say where each picture
stands, and which picture it is.

    kitten @ --to <socket> get-text --ansi --extent screen \
        | docs/harness/covers.py --transmits <the file of the bytes>

The report holds one line for each row of a picture, and then the identities:

- **The pictures of the screen.** More than one identity in one place, or an
  identity that stays after the screen changed, is a picture that the program
  did not take away.
- **The pictures that the program sent.** The option `--transmits` names a file
  of the bytes that the program wrote to the terminal (`script -f` writes such a
  file). **A picture of the screen that no byte of that file sent is a fault**:
  the terminal holds the placeholder of a picture that it does not have, and the
  user sees an empty place or the picture of another moment.

The identity of the graphics protocol is a number of 32 bits. Three of those
bytes stand in the colour of the letter, therefore this file names a picture
with those three numbers, as `247.231.226`.
"""

import argparse
import re
import sys

# The character of a placeholder of the graphics protocol of kitty.
PLACEHOLDER = "\U0010EEEE"

# The colour of the letter of a cell, in the two forms that a terminal writes.
COLOUR = re.compile(r"\x1b\[38[:;]2[:;](\d+)[:;](\d+)[:;](\d+)m" + re.escape(PLACEHOLDER))

# Every escape of a screen. The report needs the columns, therefore it takes
# them away before it counts.
ESCAPE = re.compile(r"\x1b\[[0-9;:]*[a-zA-Z]")

# The first chunk of a picture that the program sends. `a=T` places the picture
# and `U=1` makes that placement a virtual one, for the placeholders above.
TRANSMIT = re.compile(rb"i=(\d+),a=T,U=1")


def the_name_of(identity):
    """Gives the three numbers of the colour of an identity of 32 bits."""
    byte = identity.to_bytes(4, "big")
    return f"{byte[1]}.{byte[2]}.{byte[3]}"


def the_rows_of_the_pictures(screen):
    """Gives one answer for each row of the screen that holds a picture."""
    rows = []

    for number, line in enumerate(screen.split("\n")):
        if PLACEHOLDER not in line:
            continue

        names = sorted({".".join(found.groups()) for found in COLOUR.finditer(line)})
        letters = ESCAPE.sub("", line)

        rows.append(
            {
                "row": number,
                "column": letters.index(PLACEHOLDER),
                "cells": letters.count(PLACEHOLDER),
                "names": names,
            }
        )

    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--transmits",
        help="the file of the bytes that the program wrote to the terminal",
    )
    parser.add_argument(
        "--screen",
        help="the file of the screen with its escapes. The standard input, with no such file",
    )
    given = parser.parse_args()

    if given.screen:
        with open(given.screen, encoding="utf-8", errors="replace") as file:
            screen = file.read()
    else:
        screen = sys.stdin.read()

    rows = the_rows_of_the_pictures(screen)

    for row in rows:
        print(
            f"row {row['row']:3d} column {row['column']:3d} "
            f"cells {row['cells']:3d} the picture {' and '.join(row['names']) or '(no colour)'}"
        )

    on_the_screen = sorted({name for row in rows for name in row["names"]})
    print(f"the rows of a picture: {len(rows)}")
    print(f"the pictures of the screen: {len(on_the_screen)} {on_the_screen}")

    if not given.transmits:
        return 0

    with open(given.transmits, "rb") as file:
        bytes_of_the_terminal = file.read()

    sent = [int(found.group(1)) for found in TRANSMIT.finditer(bytes_of_the_terminal)]
    names_of_the_sent = {the_name_of(identity) for identity in sent}

    print(f"the pictures that the program sent: {len(sent)}")

    with_no_picture = [name for name in on_the_screen if name not in names_of_the_sent]

    if with_no_picture:
        print(f"**a placeholder of a picture that the program did not send**: {with_no_picture}")
        return 1

    print("every picture of the screen came from the program")
    return 0


if __name__ == "__main__":
    sys.exit(main())
