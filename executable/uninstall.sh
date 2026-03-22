#!/usr/bin/env bash
# =============================================================================
#  Fractal Language — Linux Uninstaller
#  Removes: fractal-compiler  fractal-editor  from  /usr/bin
# =============================================================================

set -euo pipefail

RED='\033[1;31m'; GREEN='\033[1;32m'; CYAN='\033[1;36m'
YELLOW='\033[1;33m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}${BOLD}  →${RESET} $*"; }
success() { echo -e "${GREEN}${BOLD}  ✓${RESET} $*"; }
warn()    { echo -e "${YELLOW}${BOLD}  !${RESET} $*"; }
die()     { echo -e "${RED}${BOLD}  ✗ Error:${RESET} $*" >&2; exit 1; }

echo ""
echo -e "${RED}${BOLD}╔══════════════════════════════════════════╗${RESET}"
echo -e "${RED}${BOLD}║      Fractal Language Uninstaller        ║${RESET}"
echo -e "${RED}${BOLD}╚══════════════════════════════════════════╝${RESET}"
echo ""

INSTALL_DIR="/usr/bin"
BINARIES=("fractal-compiler" "fractal-editor")

# ── 1. Check anything is actually installed ───────────────────────────────────
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
read -rp "$(echo -e "${YELLOW}${BOLD}  Proceed with uninstall? [y/N]:${RESET} ")" CONFIRM
case "$CONFIRM" in
    [yY][eE][sS]|[yY]) ;;
    *) echo ""; warn "Aborted."; exit 0 ;;
esac
echo ""

# ── 3. Remove binaries ────────────────────────────────────────────────────────
info "Removing binaries (sudo required)..."

for bin in "${FOUND[@]}"; do
    sudo rm -f "${INSTALL_DIR}/${bin}" \
        || die "Failed to remove '${bin}' from ${INSTALL_DIR}."
    success "  Removed ${INSTALL_DIR}/${bin}"
done

# ── 4. Verify gone ────────────────────────────────────────────────────────────
echo ""
info "Verifying removal..."

ALL_GONE=true
for bin in "${FOUND[@]}"; do
    if command -v "$bin" &>/dev/null; then
        warn "  '${bin}' still found at $(command -v "$bin") — may be a duplicate installation."
        ALL_GONE=false
    else
        success "  '${bin}' is no longer on PATH."
    fi
done

# ── 5. Optional: remove rustup ────────────────────────────────────────────────
echo ""
if command -v rustup &>/dev/null; then
    read -rp "$(echo -e "${YELLOW}${BOLD}  Also remove Rust (rustup)?${RESET} This affects all Rust projects on this machine. [y/N]: ")" RM_RUST
    case "$RM_RUST" in
        [yY][eE][sS]|[yY])
            info "Running rustup self uninstall..."
            rustup self uninstall -y || die "rustup uninstall failed."
            success "Rust removed."
            ;;
        *)
            info "Rust left in place."
            ;;
    esac
fi

# ── 6. Done ───────────────────────────────────────────────────────────────────
echo ""
if $ALL_GONE; then
    echo -e "${GREEN}${BOLD}  Fractal has been uninstalled.${RESET}"
else
    echo -e "${YELLOW}${BOLD}  Fractal partially uninstalled — see warnings above.${RESET}"
fi
echo ""
