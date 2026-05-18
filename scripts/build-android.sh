#! /bin/sh
set -euo pipefail

# ============================================================
# Android build script for habit-slot
# Generates icons, builds APK, signs and aligns.
# Run from the project root: make release-android
# Or directly: scripts/build-android.sh
#
# Required env vars (or set defaults below):
#   ANDROID_KEYSTORE  - path to .jks file
#   ANDROID_KEYPASS   - keystore + key password
#   ANDROID_KEY_ALIAS - key alias name
# ============================================================

# --------------- Config ---------------
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ICON_SOURCE="${PROJECT_ROOT}/static/devil-icon.png"
DX_OUTPUT="${PROJECT_ROOT}/target/dx/habit-slot/release/android"

# Load secrets from .env
if [ -f "${PROJECT_ROOT}/.env" ]; then
  while IFS='=' read -r key value; do
    case "$key" in
      ANDROID_KEYSTORE_PASSWORD) ANDROID_KEYSTORE_PASSWORD="${value}" ;;
      ANDROID_KEY_PASSWORD) ANDROID_KEY_PASSWORD="${value}" ;;
    esac
  done < "${PROJECT_ROOT}/.env"
fi

ANDROID_KEYSTORE="${PROJECT_ROOT}/game-release.keystore"
ANDROID_KEYPASS="${ANDROID_KEYSTORE_PASSWORD:-}"
ANDROID_KEY_ALIAS="${ANDROID_KEY_ALIAS:-habit-slot}"

# --------------- Helper: locate magick ---------------
MAGICK="magick"
if ! command -v magick >/dev/null 2>&1; then
  NIX_MAGICK=$(find /nix/store -path '*/bin/magick' -type f 2>/dev/null | head -1)
  if [ -n "${NIX_MAGICK}" ]; then
    MAGICK="${NIX_MAGICK}"
  else
    echo "ERROR: Need 'magick' (ImageMagick). Install with: nix-shell -p imagemagick" >&2
    exit 1
  fi
fi

# --------------- Android SDK tools ---------------
if [ -z "${ANDROID_HOME}" ]; then
  ANDROID_HOME="${HOME}/Android/Sdk"
fi

# Pick latest build-tools version
BUILD_TOOLS=$(ls -1 "${ANDROID_HOME}/build-tools/" 2>/dev/null | sort -V | tail -1)
if [ -n "${BUILD_TOOLS}" ]; then
  export PATH="${ANDROID_HOME}/build-tools/${BUILD_TOOLS}:${ANDROID_HOME}/platform-tools:${PATH}"
fi

# --------------- Validate ---------------
if [ ! -f "${ICON_SOURCE}" ]; then
  echo "ERROR: Icon source not found: ${ICON_SOURCE}" >&2
  exit 1
fi

if [ -z "${ANDROID_KEYPASS}" ]; then
  echo "ERROR: ANDROID_KEYSTORE_PASSWORD not found in .env" >&2
  exit 1
fi

if [ ! -f "${ANDROID_KEYSTORE}" ]; then
  echo "ERROR: Keystore not found: ${ANDROID_KEYSTORE}" >&2
  exit 1
fi

# --------------- Step 1: Bundle with dx ---------------
echo "==> Cleaning release dir & bundling with dx..."
rm -rf "${PROJECT_ROOT}/target/dx/habit-slot"

cd "${PROJECT_ROOT}"
dx bundle --platform android --release --target aarch64-linux-android

if [ ! -d "${DX_OUTPUT}/app" ]; then
  echo "ERROR: Expected Android project not found at ${DX_OUTPUT}/app" >&2
  exit 1
fi

# --------------- Step 2: Generate icons & strip webps ---------------
echo "==> Generating icons..."
cd "${DX_OUTPUT}/app"

RES_DIR="app/src/main/res"

WEBP_COUNT=$(find "${RES_DIR}" -name "*.webp" -type f | wc -l)
if [ "${WEBP_COUNT}" -gt 0 ]; then
  echo "    Removing ${WEBP_COUNT} .webp files from dx bundle..."
  find "${RES_DIR}" -name "*.webp" -type f -delete
fi

rm -f "${RES_DIR}/mipmap-anydpi-v26/ic_launcher.xml" 2>/dev/null || true

SIZES="mdpi:48 hdpi:72 xhdpi:96 xxhdpi:144 xxxhdpi:192"

for spec in ${SIZES}; do
  density=$(echo "${spec}" | cut -d: -f1)
  size=$(echo "${spec}" | cut -d: -f2)

  target_dir="${RES_DIR}/mipmap-${density}"
  mkdir -p "${target_dir}"

  echo "    ${density}: ${size}px"
  ${MAGICK} "${ICON_SOURCE}" -resize "${size}x${size}" "${target_dir}/ic_launcher.png"
done

# --------------- Step 3: Build the APK ---------------
echo "==> Building with Gradle..."
./gradlew assembleRelease

RELEASE_APK="app/build/outputs/apk/release/app-release.apk"

if [ ! -f "${RELEASE_APK}" ]; then
  echo "ERROR: APK not found at ${RELEASE_APK}" >&2
  exit 1
fi

# Copy to project root release-build/
mkdir -p "${PROJECT_ROOT}/release-build"
cp -f "${DX_OUTPUT}/app/${RELEASE_APK}" "${PROJECT_ROOT}/release-build/habit-slot-release.apk"

FINAL="${PROJECT_ROOT}/release-build/habit-slot-release.apk"

echo ""
echo "==> Done! Signed APK: ${FINAL}"
echo "    To install: adb install -r ${FINAL}"
