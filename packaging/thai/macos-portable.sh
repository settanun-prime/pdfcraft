#!/usr/bin/env bash
# Pack the app packaging/macos/package.sh just built as a portable zip: the app and portable.txt
# in one folder, so settings stay beside it (crates/ui-egui/src/platform/portable.rs).
#   packaging/thai/macos-portable.sh <name>   e.g. 0.18.1-th.4 -> dist/release/pdfcraft-<name>-macos-aarch64-portable.zip
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
NAME="${1:?usage: macos-portable.sh <version-label>}"
APP="${CARGO_TARGET_DIR:-$ROOT/target}/macos-package/PdfCraft.app"
DIST="${DIST:-$ROOT/dist/release}"
[[ -d "$APP" ]] || { echo "no $APP: run packaging/macos/package.sh first" >&2; exit 1; }
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
FOLDER="$STAGE/PdfCraft TH Portable"
mkdir -p "$FOLDER"
ditto "$APP" "$FOLDER/PdfCraft TH.app"
cp "$ROOT/packaging/thai/portable.txt" "$FOLDER/portable.txt"
codesign --verify --strict --deep "$FOLDER/PdfCraft TH.app"
mkdir -p "$DIST"
OUT="$DIST/pdfcraft-$NAME-macos-aarch64-portable.zip"
rm -f "$OUT"
ditto -c -k --keepParent "$FOLDER" "$OUT"
ls -l "$OUT"
