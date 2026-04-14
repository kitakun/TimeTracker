#!/usr/bin/env bash
set -euo pipefail

# Build TimeTracker for Ubuntu/Linux.
#
# Usage examples:
#   ./scripts/build-ubuntu.sh
#   ./scripts/build-ubuntu.sh --bundle appimage
#   ./scripts/build-ubuntu.sh --bundle deb
#   ./scripts/build-ubuntu.sh --bundle all --skip-frontend
#   ./scripts/build-ubuntu.sh --debug

RELEASE=true
TARGET="x86_64-unknown-linux-gnu"
BUNDLE="appimage"
SKIP_FRONTEND=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug)
      RELEASE=false
      shift
      ;;
    --release)
      RELEASE=true
      shift
      ;;
    --target)
      TARGET="${2:-}"
      shift 2
      ;;
    --bundle)
      BUNDLE="${2:-}"
      shift 2
      ;;
    --skip-frontend)
      SKIP_FRONTEND=true
      shift
      ;;
    *)
      echo "Unknown argument: $1"
      exit 1
      ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MODE="release"
[[ "${RELEASE}" == "false" ]] && MODE="debug"

step() {
  echo
  echo ">> $1"
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "ERROR: '$1' not found in PATH."
    exit 1
  fi
}

step "Checking prerequisites"
require_command cargo
require_command rustup
require_command npm

step "Ensuring Rust target: ${TARGET}"
rustup target add "${TARGET}"

TAURI_CLI=""
if [[ -x "${REPO_ROOT}/node_modules/.bin/tauri" ]]; then
  TAURI_CLI="${REPO_ROOT}/node_modules/.bin/tauri"
elif command -v cargo-tauri >/dev/null 2>&1; then
  TAURI_CLI="cargo-tauri"
else
  step "Installing tauri-cli via cargo (one-time)"
  cargo install tauri-cli --version "^2" --locked
  TAURI_CLI="cargo-tauri"
fi

cd "${REPO_ROOT}"

if [[ "${SKIP_FRONTEND}" == "false" ]]; then
  step "Installing npm dependencies"
  npm install --prefer-offline
fi

step "Running Tauri build"
BUILD_ARGS=(build --target "${TARGET}")
if [[ "${RELEASE}" == "false" ]]; then
  BUILD_ARGS+=(--debug)
fi

case "${BUNDLE,,}" in
  appimage) BUILD_ARGS+=(--bundles appimage) ;;
  deb) BUILD_ARGS+=(--bundles deb) ;;
  rpm) BUILD_ARGS+=(--bundles rpm) ;;
  all) ;;
  *)
    echo "Unknown --bundle value '${BUNDLE}'."
    echo "Valid values: appimage | deb | rpm | all"
    exit 1
    ;;
esac

echo "Command: ${TAURI_CLI} ${BUILD_ARGS[*]}"
"${TAURI_CLI}" "${BUILD_ARGS[@]}"

step "Locating build artifacts"
TARGET_DIR="${REPO_ROOT}/src-tauri/target/${TARGET}/${MODE}/bundle"
ARTIFACTS=()

if [[ "${BUNDLE,,}" == "appimage" || "${BUNDLE,,}" == "all" ]]; then
  while IFS= read -r f; do ARTIFACTS+=("$f"); done < <(rg --files "${TARGET_DIR}" -g "*.AppImage")
fi
if [[ "${BUNDLE,,}" == "deb" || "${BUNDLE,,}" == "all" ]]; then
  while IFS= read -r f; do ARTIFACTS+=("$f"); done < <(rg --files "${TARGET_DIR}" -g "*.deb")
fi
if [[ "${BUNDLE,,}" == "rpm" || "${BUNDLE,,}" == "all" ]]; then
  while IFS= read -r f; do ARTIFACTS+=("$f"); done < <(rg --files "${TARGET_DIR}" -g "*.rpm")
fi

if [[ ${#ARTIFACTS[@]} -eq 0 ]]; then
  echo "No artifacts found under: ${TARGET_DIR}"
  exit 0
fi

step "Copying artifacts to scripts folder"
for artifact in "${ARTIFACTS[@]}"; do
  cp -f "${artifact}" "${SCRIPT_DIR}/"
  echo "Copied: ${SCRIPT_DIR}/$(basename "${artifact}")"
done

echo
echo "OK Linux build complete"
