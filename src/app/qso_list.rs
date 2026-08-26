use super::*;
use digital_ham_radio_logbook::database::{QsoListItem, QsoPage, DEFAULT_PAGE_SIZE};
use digital_ham_radio_logbook::domain::ModeMetadata;

#[derive(Debug, Clone)]
pub(crate) enum LogbookQuery {
    General(String),
    Dmr(DmrFilter),
    Ft8(Ft8Filter),
    Dstar(DstarFilter),
    Ysf(YsfFilter),
}

#[derive(Debug, Clone)]
pub(crate) struct LogbookViewState {
    pub(crate) query: LogbookQuery,
    pub(crate) offset: usize,
}

impl Default for LogbookViewState {
    fn default() -> Self {
        Self {
            query: LogbookQuery::General(String::new()),
            offset: 0,
        }
    }
}

pub(crate) type SharedLogbookViewState = Rc<RefCell<LogbookViewState>>;

pub(crate) fn connect_search_handler(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    state: &SharedLogbookViewState,
) {
    let weak_ui = ui.as_weak();
    let repository = Rc::clone(repository);
    let state = Rc::clone(state);
    ui.on_search_qso(move |query| {
        let Some(ui) = weak_ui.upgrade() else { return };
        let previous_state = state.borrow().clone();
        {
            let mut state = state.borrow_mut();
            state.query = LogbookQuery::General(query.to_string());
            state.offset = 0;
        }
        match refresh_qso_list(&ui, &repository, &state) {
            Ok(()) => set_status(&ui, "Search completed", STATUS_INFO),
            Err(error) => {
                *state.borrow_mut() = previous_state;
                set_status(&ui, format!("Could not search QSOs: {error}"), STATUS_ERROR);
            }
        }
    });
}

pub(crate) fn connect_pagination_handlers(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    state: &SharedLogbookViewState,
) {
    let weak_ui = ui.as_weak();
    let repository_previous = Rc::clone(repository);
    let previous_state = Rc::clone(state);
    ui.on_previous_logbook_page(move || {
        let Some(ui) = weak_ui.upgrade() else { return };
        let previous_offset = {
            let mut state = previous_state.borrow_mut();
            let previous_offset = state.offset;
            state.offset = state.offset.saturating_sub(DEFAULT_PAGE_SIZE);
            previous_offset
        };
        if let Err(error) = refresh_qso_list(&ui, &repository_previous, &previous_state) {
            previous_state.borrow_mut().offset = previous_offset;
            set_status(
                &ui,
                format!("Could not load previous page: {error}"),
                STATUS_ERROR,
            );
        }
    });

    let weak_ui = ui.as_weak();
    let repository_next = Rc::clone(repository);
    let next_state = Rc::clone(state);
    ui.on_next_logbook_page(move || {
        let Some(ui) = weak_ui.upgrade() else { return };
        let previous_offset = {
            let mut state = next_state.borrow_mut();
            let previous_offset = state.offset;
            state.offset = state.offset.saturating_add(DEFAULT_PAGE_SIZE);
            previous_offset
        };
        if let Err(error) = refresh_qso_list(&ui, &repository_next, &next_state) {
            next_state.borrow_mut().offset = previous_offset;
            set_status(
                &ui,
                format!("Could not load next page: {error}"),
                STATUS_ERROR,
            );
        }
    });
}

pub(crate) fn connect_delete_handler(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    state: &SharedLogbookViewState,
) {
    let weak_ui = ui.as_weak();
    let repository = Rc::clone(repository);
    let state = Rc::clone(state);
    ui.on_delete_qso(move |id| {
        let Some(ui) = weak_ui.upgrade() else { return };
        let id = match id.parse::<i64>() {
            Ok(id) => id,
            Err(error) => {
                set_status(&ui, format!("Could not delete QSO: {error}"), STATUS_ERROR);
                return;
            }
        };
        match repository.delete(id) {
            Ok(true) => {
                let refresh_result = refresh_qso_list(&ui, &repository, &state);
                let clear_result = if ui.get_editing_id() == id.to_string() {
                    clear_editor(&ui)
                } else {
                    Ok(())
                };
                match (refresh_result, clear_result) {
                    (Ok(()), Ok(())) => set_status(&ui, "QSO deleted", STATUS_SUCCESS),
                    (refresh, clear) => {
                        let details = refresh
                            .err()
                            .map(|error| format!("Logbook refresh failed: {error}"))
                            .or_else(|| {
                                clear
                                    .err()
                                    .map(|error| format!("editor reset failed: {error}"))
                            })
                            .unwrap_or_else(|| "presentation update failed".to_owned());
                        logging::error(&format!(
                            "QSO deleted but presentation update failed: {details}"
                        ));
                        set_status(
                            &ui,
                            "QSO deleted, but the display could not be refreshed. The database remains consistent.",
                            STATUS_WARNING,
                        );
                    }
                }
            }
            Ok(false) => set_status(&ui, "Could not delete QSO: QSO no longer exists", STATUS_ERROR),
            Err(error) => set_status(&ui, format!("Could not delete QSO: {error}"), STATUS_ERROR),
        }
    });
}

pub(crate) fn refresh_qso_list(
    ui: &MainWindow,
    repository: &QsoRepository,
    state: &SharedLogbookViewState,
) -> Result<(), Box<dyn Error>> {
    let (query, requested_offset) = {
        let state = state.borrow();
        (state.query.clone(), state.offset)
    };
    let mut page = search_page(repository, &query, requested_offset)?;
    let valid_offset = clamp_page_offset(page.offset, page.total, page.limit);
    if valid_offset != page.offset {
        page = search_page(repository, &query, valid_offset)?;
    }
    state.borrow_mut().offset = page.offset;
    refresh_page(ui, page)
}

fn search_page(
    repository: &QsoRepository,
    query: &LogbookQuery,
    offset: usize,
) -> Result<QsoPage, Box<dyn Error>> {
    Ok(match query {
        LogbookQuery::General(query) => repository.search_page(query, offset, DEFAULT_PAGE_SIZE)?,
        LogbookQuery::Dmr(filter) => {
            repository.search_dmr_page(filter, offset, DEFAULT_PAGE_SIZE)?
        }
        LogbookQuery::Ft8(filter) => {
            repository.search_ft8_page(filter, offset, DEFAULT_PAGE_SIZE)?
        }
        LogbookQuery::Dstar(filter) => {
            repository.search_dstar_page(filter, offset, DEFAULT_PAGE_SIZE)?
        }
        LogbookQuery::Ysf(filter) => {
            repository.search_ysf_page(filter, offset, DEFAULT_PAGE_SIZE)?
        }
    })
}

fn refresh_page(ui: &MainWindow, page: QsoPage) -> Result<(), Box<dyn Error>> {
    let page_count = page_count(page.total, page.limit);
    let page_number = if page_count == 0 {
        0
    } else {
        page.offset / page.limit + 1
    };
    ui.set_logbook_page_offset(saturating_i32(page.offset));
    ui.set_logbook_page_limit(saturating_i32(page.limit));
    ui.set_logbook_page_total(saturating_i32(page.total));
    ui.set_logbook_page_number(saturating_i32(page_number));
    ui.set_logbook_page_count(saturating_i32(page_count));
    refresh_rows(ui, page.items)
}

pub(crate) fn refresh_rows(ui: &MainWindow, items: Vec<QsoListItem>) -> Result<(), Box<dyn Error>> {
    let rows = items
        .into_iter()
        .map(|item| {
            let qso = item.qso;
            let (dmr, ft8, dstar, ysf) = match item.metadata {
                ModeMetadata::Dmr(metadata) => (Some(metadata), None, None, None),
                ModeMetadata::Ft8(metadata) => (None, Some(metadata), None, None),
                ModeMetadata::Dstar(metadata) => (None, None, Some(metadata), None),
                ModeMetadata::Ysf(metadata) => (None, None, None, Some(metadata)),
                ModeMetadata::Generic => (None, None, None, None),
            };
            let datetime = format_utc_datetime(qso.datetime_start_utc)?;
            let route_summary = dmr
                .as_ref()
                .map(format_dmr_route)
                .or_else(|| ft8.as_ref().map(format_ft8_summary))
                .or_else(|| dstar.as_ref().map(format_dstar_summary))
                .or_else(|| ysf.as_ref().map(format_ysf_summary))
                .unwrap_or_default();
            Ok(QsoRow {
                id: SharedString::from(qso.id.to_string()),
                callsign: qso.callsign.into(),
                timestamp: datetime.clone().into(),
                datetime_input: datetime.into(),
                frequency: format_frequency(qso.frequency_hz).into(),
                frequency_input: format_frequency_input(qso.frequency_hz).into(),
                band: qso.band.unwrap_or_default().into(),
                mode: qso.mode.into(),
                rst_sent: qso.rst_sent.unwrap_or_default().into(),
                rst_received: qso.rst_received.unwrap_or_default().into(),
                grid: qso.grid_locator.unwrap_or_default().into(),
                name: qso.name.unwrap_or_default().into(),
                qth: qso.qth.unwrap_or_default().into(),
                notes: qso.notes.into(),
                route_summary: route_summary.into(),
                dmr_remote_id: optional_number(dmr.as_ref().and_then(|v| v.remote_dmr_id)).into(),
                dmr_local_id: optional_number(dmr.as_ref().and_then(|v| v.local_dmr_id)).into(),
                dmr_talkgroup: optional_number(dmr.as_ref().and_then(|v| v.talkgroup)).into(),
                dmr_timeslot: optional_number(dmr.as_ref().and_then(|v| v.timeslot)).into(),
                dmr_color_code: optional_number(dmr.as_ref().and_then(|v| v.color_code)).into(),
                dmr_network: dmr
                    .as_ref()
                    .and_then(|v| v.network.clone())
                    .unwrap_or_default()
                    .into(),
                dmr_call_type: dmr
                    .as_ref()
                    .map(|v| v.call_type.as_str())
                    .unwrap_or("")
                    .into(),
                dmr_access_type: dmr
                    .as_ref()
                    .map(|v| v.access_type.as_str())
                    .unwrap_or("")
                    .into(),
                dmr_repeater: dmr
                    .as_ref()
                    .and_then(|v| v.repeater_callsign.clone())
                    .unwrap_or_default()
                    .into(),
                dmr_hotspot: dmr
                    .as_ref()
                    .and_then(|v| v.hotspot.clone())
                    .unwrap_or_default()
                    .into(),
                dmr_notes: dmr
                    .as_ref()
                    .map(|v| v.notes.clone())
                    .unwrap_or_default()
                    .into(),
                ft8_snr_sent: optional_number(ft8.as_ref().and_then(|v| v.snr_sent_db)).into(),
                ft8_snr_received: optional_number(ft8.as_ref().and_then(|v| v.snr_received_db))
                    .into(),
                ft8_power: optional_number(ft8.as_ref().and_then(|v| v.power_watts)).into(),
                ft8_audio_frequency: optional_number(
                    ft8.as_ref().and_then(|v| v.audio_frequency_hz),
                )
                .into(),
                ft8_source_software: ft8
                    .as_ref()
                    .and_then(|v| v.source_software.clone())
                    .unwrap_or_default()
                    .into(),
                ft8_protocol: ft8
                    .as_ref()
                    .and_then(|v| v.protocol.clone())
                    .unwrap_or_default()
                    .into(),
                ft8_final_message: ft8
                    .as_ref()
                    .and_then(|v| v.final_message.clone())
                    .unwrap_or_default()
                    .into(),
                dstar_reflector: dstar
                    .as_ref()
                    .and_then(|v| v.reflector.clone())
                    .unwrap_or_default()
                    .into(),
                dstar_module: dstar
                    .as_ref()
                    .and_then(|v| v.module.clone())
                    .unwrap_or_default()
                    .into(),
                dstar_mycall: dstar
                    .as_ref()
                    .and_then(|v| v.mycall.clone())
                    .unwrap_or_default()
                    .into(),
                dstar_urcall: dstar
                    .as_ref()
                    .and_then(|v| v.urcall.clone())
                    .unwrap_or_default()
                    .into(),
                dstar_rpt1: dstar
                    .as_ref()
                    .and_then(|v| v.rpt1.clone())
                    .unwrap_or_default()
                    .into(),
                dstar_rpt2: dstar
                    .as_ref()
                    .and_then(|v| v.rpt2.clone())
                    .unwrap_or_default()
                    .into(),
                dstar_notes: dstar
                    .as_ref()
                    .map(|v| v.notes.clone())
                    .unwrap_or_default()
                    .into(),
                ysf_room: ysf
                    .as_ref()
                    .and_then(|v| v.room.clone())
                    .unwrap_or_default()
                    .into(),
                ysf_wires_x_node: ysf
                    .as_ref()
                    .and_then(|v| v.wires_x_node.clone())
                    .unwrap_or_default()
                    .into(),
                ysf_repeater: ysf
                    .as_ref()
                    .and_then(|v| v.repeater.clone())
                    .unwrap_or_default()
                    .into(),
                ysf_network: ysf
                    .as_ref()
                    .and_then(|v| v.network.clone())
                    .unwrap_or_default()
                    .into(),
                ysf_access_type: ysf
                    .as_ref()
                    .map(|v| v.access_type.as_str())
                    .unwrap_or("")
                    .into(),
                ysf_tx_dg_id: optional_number(ysf.as_ref().and_then(|v| v.tx_dg_id)).into(),
                ysf_rx_dg_id: optional_number(ysf.as_ref().and_then(|v| v.rx_dg_id)).into(),
                ysf_notes: ysf
                    .as_ref()
                    .map(|v| v.notes.clone())
                    .unwrap_or_default()
                    .into(),
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    ui.set_qsos(ModelRc::new(VecModel::from(rows)));
    Ok(())
}

fn clamp_page_offset(offset: usize, total: usize, limit: usize) -> usize {
    if total == 0 || limit == 0 {
        0
    } else {
        offset.min((total - 1) / limit * limit)
    }
}

fn page_count(total: usize, limit: usize) -> usize {
    if total == 0 || limit == 0 {
        0
    } else {
        1 + (total - 1) / limit
    }
}

fn saturating_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
fn optional_number<T: ToString>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
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

fn format_dstar_summary(metadata: &DStarMetadata) -> String {
    let mut parts = Vec::new();
    if let Some(reflector) = &metadata.reflector {
        parts.push(reflector.clone());
    }
    if let Some(module) = &metadata.module {
        parts.push(format!("Module {module}"));
    }
    if let Some(rpt1) = &metadata.rpt1 {
        parts.push(format!("RPT1 {rpt1}"));
    }
    parts.join(" · ")
}

fn format_ysf_summary(metadata: &YsfMetadata) -> String {
    let mut parts = Vec::new();
    if let Some(room) = &metadata.room {
        parts.push(room.clone());
    }
    if let Some(node) = &metadata.wires_x_node {
        parts.push(format!("Node {node}"));
    }
    if let Some(tx) = metadata.tx_dg_id {
        parts.push(format!("DG-ID TX {tx}"));
    }
    if let Some(rx) = metadata.rx_dg_id {
        parts.push(format!("RX {rx}"));
    }
    if parts.is_empty() {
        parts.push(metadata.access_type.as_str().to_owned());
    }
    parts.join(" · ")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_page_count_without_overflow() {
        assert_eq!(page_count(0, 100), 0);
        assert_eq!(page_count(1, 100), 1);
        assert_eq!(page_count(100, 100), 1);
        assert_eq!(page_count(101, 100), 2);
        assert_eq!(page_count(usize::MAX, 100), 1 + (usize::MAX - 1) / 100);
    }

    #[test]
    fn clamps_offset_to_a_valid_page() {
        assert_eq!(clamp_page_offset(200, 250, 100), 200);
        assert_eq!(clamp_page_offset(300, 250, 100), 200);
        assert_eq!(clamp_page_offset(100, 100, 100), 0);
        assert_eq!(clamp_page_offset(usize::MAX, 0, 100), 0);
        assert_eq!(clamp_page_offset(42, 10, 0), 0);
    }

    #[test]
    fn saturates_values_for_slint_ints() {
        assert_eq!(saturating_i32(42), 42);
        assert_eq!(saturating_i32(usize::MAX), i32::MAX);
    }

    #[test]
    fn formats_compact_ysf_route_summary() {
        let metadata = YsfMetadata::from_input(YsfMetadataInput {
            room: "Brazil Room".into(),
            wires_x_node: "PY2YSF".into(),
            tx_dg_id: "10".into(),
            rx_dg_id: "20".into(),
            access_type: "hotspot".into(),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            format_ysf_summary(&metadata),
            "Brazil Room · Node PY2YSF · DG-ID TX 10 · RX 20"
        );
    }

    #[test]
    fn formats_compact_dstar_route_summary() {
        let metadata = DStarMetadata::from_input(DStarMetadataInput {
            reflector: "REF001 C".into(),
            module: "C".into(),
            rpt1: "PY2XYZ B".into(),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(
            format_dstar_summary(&metadata),
            "REF001 C · Module C · RPT1 PY2XYZ B"
        );
    }
}
