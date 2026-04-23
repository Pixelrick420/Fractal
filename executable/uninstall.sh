#!/usr/bin/env bash
# =============================================================================
#  Fractal Language - Uninstaller
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
echo -e "${RED}${BOLD}╔══════════════════════════════════════════╗${RESET}"
echo -e "${RED}${BOLD}║      Fractal Language Uninstaller        ║${RESET}"
echo -e "${RED}${BOLD}╚══════════════════════════════════════════╝${RESET}"
echo ""

# ── Privilege helper ──────────────────────────────────────────────────────────
if [[ "$(id -u)" -eq 0 ]]; then
    SUDO=""
else
    command -v sudo &>/dev/null \
        || die "'sudo' is not installed and you are not root.\n  Please run this uninstaller as root, or install sudo first."
    SUDO="sudo"
fi

INSTALL_DIR="/usr/bin"
BINARIES=("fractal-compiler" "fractal-editor")

# ── 1. Find what is installed ─────────────────────────────────────────────────
FOUND=()
for bin in "${BINARIES[@]}"; do
    [[ -f "${INSTALL_DIR}/${bin}" ]] && FOUND+=("$bin")
done

if [[ ${#FOUND[@]} -eq 0 ]]; then
    warn "No Fractal binaries found in ${INSTALL_DIR}. Nothing to remove."
    exit 0
fi

info "The following will be removed from ${INSTALL_DIR}:"
for bin in "${FOUND[@]}"; do
    echo -e "    ${BOLD}${bin}${RESET}"
done
echo ""

# ── 2. Confirm ────────────────────────────────────────────────────────────────
read -rp "$(echo -e "${YELLOW}${BOLD}  Proceed? [y/N]:${RESET} ")" CONFIRM
case "$CONFIRM" in
    [yY][eE][sS]|[yY]) ;;
    *) echo ""; warn "Aborted."; exit 0 ;;
esac
echo ""

# ── 3. Remove binaries ────────────────────────────────────────────────────────
info "Removing binaries..."

for bin in "${FOUND[@]}"; do
    $SUDO rm -f "${INSTALL_DIR}/${bin}" \
        || die "Failed to remove '${bin}' - make sure you have sufficient privileges."
    success "Removed ${INSTALL_DIR}/${bin}"
done

# ── 4. Verify removed ─────────────────────────────────────────────────────────
ALL_GONE=true
for bin in "${FOUND[@]}"; do
    if command -v "$bin" &>/dev/null; then
        LOCATION=$(command -v "$bin")
        warn "'${bin}' still found at ${LOCATION} - there may be another copy outside ${INSTALL_DIR}."
        ALL_GONE=false
    fi
done

# ── 5. Clean up shell rc entries ──────────────────────────────────────────────
RC_FILES=(
    "$HOME/.bashrc"
    "$HOME/.zshrc"
    "$HOME/.profile"
    "$HOME/.kshrc"
    "$HOME/.config/fish/config.fish"
)
for rc in "${RC_FILES[@]}"; do
    if [[ -f "$rc" ]] && grep -q 'Added by Fractal installer' "$rc" 2>/dev/null; then
        sed -i '/# Added by Fractal installer/{N;N;d}' "$rc" 2>/dev/null || true
        info "Cleaned up Fractal installer entries from ${rc}."
    fi
done

# ── 6. Optional: remove Rust ──────────────────────────────────────────────────
echo ""
if command -v rustup &>/dev/null; then
    echo -e "${YELLOW}${BOLD}  Rust (rustup) is installed on this machine.${RESET}"
    warn "Removing Rust will affect ALL Rust projects, not just Fractal."
    read -rp "$(echo -e "${YELLOW}${BOLD}  Remove Rust as well? [y/N]:${RESET} ")" RM_RUST
    case "$RM_RUST" in
        [yY][eE][sS]|[yY])
            info "Running rustup self uninstall..."
            rustup self uninstall -y \
                || die "rustup uninstall failed - try running 'rustup self uninstall' manually."
            success "Rust removed."
            ;;
        *)
            info "Rust left in place."
            ;;
    esac
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
if $ALL_GONE; then
    echo -e "${GREEN}${BOLD}  Fractal has been uninstalled.${RESET}"
else
    echo -e "${YELLOW}${BOLD}  Fractal partially uninstalled - see warnings above.${RESET}"
fi
echo ""
