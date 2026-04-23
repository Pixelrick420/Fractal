#!/usr/bin/env bash
# =============================================================================
#  Fractal Language - Installer
#  https://github.com/Pixelrick420/Fractal
# =============================================================================

set -euo pipefail

# ── Colours ───────────────────────────────────────────────────────────────────
RED='\033[1;31m'; GREEN='\033[1;32m'; CYAN='\033[1;36m'
YELLOW='\033[1;33m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}${BOLD}  →${RESET} $*"; }
success() { echo -e "${GREEN}${BOLD}  ✓${RESET} $*"; }
warn()    { echo -e "${YELLOW}${BOLD}  !${RESET} $*"; }
die()     { echo -e "\n${RED}${BOLD}  ✗ Error:${RESET} $*\n" >&2; exit 1; }

# ── Banner ────────────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}${BOLD}╔══════════════════════════════════════════╗${RESET}"
echo -e "${CYAN}${BOLD}║       Fractal Language Installer         ║${RESET}"
echo -e "${CYAN}${BOLD}║  github.com/Pixelrick420/Fractal         ║${RESET}"
echo -e "${CYAN}${BOLD}╚══════════════════════════════════════════╝${RESET}"
echo ""

# ── Constants ─────────────────────────────────────────────────────────────────
REPO="Pixelrick420/Fractal"
INSTALL_DIR="/usr/bin"
BINARIES=("fractal-compiler" "fractal-editor")

# ── 1. Platform check ─────────────────────────────────────────────────────────
[[ "$(uname -s)" == "Linux" ]] || die "This installer only supports Linux."

ARCH=$(uname -m)
[[ "$ARCH" == "x86_64" ]] || die "Unsupported architecture: ${ARCH}. Only x86_64 is supported."

# ── 2. Package manager helper ─────────────────────────────────────────────────
# Installs a package using whatever package manager is available.
# Must be called before SUDO is set (used to bootstrap curl/sudo themselves).
_install_pkg() {
    local pkg="$1"
    if command -v apt-get &>/dev/null; then
        info "Auto-installing '${pkg}' via apt-get..."
        apt-get update -qq 2>/dev/null | tail -1 || true
        DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "$pkg" \
            || die "Failed to install '${pkg}' with apt-get."
    elif command -v dnf &>/dev/null; then
        info "Auto-installing '${pkg}' via dnf..."
        dnf install -y -q "$pkg" \
            || die "Failed to install '${pkg}' with dnf."
    elif command -v yum &>/dev/null; then
        info "Auto-installing '${pkg}' via yum..."
        yum install -y -q "$pkg" \
            || die "Failed to install '${pkg}' with yum."
    elif command -v zypper &>/dev/null; then
        info "Auto-installing '${pkg}' via zypper..."
        zypper install -y "$pkg" \
            || die "Failed to install '${pkg}' with zypper."
    elif command -v pacman &>/dev/null; then
        info "Auto-installing '${pkg}' via pacman..."
        pacman -Sy --noconfirm "$pkg" \
            || die "Failed to install '${pkg}' with pacman."
    else
        die "No supported package manager found. Please install '${pkg}' manually."
    fi
}

# ── 3. Bootstrap curl (needed for everything that follows) ────────────────────
info "Checking dependencies..."

if ! command -v curl &>/dev/null; then
    # _install_pkg runs without sudo because we either are root, or will
    # handle the non-root case below after curl is available.
    if [[ "$(id -u)" -ne 0 ]]; then
        die "'curl' is not installed and you are not root.\n  Please install curl first:\n    Debian/Ubuntu: apt-get install curl\n    Fedora/RHEL:   dnf install curl"
    fi
    _install_pkg curl
fi

# ── 4. Privilege helper ───────────────────────────────────────────────────────
# Sets SUDO="" when already root, "sudo" otherwise.
# Installs sudo automatically if we are root and it is missing.
if [[ "$(id -u)" -eq 0 ]]; then
    if ! command -v sudo &>/dev/null; then
        _install_pkg sudo
    fi
    SUDO=""   # root doesn't need sudo prefix
else
    if ! command -v sudo &>/dev/null; then
        die "'sudo' is not installed and you are not root.\n  Please run this installer as root, or install sudo first."
    fi
    SUDO="sudo"
fi

# ── 5. Optional helpers ───────────────────────────────────────────────────────
if ! command -v xxd &>/dev/null; then
    warn "'xxd' not found - binary verification will be skipped."
    HAS_XXD=false
else
    HAS_XXD=true
fi

if command -v jq &>/dev/null; then
    USE_JQ=true
else
    USE_JQ=false
fi

success "Dependencies OK."

# ── 6. Note on binaries ──────────────────────────────────────────────────────
# Release binaries are built with musl (fully static for the compiler, minimal
# dynamic deps for the editor - only libGL/libEGL + X11/Wayland, which every
# desktop Linux provides). No GLIBC version check is needed.
GLIBC_OK=true   # always true - musl binaries have no GLIBC requirement

# ── 7. Rust installer (only called when a source build is needed) ─────────────
RUST_JUST_INSTALLED=false

ensure_rust() {
    if command -v rustc &>/dev/null; then
        RUST_VER=$(rustc --version 2>/dev/null | awk '{print $2}')
        success "Rust ${RUST_VER} already installed."
        return
    fi

    info "Rust not found - installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --no-modify-path --profile minimal \
        || die "rustup installation failed.\n  Check your internet connection and try again."

    # shellcheck source=/dev/null
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env" \
        || export PATH="$HOME/.cargo/bin:$PATH"

    rustc --version &>/dev/null \
        || die "rustc not found after install - PATH may not include ~/.cargo/bin."

    RUST_VER=$(rustc --version | awk '{print $2}')
    success "Rust ${RUST_VER} installed."
    RUST_JUST_INSTALLED=true

    local SHELL_NAME RC_FILES
    SHELL_NAME=$(basename "${SHELL:-bash}")
    case "$SHELL_NAME" in
        zsh)  RC_FILES=("$HOME/.zshrc") ;;
        fish) RC_FILES=("$HOME/.config/fish/config.fish") ;;
        ksh)  RC_FILES=("$HOME/.kshrc") ;;
        *)    RC_FILES=("$HOME/.bashrc") ;;
    esac
    RC_FILES+=("$HOME/.profile")

    for rc in "${RC_FILES[@]}"; do
        if [[ -f "$rc" ]] && ! grep -q '\.cargo/env\|\.cargo/bin' "$rc" 2>/dev/null; then
            {
                echo ''
                echo '# Added by Fractal installer'
                echo '. "$HOME/.cargo/env"'
            } >> "$rc"
            info "Updated ${rc} to include Cargo in PATH."
        fi
    done
}

# ── 8. Fetch latest release metadata ─────────────────────────────────────────
info "Fetching latest release from GitHub..."

API_URL="https://api.github.com/repos/${REPO}/releases/latest"
RELEASE_JSON=$(curl -fsSL --max-time 15 "$API_URL" 2>/dev/null) \
    || die "Could not reach GitHub API.\n  Check your internet connection: curl ${API_URL}"

if $USE_JQ; then
    TAG=$(echo "$RELEASE_JSON"          | jq -r '.tag_name // empty')
    RELEASE_NAME=$(echo "$RELEASE_JSON" | jq -r '.name     // empty')
    ASSET_NAMES=$(echo "$RELEASE_JSON"  | jq -r '.assets[].name' 2>/dev/null || true)
else
    TAG=$(echo "$RELEASE_JSON" \
            | grep '"tag_name"' | head -1 \
            | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    RELEASE_NAME=$(echo "$RELEASE_JSON" \
            | grep '"name"' | head -1 \
            | sed 's/.*"name": *"\([^"]*\)".*/\1/')
    ASSET_NAMES=$(echo "$RELEASE_JSON" \
            | grep '"name"' \
            | sed 's/.*"name": *"\([^"]*\)".*/\1/')
fi

[[ -n "$TAG" && "$TAG" != "null" ]] \
    || die "Could not read the latest release tag.\n  The repository may not have any published releases yet."

success "Latest release: ${BOLD}${RELEASE_NAME:-$TAG}${RESET} (${TAG})"

# ── 9. Decide download strategy ───────────────────────────────────────────────
# Release binaries are musl builds - download them directly.
# Fall back to source build only if the asset is missing from the release
# (e.g. a dev build that pre-dates the musl pipeline).
BUILD_STRATEGY="download"

# Check all expected binaries exist as release assets
for bin in "${BINARIES[@]}"; do
    if ! echo "$ASSET_NAMES" | grep -qF "${bin}"; then
        warn "'${bin}' not found in release assets - will build from source."
        BUILD_STRATEGY="source"
        break
    fi
done

info "Install strategy: ${BOLD}${BUILD_STRATEGY}${RESET}"

# ── 10. Download or build ─────────────────────────────────────────────────────
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"

# Returns 1 on HTTP 404 (asset missing), calls die() on other errors.
download_binary() {
    local bin="$1" remote_name="$2"
    local url="${BASE_URL}/${remote_name}"
    info "Downloading ${remote_name}..."
    local http_code
    http_code=$(curl -fsSL --max-time 120 --write-out '%{http_code}' \
                    -o "${TMP_DIR}/${bin}" "$url" 2>/dev/null) || true
    if [[ "$http_code" == "404" ]]; then
        return 1
    elif [[ "$http_code" != "200" ]]; then
        die "Download failed for '${remote_name}' (HTTP ${http_code}).\n  URL: ${url}"
    fi
    chmod +x "${TMP_DIR}/${bin}"
    success "${bin} downloaded."
}

do_source_build() {
    warn "Building from source - this may take a few minutes..."
    ensure_rust

    # Add the fully-static musl target so the result has no GLIBC dependency
    info "Adding musl target to Rust toolchain..."
    rustup target add x86_64-unknown-linux-musl 2>/dev/null || true

    # musl-tools provides the musl-gcc linker wrapper on Debian/Ubuntu
    if command -v apt-get &>/dev/null; then
        DEBIAN_FRONTEND=noninteractive apt-get install -y -qq musl-tools 2>/dev/null || true
    fi

    local BUILD_TMP
    BUILD_TMP=$(mktemp -d)

    info "Cloning Fractal repository..."
    command -v git &>/dev/null || _install_pkg git
    git clone --depth 1 "https://github.com/${REPO}.git" "${BUILD_TMP}/fractal" \
        || die "git clone failed. Check your internet connection."

    cd "${BUILD_TMP}/fractal"

    local CARGO_TARGET_FLAG OUT_TARGET_DIR
    if rustup target list --installed 2>/dev/null | grep -q 'x86_64-unknown-linux-musl'; then
        CARGO_TARGET_FLAG="--target x86_64-unknown-linux-musl"
        OUT_TARGET_DIR="target/x86_64-unknown-linux-musl/release"
    else
        CARGO_TARGET_FLAG=""
        OUT_TARGET_DIR="target/release"
    fi

    info "Compiling Fractal..."
    # shellcheck disable=SC2086
    cargo build --release $CARGO_TARGET_FLAG \
        || die "cargo build failed."

    for bin in "${BINARIES[@]}"; do
        if [[ -f "${OUT_TARGET_DIR}/${bin}" ]]; then
            cp "${OUT_TARGET_DIR}/${bin}" "${TMP_DIR}/${bin}"
            chmod +x "${TMP_DIR}/${bin}"
            success "Built ${bin}."
        else
            die "Expected binary '${bin}' not found in ${OUT_TARGET_DIR}/ after build."
        fi
    done

    cd /
    rm -rf "$BUILD_TMP"
}

case "$BUILD_STRATEGY" in
    download)
        FAILED=false
        for bin in "${BINARIES[@]}"; do
            if ! download_binary "$bin" "$bin"; then
                warn "'${bin}' download failed - falling back to source build."
                FAILED=true
                break
            fi
        done
        if $FAILED; then
            do_source_build
        fi
        ;;
    source)
        do_source_build
        ;;
esac

# ── 10b. Ensure Rust + C linker are installed ───────────────────────────────
# fractal-compiler is a transpiler: it converts .fr → .rs then calls rustc
# to produce the final binary. rustc AND a C linker (cc/gcc) must be on PATH.
ensure_rust

if ! command -v cc &>/dev/null && ! command -v gcc &>/dev/null; then
    info "C linker not found - installing gcc..."
    _install_pkg gcc
fi

# ── 11. Verify ELF magic bytes ────────────────────────────────────────────────
if $HAS_XXD; then
    info "Verifying binaries..."
    for bin in "${BINARIES[@]}"; do
        MAGIC=$(xxd -l4 "${TMP_DIR}/${bin}" 2>/dev/null | awk 'NR==1{print $2$3}')
        if [[ "$MAGIC" != "7f454c46" ]]; then
            FIRST_LINE=$(head -1 "${TMP_DIR}/${bin}" 2>/dev/null || true)
            die "'${bin}' is not a valid Linux ELF binary.\n  First bytes: ${FIRST_LINE}\n  The asset may be corrupted or mis-named in the release."
        fi
    done
    success "Binaries verified."
fi

# ── 12. Install ───────────────────────────────────────────────────────────────
info "Installing to ${INSTALL_DIR}..."

for bin in "${BINARIES[@]}"; do
    $SUDO install -m 0755 "${TMP_DIR}/${bin}" "${INSTALL_DIR}/${bin}" \
        || die "Failed to install '${bin}' to ${INSTALL_DIR}."
    success "Installed ${bin} → ${INSTALL_DIR}/${bin}"
done

# ── 13. Smoke test ────────────────────────────────────────────────────────────
ALL_OK=true
for bin in "${BINARIES[@]}"; do
    if ! command -v "$bin" &>/dev/null; then
        warn "'${bin}' not found on PATH - ${INSTALL_DIR} may not be in your PATH."
        ALL_OK=false
    fi
done

# ── 14. Source shell rc so Rust is immediately available ──────────────────────
if $RUST_JUST_INSTALLED; then
    SHELL_NAME=$(basename "${SHELL:-bash}")
    case "$SHELL_NAME" in
        zsh)  RC="$HOME/.zshrc" ;;
        fish) RC="" ;;
        ksh)  RC="$HOME/.kshrc" ;;
        *)    RC="$HOME/.bashrc" ;;
    esac
    if [[ -n "$RC" && -f "$RC" ]]; then
        # shellcheck source=/dev/null
        source "$RC" 2>/dev/null || true
        info "Sourced ${RC} - Rust is now active in this session."
    fi
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}╔══════════════════════════════════════════╗${RESET}"
echo -e "${GREEN}${BOLD}║     Fractal installed successfully!      ║${RESET}"
echo -e "${GREEN}${BOLD}╚══════════════════════════════════════════╝${RESET}"
echo ""
echo -e "  ${BOLD}fractal-compiler${RESET} file.fr          - compile"
echo -e "  ${BOLD}fractal-compiler${RESET} debug file.fr    - compile with debug info"
echo -e "  ${BOLD}fractal-editor${RESET}                    - launch the editor"
echo ""

if ! $ALL_OK; then
    warn "Open a new terminal if the commands are not found."
fi
