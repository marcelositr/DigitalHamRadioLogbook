const MAIN: &str = include_str!("../ui/main.slint");
const DESIGN: &str = include_str!("../ui/design-system.slint");
const SHELL: &str = include_str!("../ui/components/app-shell.slint");
const EDITOR: &str = include_str!("../ui/pages/qso-editor-page.slint");
const TOOLS: &str = include_str!("../ui/pages/tools-page.slint");
const SETTINGS: &str = include_str!("../ui/pages/settings-page.slint");

#[test]
fn main_window_uses_persistent_desktop_shell() {
    assert!(MAIN.contains("DesktopMenuBar"));
    assert!(MAIN.contains("AppSidebar"));
    assert!(MAIN.contains("ContextBar"));
    assert!(MAIN.contains("StatusBar"));
    assert!(MAIN.contains("sidebar-collapsed"));
}

#[test]
fn material_desktop_design_system_keeps_restrained_roles() {
    for token in [
        "surface-selected",
        "border-subtle",
        "text-secondary",
        "primary-hover",
        "warning-surface",
        "success-surface",
    ] {
        assert!(DESIGN.contains(token), "missing design token: {token}");
    }
    assert!(DESIGN.contains("4 px desktop spacing grid"));
    assert!(DESIGN.contains("Used for actions, focus and selection, never decoration"));
}

#[test]
fn shell_preserves_four_product_workspaces() {
    for label in ["Logbook", "New QSO", "Tools", "Settings"] {
        assert!(SHELL.contains(label), "missing shell destination: {label}");
    }
    for group in ["Operation", "Data", "System"] {
        assert!(SHELL.contains(group), "missing shell group: {group}");
    }
}

#[test]
fn rust_facing_qso_shortcuts_remain_in_main_contract() {
    for shortcut in [
        "event.modifiers.control && event.text == \"n\"",
        "event.modifiers.control && event.text == \"s\"",
        "event.modifiers.control && event.text == Key.Return",
        "event.modifiers.control && event.text == \"f\"",
    ] {
        assert!(
            MAIN.contains(shortcut),
            "missing shortcut contract: {shortcut}"
        );
    }
}

#[test]
fn editor_retains_mode_specific_workspaces_and_fixed_actions() {
    for section in [
        "DMR details",
        "FT8 details",
        "D-STAR details",
        "YSF / C4FM details",
    ] {
        assert!(
            EDITOR.contains(section),
            "missing editor section: {section}"
        );
    }
    for action in ["Save & New", "Cancel", "Save QSO", "Save changes"] {
        assert!(EDITOR.contains(action), "missing editor action: {action}");
    }
}

#[test]
fn tools_remains_split_by_interoperability_health_and_backup() {
    assert!(TOOLS.contains("ADIF import and export"));
    assert!(TOOLS.contains("Data health"));
    assert!(TOOLS.contains("Database backup"));
    assert!(TOOLS.contains("Export current results"));
    assert!(TOOLS.contains("Check data health"));
}

#[test]
fn settings_keeps_local_identity_primary_and_external_links_explicit() {
    assert!(SETTINGS.contains("Local station"));
    assert!(SETTINGS.contains("Primary operating identity"));
    assert!(SETTINGS.contains("External lookup links"));
    assert!(SETTINGS.contains("External websites open only after an explicit click"));
}
