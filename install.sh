#!/usr/bin/env bash
#
# Burrow — one-liner install script
#
#   curl -fsSL https://raw.githubusercontent.com/Simplinity/burrow/master/install.sh | bash
#
# Installs burrowd + burrow CLI to /usr/local/bin (or ~/.local/bin if no root).
# Requires: curl, tar, and a 64-bit OS (Linux or macOS).

set -euo pipefail

REPO="Simplinity/burrow"
VERSION="${BURROW_VERSION:-latest}"
INSTALL_DIR="${BURROW_INSTALL_DIR:-}"

# ── Helpers ──────────────────────────────────────────────────────

info()  { printf '\033[36m→\033[0m %s\n' "$*"; }
ok()    { printf '\033[32m✓\033[0m %s\n' "$*"; }
err()   { printf '\033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  os="linux" ;;
        Darwin) os="darwin" ;;
        *)      err "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)             err "Unsupported architecture: $arch" ;;
    esac

    echo "${os}-${arch}"
}

resolve_version() {
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')
        [ -n "$VERSION" ] || err "Could not determine latest version"
    fi
}

pick_install_dir() {
    if [ -n "$INSTALL_DIR" ]; then
        return
    fi
    if [ -w /usr/local/bin ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ -d "$HOME/.local/bin" ] || mkdir -p "$HOME/.local/bin" 2>/dev/null; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        err "Cannot write to /usr/local/bin or ~/.local/bin. Set BURROW_INSTALL_DIR."
    fi
}

check_path() {
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            printf '\n\033[33m!\033[0m %s is not in your PATH.\n' "$INSTALL_DIR"
            echo "  Add it:  export PATH=\"$INSTALL_DIR:\$PATH\""
            ;;
    esac
}

# ── Main ─────────────────────────────────────────────────────────

main() {
    echo ""
    echo "  ┌──────────────────────────────────┐"
    echo "  │  burrow installer                 │"
    echo "  │  The internet, minus the parts    │"
    echo "  │  that made you hate the internet. │"
    echo "  └──────────────────────────────────┘"
    echo ""

    local platform
    platform="$(detect_platform)"
    info "Detected platform: $platform"

    resolve_version
    info "Installing version: $VERSION"

    pick_install_dir
    info "Install directory: $INSTALL_DIR"

    local tarball="burrow-${VERSION}-${platform}.tar.gz"
    local url="https://github.com/${REPO}/releases/download/${VERSION}/${tarball}"

    info "Downloading $tarball..."
    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    curl -fsSL "$url" -o "$tmpdir/$tarball" \
        || err "Download failed. Check that ${VERSION} has a release for ${platform}."

    info "Extracting..."
    tar -xzf "$tmpdir/$tarball" -C "$tmpdir"

    # Install binaries
    for bin in burrowd burrow; do
        if [ -f "$tmpdir/$bin" ]; then
            install -m 755 "$tmpdir/$bin" "$INSTALL_DIR/$bin"
            ok "Installed $bin → $INSTALL_DIR/$bin"
        fi
    done

    check_path

    echo ""
    ok "Done! Get started:"
    echo ""
    echo "    burrow init myblog"
    echo "    burrow new \"My first post\""
    echo "    burrowd"
    echo ""
}

main "$@"
