#!/usr/bin/env bash
# The harness that drives the real program inside a window of kitty, for the
# covers. See docs/HANDOVER.md and T-137.
#
# **`drive.sh` cannot measure a cover of a user.** The program inside tmux asks
# the terminal nothing, therefore it draws the covers with blocks of Unicode
# (`asks_the_terminal` of `src/ui/cover.rs`). A user of kitty, of ghostty, or of
# WezTerm gets the graphics protocol, and **the letters of a screen of tmux hold
# no byte of that protocol**.
#
# This harness opens a window of kitty of its own, with the remote control of
# kitty. Therefore a session can press a key, read the screen, and see the
# covers of the protocol that the user sees.
#
# The window stands on the screen of the user. **Ask the user before you open
# it**, as you ask before a measurement that plays sound.
#
# Give this file to your shell, and then use the functions:
#
#   source docs/harness/kitty.sh
#   start_the_kitty                       # the window of the measurement
#   start_the_program_in_kitty            # it comes back at the first frame
#   press_in_kitty j; the_kitty_screen | head -20
#   the_covers                            # the pictures of the screen
#   stop_the_kitty
#
# The program runs under `script`, therefore `$THE_BYTES` holds every byte that
# the program wrote to the terminal: `the_covers` reads that file and it says if
# a picture of the screen came from the program.

set -o pipefail

# The socket of the remote control of kitty. A second harness needs a second
# name.
: "${KITTY_SOCKET:=/run/user/$(id -u)/toutui-harness.sock}"

# The size of the window, in cells. A width of 80 columns takes the covers away,
# and a view of the covers needs 160.
: "${COLUMNS_OF_THE_SCREEN:=160}"
: "${ROWS_OF_THE_SCREEN:=45}"

# The longest time of a poll, in seconds, and the time between two looks.
: "${TIMEOUT:=40}"
: "${LOOK_AGAIN:=0.2}"

# The isolated configuration of the sandbox. **Never the configuration of the
# user**, and never the server of the user.
: "${ABS:=$HOME/.local/share/toutui-abs-test}"
: "${TOUTUI_HARNESS_BINARY:=./target/debug/toutui}"

# The file of the bytes that the program wrote to the terminal.
: "${THE_BYTES:=${TMPDIR:-/tmp}/toutui-harness-bytes}"

# The remote control of kitty.
the_kitty() {
    kitten @ --to "unix:$KITTY_SOCKET" "$@"
}

# Opens the window of the measurement. It gives no fault for a window that
# stands already.
start_the_kitty() {
    if the_kitty ls > /dev/null 2>&1; then
        return 0
    fi

    setsid kitty \
        -o allow_remote_control=yes \
        -o confirm_os_window_close=0 \
        --listen-on "unix:$KITTY_SOCKET" \
        --title toutui-harness \
        bash --norc > /dev/null 2>&1 &

    local start
    start=$(date +%s%3N)

    while ! the_kitty ls > /dev/null 2>&1; do
        if (( $(date +%s%3N) - start > TIMEOUT * 1000 )); then
            echo "the window of kitty did not answer in $TIMEOUT s" >&2
            return 1
        fi
        sleep "$LOOK_AGAIN"
    done

    the_kitty resize-os-window \
        --width "$COLUMNS_OF_THE_SCREEN" \
        --height "$ROWS_OF_THE_SCREEN" \
        --unit cells > /dev/null 2>&1

    # A window that a compositor holds keeps its size. Therefore the screen of
    # the measurement can be wider than the two values above, and every function
    # of this file reads the true size.
    return 0
}

# Closes the window. It gives no fault for a window that does not stand.
stop_the_kitty() {
    the_kitty close-os-window > /dev/null 2>&1
    return 0
}

# Gives the size of the screen, as "columns rows".
the_kitty_size() {
    the_kitty ls | python3 -c '
import json, sys
window = json.load(sys.stdin)[0]["tabs"][0]["windows"][0]
print(window["columns"], window["lines"])
'
}

# Starts the program, and it comes back at the first frame of the Home view.
#
# The first argument holds more variables of the environment, for example
# "TOUTUI_EBOOK_CACHE_BYTES=200000".
start_the_program_in_kitty() {
    local more="$1"
    local marker="${2:-Continue Listening}"

    : > "$ABS/toutui-config/toutui/toutui.log"
    : > "$THE_BYTES"

    the_kitty send-text "script -q -f -c '\
env XDG_CONFIG_HOME=$ABS/toutui-config XDG_DATA_HOME=$ABS/toutui-data \
$more $TOUTUI_HARNESS_BINARY' $THE_BYTES
"

    wait_for_in_kitty "$marker"
}

# Stops the program with the key that closes it.
stop_the_program_in_kitty() {
    press_in_kitty Q
    sleep 1
    return 0
}

# Presses a key, or a sequence of keys. The keys go as letters, therefore a key
# of a name (Enter, Tab) needs `the_kitty send-key`.
#
#   press_in_kitty j
#   press_in_kitty RRR
press_in_kitty() {
    the_kitty send-text "$@"
}

# Gives the whole screen, with no escape.
the_kitty_screen() {
    the_kitty get-text --extent screen
}

# Gives the whole screen with its escapes. The covers stand in those escapes.
the_kitty_screen_with_the_escapes() {
    the_kitty get-text --ansi --extent screen
}

# Waits for a text of the screen. It gives 0 when the text came, and 1 when the
# time went by.
wait_for_in_kitty() {
    local marker="$1"
    local limit="${2:-$TIMEOUT}"
    local start
    start=$(date +%s%3N)

    while :; do
        if the_kitty_screen 2>/dev/null | grep -qF -- "$marker"; then
            printf 'the marker "%s" came after %d ms\n' \
                "$marker" "$(( $(date +%s%3N) - start ))" >&2
            return 0
        fi

        if (( $(date +%s%3N) - start > limit * 1000 )); then
            printf 'the marker "%s" did not come in %s s. The screen holds:\n' \
                "$marker" "$limit" >&2
            the_kitty_screen 2>/dev/null | grep -v '^ *$' | head -12 >&2
            return 1
        fi

        sleep "$LOOK_AGAIN"
    done
}

# Gives the pictures of the screen: the place of each row of a picture, the
# identity of that picture, and the pictures that the program sent.
#
# It gives 1 when the screen holds the placeholder of a picture that the program
# did not send. See docs/harness/covers.py.
the_covers() {
    local where
    where="$(dirname "${BASH_SOURCE[0]}")"

    the_kitty_screen_with_the_escapes | python3 "$where/covers.py" --transmits "$THE_BYTES"
}

# Gives the lines of the log of the program that hold a text.
the_log_of_the_kitty() {
    grep -E "${1:-.}" "$ABS/toutui-config/toutui/toutui.log" | tail -"${2:-10}"
}
