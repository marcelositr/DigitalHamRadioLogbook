use super::*;
use digital_ham_radio_logbook::domain::ModeMetadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveIntent {
    Save,
    SaveAndNew,
}

#[derive(Clone)]
struct PreparedQso {
    qso: NewQso,
    metadata: ModeMetadata,
    editing_id: Option<i64>,
}

#[derive(Clone)]
struct PendingDuplicate {
    prepared: PreparedQso,
    snapshot: EditorSnapshot,
    intent: SaveIntent,
}

#[derive(Default)]
struct SaveState {
    pending_duplicate: Option<PendingDuplicate>,
    committed_snapshot: Option<EditorSnapshot>,
}

pub(crate) fn connect_mode_handler(ui: &MainWindow) {
    let weak_ui = ui.as_weak();
    ui.on_mode_input_changed(move |mode| {
        if let Some(ui) = weak_ui.upgrade() {
            ui.set_mode_kind(mode_kind(mode.as_str()));
        }
    });
}

fn mode_kind(mode: &str) -> i32 {
    match mode.trim().to_ascii_uppercase().as_str() {
        "DMR" => 1,
        "FT8" => 2,
        "DSTAR" | "D-STAR" => 3,
        "YSF" | "C4FM" | "SYSTEM FUSION" => 4,
        _ => 0,
    }
}

fn canonical_mode(mode: &str) -> String {
    match mode.trim().to_ascii_uppercase().as_str() {
        "YSF" | "C4FM" | "SYSTEM FUSION" => "C4FM".to_owned(),
        "D-STAR" => "DSTAR".to_owned(),
        _ => mode.trim().to_ascii_uppercase(),
    }
}

pub(crate) fn connect_save_handler(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    state: &SharedLogbookViewState,
    baseline: &Rc<RefCell<EditorSnapshot>>,
) {
    let saving = Rc::new(Cell::new(false));
    let save_state = Rc::new(RefCell::new(SaveState::default()));

    connect_save_action(
        ui,
        repository,
        state,
        baseline,
        &saving,
        &save_state,
        SaveIntent::Save,
    );
    connect_save_action(
        ui,
        repository,
        state,
        baseline,
        &saving,
        &save_state,
        SaveIntent::SaveAndNew,
    );

    let weak_ui = ui.as_weak();
    let save_state_review = Rc::clone(&save_state);
    ui.on_review_duplicate(move || {
        if let Some(ui) = weak_ui.upgrade() {
            save_state_review.borrow_mut().pending_duplicate = None;
            ui.set_duplicate_warning_visible(false);
            set_status(
                &ui,
                "Duplicate not saved; review the unchanged form",
                STATUS_INFO,
            );
        }
    });

    let weak_ui = ui.as_weak();
    let repository = Rc::clone(repository);
    let view_state = Rc::clone(state);
    let baseline = Rc::clone(baseline);
    let saving_anyway = Rc::clone(&saving);
    let save_state_anyway = Rc::clone(&save_state);
    ui.on_save_duplicate_anyway(move || {
        if !begin_action(&saving_anyway) {
            return;
        }
        let Some(ui) = weak_ui.upgrade() else {
            saving_anyway.set(false);
            return;
        };
        let pending = save_state_anyway.borrow_mut().pending_duplicate.take();
        ui.set_duplicate_warning_visible(false);
        if let Some(pending) = pending {
            let current = editor_snapshot(&ui);
            if current == pending.snapshot {
                finish_save(
                    &ui,
                    &repository,
                    &view_state,
                    &baseline,
                    &save_state_anyway,
                    pending.prepared,
                    pending.intent,
                );
            } else {
                set_status(
                    &ui,
                    "Form changed; validating and checking duplicates again",
                    STATUS_INFO,
                );
                process_save_request(
                    &ui,
                    &repository,
                    &view_state,
                    &baseline,
                    &save_state_anyway,
                    pending.intent,
                );
            }
        }
        saving_anyway.set(false);
    });
}

#[allow(clippy::too_many_arguments)]
fn connect_save_action(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    state: &SharedLogbookViewState,
    baseline: &Rc<RefCell<EditorSnapshot>>,
    saving: &Rc<Cell<bool>>,
    save_state: &Rc<RefCell<SaveState>>,
    intent: SaveIntent,
) {
    let weak_ui = ui.as_weak();
    let repository = Rc::clone(repository);
    let view_state = Rc::clone(state);
    let baseline = Rc::clone(baseline);
    let saving = Rc::clone(saving);
    let save_state = Rc::clone(save_state);
    let callback = move || {
        if !begin_action(&saving) {
            return;
        }
        if let Some(ui) = weak_ui.upgrade() {
            process_save_request(
                &ui,
                &repository,
                &view_state,
                &baseline,
                &save_state,
                intent,
            );
        }
        saving.set(false);
    };
    match intent {
        SaveIntent::Save => ui.on_save_qso(callback),
        SaveIntent::SaveAndNew => ui.on_save_and_new_qso(callback),
    }
}

fn begin_action(saving: &Cell<bool>) -> bool {
    !saving.replace(true)
}

fn process_save_request(
    ui: &MainWindow,
    repository: &QsoRepository,
    view_state: &SharedLogbookViewState,
    baseline: &Rc<RefCell<EditorSnapshot>>,
    save_state: &Rc<RefCell<SaveState>>,
    intent: SaveIntent,
) {
    if intent == SaveIntent::SaveAndNew && !ui.get_editing_id().is_empty() {
        set_status(
            ui,
            "Save & New is available only for a new QSO",
            STATUS_WARNING,
        );
        return;
    }
    let snapshot = editor_snapshot(ui);
    if save_state.borrow().committed_snapshot.as_ref() == Some(&snapshot) {
        set_status(
            ui,
            "This unchanged form was already committed; no second write was made",
            STATUS_WARNING,
        );
        return;
    }
    let prepared = match prepare_form(ui) {
        Ok(prepared) => prepared,
        Err(error) => {
            set_status(ui, format!("Could not save QSO: {error}"), STATUS_ERROR);
            return;
        }
    };
    match repository.find_qso_identity_match(&prepared.qso, prepared.editing_id) {
        Ok(Some(id)) => {
            save_state.borrow_mut().pending_duplicate = Some(PendingDuplicate {
                prepared,
                snapshot,
                intent,
            });
            ui.set_duplicate_warning_text(
                format!("An exact QSO identity already exists (record #{id}). Review the form or save anyway.").into(),
            );
            ui.set_duplicate_warning_visible(true);
            set_status(
                ui,
                "Possible duplicate found; nothing has been written",
                STATUS_WARNING,
            );
        }
        Ok(None) => finish_save(
            ui, repository, view_state, baseline, save_state, prepared, intent,
        ),
        Err(error) => set_status(
            ui,
            format!("Could not check duplicates: {error}"),
            STATUS_ERROR,
        ),
    }
}

fn prepare_form(ui: &MainWindow) -> Result<PreparedQso, Box<dyn Error>> {
    let qso = NewQso::new(
        ui.get_callsign_text().as_str(),
        parse_utc_datetime(ui.get_datetime_text().as_str())?,
        parse_mhz_to_hz(ui.get_frequency_text().as_str())?,
        canonical_mode(ui.get_mode_text().as_str()),
    )?
    .with_common_fields(CommonQsoFields {
        band_override: ui.get_band_text().to_string(),
        rst_sent: ui.get_rst_sent_text().to_string(),
        rst_received: ui.get_rst_received_text().to_string(),
        grid_locator: ui.get_grid_text().to_string(),
        name: ui.get_name_text().to_string(),
        qth: ui.get_qth_text().to_string(),
        notes: ui.get_notes_text().to_string(),
    })?;
    let metadata = match qso.mode.as_str() {
        "DMR" => ModeMetadata::Dmr(DmrMetadata::from_input(DmrMetadataInput {
            remote_dmr_id: ui.get_dmr_remote_id_text().to_string(),
            local_dmr_id: ui.get_dmr_local_id_text().to_string(),
            talkgroup: ui.get_dmr_talkgroup_text().to_string(),
            timeslot: ui.get_dmr_timeslot_text().to_string(),
            color_code: ui.get_dmr_color_code_text().to_string(),
            network: ui.get_dmr_network_text().to_string(),
            call_type: ui.get_dmr_call_type_text().to_string(),
            access_type: ui.get_dmr_access_type_text().to_string(),
            repeater_callsign: ui.get_dmr_repeater_text().to_string(),
            hotspot: ui.get_dmr_hotspot_text().to_string(),
            notes: ui.get_dmr_notes_text().to_string(),
            ..Default::default()
        })?),
        "DSTAR" => ModeMetadata::Dstar(DStarMetadata::from_input(DStarMetadataInput {
            reflector: ui.get_dstar_reflector_text().to_string(),
            module: ui.get_dstar_module_text().to_string(),
            mycall: ui.get_dstar_mycall_text().to_string(),
            urcall: ui.get_dstar_urcall_text().to_string(),
            rpt1: ui.get_dstar_rpt1_text().to_string(),
            rpt2: ui.get_dstar_rpt2_text().to_string(),
            notes: ui.get_dstar_notes_text().to_string(),
        })?),
        "C4FM" => ModeMetadata::Ysf(YsfMetadata::from_input(YsfMetadataInput {
            room: ui.get_ysf_room_text().to_string(),
            wires_x_node: ui.get_ysf_wires_x_node_text().to_string(),
            repeater: ui.get_ysf_repeater_text().to_string(),
            network: ui.get_ysf_network_text().to_string(),
            access_type: ui.get_ysf_access_type_text().to_string(),
            tx_dg_id: ui.get_ysf_tx_dg_id_text().to_string(),
            rx_dg_id: ui.get_ysf_rx_dg_id_text().to_string(),
            notes: ui.get_ysf_notes_text().to_string(),
        })?),
        "FT8" => ModeMetadata::Ft8(Ft8Metadata::from_input(Ft8MetadataInput {
            snr_sent_db: ui.get_ft8_snr_sent_text().to_string(),
            snr_received_db: ui.get_ft8_snr_received_text().to_string(),
            power_watts: ui.get_ft8_power_text().to_string(),
            audio_frequency_hz: ui.get_ft8_audio_frequency_text().to_string(),
            source_software: ui.get_ft8_source_software_text().to_string(),
            protocol: ui.get_ft8_protocol_text().to_string(),
            final_message: ui.get_ft8_final_message_text().to_string(),
        })?),
        _ => ModeMetadata::Generic,
    };
    let editing_id = match ui.get_editing_id().as_str() {
        "" => None,
        value => Some(value.parse()?),
    };
    Ok(PreparedQso {
        qso,
        metadata,
        editing_id,
    })
}

fn persist_prepared(
    repository: &QsoRepository,
    prepared: &PreparedQso,
    now_utc: i64,
) -> Result<&'static str, Box<dyn Error>> {
    let changed = match (prepared.editing_id, &prepared.metadata) {
        (None, ModeMetadata::Dmr(m)) => {
            repository.insert_dmr(&prepared.qso, m, now_utc)?;
            true
        }
        (None, ModeMetadata::Dstar(m)) => {
            repository.insert_dstar(&prepared.qso, m, now_utc)?;
            true
        }
        (None, ModeMetadata::Ysf(m)) => {
            repository.insert_ysf(&prepared.qso, m, now_utc)?;
            true
        }
        (None, ModeMetadata::Ft8(m)) => {
            repository.insert_ft8(&prepared.qso, m, now_utc)?;
            true
        }
        (None, ModeMetadata::Generic) => {
            repository.insert(&prepared.qso, now_utc)?;
            true
        }
        (Some(id), ModeMetadata::Dmr(m)) => repository.update_dmr(id, &prepared.qso, m, now_utc)?,
        (Some(id), ModeMetadata::Dstar(m)) => {
            repository.update_dstar(id, &prepared.qso, m, now_utc)?
        }
        (Some(id), ModeMetadata::Ysf(m)) => repository.update_ysf(id, &prepared.qso, m, now_utc)?,
        (Some(id), ModeMetadata::Ft8(m)) => repository.update_ft8(id, &prepared.qso, m, now_utc)?,
        (Some(id), ModeMetadata::Generic) => repository.update(id, &prepared.qso, now_utc)?,
    };
    if !changed {
        return Err("QSO no longer exists".into());
    }
    Ok(if prepared.editing_id.is_none() {
        "QSO saved"
    } else {
        "QSO updated"
    })
}

fn finish_save(
    ui: &MainWindow,
    repository: &QsoRepository,
    view_state: &SharedLogbookViewState,
    baseline: &Rc<RefCell<EditorSnapshot>>,
    save_state: &Rc<RefCell<SaveState>>,
    prepared: PreparedQso,
    intent: SaveIntent,
) {
    let committed_snapshot = editor_snapshot(ui);
    let now_utc = match current_utc_timestamp() {
        Ok(value) => value,
        Err(error) => {
            set_status(ui, format!("Could not save QSO: {error}"), STATUS_ERROR);
            return;
        }
    };
    let message = match persist_prepared(repository, &prepared, now_utc) {
        Ok(message) => message,
        Err(error) => {
            set_status(ui, format!("Could not save QSO: {error}"), STATUS_ERROR);
            return;
        }
    };
    save_state.borrow_mut().committed_snapshot = Some(committed_snapshot);
    if let Err(error) = refresh_qso_list(ui, repository, view_state) {
        logging::error(&format!(
            "QSO committed but logbook refresh failed: {error}"
        ));
        set_status(ui, format!("{message}, but refresh failed. Data is safe; this unchanged form cannot be committed again."), STATUS_WARNING);
        return;
    }
    if let Err(error) = clear_editor(ui) {
        logging::error(&format!("QSO committed but editor reset failed: {error}"));
        set_status(ui, format!("{message}, but editor reset failed. Data is safe; this unchanged form cannot be committed again."), STATUS_WARNING);
        return;
    }
    *baseline.borrow_mut() = editor_snapshot(ui);
    save_state.borrow_mut().committed_snapshot = None;
    set_status(ui, message, STATUS_SUCCESS);
    if intent == SaveIntent::SaveAndNew {
        ui.set_active_page(1);
        ui.invoke_focus_callsign();
    } else {
        ui.set_active_page(0);
    }
}

pub(crate) fn clear_editor(ui: &MainWindow) -> Result<(), Box<dyn Error>> {
    ui.set_editing_id("".into());
    ui.set_callsign_text("".into());
    ui.set_datetime_text(format_utc_datetime(current_utc_timestamp()?)?.into());
    ui.set_mode_text("".into());
    ui.set_mode_kind(0);
    ui.set_frequency_text("".into());
    ui.set_band_text("".into());
    ui.set_rst_sent_text("".into());
    ui.set_rst_received_text("".into());
    ui.set_grid_text("".into());
    ui.set_name_text("".into());
    ui.set_qth_text("".into());
    ui.set_notes_text("".into());
    ui.set_dmr_remote_id_text("".into());
    ui.set_dmr_local_id_text("".into());
    ui.set_dmr_talkgroup_text("".into());
    ui.set_dmr_timeslot_text("".into());
    ui.set_dmr_color_code_text("".into());
    ui.set_dmr_network_text("".into());
    ui.set_dmr_call_type_text("group".into());
    ui.set_dmr_access_type_text("simplex".into());
    ui.set_dmr_repeater_text("".into());
    ui.set_dmr_hotspot_text("".into());
    ui.set_dmr_notes_text("".into());
    ui.set_ft8_snr_sent_text("".into());
    ui.set_ft8_snr_received_text("".into());
    ui.set_ft8_power_text("".into());
    ui.set_ft8_audio_frequency_text("".into());
    ui.set_ft8_source_software_text("".into());
    ui.set_ft8_protocol_text("FT8".into());
    ui.set_ft8_final_message_text("".into());
    ui.set_dstar_reflector_text("".into());
    ui.set_dstar_module_text("".into());
    ui.set_dstar_mycall_text("".into());
    ui.set_dstar_urcall_text("".into());
    ui.set_dstar_rpt1_text("".into());
    ui.set_dstar_rpt2_text("".into());
    ui.set_dstar_notes_text("".into());
    ui.set_ysf_room_text("".into());
    ui.set_ysf_wires_x_node_text("".into());
    ui.set_ysf_repeater_text("".into());
    ui.set_ysf_network_text("".into());
    ui.set_ysf_access_type_text("simplex".into());
    ui.set_ysf_tx_dg_id_text("".into());
    ui.set_ysf_rx_dg_id_text("".into());
    ui.set_ysf_notes_text("".into());
    ui.set_duplicate_warning_visible(false);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    static NEXT_DATABASE: AtomicU64 = AtomicU64::new(0);
    const FIRST_UTC: i64 = 1_700_000_000;
    const SECOND_UTC: i64 = 1_700_000_060;

    struct OperationalFixture {
        ui: MainWindow,
        repository: Rc<QsoRepository>,
        path: PathBuf,
        _lock: MutexGuard<'static, ()>,
    }

    impl OperationalFixture {
        fn new() -> Self {
            let lock = TEST_LOCK
                .get_or_init(|| Mutex::new(()))
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let path = std::env::temp_dir().join(format!(
                "digital-ham-radio-logbook-qso-editor-{}-{}.sqlite3",
                std::process::id(),
                NEXT_DATABASE.fetch_add(1, Ordering::Relaxed)
            ));
            let repository = Rc::new(QsoRepository::open(&path).unwrap());
            let ui = MainWindow::new().unwrap();
            let state = Rc::new(RefCell::new(qso_list::LogbookViewState::default()));
            refresh_qso_list(&ui, &repository, &state).unwrap();
            let baseline = Rc::new(RefCell::new(editor_snapshot(&ui)));
            connect_save_handler(&ui, &repository, &state, &baseline);
            Self {
                ui,
                repository,
                path,
                _lock: lock,
            }
        }

        fn fill(&self, callsign: &str, utc: i64, frequency: &str) {
            self.ui.set_callsign_text(callsign.into());
            self.ui
                .set_datetime_text(format_utc_datetime(utc).unwrap().into());
            self.ui.set_frequency_text(frequency.into());
            self.ui.set_mode_text("FM".into());
            self.ui.set_notes_text("operational test".into());
        }

        fn seed(&self, callsign: &str, utc: i64, frequency_hz: i64) -> i64 {
            self.repository
                .insert(
                    &NewQso::new(callsign, utc, frequency_hz, "FM").unwrap(),
                    utc,
                )
                .unwrap()
        }

        fn persisted(&self) -> Vec<digital_ham_radio_logbook::domain::Qso> {
            self.repository.list().unwrap()
        }
    }

    impl Drop for OperationalFixture {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn assert_editor_cleared_with_new_utc(ui: &MainWindow) {
        assert!(ui.get_editing_id().is_empty());
        assert!(ui.get_callsign_text().is_empty());
        assert!(ui.get_frequency_text().is_empty());
        assert!(ui.get_mode_text().is_empty());
        assert!(!ui.get_datetime_text().is_empty());
    }

    fn new_generic_save_persists_once_and_updates_editor_and_list() {
        let fixture = OperationalFixture::new();
        fixture.fill("PY2ABC", FIRST_UTC, "145.500");

        fixture.ui.invoke_save_qso();

        let persisted = fixture.persisted();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].callsign, "PY2ABC");
        assert_eq!(fixture.ui.get_qsos().row_count(), 1);
        assert_eq!(fixture.ui.get_active_page(), 0);
        assert_editor_cleared_with_new_utc(&fixture.ui);
    }

    fn save_and_new_persists_first_and_leaves_fresh_unpersisted_editor() {
        let fixture = OperationalFixture::new();
        fixture.fill("PY2NEW", FIRST_UTC, "145.500");

        fixture.ui.invoke_save_and_new_qso();

        assert_eq!(fixture.persisted().len(), 1);
        assert_eq!(fixture.ui.get_qsos().row_count(), 1);
        assert_eq!(fixture.ui.get_active_page(), 1);
        assert_editor_cleared_with_new_utc(&fixture.ui);
    }

    fn invalid_frequency_or_callsign_save_and_new_preserves_form_and_inserts_nothing() {
        let fixture = OperationalFixture::new();
        fixture.fill("PY2BAD", FIRST_UTC, "not-a-frequency");
        let invalid_frequency = editor_snapshot(&fixture.ui);
        fixture.ui.invoke_save_and_new_qso();
        assert!(fixture.persisted().is_empty());
        assert_eq!(editor_snapshot(&fixture.ui), invalid_frequency);

        fixture.ui.set_frequency_text("145.500".into());
        fixture.ui.set_callsign_text("".into());
        let invalid_callsign = editor_snapshot(&fixture.ui);
        fixture.ui.invoke_save_and_new_qso();
        assert!(fixture.persisted().is_empty());
        assert_eq!(editor_snapshot(&fixture.ui), invalid_callsign);
    }

    fn exact_duplicate_save_then_review_writes_nothing_and_preserves_form() {
        let fixture = OperationalFixture::new();
        fixture.seed("PY2DUP", FIRST_UTC, 145_500_000);
        fixture.fill("py2dup", FIRST_UTC, "145.500");
        let form = editor_snapshot(&fixture.ui);

        fixture.ui.invoke_save_qso();
        assert!(fixture.ui.get_duplicate_warning_visible());
        assert_eq!(fixture.persisted().len(), 1);
        fixture.ui.invoke_review_duplicate();

        assert!(!fixture.ui.get_duplicate_warning_visible());
        assert_eq!(fixture.persisted().len(), 1);
        assert_eq!(editor_snapshot(&fixture.ui), form);
    }

    fn exact_duplicate_save_and_new_then_save_anyway_persists_and_clears() {
        let fixture = OperationalFixture::new();
        fixture.seed("PY2DUP", FIRST_UTC, 145_500_000);
        fixture.fill("PY2DUP", FIRST_UTC, "145.500");

        fixture.ui.invoke_save_and_new_qso();
        assert!(fixture.ui.get_duplicate_warning_visible());
        assert_eq!(fixture.persisted().len(), 1);
        fixture.ui.invoke_save_duplicate_anyway();

        assert_eq!(fixture.persisted().len(), 2);
        assert_eq!(fixture.ui.get_active_page(), 1);
        assert_editor_cleared_with_new_utc(&fixture.ui);
    }

    fn editing_same_identity_excludes_self_and_updates_without_warning() {
        let fixture = OperationalFixture::new();
        let id = fixture.seed("PY2EDIT", FIRST_UTC, 145_500_000);
        fixture.fill("PY2EDIT", FIRST_UTC, "145.500");
        fixture.ui.set_editing_id(id.to_string().into());
        fixture.ui.set_notes_text("updated".into());

        fixture.ui.invoke_save_qso();

        let persisted = fixture.persisted();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].notes, "updated");
        assert!(!fixture.ui.get_duplicate_warning_visible());
        assert_editor_cleared_with_new_utc(&fixture.ui);
    }

    fn editing_to_another_qso_identity_shows_warning_without_update() {
        let fixture = OperationalFixture::new();
        let editing_id = fixture.seed("PY2ONE", FIRST_UTC, 145_500_000);
        fixture.seed("PY2TWO", SECOND_UTC, 433_500_000);
        fixture.fill("PY2TWO", SECOND_UTC, "433.500");
        fixture.ui.set_editing_id(editing_id.to_string().into());
        let form = editor_snapshot(&fixture.ui);

        fixture.ui.invoke_save_qso();

        assert!(fixture.ui.get_duplicate_warning_visible());
        assert_eq!(fixture.persisted().len(), 2);
        assert_eq!(editor_snapshot(&fixture.ui), form);
    }

    #[test]
    fn operational_save_flows_use_real_callbacks_and_file_repository() {
        new_generic_save_persists_once_and_updates_editor_and_list();
        save_and_new_persists_first_and_leaves_fresh_unpersisted_editor();
        invalid_frequency_or_callsign_save_and_new_preserves_form_and_inserts_nothing();
        exact_duplicate_save_then_review_writes_nothing_and_preserves_form();
        exact_duplicate_save_and_new_then_save_anyway_persists_and_clears();
        editing_same_identity_excludes_self_and_updates_without_warning();
        editing_to_another_qso_identity_shows_warning_without_update();
    }

    #[test]
    fn normalizes_specialized_mode_for_the_editor() {
        assert_eq!(mode_kind(" dmr "), 1);
        assert_eq!(mode_kind("D-STAR"), 3);
        assert_eq!(canonical_mode("ysf"), "C4FM");
        assert_eq!(canonical_mode("d-star"), "DSTAR");
    }
    #[test]
    fn double_action_guard_accepts_only_first_action() {
        let saving = Cell::new(false);
        assert!(begin_action(&saving));
        assert!(!begin_action(&saving));
        saving.set(false);
        assert!(begin_action(&saving));
    }
    #[test]
    fn pending_duplicate_preserves_intent_and_snapshot() {
        let pending = PendingDuplicate {
            prepared: PreparedQso {
                qso: NewQso::new("PY2ABC", 1, 145_000_000, "FM").unwrap(),
                metadata: ModeMetadata::Generic,
                editing_id: None,
            },
            snapshot: EditorSnapshot(vec!["form".into()]),
            intent: SaveIntent::SaveAndNew,
        };
        assert_eq!(pending.intent, SaveIntent::SaveAndNew);
        assert_eq!(pending.snapshot, EditorSnapshot(vec!["form".into()]));
    }
}
