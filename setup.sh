#!/usr/bin/env bash
# =============================================================================
#  Fractal Language — Setup Script
#  Supports: Ubuntu 24.10+
#  Installs: fractal-compiler, fractal-editor, minimal rustc
# =============================================================================

set -euo pipefail

# ── Colour helpers ────────────────────────────────────────────────────────────
RED='\033[1;31m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
CYAN='\033[1;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo -e "${CYAN}${BOLD}  →${RESET}  $*"; }
success() { echo -e "${GREEN}${BOLD}  ✓${RESET}  $*"; }
warn()    { echo -e "${YELLOW}${BOLD}  !${RESET}  $*"; }
die()     { echo -e "${RED}${BOLD}  ✗  ERROR:${RESET}  $*" >&2; exit 1; }

# ── Banner ────────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}${BLUE}╔══════════════════════════════════════════╗${RESET}"
echo -e "${BOLD}${BLUE}║         Fractal Language  —  Setup       ║${RESET}"
echo -e "${BOLD}${BLUE}╚══════════════════════════════════════════╝${RESET}"
echo ""

# ── Configuration — edit these to match your release infrastructure ───────────
FRACTAL_VERSION="latest"
GITHUB_REPO="harikrishnanr/fractal"          # adjust to your actual repo
RELEASE_BASE="https://github.com/${GITHUB_REPO}/releases/latest/download"
COMPILER_BIN_URL="${RELEASE_BASE}/fractal-compiler-linux-x86_64"
EDITOR_BIN_URL="${RELEASE_BASE}/fractal-editor-linux-x86_64"
ICON_URL="${RELEASE_BASE}/fractal-icon.png"   # 256×256 PNG app icon

INSTALL_DIR="${HOME}/.local/bin"
DATA_DIR="${HOME}/.local/share/fractal"
ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
DESKTOP_DIR="${HOME}/.local/share/applications"
# ─────────────────────────────────────────────────────────────────────────────

# ── Preflight checks ──────────────────────────────────────────────────────────
info "Checking system requirements…"

if [[ "$(uname -s)" != "Linux" ]]; then
    die "This script is for Linux only."
fi

# Require Ubuntu 24.10+ (VERSION_ID >= 24.10)
if [[ -f /etc/os-release ]]; then
    . /etc/os-release
    if [[ "${ID:-}" != "ubuntu" ]]; then
        warn "This script targets Ubuntu. Detected: ${PRETTY_NAME:-unknown}. Continuing anyway."
    else
        # Compare version numbers: split on '.' and compare major/minor
        IFS='.' read -r _major _minor _rest <<< "${VERSION_ID:-0.0}"
        if (( _major < 24 )) || { (( _major == 24 )) && (( _minor < 10 )); }; then
            die "Ubuntu 24.10 or higher is required. Detected: ${VERSION_ID}."
        fi
    fi
fi

# Check we're not running as root (rustup refuses to run as root)
if [[ "${EUID}" -eq 0 ]]; then
    die "Do not run this script as root. Run it as your normal user account."
fi

# ── Dependencies (curl, wget, xdg-utils) ──────────────────────────────────────
info "Installing system dependencies…"
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
    curl \
    wget \
    ca-certificates \
    xdg-utils \
    libxcb-render0 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    libxkbcommon0 \
    libgtk-3-0 \
    > /dev/null 2>&1
success "System dependencies ready."

# ── Install minimal Rust ───────────────────────────────────────────────────────
info "Checking for rustc…"

if command -v rustc &>/dev/null; then
    RUSTC_VER="$(rustc --version 2>/dev/null | awk '{print $2}')"
    success "rustc already installed (${RUSTC_VER}). Skipping."
else
    info "Installing minimal Rust toolchain via rustup…"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- \
            --profile minimal \
            --default-toolchain stable \
            --no-modify-path \
            -y \
        > /dev/null 2>&1

    # Source the cargo environment for the rest of this script
    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env"
    success "rustc $(rustc --version | awk '{print $2}') installed."
fi

CARGO_BIN="${HOME}/.cargo/bin"

# ── Create install directories ────────────────────────────────────────────────
info "Creating install directories…"
mkdir -p "${INSTALL_DIR}"
mkdir -p "${DATA_DIR}"
mkdir -p "${ICON_DIR}"
mkdir -p "${DESKTOP_DIR}"
success "Directories ready."

# ── Download Fractal binaries ─────────────────────────────────────────────────
info "Downloading fractal-compiler…"
if curl -fsSL "${COMPILER_BIN_URL}" -o "${INSTALL_DIR}/fractal-compiler"; then
    chmod +x "${INSTALL_DIR}/fractal-compiler"
    success "fractal-compiler installed → ${INSTALL_DIR}/fractal-compiler"
else
    die "Failed to download fractal-compiler from:\n    ${COMPILER_BIN_URL}"
fi

info "Downloading fractal-editor…"
if curl -fsSL "${EDITOR_BIN_URL}" -o "${INSTALL_DIR}/fractal-editor"; then
    chmod +x "${INSTALL_DIR}/fractal-editor"
    success "fractal-editor installed → ${INSTALL_DIR}/fractal-editor"
else
    die "Failed to download fractal-editor from:\n    ${EDITOR_BIN_URL}"
fi

# ── Download app icon ─────────────────────────────────────────────────────────
info "Downloading app icon…"
ICON_PATH="${ICON_DIR}/fractal-editor.png"
if curl -fsSL "${ICON_URL}" -o "${ICON_PATH}" 2>/dev/null; then
    success "Icon installed → ${ICON_PATH}"
else
    # Fall back to a simple generated icon so the .desktop file still works
    warn "Could not download icon. Using fallback."
    ICON_PATH="fractal-editor"   # let the desktop environment find a generic one
fi

# ── PATH configuration ────────────────────────────────────────────────────────
info "Configuring PATH…"

# Add ~/.local/bin and ~/.cargo/bin to PATH in shell rc files if not already present
configure_path_in_file() {
    local RC_FILE="$1"
    local MARKER="# >>> fractal path >>>"

    if [[ ! -f "${RC_FILE}" ]]; then
        return
    fi

    if grep -qF "${MARKER}" "${RC_FILE}" 2>/dev/null; then
        return   # already configured
    fi

    cat >> "${RC_FILE}" << EOF

${MARKER}
export PATH="\${HOME}/.local/bin:\${HOME}/.cargo/bin:\${PATH}"
# <<< fractal path <<<
EOF
}

configure_path_in_file "${HOME}/.bashrc"
configure_path_in_file "${HOME}/.bash_profile"
configure_path_in_file "${HOME}/.profile"

# Also do zshrc if zsh is present
if [[ -f "${HOME}/.zshrc" ]]; then
    configure_path_in_file "${HOME}/.zshrc"
fi

# Make PATH live for the rest of this script
export PATH="${INSTALL_DIR}:${CARGO_BIN}:${PATH}"

success "PATH configured."

# ── .desktop file ─────────────────────────────────────────────────────────────
info "Creating .desktop entry…"

DESKTOP_FILE="${DESKTOP_DIR}/fractal-editor.desktop"

cat > "${DESKTOP_FILE}" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Fractal Editor
GenericName=Code Editor
Comment=IDE for the Fractal programming language
Exec=${INSTALL_DIR}/fractal-editor %f
Icon=${ICON_PATH}
Terminal=false
StartupNotify=true
StartupWMClass=fractal-editor
Categories=Development;IDE;TextEditor;
MimeType=text/x-fractal;
Keywords=fractal;code;editor;compiler;programming;
EOF

chmod 644 "${DESKTOP_FILE}"

# Register the .fr file extension with the MIME system
MIME_FILE="${HOME}/.local/share/mime/packages/fractal.xml"
mkdir -p "$(dirname "${MIME_FILE}")"
cat > "${MIME_FILE}" << EOF
<?xml version="1.0" encoding="utf-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-fractal">
    <comment>Fractal source file</comment>
    <glob pattern="*.fr"/>
  </mime-type>
</mime-info>
EOF

# Update MIME and desktop databases (suppress output; these may warn if running
# outside a full desktop session, e.g. in a minimal container — that's fine)
update-mime-database "${HOME}/.local/share/mime" 2>/dev/null || true
update-desktop-database "${DESKTOP_DIR}" 2>/dev/null || true
xdg-mime default fractal-editor.desktop text/x-fractal 2>/dev/null || true

success ".desktop entry created."

# ── Verify installation ───────────────────────────────────────────────────────
echo ""
info "Verifying installation…"

ALL_OK=true

if command -v fractal-compiler &>/dev/null; then
    success "fractal-compiler  — $(fractal-compiler --version 2>/dev/null || echo 'ok')"
else
    warn "fractal-compiler not found on PATH (you may need to restart your shell)."
    ALL_OK=false
fi

if command -v fractal-editor &>/dev/null; then
    success "fractal-editor    — found"
else
    warn "fractal-editor not found on PATH (you may need to restart your shell)."
    ALL_OK=false
fi

if command -v rustc &>/dev/null; then
    success "rustc             — $(rustc --version)"
else
    warn "rustc not found on PATH."
    ALL_OK=false
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}╔══════════════════════════════════════════╗${RESET}"
echo -e "${BOLD}${GREEN}║         Installation Complete  ✓         ║${RESET}"
echo -e "${BOLD}${GREEN}╚══════════════════════════════════════════╝${RESET}"
echo ""

if [[ "${ALL_OK}" == "true" ]]; then
    echo -e "  Run ${BOLD}fractal-editor${RESET} to launch the IDE."
    echo -e "  Run ${BOLD}fractal-compiler <file.fr>${RESET} to compile from the terminal."
else
    echo -e "  ${YELLOW}Restart your terminal (or run ${BOLD}source ~/.bashrc${RESET}${YELLOW}),"
    echo -e "  then run ${BOLD}fractal-editor${RESET}${YELLOW} to launch the IDE.${RESET}"
fi
echo ""
