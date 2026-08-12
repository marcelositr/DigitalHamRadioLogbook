#!/bin/sh
set -eu

usage() {
    cat <<'EOF'
Usage: packaging/linux/make-release.sh [OUTPUT_DIRECTORY]

Build a locked release and create a minimal Linux tar.gz plus SHA-256 file.
The default output directory is dist/.
EOF
}

case ${1:-} in --help|-h) usage; exit 0 ;; esac
[ "$#" -le 1 ] || { usage >&2; exit 2; }

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
OUTPUT_DIR=${1:-$ROOT_DIR/dist}
case $OUTPUT_DIR in /*) ;; *) OUTPUT_DIR=$ROOT_DIR/$OUTPUT_DIR ;; esac

PACKAGE=digital-ham-radio-logbook
APP_ID=io.github.marcelositr.DigitalHamRadioLogbook
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | sed -n '1p')
[ -n "$VERSION" ] || { printf 'Could not determine package version.\n' >&2; exit 1; }
ARCH=$(uname -m)
RELEASE_NAME=$PACKAGE-$VERSION-linux-$ARCH
STAGING_PARENT=$OUTPUT_DIR/.staging.$$
STAGING=$STAGING_PARENT/$RELEASE_NAME
ARCHIVE=$OUTPUT_DIR/$RELEASE_NAME.tar.gz

cleanup() { rm -rf -- "$STAGING_PARENT"; }
trap cleanup EXIT HUP INT TERM

cd "$ROOT_DIR"
cargo build --locked --release
BINARY=$ROOT_DIR/target/release/$PACKAGE
[ -x "$BINARY" ] || { printf 'Release binary not found: %s\n' "$BINARY" >&2; exit 1; }

if command -v ldd >/dev/null 2>&1; then
    ldd_output=$(ldd "$BINARY" 2>&1) || { printf '%s\n' "$ldd_output" >&2; exit 1; }
    printf '%s\n' "$ldd_output"
    if printf '%s\n' "$ldd_output" | grep 'not found' >/dev/null 2>&1; then
        printf 'Refusing release with unresolved shared libraries.\n' >&2
        exit 1
    fi
else
    printf 'ldd not available; shared-library validation skipped.\n' >&2
fi

mkdir -p -- "$STAGING/bin" "$STAGING/share/applications" \
    "$STAGING/share/icons/hicolor/scalable/apps" "$STAGING/docs" "$OUTPUT_DIR"
cp -- "$BINARY" "$STAGING/bin/$PACKAGE"
cp -- "$SCRIPT_DIR/install.sh" "$SCRIPT_DIR/uninstall.sh" "$STAGING/"
cp -- "$SCRIPT_DIR/$APP_ID.desktop.in" "$STAGING/share/applications/"
cp -- "$ROOT_DIR/assets/$APP_ID.svg" "$STAGING/share/icons/hicolor/scalable/apps/"
cp -- "$ROOT_DIR/LICENSE" "$STAGING/"
cp -- "$ROOT_DIR/docs/LINUX-DISTRIBUTION.md" "$STAGING/docs/"
chmod 755 "$STAGING/install.sh" "$STAGING/uninstall.sh" "$STAGING/bin/$PACKAGE"

tar -C "$STAGING_PARENT" -czf "$ARCHIVE" "$RELEASE_NAME"
if command -v sha256sum >/dev/null 2>&1; then
    (cd "$OUTPUT_DIR" && sha256sum "$RELEASE_NAME.tar.gz" >"$RELEASE_NAME.tar.gz.sha256")
elif command -v shasum >/dev/null 2>&1; then
    (cd "$OUTPUT_DIR" && shasum -a 256 "$RELEASE_NAME.tar.gz" >"$RELEASE_NAME.tar.gz.sha256")
else
    printf 'Neither sha256sum nor shasum is available.\n' >&2
    exit 1
fi

printf 'Created %s\nCreated %s.sha256\n' "$ARCHIVE" "$ARCHIVE"
