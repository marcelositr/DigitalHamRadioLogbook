#!/bin/sh
set -eu

usage() {
    cat <<'EOF'
Usage: packaging/linux/make-additional-packages.sh BINARY [OUTPUT_DIRECTORY]

Create .deb and AppImage packages plus SHA-256 files from an existing release
binary. This script does not compile or modify the binary.

Environment:
  APPIMAGETOOL  Path to appimagetool (default: appimagetool from PATH)
EOF
}

case ${1:-} in --help|-h) usage; exit 0 ;; esac
[ "$#" -ge 1 ] && [ "$#" -le 2 ] || { usage >&2; exit 2; }

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
BINARY=$1
OUTPUT_DIR=${2:-$ROOT_DIR/dist}
case $BINARY in /*) ;; *) BINARY=$ROOT_DIR/$BINARY ;; esac
case $OUTPUT_DIR in /*) ;; *) OUTPUT_DIR=$ROOT_DIR/$OUTPUT_DIR ;; esac

PACKAGE=digital-ham-radio-logbook
APP_ID=io.github.marcelositr.DigitalHamRadioLogbook
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | sed -n '1p')
[ -n "$VERSION" ] || { printf 'Could not determine package version.\n' >&2; exit 1; }
[ -x "$BINARY" ] || { printf 'Release binary is not executable: %s\n' "$BINARY" >&2; exit 1; }

case $(uname -m) in
    x86_64) DEB_ARCH=amd64; APPIMAGE_ARCH=x86_64 ;;
    aarch64) DEB_ARCH=arm64; APPIMAGE_ARCH=aarch64 ;;
    *) printf 'Unsupported package architecture: %s\n' "$(uname -m)" >&2; exit 1 ;;
esac
DEB_VERSION=$(printf '%s' "$VERSION" | sed 's/-rc\./~rc/')
DEB_FILE_VERSION=$(printf '%s' "$VERSION" | sed 's/-rc\./.rc/')
APPIMAGETOOL=${APPIMAGETOOL:-appimagetool}
command -v dpkg-deb >/dev/null 2>&1 || { printf 'Required tool not found: dpkg-deb\n' >&2; exit 1; }
command -v "$APPIMAGETOOL" >/dev/null 2>&1 || { printf 'Required tool not found: %s\n' "$APPIMAGETOOL" >&2; exit 1; }
command -v sha256sum >/dev/null 2>&1 || { printf 'Required tool not found: sha256sum\n' >&2; exit 1; }

for source in \
    "$SCRIPT_DIR/$APP_ID.desktop.in" \
    "$ROOT_DIR/assets/$APP_ID.svg" \
    "$ROOT_DIR/assets/$APP_ID.appdata.xml" \
    "$ROOT_DIR/LICENSE"
do
    [ -f "$source" ] || { printf 'Required package file not found: %s\n' "$source" >&2; exit 1; }
done

mkdir -p -- "$OUTPUT_DIR"
WORK=$OUTPUT_DIR/.additional-packages.$$
DEB_ROOT=$WORK/deb
APPDIR=$WORK/AppDir
cleanup() { rm -rf -- "$WORK"; }
trap cleanup EXIT HUP INT TERM

mkdir -p -- "$DEB_ROOT/DEBIAN" "$DEB_ROOT/usr/bin" \
    "$DEB_ROOT/usr/share/applications" \
    "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps" \
    "$DEB_ROOT/usr/share/metainfo" \
    "$DEB_ROOT/usr/share/doc/$PACKAGE"
cp -- "$BINARY" "$DEB_ROOT/usr/bin/$PACKAGE"
sed 's|@EXECUTABLE@|/usr/bin/digital-ham-radio-logbook|g' \
    "$SCRIPT_DIR/$APP_ID.desktop.in" >"$DEB_ROOT/usr/share/applications/$APP_ID.desktop"
cp -- "$ROOT_DIR/assets/$APP_ID.svg" "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/"
cp -- "$ROOT_DIR/assets/$APP_ID.appdata.xml" "$DEB_ROOT/usr/share/metainfo/"
cp -- "$ROOT_DIR/LICENSE" "$DEB_ROOT/usr/share/doc/$PACKAGE/copyright"
cat >"$DEB_ROOT/DEBIAN/control" <<EOF
Package: $PACKAGE
Version: $DEB_VERSION
Section: hamradio
Priority: optional
Architecture: $DEB_ARCH
Depends: libc6, libfontconfig1, libfreetype6
Maintainer: Marcelo Trindade <marcelositr@users.noreply.github.com>
Homepage: https://github.com/marcelositr/DigitalHamRadioLogbook
Description: Local and offline digital amateur radio logbook
 Desktop logbook for Generic, DMR, FT8, D-STAR and YSF/C4FM contacts,
 using a local SQLite database and ADIF interoperability.
EOF
find "$DEB_ROOT" -type d -exec chmod 755 {} +
find "$DEB_ROOT" -type f -exec chmod 644 {} +
chmod 755 "$DEB_ROOT/usr/bin/$PACKAGE"
# GitHub release assets sanitize '~' in filenames. Keep Debian's '~rc' inside
# package metadata, but use a stable '.rc' asset filename whose checksum remains valid.
DEB_FILE=$OUTPUT_DIR/${PACKAGE}_${DEB_FILE_VERSION}_${DEB_ARCH}.deb
dpkg-deb --root-owner-group --build "$DEB_ROOT" "$DEB_FILE"

mkdir -p -- "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/scalable/apps" "$APPDIR/usr/share/metainfo"
cp -- "$BINARY" "$APPDIR/usr/bin/$PACKAGE"
sed 's|@EXECUTABLE@|digital-ham-radio-logbook|g' \
    "$SCRIPT_DIR/$APP_ID.desktop.in" >"$APPDIR/$APP_ID.desktop"
cp -- "$APPDIR/$APP_ID.desktop" "$APPDIR/usr/share/applications/"
cp -- "$ROOT_DIR/assets/$APP_ID.svg" "$APPDIR/$APP_ID.svg"
cp -- "$ROOT_DIR/assets/$APP_ID.svg" "$APPDIR/usr/share/icons/hicolor/scalable/apps/"
cp -- "$ROOT_DIR/assets/$APP_ID.appdata.xml" "$APPDIR/usr/share/metainfo/"
ln -s "$APP_ID.svg" "$APPDIR/.DirIcon"
cat >"$APPDIR/AppRun" <<'EOF'
#!/bin/sh
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$HERE/usr/bin/digital-ham-radio-logbook" "$@"
EOF
chmod 755 "$APPDIR/AppRun" "$APPDIR/usr/bin/$PACKAGE"
APPIMAGE_FILE=$OUTPUT_DIR/${PACKAGE}-${VERSION}-${APPIMAGE_ARCH}.AppImage
ARCH=$APPIMAGE_ARCH "$APPIMAGETOOL" "$APPDIR" "$APPIMAGE_FILE"
chmod 755 "$APPIMAGE_FILE"

for artifact in "$DEB_FILE" "$APPIMAGE_FILE"; do
    checksum_line=$(sha256sum "$artifact")
    checksum_value=${checksum_line%% *}
    printf '%s  %s\n' "$checksum_value" "$(basename "$artifact")" >"$artifact.sha256"
done

trap - EXIT HUP INT TERM
rm -rf -- "$WORK"
printf 'Created %s\nCreated %s\n' \
    "$DEB_FILE" "$DEB_FILE.sha256" "$APPIMAGE_FILE" "$APPIMAGE_FILE.sha256"
