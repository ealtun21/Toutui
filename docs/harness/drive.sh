#!/usr/bin/env bash
# The harness that drives the real program inside tmux. See docs/HANDOVER.md.
#
# **A fixed `sleep` is the largest waste of a session.** A measurement of
# 2026-08-11: the first frame of the program comes after 673 milliseconds, and the
# session before it slept 17 seconds before each of about 30 measurements. That is
# about eight minutes of nothing.
#
# Every function of this file polls for a marker, and every poll holds a timeout.
# A poll with no timeout holds a session for ever.
#
# Give this file to your shell, and then use the functions:
#
#   source docs/harness/drive.sh
#   start_the_program                       # it comes back at the first frame
#   press j; wait_for "Alice in Wonderland"
#   the_screen | head -20
#   stop_the_program
#
# The variables of the sandbox come from docs/TEST-SERVER.md. Set
# TOUTUI_HARNESS_BINARY for a build of the release.

set -o pipefail

# The name of the session of tmux. A second harness needs a second name.
: "${SESSION:=check}"

# The size of the terminal. A width of 80 columns takes the covers away, and a
# view of the covers needs 160.
: "${COLUMNS_OF_THE_SCREEN:=160}"
: "${ROWS_OF_THE_SCREEN:=45}"

# The longest time of a poll, in seconds.
: "${TIMEOUT:=30}"

# The time between two looks at the screen, in seconds.
: "${LOOK_AGAIN:=0.2}"

# The isolated configuration of the sandbox. **Never the configuration of the
# user**, and never the server of the user.
: "${ABS:=$HOME/.local/share/toutui-abs-test}"
: "${TOUTUI_HARNESS_BINARY:=./target/debug/toutui}"

# The null device of ALSA for `cargo test`. **It does not silence the real
# program**: ask the user before a measurement that plays sound. See the memory
# of the harness and T-68.
: "${ASOUND:=}"

# Gives the whole screen. `-e` keeps the colours.
the_screen() {
    tmux capture-pane -p ${1:+-e} -t "$SESSION"
}

# Waits for a text of the screen. It gives 0 when the text came, and 1 when the
# time went by.
#
#   wait_for "Continue Listening"
#   wait_for "the part of the stream" 60      # a longer timeout
wait_for() {
    local marker="$1"
    local limit="${2:-$TIMEOUT}"
    local start
    start=$(date +%s%3N)

    while :; do
        if the_screen 2>/dev/null | grep -qF -- "$marker"; then
            printf 'the marker "%s" came after %d ms\n' \
                "$marker" "$(( $(date +%s%3N) - start ))" >&2
            return 0
        fi

        if (( $(date +%s%3N) - start > limit * 1000 )); then
            printf 'the marker "%s" did not come in %s s. The screen holds:\n' \
                "$marker" "$limit" >&2
            the_screen 2>/dev/null | grep -v '^ *$' | head -12 >&2
            return 1
        fi

        sleep "$LOOK_AGAIN"
    done
}

# Waits while a text stands on the screen. A message of the program lives six
# seconds, therefore a measurement of the message that comes after it must wait
# for the first one to go.
wait_while() {
    local marker="$1"
    local limit="${2:-$TIMEOUT}"
    local start
    start=$(date +%s%3N)

    while the_screen 2>/dev/null | grep -qF -- "$marker"; do
        if (( $(date +%s%3N) - start > limit * 1000 )); then
            printf 'the marker "%s" stays after %s s\n' "$marker" "$limit" >&2
            return 1
        fi
        sleep "$LOOK_AGAIN"
    done

    return 0
}

# Starts the program, and it comes back at the first frame of the Home view.
#
# The first argument holds more variables of the environment, for example
# "TOUTUI_EBOOK_CACHE_BYTES=200000".
start_the_program() {
    local more="$1"
    local marker="${2:-Continue Listening}"

    stop_the_program

    : > "$ABS/toutui-config/toutui/toutui.log"

    tmux new-session -d -s "$SESSION" \
        -x "$COLUMNS_OF_THE_SCREEN" -y "$ROWS_OF_THE_SCREEN" \
        "env XDG_CONFIG_HOME=$ABS/toutui-config XDG_DATA_HOME=$ABS/toutui-data \
             ${ASOUND:+ALSA_CONFIG_PATH=$ASOUND} $more $TOUTUI_HARNESS_BINARY"

    wait_for "$marker" 40
}

# Stops the program. It gives no fault for a session that does not stand.
stop_the_program() {
    tmux kill-session -t "$SESSION" 2>/dev/null
    return 0
}

# Presses a key, or a sequence of keys.
#
#   press j
#   press / ; press "carroll" ; press Enter
press() {
    tmux send-keys -t "$SESSION" "$@"
}

# Gives the lines of the log of the program that hold a text.
the_log() {
    grep -E "${1:-.}" "$ABS/toutui-config/toutui/toutui.log" | tail -"${2:-10}"
}

# Says that the program stands and that it answers. A program that stopped gives
# a screen with no line.
the_program_stands() {
    tmux has-session -t "$SESSION" 2>/dev/null
}
