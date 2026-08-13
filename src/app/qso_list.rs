use super::*;

pub(crate) fn connect_search_handler(ui: &MainWindow, repository: &Rc<QsoRepository>) {
    let weak_ui = ui.as_weak();
    let repository = Rc::clone(repository);
    ui.on_search_qso(move |query| {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };

        match refresh_qso_list(&ui, &repository, query.as_str()) {
            Ok(()) => set_status(&ui, "Search completed", STATUS_INFO),
            Err(error) => set_status(&ui, format!("Could not search QSOs: {error}"), STATUS_ERROR),
        }
    });
}

pub(crate) fn connect_delete_handler(ui: &MainWindow, repository: &Rc<QsoRepository>) {
    let weak_ui = ui.as_weak();
    let repository = Rc::clone(repository);
    ui.on_delete_qso(move |id| {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };

        let result = (|| -> Result<(), Box<dyn Error>> {
            let id = id.parse::<i64>()?;
            if !repository.delete(id)? {
                return Err("QSO no longer exists".into());
            }
            refresh_qso_list(&ui, &repository, ui.get_search_text().as_str())?;
            if ui.get_editing_id() == id.to_string() {
                clear_editor(&ui)?;
            }
            Ok(())
        })();

        match result {
            Ok(()) => set_status(&ui, "QSO deleted", STATUS_SUCCESS),
            Err(error) => set_status(&ui, format!("Could not delete QSO: {error}"), STATUS_ERROR),
        }
    });
}
pub(crate) fn refresh_qso_list(
    ui: &MainWindow,
    repository: &QsoRepository,
    query: &str,
) -> Result<(), Box<dyn Error>> {
    refresh_rows(ui, repository.search(query)?, repository)
}

pub(crate) fn refresh_rows(
    ui: &MainWindow,
    qsos: Vec<digital_ham_radio_logbook::domain::Qso>,
    repository: &QsoRepository,
) -> Result<(), Box<dyn Error>> {
    let rows = qsos
        .into_iter()
        .map(|qso| {
            let datetime = format_utc_datetime(qso.datetime_start_utc)?;
            let dmr = repository.get_dmr_metadata(qso.id)?;
            let ft8 = repository.get_ft8_metadata(qso.id)?;
            let route_summary = dmr
                .as_ref()
                .map(format_dmr_route)
                .or_else(|| ft8.as_ref().map(format_ft8_summary))
                .unwrap_or_default();
            Ok(QsoRow {
                id: SharedString::from(qso.id.to_string()),
                callsign: SharedString::from(qso.callsign),
                timestamp: SharedString::from(datetime.clone()),
                datetime_input: SharedString::from(datetime),
                frequency: SharedString::from(format_frequency(qso.frequency_hz)),
                frequency_input: SharedString::from(format_frequency_input(qso.frequency_hz)),
                band: SharedString::from(qso.band.unwrap_or_default()),
                mode: SharedString::from(qso.mode),
                rst_sent: SharedString::from(qso.rst_sent.unwrap_or_default()),
                rst_received: SharedString::from(qso.rst_received.unwrap_or_default()),
                grid: SharedString::from(qso.grid_locator.unwrap_or_default()),
                name: SharedString::from(qso.name.unwrap_or_default()),
                qth: SharedString::from(qso.qth.unwrap_or_default()),
                notes: SharedString::from(qso.notes),
                route_summary: route_summary.into(),
                dmr_remote_id: optional_number(dmr.as_ref().and_then(|value| value.remote_dmr_id))
                    .into(),
                dmr_local_id: optional_number(dmr.as_ref().and_then(|value| value.local_dmr_id))
                    .into(),
                dmr_talkgroup: optional_number(dmr.as_ref().and_then(|value| value.talkgroup))
                    .into(),
                dmr_timeslot: optional_number(dmr.as_ref().and_then(|value| value.timeslot)).into(),
                dmr_color_code: optional_number(dmr.as_ref().and_then(|value| value.color_code))
                    .into(),
                dmr_network: dmr
                    .as_ref()
                    .and_then(|value| value.network.clone())
                    .unwrap_or_default()
                    .into(),
                dmr_call_type: dmr
                    .as_ref()
                    .map(|value| value.call_type.as_str())
                    .unwrap_or("")
                    .into(),
                dmr_access_type: dmr
                    .as_ref()
                    .map(|value| value.access_type.as_str())
                    .unwrap_or("")
                    .into(),
                dmr_repeater: dmr
                    .as_ref()
                    .and_then(|value| value.repeater_callsign.clone())
                    .unwrap_or_default()
                    .into(),
                dmr_hotspot: dmr
                    .as_ref()
                    .and_then(|value| value.hotspot.clone())
                    .unwrap_or_default()
                    .into(),
                dmr_notes: dmr
                    .as_ref()
                    .map(|value| value.notes.clone())
                    .unwrap_or_default()
                    .into(),
                ft8_snr_sent: optional_number(ft8.as_ref().and_then(|value| value.snr_sent_db))
                    .into(),
                ft8_snr_received: optional_number(
                    ft8.as_ref().and_then(|value| value.snr_received_db),
                )
                .into(),
                ft8_power: optional_number(ft8.as_ref().and_then(|value| value.power_watts)).into(),
                ft8_audio_frequency: optional_number(
                    ft8.as_ref().and_then(|value| value.audio_frequency_hz),
                )
                .into(),
                ft8_source_software: ft8
                    .as_ref()
                    .and_then(|value| value.source_software.clone())
                    .unwrap_or_default()
                    .into(),
                ft8_protocol: ft8
                    .as_ref()
                    .and_then(|value| value.protocol.clone())
                    .unwrap_or_default()
                    .into(),
                ft8_final_message: ft8
                    .as_ref()
                    .and_then(|value| value.final_message.clone())
                    .unwrap_or_default()
                    .into(),
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    ui.set_qsos(ModelRc::new(VecModel::from(rows)));
    Ok(())
}

fn optional_number<T: ToString>(value: Option<T>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn format_dmr_route(metadata: &DmrMetadata) -> String {
    let access = metadata
        .repeater_callsign
        .as_deref()
        .or(metadata.hotspot.as_deref())
        .unwrap_or(metadata.access_type.as_str());
    let mut parts = vec![access.to_owned()];
    if let Some(network) = &metadata.network {
        parts.push(network.clone());
    }
    if let Some(talkgroup) = metadata.talkgroup {
        parts.push(format!("TG {talkgroup}"));
    }
    parts.join(" → ")
}

fn format_ft8_summary(metadata: &Ft8Metadata) -> String {
    let mut parts = Vec::new();
    if let Some(snr) = metadata.snr_received_db {
        parts.push(format!("SNR {snr} dB"));
    }
    if let Some(power) = metadata.power_watts {
        parts.push(format!("{power} W"));
    }
    if let Some(message) = &metadata.final_message {
        parts.push(message.clone());
    }
    parts.join(" · ")
}
