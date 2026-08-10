#!/usr/bin/env bash
# Write the paths and the commands that remove toutui from macOS.
#
# Use:
#   curl -LsSf https://raw.githubusercontent.com/ealtun21/Toutui/main/macos/uninstall.sh | bash
#
# THE SCRIPT DELETES NOTHING. The script writes the paths and the commands.
# You read each command, and you run the commands that you want.
#
# The program has the same function: `toutui --uninstall`. Use that command
# first. This script answers the conditions in which that command cannot run:
#
#   1. The binary is already absent, and the configuration is still on the
#      disk.
#   2. A browser received the archive of the release. macOS then puts the
#      attribute com.apple.quarantine on the files, and Gatekeeper stops the
#      binary. The user cannot run the program at all.
#
# The fork gives no bundle of an application for macOS. Therefore this list
# names no bundle. See T-31 in docs/TAKEOVER-BACKLOG.md.

set -euo pipefail

# Gives the directory of configuration. The rule agrees with config_dir() in
# install.sh and with paths::config_dir in the program.
config_dir() {
    if [ -n "${XDG_CONFIG_HOME:-}" ]; then
        echo "${XDG_CONFIG_HOME}/toutui"
    else
        echo "${HOME}/Library/Preferences/toutui"
    fi
}

# Gives the directory of the data. The rule agrees with paths::data_dir in the
# program. macOS uses the same path as Linux here.
data_dir() {
    if [ -n "${XDG_DATA_HOME:-}" ]; then
        echo "${XDG_DATA_HOME}/toutui"
    else
        echo "${HOME}/.local/share/toutui"
    fi
}

# Gives the path of the binary. The script looks in PATH first, because the
# user can install the binary in /usr/local/bin or in ${HOME}/.cargo/bin.
binary_path() {
    command -v toutui 2>/dev/null || true
}

# Tells if a path is inside the home directory of the user.
#
# The user owns such a path, therefore the command needs no sudo. A path in
# /usr/local/bin belongs to the system, and that path needs sudo.
inside_home() {
    case "$1" in
        "${HOME}"/*) return 0 ;;
        *) return 1 ;;
    esac
}

# Writes the command that deletes one path.
# $1 = the path, $2 = "dir" for a directory or "file" for a file.
write_command() {
    local path="$1" kind="$2" flags root

    if [ "$kind" = "dir" ]; then
        flags="-rf"
    else
        flags="-f"
    fi

    if inside_home "$path"; then
        root=""
    else
        root="sudo "
    fi

    printf "    %srm %s '%s'\n" "$root" "$flags" "$path"
}

main() {
    local config data binary

    config=$(config_dir)
    data=$(data_dir)
    binary=$(binary_path)

    echo "This script deletes nothing. It writes the paths and the commands."
    echo
    echo "Your installation put these paths on the disk:"
    echo
    echo "    the configuration, the secret key, the database, and the log"
    echo "        ${config}"
    echo "        it holds config.toml, .env, db.sqlite3, and toutui.log"
    echo "    the binary"
    if [ -n "$binary" ]; then
        echo "        ${binary}"
    else
        echo "        no binary toutui is in your PATH"
        echo "        look in /usr/local/bin and in ${HOME}/.cargo/bin"
    fi
    echo "    the downloads for the offline mode"
    echo "        ${data}"
    echo "        it holds downloads/<user>"
    echo
    echo "Run these commands to delete the paths. Read each command first:"
    echo
    write_command "$config" dir
    if [ -n "$binary" ]; then
        write_command "$binary" file
    else
        echo "    Find the binary first, then delete it."
    fi
    write_command "$data" dir
    echo
    echo "Keep the configuration if you want to install the program again."
}

main "$@"
