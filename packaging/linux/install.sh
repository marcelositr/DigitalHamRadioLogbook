#!/bin/sh
set -eu

APP_ID=io.github.marcelositr.DigitalHamRadioLogbook
BINARY=digital-ham-radio-logbook
DRY_RUN=false

usage() {
    cat <<'EOF'
Usage: ./install.sh [--dry-run] [--help]

Install Digital Ham Radio Logbook for the current user using XDG paths.
  --dry-run  Print the operations without changing the filesystem
  --help     Show this help
EOF
}

for arg do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --help|-h) usage; exit 0 ;;
        *) printf 'Unknown option: %s\n' "$arg" >&2; usage >&2; exit 2 ;;
    esac
done

SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
PAYLOAD_DIR=$SCRIPT_DIR
[ -f "$PAYLOAD_DIR/bin/$BINARY" ] || { printf 'Missing release payload: bin/%s\n' "$BINARY" >&2; exit 1; }
[ -f "$PAYLOAD_DIR/share/applications/$APP_ID.desktop.in" ] || { printf 'Missing desktop template.\n' >&2; exit 1; }
[ -f "$PAYLOAD_DIR/share/icons/hicolor/scalable/apps/$APP_ID.svg" ] || { printf 'Missing application icon.\n' >&2; exit 1; }
[ -f "$PAYLOAD_DIR/uninstall.sh" ] || { printf 'Missing uninstall.sh.\n' >&2; exit 1; }

HOME=${HOME:?HOME is not set}
DATA_HOME=${XDG_DATA_HOME:-$HOME/.local/share}
BIN_HOME=${XDG_BIN_HOME:-$HOME/.local/bin}
APP_DIR=$DATA_HOME/$APP_ID
BIN_DEST=$BIN_HOME/$BINARY
DESKTOP_DEST=$DATA_HOME/applications/$APP_ID.desktop
ICON_DEST=$DATA_HOME/icons/hicolor/scalable/apps/$APP_ID.svg
UNINSTALL_DEST=$APP_DIR/uninstall.sh
MANIFEST=$APP_DIR/install-manifest

show_plan() {
    printf '%s\n' "Install $BIN_DEST" "Install $DESKTOP_DEST" "Install $ICON_DEST" \
        "Install $UNINSTALL_DEST" "Write $MANIFEST"
}

if [ "$DRY_RUN" = true ]; then
    show_plan
    exit 0
fi

umask 077
for dir in "$BIN_HOME" "$DATA_HOME/applications" "$DATA_HOME/icons/hicolor/scalable/apps" "$APP_DIR"; do
    if [ -L "$dir" ]; then
        printf 'Refusing symbolic-link directory: %s\n' "$dir" >&2
        exit 1
    fi
    mkdir -p "$dir"
done

TMP_FILES=
cleanup() {
    old_ifs=$IFS
    IFS='
'
    for file in $TMP_FILES; do rm -f "$file"; done
    IFS=$old_ifs
}
trap cleanup EXIT HUP INT TERM

install_file() {
    source=$1 destination=$2 mode=$3
    temporary=${destination}.tmp.$$
    TMP_FILES="$TMP_FILES
$temporary"
    cp "$source" "$temporary"
    chmod "$mode" "$temporary"
    mv -f "$temporary" "$destination"
}

install_file "$PAYLOAD_DIR/bin/$BINARY" "$BIN_DEST" 755
install_file "$PAYLOAD_DIR/share/icons/hicolor/scalable/apps/$APP_ID.svg" "$ICON_DEST" 644
install_file "$PAYLOAD_DIR/uninstall.sh" "$UNINSTALL_DEST" 755

escaped_exec=$(printf '%s' "$BIN_DEST" | sed 's/[\\&|]/\\&/g; s/"/\\\\"/g')
desktop_tmp=${DESKTOP_DEST}.tmp.$$
TMP_FILES="$TMP_FILES
$desktop_tmp"
sed "s|@EXECUTABLE@|$escaped_exec|g" "$PAYLOAD_DIR/share/applications/$APP_ID.desktop.in" >"$desktop_tmp"
chmod 644 "$desktop_tmp"
mv -f "$desktop_tmp" "$DESKTOP_DEST"

manifest_tmp=${MANIFEST}.tmp.$$
TMP_FILES="$TMP_FILES
$manifest_tmp"
printf '%s\n' "$BIN_DEST" "$DESKTOP_DEST" "$ICON_DEST" "$UNINSTALL_DEST" >"$manifest_tmp"
chmod 600 "$manifest_tmp"
mv -f "$manifest_tmp" "$MANIFEST"

trap - EXIT HUP INT TERM
printf 'Installed Digital Ham Radio Logbook.\nRun: %s\nUninstall: %s\n' "$BIN_DEST" "$UNINSTALL_DEST"
