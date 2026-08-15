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
CHECKSUM=$ARCHIVE.sha256
ARCHIVE_TMP=$OUTPUT_DIR/.$RELEASE_NAME.tar.gz.tmp.$$
CHECKSUM_TMP=$OUTPUT_DIR/.$RELEASE_NAME.tar.gz.sha256.tmp.$$

cleanup() { rm -rf -- "$STAGING_PARENT"; rm -f -- "$ARCHIVE_TMP" "$CHECKSUM_TMP"; }
trap cleanup EXIT HUP INT TERM

for tool in cargo tar sed uname mkdir cp chmod mv rm find; do
    command -v "$tool" >/dev/null 2>&1 || {
        printf 'Required tool not found: %s\n' "$tool" >&2
        exit 1
    }
done
if command -v sha256sum >/dev/null 2>&1; then
    HASH_TOOL=sha256sum
elif command -v shasum >/dev/null 2>&1; then
    HASH_TOOL=shasum
else
    printf 'Neither sha256sum nor shasum is available.\n' >&2
    exit 1
fi
for source in \
    "$SCRIPT_DIR/install.sh" \
    "$SCRIPT_DIR/uninstall.sh" \
    "$SCRIPT_DIR/$APP_ID.desktop.in" \
    "$ROOT_DIR/assets/$APP_ID.svg" \
    "$ROOT_DIR/LICENSE" \
    "$ROOT_DIR/docs/LINUX-DISTRIBUTION.md"
do
    [ -f "$source" ] || { printf 'Required release file not found: %s\n' "$source" >&2; exit 1; }
done
mkdir -p -- "$OUTPUT_DIR"
[ -d "$OUTPUT_DIR" ] && [ -w "$OUTPUT_DIR" ] || {
    printf 'Output directory is not writable: %s\n' "$OUTPUT_DIR" >&2
    exit 1
}

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
    "$STAGING/share/icons/hicolor/scalable/apps" "$STAGING/docs"
cp -- "$BINARY" "$STAGING/bin/$PACKAGE"
cp -- "$SCRIPT_DIR/install.sh" "$SCRIPT_DIR/uninstall.sh" "$STAGING/"
cp -- "$SCRIPT_DIR/$APP_ID.desktop.in" "$STAGING/share/applications/"
cp -- "$ROOT_DIR/assets/$APP_ID.svg" "$STAGING/share/icons/hicolor/scalable/apps/"
cp -- "$ROOT_DIR/LICENSE" "$STAGING/"
cp -- "$ROOT_DIR/docs/LINUX-DISTRIBUTION.md" "$STAGING/docs/"
find "$STAGING" -type d -exec chmod 755 {} +
find "$STAGING" -type f -exec chmod 644 {} +
chmod 755 "$STAGING/install.sh" "$STAGING/uninstall.sh" "$STAGING/bin/$PACKAGE"

# GNU tar and gzip can remove timestamps, ownership, ordering and gzip header
# variance. Other tar implementations retain the portable release path.
if tar --version 2>/dev/null | grep 'GNU tar' >/dev/null 2>&1 && command -v gzip >/dev/null 2>&1; then
    SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-0}
    tar -C "$STAGING_PARENT" --sort=name --format=posix \
        --mtime="@$SOURCE_DATE_EPOCH" --owner=0 --group=0 --numeric-owner \
        --pax-option=delete=atime,delete=ctime -cf - "$RELEASE_NAME" |
        gzip -n >"$ARCHIVE_TMP"
else
    printf 'GNU tar and gzip not both available; archive metadata is not normalized.\n' >&2
    tar -C "$STAGING_PARENT" -czf "$ARCHIVE_TMP" "$RELEASE_NAME"
fi

if [ "$HASH_TOOL" = sha256sum ]; then
    checksum_line=$(sha256sum "$ARCHIVE_TMP")
else
    checksum_line=$(shasum -a 256 "$ARCHIVE_TMP")
fi
checksum_value=${checksum_line%% *}
[ -n "$checksum_value" ] || { printf 'Could not calculate archive checksum.\n' >&2; exit 1; }
printf '%s  %s\n' "$checksum_value" "$RELEASE_NAME.tar.gz" >"$CHECKSUM_TMP"

# Both files are complete before publication. The checksum is the commit marker:
# remove any previous marker, replace the archive, then publish its new checksum.
rm -f -- "$CHECKSUM"
mv -f -- "$ARCHIVE_TMP" "$ARCHIVE"
mv -f -- "$CHECKSUM_TMP" "$CHECKSUM"

trap - EXIT HUP INT TERM
rm -rf -- "$STAGING_PARENT"
printf 'Created %s\nCreated %s\n' "$ARCHIVE" "$CHECKSUM"
