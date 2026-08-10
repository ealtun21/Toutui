#!/usr/bin/env bash
# Install toutui.
#
# Use:
#   curl -LsSf https://raw.githubusercontent.com/ealtun21/Toutui/main/install.sh | bash
#
# The script receives the archive of the last release, it compares the sum
# with SHA256SUMS of that release, and it installs the binary. The script
# installs no other program, because toutui plays the audio itself.

set -euo pipefail

REPO="ealtun21/Toutui"
API="https://api.github.com/repos/${REPO}/releases/latest"
BIN_DIR="${TOUTUI_BIN_DIR:-/usr/local/bin}"

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

sum_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

main() {
    [ "$(id -u)" -ne 0 ] || fail "Do not run this script as root."

    command -v curl >/dev/null 2>&1 || fail "Install curl first."
    command -v tar  >/dev/null 2>&1 || fail "Install tar first."

    local target archive tag tmp
    target=$(identify_target)
    archive="toutui-${target}.tar.gz"

    tag=$(curl -sSfL "$API" | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
    [ -n "$tag" ] || fail "The repository has no release."

    echo "[INFO] The last release is ${tag}."

    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    local base="https://github.com/${REPO}/releases/download/${tag}"
    curl -sSfL "${base}/${archive}" -o "${tmp}/${archive}"
    curl -sSfL "${base}/SHA256SUMS" -o "${tmp}/SHA256SUMS"

    local expected actual
    expected=$(grep " ${archive}\$" "${tmp}/SHA256SUMS" | awk '{print $1}')
    [ -n "$expected" ] || fail "SHA256SUMS has no sum for ${archive}."
    actual=$(sum_of "${tmp}/${archive}")
    [ "$expected" = "$actual" ] || fail "The sum of ${archive} is not correct."

    echo "[INFO] The sum is correct."

    tar -xzf "${tmp}/${archive}" -C "$tmp"
    sudo install -m 755 "${tmp}/toutui" "${BIN_DIR}/toutui"
    echo "[INFO] The binary is in ${BIN_DIR}/toutui."

    local config
    config=$(config_dir)
    mkdir -p "$config"
    if [ ! -f "${config}/config.toml" ]; then
        curl -sSfL "${base}/config.example.toml" -o "${config}/config.toml"
        echo "[INFO] The configuration is in ${config}/config.toml."
    fi

    if [ ! -f "${config}/.env" ]; then
        local key
        key=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')
        ( umask 077; echo "TOUTUI_SECRET_KEY=${key}" > "${config}/.env" )
        echo "[INFO] The secret key is in ${config}/.env."
    fi

    if [ "$(uname -s)" = "Linux" ]; then
        mkdir -p "${HOME}/.local/share/applications"
        curl -sSfL "${base}/toutui.desktop" \
            -o "${HOME}/.local/share/applications/toutui.desktop"
    fi

    echo "[DONE] Type toutui to start the program."
}

main "$@"
