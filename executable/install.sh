#!/usr/bin/env bash
# =============================================================================
#  Fractal Language — Linux Installer
#  Repo  : https://github.com/Pixelrick420/Fractal
#  Installs: fractal-compiler  fractal-editor  →  /usr/bin
#  Rust  : installed via rustup if not already present
# =============================================================================

set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
RED='\033[1;31m'; GREEN='\033[1;32m'; CYAN='\033[1;36m'
YELLOW='\033[1;33m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}${BOLD}  →${RESET} $*"; }
success() { echo -e "${GREEN}${BOLD}  ✓${RESET} $*"; }
warn()    { echo -e "${YELLOW}${BOLD}  !${RESET} $*"; }
die()     { echo -e "${RED}${BOLD}  ✗ Error:${RESET} $*" >&2; exit 1; }

# ── Banner ────────────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}${BOLD}╔══════════════════════════════════════════╗${RESET}"
echo -e "${CYAN}${BOLD}║       Fractal Language Installer         ║${RESET}"
echo -e "${CYAN}${BOLD}║  github.com/Pixelrick420/Fractal          ║${RESET}"
echo -e "${CYAN}${BOLD}╚══════════════════════════════════════════╝${RESET}"
echo ""

# ── Constants ────────────────────────────────────────────────────────────────
REPO="Pixelrick420/Fractal"
INSTALL_DIR="/usr/bin"
BINARIES=("fractal-compiler" "fractal-editor")

# ── 1. Check for required tools ───────────────────────────────────────────────
info "Checking dependencies..."

for cmd in curl sudo; do
    command -v "$cmd" &>/dev/null || die "'$cmd' is required but not installed. Please install it and re-run."
done

# jq is optional — we can parse JSON with grep/sed if absent
if command -v jq &>/dev/null; then
    USE_JQ=true
else
    USE_JQ=false
    warn "'jq' not found — falling back to grep/sed for JSON parsing."
fi

# ── 2. Install Rust via rustup (if rustc not present) ─────────────────────────
if command -v rustc &>/dev/null; then
    RUST_VER=$(rustc --version 2>/dev/null | awk '{print $2}')
    success "Rust already installed (${RUST_VER}) — skipping rustup."
else
    info "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path \
        || die "rustup installation failed."

    # Make cargo/rustc available in this shell session
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"

    rustc --version &>/dev/null || die "rustc still not found after rustup install — please restart your shell and re-run."

    success "Rust installed successfully."

    # Persist PATH for future sessions if not already configured
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        if [[ -f "$rc" ]] && ! grep -q '\.cargo/env' "$rc" 2>/dev/null; then
            echo '' >> "$rc"
            echo '# Added by Fractal installer' >> "$rc"
            echo '. "$HOME/.cargo/env"' >> "$rc"
            info "Added Cargo env sourcing to $rc"
        fi
    done
fi

# ── 3. Fetch latest release metadata from GitHub ──────────────────────────────
info "Fetching latest release from GitHub..."

API_URL="https://api.github.com/repos/${REPO}/releases/latest"
RELEASE_JSON=$(curl -fsSL "$API_URL") \
    || die "Failed to reach GitHub API. Check your internet connection."

if $USE_JQ; then
    TAG=$(echo "$RELEASE_JSON" | jq -r '.tag_name')
    RELEASE_NAME=$(echo "$RELEASE_JSON" | jq -r '.name')
else
    TAG=$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    RELEASE_NAME=$(echo "$RELEASE_JSON" | grep '"name"' | head -1 | sed 's/.*"name": *"\([^"]*\)".*/\1/')
fi

[[ -n "$TAG" && "$TAG" != "null" ]] || die "Could not determine latest release tag. The repo may have no releases yet."

success "Latest release: ${BOLD}${RELEASE_NAME}${RESET} (${TAG})"

# ── 4. Download binaries ──────────────────────────────────────────────────────
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

info "Downloading binaries to temporary directory..."

BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"

for bin in "${BINARIES[@]}"; do
    DOWNLOAD_URL="${BASE_URL}/${bin}"
    info "  Downloading ${bin}..."
    curl -fsSL --progress-bar -o "${TMP_DIR}/${bin}" "$DOWNLOAD_URL" \
        || die "Failed to download '${bin}' from:\n  ${DOWNLOAD_URL}\n\n  Make sure the release assets exist and match the expected filenames."
    chmod +x "${TMP_DIR}/${bin}"
    success "  ${bin} downloaded."
done

# ── 5. Verify binaries are executable ELF files ───────────────────────────────
info "Verifying downloaded binaries..."

for bin in "${BINARIES[@]}"; do
    BIN_PATH="${TMP_DIR}/${bin}"
    MAGIC=$(xxd -l4 "$BIN_PATH" 2>/dev/null | head -1 | awk '{print $2$3}')
    # ELF magic bytes: 7f454c46
    if [[ "$MAGIC" != "7f454c46" ]]; then
        # Could be a text/HTML error page — print first line for debugging
        FIRST_LINE=$(head -1 "$BIN_PATH" 2>/dev/null || true)
        die "'${bin}' does not appear to be a Linux binary.\n  First line: ${FIRST_LINE}\n  Check that the GitHub release actually contains Linux x86-64 binaries."
    fi
done
success "Binaries verified."

# ── 6. Install to /usr/bin (requires sudo) ────────────────────────────────────
info "Installing to ${INSTALL_DIR} (sudo required)..."

for bin in "${BINARIES[@]}"; do
    sudo install -m 0755 "${TMP_DIR}/${bin}" "${INSTALL_DIR}/${bin}" \
        || die "Failed to install '${bin}' to ${INSTALL_DIR}."
    success "  Installed ${bin} → ${INSTALL_DIR}/${bin}"
done

# ── 7. Smoke-test: confirm commands are on PATH ───────────────────────────────
echo ""
info "Verifying installation..."

ALL_OK=true
for bin in "${BINARIES[@]}"; do
    if command -v "$bin" &>/dev/null; then
        success "  '${bin}' is available on PATH."
    else
        warn "  '${bin}' was installed to ${INSTALL_DIR} but is not found on PATH."
        warn "  Make sure ${INSTALL_DIR} is in your PATH (\$PATH)."
        ALL_OK=false
    fi
done

# ── 8. Done ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}╔══════════════════════════════════════════╗${RESET}"
echo -e "${GREEN}${BOLD}║     Fractal installed successfully!      ║${RESET}"
echo -e "${GREEN}${BOLD}╚══════════════════════════════════════════╝${RESET}"
echo ""
echo -e "  ${BOLD}fractal-compiler${RESET} your_program.fr   — compile a .fr file"
echo -e "  ${BOLD}fractal-compiler${RESET} debug your_program.fr  — compile with debug info"
echo -e "  ${BOLD}fractal-editor${RESET}                     — launch the GUI editor"
echo ""

if ! $ALL_OK; then
    warn "One or more binaries were not found on PATH."
    warn "If ${INSTALL_DIR} is already in your PATH, try opening a new terminal."
fi
