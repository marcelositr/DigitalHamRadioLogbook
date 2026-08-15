#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
APP_ID=io.github.marcelositr.DigitalHamRadioLogbook
PACKAGE=digital-ham-radio-logbook
TMP_ROOT=${TMPDIR:-/tmp}/digital-ham-radio-logbook-packaging-test.$$
FIXTURE=$TMP_ROOT/repository
TOOLS=$TMP_ROOT/tools
DIST=$TMP_ROOT/dist
EXTRACT=$TMP_ROOT/extract
TEST_HOME=$TMP_ROOT/home
XDG_DATA_HOME=$TMP_ROOT/xdg-data
XDG_CONFIG_HOME=$TMP_ROOT/xdg-config
XDG_BIN_HOME=$TMP_ROOT/xdg-bin

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
cleanup() { rm -rf "$TMP_ROOT"; }
trap cleanup EXIT HUP INT TERM

for tool in tar sed grep find sort cmp mkdir cp chmod rm; do
    command -v "$tool" >/dev/null 2>&1 || fail "required test tool not found: $tool"
done
if command -v sha256sum >/dev/null 2>&1; then
    VERIFY_CHECKSUM=sha256sum
elif command -v shasum >/dev/null 2>&1; then
    VERIFY_CHECKSUM=shasum
else
    fail 'sha256sum or shasum is required'
fi

mkdir -p "$FIXTURE/packaging/linux" "$FIXTURE/assets" "$FIXTURE/docs" \
    "$TOOLS" "$DIST" "$EXTRACT" "$TEST_HOME" "$XDG_DATA_HOME" \
    "$XDG_CONFIG_HOME/digital-ham-log" "$XDG_BIN_HOME"
cp "$SCRIPT_DIR/make-release.sh" "$SCRIPT_DIR/install.sh" \
    "$SCRIPT_DIR/uninstall.sh" "$SCRIPT_DIR/$APP_ID.desktop.in" \
    "$FIXTURE/packaging/linux/"
cp "$ROOT_DIR/assets/$APP_ID.svg" "$FIXTURE/assets/"
cp "$ROOT_DIR/docs/LINUX-DISTRIBUTION.md" "$FIXTURE/docs/"
cp "$ROOT_DIR/LICENSE" "$FIXTURE/"
printf '%s\n' '[package]' 'name = "digital-ham-radio-logbook"' 'version = "0.0.0-smoke"' >"$FIXTURE/Cargo.toml"

cat >"$TOOLS/cargo" <<'EOF'
#!/bin/sh
set -eu
[ "$#" -eq 3 ] && [ "$1" = build ] && [ "$2" = --locked ] && [ "$3" = --release ] || exit 64
mkdir -p target/release
cat >target/release/digital-ham-radio-logbook <<'PAYLOAD'
#!/bin/sh
printf 'smoke payload\n'
PAYLOAD
chmod 755 target/release/digital-ham-radio-logbook
EOF
cat >"$TOOLS/ldd" <<'EOF'
#!/bin/sh
printf 'smoke payload has no shared-library dependencies\n'
EOF
chmod 755 "$TOOLS/cargo" "$TOOLS/ldd"

PATH=$TOOLS:$PATH SOURCE_DATE_EPOCH=0 "$FIXTURE/packaging/linux/make-release.sh" "$DIST"
ARCHIVE=$(find "$DIST" -type f -name '*.tar.gz' -print)
CHECKSUM=$(find "$DIST" -type f -name '*.tar.gz.sha256' -print)
[ -n "$ARCHIVE" ] && [ -f "$ARCHIVE" ] || fail 'release archive was not created'
[ -n "$CHECKSUM" ] && [ -f "$CHECKSUM" ] || fail 'release checksum was not created'
[ "$(find "$DIST" -type f -name '*.tmp.*' -print)" = '' ] || fail 'temporary publication files remain'

(cd "$DIST" && if [ "$VERIFY_CHECKSUM" = sha256sum ]; then sha256sum -c "$(basename "$CHECKSUM")"; else shasum -a 256 -c "$(basename "$CHECKSUM")"; fi)
RELEASE_DIR=$(basename "$ARCHIVE" .tar.gz)
tar -tzf "$ARCHIVE" | sort >"$TMP_ROOT/contents.actual"
cat >"$TMP_ROOT/contents.expected" <<EOF
$RELEASE_DIR/
$RELEASE_DIR/LICENSE
$RELEASE_DIR/bin/
$RELEASE_DIR/bin/$PACKAGE
$RELEASE_DIR/docs/
$RELEASE_DIR/docs/LINUX-DISTRIBUTION.md
$RELEASE_DIR/install.sh
$RELEASE_DIR/share/
$RELEASE_DIR/share/applications/
$RELEASE_DIR/share/applications/$APP_ID.desktop.in
$RELEASE_DIR/share/icons/
$RELEASE_DIR/share/icons/hicolor/
$RELEASE_DIR/share/icons/hicolor/scalable/
$RELEASE_DIR/share/icons/hicolor/scalable/apps/
$RELEASE_DIR/share/icons/hicolor/scalable/apps/$APP_ID.svg
$RELEASE_DIR/uninstall.sh
EOF
sort "$TMP_ROOT/contents.expected" -o "$TMP_ROOT/contents.expected"
cmp "$TMP_ROOT/contents.expected" "$TMP_ROOT/contents.actual" || fail 'archive content differs from the release contract'
tar -xzf "$ARCHIVE" -C "$EXTRACT"
PAYLOAD_DIR=$EXTRACT/$RELEASE_DIR

printf 'preserve data\n' >"$XDG_DATA_HOME/digital-ham-log.sentinel"
printf 'preserve config\n' >"$XDG_CONFIG_HOME/digital-ham-log/config.toml"
ENV_PREFIX="HOME=$TEST_HOME XDG_DATA_HOME=$XDG_DATA_HOME XDG_CONFIG_HOME=$XDG_CONFIG_HOME XDG_BIN_HOME=$XDG_BIN_HOME"
HOME=$TEST_HOME XDG_DATA_HOME=$XDG_DATA_HOME XDG_CONFIG_HOME=$XDG_CONFIG_HOME XDG_BIN_HOME=$XDG_BIN_HOME \
    "$PAYLOAD_DIR/install.sh" --dry-run >"$TMP_ROOT/install-dry-run"
[ ! -e "$XDG_BIN_HOME/$PACKAGE" ] || fail 'install dry-run changed the filesystem'
grep "Install $XDG_BIN_HOME/$PACKAGE" "$TMP_ROOT/install-dry-run" >/dev/null || fail 'install dry-run omitted binary'

HOME=$TEST_HOME XDG_DATA_HOME=$XDG_DATA_HOME XDG_CONFIG_HOME=$XDG_CONFIG_HOME XDG_BIN_HOME=$XDG_BIN_HOME "$PAYLOAD_DIR/install.sh"
HOME=$TEST_HOME XDG_DATA_HOME=$XDG_DATA_HOME XDG_CONFIG_HOME=$XDG_CONFIG_HOME XDG_BIN_HOME=$XDG_BIN_HOME "$PAYLOAD_DIR/install.sh"
[ -x "$XDG_BIN_HOME/$PACKAGE" ] || fail 'binary missing after reinstall'
grep "Exec=\"$XDG_BIN_HOME/$PACKAGE\"" "$XDG_DATA_HOME/applications/$APP_ID.desktop" >/dev/null || fail 'desktop entry has wrong executable'

INSTALLED_UNINSTALL=$XDG_DATA_HOME/$APP_ID/uninstall.sh
HOME=$TEST_HOME XDG_DATA_HOME=$XDG_DATA_HOME XDG_CONFIG_HOME=$XDG_CONFIG_HOME XDG_BIN_HOME=$XDG_BIN_HOME \
    "$INSTALLED_UNINSTALL" --dry-run >"$TMP_ROOT/uninstall-dry-run"
[ -x "$XDG_BIN_HOME/$PACKAGE" ] || fail 'uninstall dry-run changed the filesystem'
HOME=$TEST_HOME XDG_DATA_HOME=$XDG_DATA_HOME XDG_CONFIG_HOME=$XDG_CONFIG_HOME XDG_BIN_HOME=$XDG_BIN_HOME "$INSTALLED_UNINSTALL"
[ ! -e "$XDG_BIN_HOME/$PACKAGE" ] || fail 'binary remains after uninstall'
[ ! -e "$XDG_DATA_HOME/applications/$APP_ID.desktop" ] || fail 'desktop entry remains after uninstall'
[ "$(cat "$XDG_DATA_HOME/digital-ham-log.sentinel")" = 'preserve data' ] || fail 'XDG data was modified'
[ "$(cat "$XDG_CONFIG_HOME/digital-ham-log/config.toml")" = 'preserve config' ] || fail 'XDG config was modified'

printf 'Linux packaging smoke test passed.\n'
