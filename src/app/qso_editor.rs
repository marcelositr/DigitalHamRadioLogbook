use super::*;

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
        _ => 0,
    }
}

pub(crate) fn connect_save_handler(ui: &MainWindow, repository: &Rc<QsoRepository>) {
    let weak_ui = ui.as_weak();
    let repository = Rc::clone(repository);
    ui.on_save_qso(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };

        let result = save_form(&ui, &repository);
        match result {
            Ok(message) => {
                set_status(&ui, message, STATUS_SUCCESS);
                ui.set_active_page(0);
            }
            Err(error) => set_status(&ui, format!("Could not save QSO: {error}"), STATUS_ERROR),
        }
    });
}

fn save_form(ui: &MainWindow, repository: &QsoRepository) -> Result<&'static str, Box<dyn Error>> {
    let now_utc = current_utc_timestamp()?;
    let datetime_start_utc = parse_utc_datetime(ui.get_datetime_text().as_str())?;
    let frequency_hz = parse_mhz_to_hz(ui.get_frequency_text().as_str())?;
    let qso = NewQso::new(
        ui.get_callsign_text().as_str(),
        datetime_start_utc,
        frequency_hz,
        ui.get_mode_text().as_str(),
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

    let id = ui.get_editing_id();
    let is_dmr = qso.mode == "DMR";
    if is_dmr {
        let metadata = DmrMetadata::from_input(DmrMetadataInput {
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
        })?;
        if id.is_empty() {
            repository.insert_dmr(&qso, &metadata, now_utc)?;
        } else if !repository.update_dmr(id.parse()?, &qso, &metadata, now_utc)? {
            return Err("QSO no longer exists".into());
        }
    } else if qso.mode == "FT8" {
        let metadata = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_sent_db: ui.get_ft8_snr_sent_text().to_string(),
            snr_received_db: ui.get_ft8_snr_received_text().to_string(),
            power_watts: ui.get_ft8_power_text().to_string(),
            audio_frequency_hz: ui.get_ft8_audio_frequency_text().to_string(),
            source_software: ui.get_ft8_source_software_text().to_string(),
            protocol: ui.get_ft8_protocol_text().to_string(),
            final_message: ui.get_ft8_final_message_text().to_string(),
        })?;
        if id.is_empty() {
            repository.insert_ft8(&qso, &metadata, now_utc)?;
        } else if !repository.update_ft8(id.parse()?, &qso, &metadata, now_utc)? {
            return Err("QSO no longer exists".into());
        }
    } else if id.is_empty() {
        repository.insert(&qso, now_utc)?;
    } else if !repository.update(id.parse()?, &qso, now_utc)? {
        return Err("QSO no longer exists".into());
    }

    refresh_qso_list(ui, repository, ui.get_search_text().as_str())?;
    clear_editor(ui)?;
    Ok(if id.is_empty() {
        "QSO saved"
    } else {
        "QSO updated"
    })
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_specialized_mode_for_the_editor() {
        for value in ["DMR", "dmr", "Dmr", " DMR "] {
            assert_eq!(mode_kind(value), 1);
        }
        for value in ["FT8", "ft8", "Ft8", " FT8 "] {
            assert_eq!(mode_kind(value), 2);
        }
        for value in ["M17", "", "FT-8"] {
            assert_eq!(mode_kind(value), 0);
        }
    }
}
