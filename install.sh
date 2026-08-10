#!/usr/bin/env bash
# Install toutui.
#
# Use:
#   curl -LsSf https://raw.githubusercontent.com/ealtun21/Toutui/main/install.sh | bash
#
# The script receives the archive of the last release, it compares the sum
# with SHA256SUMS of that release, and it installs the binary. If the command
# gh is on the system, the script also tests the proof of the origin of the
# archive, and it stops if that proof is not correct. The script installs no
# other program, because toutui plays the audio itself. The script asks for a
# password with sudo, if the directory of the binary needs one.

set -euo pipefail

REPO="ealtun21/Toutui"
API="https://api.github.com/repos/${REPO}/releases/latest"
BIN_DIR="${TOUTUI_BIN_DIR:-/usr/local/bin}"
tmp=""

# The largest file that the script accepts, in bytes. A host that sends
# without end must not fill the disk. The number agrees with
# MAX_DOWNLOAD_BYTES in src/update/install.rs. See T-30.
MAX_BYTES=$((200 * 1024 * 1024))

# The workflow that has permission to make an archive of a release.
# `--signer-workflow` refuses a proof that a different workflow made.
SIGNER_WORKFLOW="${REPO}/.github/workflows/release.yml"

fail() {
    echo "[ERROR] $1" >&2
    exit 1
}

identify_target() {
    local os arch
    os=$(uname -s)
    arch=$(uname -m)

    case "$os" in
        Darwin) echo "universal-apple-darwin" ;;
        Linux)
            case "$arch" in
                x86_64)  echo "x86_64-unknown-linux-gnu" ;;
                aarch64) echo "aarch64-unknown-linux-gnu" ;;
                *) fail "Linux $arch has no archive. Use: cargo install --git https://github.com/${REPO}" ;;
            esac
            ;;
        *) fail "$os has no archive. Use: cargo install --git https://github.com/${REPO}" ;;
    esac
}

config_dir() {
    if [ -n "${XDG_CONFIG_HOME:-}" ]; then
        echo "${XDG_CONFIG_HOME}/toutui"
    elif [ "$(uname -s)" = "Darwin" ]; then
        echo "${HOME}/Library/Preferences/toutui"
    else
        echo "${HOME}/.config/toutui"
    fi
}

size_of() {
    wc -c < "$1" | tr -d ' '
}

# Receives one address into one file, with a limit on the size.
# $1 = the address, $2 = the file.
#
# `--max-filesize` stops a download whose header gives a size above the
# limit. That header is not enough, because a host can send no header at all,
# and the download then had no limit. Therefore `head` takes one byte more
# than the limit, and the size of the file on the disk tells if the host sent
# too much. `head` also closes the pipe, thus the download stops.
receive() {
    local url="$1" out="$2"
    local status=0

    curl -sSfL --max-filesize "$MAX_BYTES" "$url" \
        | head -c "$((MAX_BYTES + 1))" > "$out" || status=$?

    # The exit status 63 is the limit of curl itself. A newer curl stops the
    # download at the limit, and an older curl stops only when the header
    # gives a size above the limit.
    if [ "$status" -eq 63 ] || [ "$(size_of "$out")" -gt "$MAX_BYTES" ]; then
        fail "The address ${url} sends more than ${MAX_BYTES} bytes."
    fi

    return "$status"
}

sum_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

# Compares the sum of a file on the disk with its sum in SHA256SUMS.
# $1 = the file on the disk, $2 = its name in SHA256SUMS, $3 = SHA256SUMS.
sum_agrees() {
    local file="$1" name="$2" sums="$3"
    local expected actual
    expected=$(awk -v n="$name" '$2 == n || $2 == "*" n {print $1}' "$sums") || true
    [ -n "$expected" ] || return 1
    actual=$(sum_of "$file")
    [ "$expected" = "$actual" ]
}

# Tests the proof of the origin of one file. $1 = the file.
#
# The sum SHA-256 comes from SHA256SUMS of the same release. Therefore the sum
# finds a download that stops, and it does not find a release that a different
# person made. The workflow release.yml makes a proof of each archive, and gh
# reads that proof. See T-29.
#
# A refusal of the proof stops the script. A condition that stops the test
# itself, for example an account that gh does not have, gives a warning only,
# because the archive can still be correct.
verify_proof() {
    local file="$1"
    local out lower status=0

    if ! command -v gh >/dev/null 2>&1; then
        echo "[WARN] The script did not test the proof of the origin, because gh is not on this system." >&2
        echo "[WARN] The script tested the sum SHA-256 only. That sum comes from the same release." >&2
        echo "[WARN] Install gh from https://cli.github.com to test the proof." >&2
        return 0
    fi

    out=$(gh attestation verify "$file" --repo "$REPO" \
        --signer-workflow "$SIGNER_WORKFLOW" 2>&1) || status=$?

    if [ "$status" -eq 0 ]; then
        echo "[INFO] The proof of the origin is correct. The workflow of ${REPO} made this archive."
        return 0
    fi

    lower=$(printf '%s' "$out" | tr '[:upper:]' '[:lower:]')
    case "$lower" in
        *"unknown flag"*|*"unknown command"*|*"unknown shorthand"*|*"unknown subcommand"*)
            echo "[WARN] The command gh on this system is too old to test the proof." >&2
            ;;
        *"auth login"*|*"authentication"*|*"not logged in"*|*"http 401"*|*"http 403"*|*"bad credentials"*)
            echo "[WARN] The command gh has no account. Run gh auth login to test the proof." >&2
            ;;
        *"no such host"*|*"dial tcp"*|*"connection refused"*|*"i/o timeout"*)
            echo "[WARN] The command gh did not reach GitHub. The script did not test the proof." >&2
            ;;
        *)
            fail "The proof of the origin of ${file} is not correct. The script installed nothing.
${out}"
            ;;
    esac

    echo "[WARN] The script tested the sum SHA-256 only." >&2
    return 0
}

warn_if_no_alsa() {
    # The program connects to libasound at the time it runs. A system with a
    # desktop has that library. A small system does not, and the loader then
    # gives a message that names no package.
    if [ "$(uname -s)" != "Linux" ]; then
        return 0
    fi
    if ldconfig -p 2>/dev/null | grep -q 'libasound\.so\.2'; then
        return 0
    fi
    echo "[WARN] The library libasound.so.2 is not on this system." >&2
    echo "[WARN] toutui needs it to play audio. Install one of these:" >&2
    echo "           Debian, Ubuntu:  sudo apt install libasound2" >&2
    echo "           Fedora, RHEL:    sudo dnf install alsa-lib" >&2
    echo "           Arch:            sudo pacman -S alsa-lib" >&2
    echo "           openSUSE:        sudo zypper install libasound2" >&2
}

main() {
    [ "$(id -u)" -ne 0 ] || fail "Do not run this script as root."

    command -v curl >/dev/null 2>&1 || fail "Install curl first."
    command -v tar  >/dev/null 2>&1 || fail "Install tar first."

    local target archive tag
    target=$(identify_target)
    archive="toutui-${target}.tar.gz"

    # The temporary directory comes before the first download, because the
    # answer of the API goes into a file as well. A file has a size that the
    # script can test, and a variable does not.
    tmp=$(mktemp -d)
    trap 'if [ -n "${tmp:-}" ]; then rm -rf "$tmp"; fi' EXIT

    receive "$API" "${tmp}/latest.json" \
        || fail "GitHub does not answer. Try again after some minutes."
    tag=$(sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "${tmp}/latest.json" | head -1)
    [ -n "$tag" ] || fail "The repository has no release."

    echo "[INFO] The last release is ${tag}."

    local base="https://github.com/${REPO}/releases/download/${tag}"
    receive "${base}/${archive}" "${tmp}/${archive}" \
        || fail "The archive ${archive} did not arrive."
    receive "${base}/SHA256SUMS" "${tmp}/SHA256SUMS" \
        || fail "SHA256SUMS did not arrive."

    sum_agrees "${tmp}/${archive}" "$archive" "${tmp}/SHA256SUMS" \
        || fail "The sum of ${archive} is not correct."

    echo "[INFO] The sum is correct."

    verify_proof "${tmp}/${archive}"

    tar -xzf "${tmp}/${archive}" -C "$tmp"

    mkdir -p "$BIN_DIR" 2>/dev/null || true
    if [ -w "$BIN_DIR" ]; then
        install -m 755 "${tmp}/toutui" "${BIN_DIR}/toutui"
    else
        command -v sudo >/dev/null 2>&1 \
            || fail "Install sudo, or give TOUTUI_BIN_DIR a directory that you own."
        sudo install -m 755 "${tmp}/toutui" "${BIN_DIR}/toutui"
    fi
    echo "[INFO] The binary is in ${BIN_DIR}/toutui."

    local config
    config=$(config_dir)
    mkdir -p "$config"
    if [ ! -f "${config}/config.toml" ]; then
        if receive "${base}/config.example.toml" "${tmp}/config.example.toml"; then
            if sum_agrees "${tmp}/config.example.toml" "config.example.toml" "${tmp}/SHA256SUMS"; then
                cp "${tmp}/config.example.toml" "${config}/config.toml"
                echo "[INFO] The configuration is in ${config}/config.toml."
            else
                echo "[WARN] The sum of config.example.toml is not correct. Make ${config}/config.toml by hand." >&2
            fi
        else
            echo "[WARN] The configuration did not arrive. Make ${config}/config.toml by hand." >&2
        fi
    fi

    if [ ! -f "${config}/.env" ]; then
        local key
        key=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')
        ( umask 077; echo "TOUTUI_SECRET_KEY=${key}" > "${config}/.env" )
        echo "[INFO] The secret key is in ${config}/.env."
    fi

    if [ "$(uname -s)" = "Linux" ]; then
        mkdir -p "${HOME}/.local/share/applications"
        if receive "${base}/toutui.desktop" "${tmp}/toutui.desktop"; then
            if sum_agrees "${tmp}/toutui.desktop" "toutui.desktop" "${tmp}/SHA256SUMS"; then
                cp "${tmp}/toutui.desktop" "${HOME}/.local/share/applications/toutui.desktop"
            else
                echo "[WARN] The sum of toutui.desktop is not correct. The launcher entry is not installed." >&2
            fi
        else
            echo "[WARN] The launcher entry did not arrive. Toutui still runs from the command line." >&2
        fi
    fi

    warn_if_no_alsa

    echo "[DONE] Type toutui to start the program."
}

main "$@"
