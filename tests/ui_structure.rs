const MAIN: &str = include_str!("../ui/main.slint");
const FOUNDATION: &str = include_str!("../ui/design-system.slint");
const SHELL: &str = include_str!("../ui/components/app-shell.slint");
const LOGBOOK: &str = include_str!("../ui/pages/logbook-page.slint");
const EDITOR: &str = include_str!("../ui/pages/qso-editor-page.slint");
const TOOLS: &str = include_str!("../ui/pages/tools-page.slint");
const SETTINGS: &str = include_str!("../ui/pages/settings-page.slint");

#[test]
fn main_window_uses_native_desktop_structure() {
    assert!(MAIN.contains("MenuBar"));
    assert!(MAIN.contains("AppSidebar"));
    assert!(MAIN.contains("StatusBar"));
    assert!(MAIN.contains("preferred-width: 1050px"));
    assert!(MAIN.contains("preferred-height: 680px"));
    assert!(!MAIN.contains("DesktopMenuBar"));
    assert!(!MAIN.contains("ContextBar"));
    assert!(!MAIN.contains("\n    width: 1050px;"));
}

#[test]
fn visual_foundation_uses_slint_style_instead_of_parallel_theme() {
    assert!(FOUNDATION.contains("Palette"));
    assert!(FOUNDATION.contains("StyleMetrics"));
    assert!(FOUNDATION.contains("FormField"));
    assert!(FOUNDATION.contains("TextAction"));
    assert!(!FOUNDATION.contains("export global Theme"));
    assert!(!FOUNDATION.contains("accent-strong"));
    assert!(!FOUNDATION.contains("surface-raised"));
}

#[test]
fn shell_preserves_four_product_workspaces_without_category_chrome() {
    for label in ["Logbook", "New QSO", "Tools", "Settings"] {
        assert!(SHELL.contains(label), "missing shell destination: {label}");
    }

    assert!(SHELL.contains("Palette.background"));
    assert!(!SHELL.contains("OPERATION"));
    assert!(!SHELL.contains("LOCAL-FIRST"));
    assert!(!SHELL.contains("ContextBar"));
}

#[test]
fn rust_facing_qso_shortcuts_remain_in_main_contract() {
    for shortcut in [
        "event.modifiers.control && event.text == \"n\"",
        "event.text == \"s\"",
        "event.text == Key.Return",
        "event.modifiers.control && event.text == \"f\"",
    ] {
        assert!(
            MAIN.contains(shortcut),
            "missing shortcut contract: {shortcut}"
        );
    }
}

#[test]
fn logbook_is_a_data_workspace_not_a_card_stack() {
    assert!(LOGBOOK.contains("ListView"));
    assert!(LOGBOOK.contains("Route / signal"));
    assert!(LOGBOOK.contains("New QSO"));
    assert!(LOGBOOK.contains("GroupBox"));
    assert!(!LOGBOOK.contains("ModeBadge"));
    assert!(!LOGBOOK.contains("FilterChip"));
    assert!(!LOGBOOK.contains("QSO ACTIVITY"));
}

#[test]
fn editor_uses_native_groupboxes_for_all_mode_workspaces() {
    assert!(EDITOR.contains("GroupBox"));
    for section in [
        "Contact",
        "Station and report",
        "DMR",
        "FT8",
        "D-STAR",
        "YSF / C4FM",
        "Notes",
    ] {
        assert!(EDITOR.contains(section), "missing editor section: {section}");
    }

    for action in ["Save & New", "Cancel", "Save QSO", "Save changes"] {
        assert!(EDITOR.contains(action), "missing editor action: {action}");
    }

    assert!(!EDITOR.contains("Panel"));
    assert!(!EDITOR.contains("DMR DETAILS"));
}

#[test]
fn tools_remains_split_by_interoperability_health_and_backup() {
    assert!(TOOLS.contains("ADIF import and export"));
    assert!(TOOLS.contains("Data health"));
    assert!(TOOLS.contains("Database backup"));
    assert!(TOOLS.contains("Export current results"));
    assert!(TOOLS.contains("Check data health"));
    assert!(TOOLS.contains("GroupBox"));
    assert!(!TOOLS.contains("Panel"));
}

#[test]
fn settings_keeps_local_identity_primary_and_external_links_explicit() {
    assert!(SETTINGS.contains("Local station"));
    assert!(SETTINGS.contains("External lookup links"));
    assert!(SETTINGS.contains(
        "External websites open only after an explicit lookup action"
    ));
    assert!(SETTINGS.contains("GroupBox"));
    assert!(!SETTINGS.contains("Panel"));
}
