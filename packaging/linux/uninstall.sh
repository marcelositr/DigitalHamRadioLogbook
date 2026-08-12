#!/bin/sh
set -eu

APP_ID=io.github.marcelositr.DigitalHamRadioLogbook
BINARY=digital-ham-radio-logbook
DRY_RUN=false

usage() {
    cat <<'EOF'
Usage: uninstall.sh [--dry-run] [--help]

Remove only application files installed for the current user.
User data and configuration under digital-ham-log are always preserved.
EOF
}

for arg do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --help|-h) usage; exit 0 ;;
        *) printf 'Unknown option: %s\n' "$arg" >&2; usage >&2; exit 2 ;;
    esac
done

HOME=${HOME:?HOME is not set}
DATA_HOME=${XDG_DATA_HOME:-$HOME/.local/share}
BIN_HOME=${XDG_BIN_HOME:-$HOME/.local/bin}
APP_DIR=$DATA_HOME/$APP_ID
MANIFEST=$APP_DIR/install-manifest

remove_file() {
    path=$1
    if [ "$DRY_RUN" = true ]; then
        printf 'Remove %s\n' "$path"
    else
        rm -f "$path"
    fi
}

# Use a fixed allowlist: manifest contents can never authorize arbitrary deletion.
for path in \
    "$BIN_HOME/$BINARY" \
    "$DATA_HOME/applications/$APP_ID.desktop" \
    "$DATA_HOME/icons/hicolor/scalable/apps/$APP_ID.svg" \
    "$APP_DIR/uninstall.sh"
do
    if [ -f "$path" ] || [ -L "$path" ] || [ "$DRY_RUN" = true ]; then
        remove_file "$path"
    fi
done

remove_file "$MANIFEST"
if [ "$DRY_RUN" = true ]; then
    printf 'Remove empty directory %s\n' "$APP_DIR"
else
    rmdir "$APP_DIR" 2>/dev/null || :
fi

printf 'Digital Ham Radio Logbook application files removed.\n'
printf 'Data and configuration in digital-ham-log were preserved.\n'
