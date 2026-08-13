use super::*;

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct EditorSnapshot(pub(super) Vec<String>);

pub(super) fn has_unsaved_changes(current: &EditorSnapshot, baseline: &EditorSnapshot) -> bool {
    current != baseline
}

pub(super) fn has_pending_exit_work(
    active_page: i32,
    current: &EditorSnapshot,
    baseline: &EditorSnapshot,
    adif_preview_pending: bool,
) -> bool {
    (active_page == 1 && has_unsaved_changes(current, baseline)) || adif_preview_pending
}

pub(crate) fn editor_snapshot(ui: &MainWindow) -> EditorSnapshot {
    EditorSnapshot(vec![
        ui.get_editing_id().to_string(),
        ui.get_callsign_text().to_string(),
        ui.get_datetime_text().to_string(),
        ui.get_mode_text().to_string(),
        ui.get_frequency_text().to_string(),
        ui.get_band_text().to_string(),
        ui.get_rst_sent_text().to_string(),
        ui.get_rst_received_text().to_string(),
        ui.get_grid_text().to_string(),
        ui.get_name_text().to_string(),
        ui.get_qth_text().to_string(),
        ui.get_notes_text().to_string(),
        ui.get_dmr_remote_id_text().to_string(),
        ui.get_dmr_local_id_text().to_string(),
        ui.get_dmr_talkgroup_text().to_string(),
        ui.get_dmr_timeslot_text().to_string(),
        ui.get_dmr_color_code_text().to_string(),
        ui.get_dmr_network_text().to_string(),
        ui.get_dmr_call_type_text().to_string(),
        ui.get_dmr_access_type_text().to_string(),
        ui.get_dmr_repeater_text().to_string(),
        ui.get_dmr_hotspot_text().to_string(),
        ui.get_dmr_notes_text().to_string(),
        ui.get_ft8_snr_sent_text().to_string(),
        ui.get_ft8_snr_received_text().to_string(),
        ui.get_ft8_power_text().to_string(),
        ui.get_ft8_audio_frequency_text().to_string(),
        ui.get_ft8_source_software_text().to_string(),
        ui.get_ft8_protocol_text().to_string(),
        ui.get_ft8_final_message_text().to_string(),
    ])
}

pub(crate) fn connect_editor_navigation_handlers(
    ui: &MainWindow,
    baseline: &Rc<RefCell<EditorSnapshot>>,
) {
    let weak_ui = ui.as_weak();
    let opened_baseline = Rc::clone(baseline);
    ui.on_editor_opened(move || {
        if let Some(ui) = weak_ui.upgrade() {
            *opened_baseline.borrow_mut() = editor_snapshot(&ui);
            ui.set_discard_editor_visible(false);
        }
    });

    let weak_ui = ui.as_weak();
    let request_baseline = Rc::clone(baseline);
    ui.on_request_page(move |target| {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        if ui.get_active_page() == 1
            && has_unsaved_changes(&editor_snapshot(&ui), &request_baseline.borrow())
        {
            ui.set_pending_page(target);
            ui.set_discard_editor_visible(true);
            set_status(&ui, "Unsaved QSO changes need confirmation", STATUS_WARNING);
            return;
        }
        if target == 1 {
            if let Err(error) = clear_editor(&ui) {
                set_status(
                    &ui,
                    format!("Could not reset date/time: {error}"),
                    STATUS_ERROR,
                );
                return;
            }
            *request_baseline.borrow_mut() = editor_snapshot(&ui);
        }
        ui.set_discard_editor_visible(false);
        ui.set_active_page(target);
    });

    let weak_ui = ui.as_weak();
    ui.on_keep_editing(move || {
        if let Some(ui) = weak_ui.upgrade() {
            ui.set_discard_editor_visible(false);
            set_status(&ui, "Continuing QSO editing", STATUS_INFO);
        }
    });

    let weak_ui = ui.as_weak();
    let discard_baseline = Rc::clone(baseline);
    ui.on_confirm_discard_editor(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let target = ui.get_pending_page();
        match clear_editor(&ui) {
            Ok(()) => {
                *discard_baseline.borrow_mut() = editor_snapshot(&ui);
                ui.set_discard_editor_visible(false);
                ui.set_active_page(target);
                set_status(&ui, "Unsaved QSO changes discarded", STATUS_INFO);
            }
            Err(error) => set_status(
                &ui,
                format!("Could not reset date/time: {error}"),
                STATUS_ERROR,
            ),
        }
    });

    let weak_ui = ui.as_weak();
    let clear_baseline = Rc::clone(baseline);
    ui.on_clear_editor(move || {
        if let Some(ui) = weak_ui.upgrade() {
            match clear_editor(&ui) {
                Ok(()) => {
                    *clear_baseline.borrow_mut() = editor_snapshot(&ui);
                    ui.set_discard_editor_visible(false);
                }
                Err(error) => set_status(
                    &ui,
                    format!("Could not reset date/time: {error}"),
                    STATUS_ERROR,
                ),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unsaved_editor_changes_without_false_positives() {
        let baseline = EditorSnapshot(vec!["PY2ABC".into(), "FT8".into(), "-18".into()]);
        assert!(!has_unsaved_changes(&baseline, &baseline));
        let common_change = EditorSnapshot(vec!["PU2XYZ".into(), "FT8".into(), "-18".into()]);
        assert!(has_unsaved_changes(&common_change, &baseline));
        let specialized_change = EditorSnapshot(vec!["PY2ABC".into(), "FT8".into(), "-12".into()]);
        assert!(has_unsaved_changes(&specialized_change, &baseline));
    }

    #[test]
    fn detects_pending_work_before_exit() {
        let baseline = EditorSnapshot(vec!["PY2ABC".into()]);
        let changed = EditorSnapshot(vec!["PU2XYZ".into()]);
        assert!(!has_pending_exit_work(1, &baseline, &baseline, false));
        assert!(has_pending_exit_work(1, &changed, &baseline, false));
        assert!(!has_pending_exit_work(0, &changed, &baseline, false));
        assert!(has_pending_exit_work(2, &baseline, &baseline, true));
    }
}
