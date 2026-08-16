#!/usr/bin/env bash
# The harness that sends a report of the mouse to the real program inside tmux.
# See docs/HANDOVER.md and the item T-316 of docs/TAKEOVER-BACKLOG.md.
#
# **A session of tmux that no person looks at has no mouse.** `tmux click` does
# not exist, and `tmux send-keys -X` drives the copy mode of tmux and not the
# program of the pane. Therefore this harness writes the bytes of the report
# itself: a terminal that holds a mouse sends those same bytes to the standard
# input of the program, and `crossterm` reads them and it makes an
# `Event::Mouse` of them.
#
# The bytes are the report SGR of the mode 1006 (`CSI < b ; x ; y M` for a press
# of a button, and the same with a small `m` for the release of it). **The
# column and the row start at 1**, as the user counts them, and the top left
# corner of the screen is the column 1 and the row 1. The value `b` holds the
# button: 0 is the button at the left, 1 is the button of the middle, 2 is the
# button at the right, 64 is the wheel up, and 65 is the wheel down; the value
# 32 above a button says that the pointer moves while that button stands down.
#
# **A program that did not enable the capture of the mouse reads these bytes
# too.** The capture tells the terminal to send the reports; it does not change
# the parser of the program. That is the value of this harness for the
# measurement of a fault: the report reaches the program of today, and the
# program of today does nothing with it.
#
# Give this file to your shell after docs/harness/drive.sh, and then use the
# functions:
#
#   source docs/harness/drive.sh
#   source docs/harness/click.sh
#   start_the_program "TOUTUI_AUDIO_DEVICE=null"
#   click 40 12                      # the button at the left, and its release
#   wheel_down 40 12
#   the_screen | head -20
#
# The variable SESSION of docs/harness/drive.sh names the session of tmux, and
# this file uses that same session.

: "${SESSION:=check}"

# The button at the left, the button of the middle, and the button at the right.
THE_BUTTON_AT_THE_LEFT=0
THE_BUTTON_OF_THE_MIDDLE=1
THE_BUTTON_AT_THE_RIGHT=2

# The wheel. A terminal sends no release for a step of the wheel.
THE_WHEEL_UP=64
THE_WHEEL_DOWN=65

# The value that says that the pointer moves while a button stands down.
THE_POINTER_MOVES=32

# Sends one report of the mouse. It takes the button, the column, the row, and
# the end of the report ("M" for a press or a move, and "m" for a release).
#
# The bytes go as hexadecimal, because the report starts with the character of
# the escape and a shell of a pipeline can lose it.
the_report_of_the_mouse() {
    local button="$1" column="$2" row="$3" end="$4"

    printf '\033[<%d;%d;%d%s' "$button" "$column" "$row" "$end" \
        | od -An -tx1 \
        | tr -s ' \n' '\n' \
        | grep -v '^$' \
        | xargs tmux send-keys -t "$SESSION" -H
}

# Presses a button of the mouse at a column and a row, and it holds it down.
#
#   press_the_button 40 12                       # the button at the left
#   press_the_button 40 12 $THE_BUTTON_AT_THE_RIGHT
press_the_button() {
    local column="$1" row="$2" button="${3:-$THE_BUTTON_AT_THE_LEFT}"
    the_report_of_the_mouse "$button" "$column" "$row" M
}

# Releases a button of the mouse at a column and a row.
release_the_button() {
    local column="$1" row="$2" button="${3:-$THE_BUTTON_AT_THE_LEFT}"
    the_report_of_the_mouse "$button" "$column" "$row" m
}

# A click: the press of a button and the release of it at the same place.
#
#   click 40 12
#   click 40 12 $THE_BUTTON_AT_THE_RIGHT
click() {
    local column="$1" row="$2" button="${3:-$THE_BUTTON_AT_THE_LEFT}"
    press_the_button "$column" "$row" "$button"
    release_the_button "$column" "$row" "$button"
}

# Moves the pointer to a column and a row while a button stands down. Use it
# between press_the_button and release_the_button for a drag.
#
#   press_the_button 20 40 ; drag_to 60 40 ; release_the_button 60 40
drag_to() {
    local column="$1" row="$2" button="${3:-$THE_BUTTON_AT_THE_LEFT}"
    the_report_of_the_mouse "$(( button + THE_POINTER_MOVES ))" \
        "$column" "$row" M
}

# One step of the wheel up at a column and a row.
wheel_up() {
    the_report_of_the_mouse "$THE_WHEEL_UP" "$1" "$2" M
}

# One step of the wheel down at a column and a row.
wheel_down() {
    the_report_of_the_mouse "$THE_WHEEL_DOWN" "$1" "$2" M
}

# Gives the number of the row of the screen that holds a text, and 0 for a text
# that no row holds. The rows start at 1, as this harness counts them.
#
#   the_row_of "Alice in Wonderland"
the_row_of() {
    tmux capture-pane -p -t "$SESSION" \
        | grep -nF -- "$1" \
        | head -1 \
        | cut -d: -f1
}
