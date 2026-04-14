#!/usr/bin/env bash
set -euo pipefail

# Build TimeTracker for macOS.
#
# Usage examples:
#   ./scripts/build-macos.sh
#   ./scripts/build-macos.sh --bundle dmg
#   ./scripts/build-macos.sh --bundle app
#   ./scripts/build-macos.sh --bundle all --skip-frontend
#   ./scripts/build-macos.sh --debug

RELEASE=true
TARGET="aarch64-apple-darwin"
BUNDLE="dmg"
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
  app) BUILD_ARGS+=(--bundles app) ;;
  dmg) BUILD_ARGS+=(--bundles dmg) ;;
  all) ;;
  *)
    echo "Unknown --bundle value '${BUNDLE}'."
    echo "Valid values: app | dmg | all"
    exit 1
    ;;
esac

echo "Command: ${TAURI_CLI} ${BUILD_ARGS[*]}"
"${TAURI_CLI}" "${BUILD_ARGS[@]}"

step "Locating build artifacts"
TARGET_DIR="${REPO_ROOT}/src-tauri/target/${TARGET}/${MODE}/bundle"
ARTIFACTS=()

if [[ "${BUNDLE,,}" == "app" || "${BUNDLE,,}" == "all" ]]; then
  while IFS= read -r f; do ARTIFACTS+=("$f"); done < <(rg --files "${TARGET_DIR}" -g "*.app")
fi
if [[ "${BUNDLE,,}" == "dmg" || "${BUNDLE,,}" == "all" ]]; then
  while IFS= read -r f; do ARTIFACTS+=("$f"); done < <(rg --files "${TARGET_DIR}" -g "*.dmg")
fi

if [[ ${#ARTIFACTS[@]} -eq 0 ]]; then
  echo "No artifacts found under: ${TARGET_DIR}"
  exit 0
fi

step "Copying artifacts to scripts folder"
for artifact in "${ARTIFACTS[@]}"; do
  cp -Rf "${artifact}" "${SCRIPT_DIR}/"
  echo "Copied: ${SCRIPT_DIR}/$(basename "${artifact}")"
done

echo
echo "OK macOS build complete"
