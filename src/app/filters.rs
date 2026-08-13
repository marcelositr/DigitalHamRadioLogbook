use super::*;

pub(crate) fn connect_dmr_filter_handlers(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    state: &SharedLogbookViewState,
) {
    let weak_ui = ui.as_weak();
    let filter_repository = Rc::clone(repository);
    let filter_state = Rc::clone(state);
    ui.on_filter_dmr(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let result = (|| -> Result<(), Box<dyn Error>> {
            let filter = DmrFilter {
                dmr_id: parse_optional_positive_u32(
                    ui.get_dmr_filter_id_text().as_str(),
                    "DMR ID",
                )?,
                talkgroup: parse_optional_positive_u32(
                    ui.get_dmr_filter_talkgroup_text().as_str(),
                    "Talkgroup",
                )?,
                network: optional_filter_text(ui.get_dmr_filter_network_text().as_str()),
                repeater: optional_filter_text(ui.get_dmr_filter_repeater_text().as_str()),
                hotspot: optional_filter_text(ui.get_dmr_filter_hotspot_text().as_str()),
                timeslot: parse_optional_timeslot(ui.get_dmr_filter_timeslot_text().as_str())?,
            };
            {
                let mut state = filter_state.borrow_mut();
                state.query = LogbookQuery::Dmr(filter);
                state.offset = 0;
            }
            refresh_qso_list(&ui, &filter_repository, &filter_state)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                ui.set_filters_applied(true);
                ui.set_filters_expanded(false);
                set_status(&ui, "DMR filters applied", STATUS_SUCCESS);
            }
            Err(error) => set_status(
                &ui,
                format!("Could not filter DMR QSOs: {error}"),
                STATUS_ERROR,
            ),
        }
    });

    let weak_ui = ui.as_weak();
    let clear_repository = Rc::clone(repository);
    let clear_state = Rc::clone(state);
    ui.on_clear_dmr_filter(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        clear_dmr_filter_fields(&ui);
        {
            let mut state = clear_state.borrow_mut();
            state.query = LogbookQuery::General(ui.get_search_text().to_string());
            state.offset = 0;
        }
        match refresh_qso_list(&ui, &clear_repository, &clear_state) {
            Ok(()) => {
                ui.set_filters_applied(false);
                ui.set_filters_expanded(false);
                set_status(&ui, "DMR filters cleared", STATUS_INFO);
            }
            Err(error) => set_status(&ui, format!("Could not reload QSOs: {error}"), STATUS_ERROR),
        }
    });
}

fn clear_dmr_filter_fields(ui: &MainWindow) {
    ui.set_dmr_filter_id_text("".into());
    ui.set_dmr_filter_talkgroup_text("".into());
    ui.set_dmr_filter_network_text("".into());
    ui.set_dmr_filter_repeater_text("".into());
    ui.set_dmr_filter_hotspot_text("".into());
    ui.set_dmr_filter_timeslot_text("".into());
}

fn parse_optional_positive_u32(
    input: &str,
    field_name: &str,
) -> Result<Option<u32>, Box<dyn Error>> {
    if input.trim().is_empty() {
        return Ok(None);
    }
    let value = input
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("{field_name} must be a positive integer"))?;
    if value == 0 {
        return Err(format!("{field_name} must be a positive integer").into());
    }
    Ok(Some(value))
}

fn parse_optional_timeslot(input: &str) -> Result<Option<u8>, Box<dyn Error>> {
    if input.trim().is_empty() {
        return Ok(None);
    }
    match input.trim().parse::<u8>() {
        Ok(value @ 1..=2) => Ok(Some(value)),
        _ => Err("Timeslot filter must be 1 or 2".into()),
    }
}

fn optional_filter_text(input: &str) -> Option<String> {
    let input = input.trim();
    (!input.is_empty()).then(|| input.to_owned())
}

pub(crate) fn connect_ft8_filter_handlers(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    state: &SharedLogbookViewState,
) {
    let weak_ui = ui.as_weak();
    let filter_repository = Rc::clone(repository);
    let filter_state = Rc::clone(state);
    ui.on_filter_ft8(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let result = (|| -> Result<(), Box<dyn Error>> {
            let filter = Ft8Filter {
                callsign: optional_filter_text(ui.get_ft8_filter_callsign_text().as_str()),
                grid: optional_filter_text(ui.get_ft8_filter_grid_text().as_str()),
                band: optional_filter_text(ui.get_ft8_filter_band_text().as_str()),
                minimum_snr_received_db: parse_optional_snr(
                    ui.get_ft8_filter_min_snr_text().as_str(),
                )?,
                maximum_snr_received_db: parse_optional_snr(
                    ui.get_ft8_filter_max_snr_text().as_str(),
                )?,
                start_utc: parse_optional_utc_datetime(ui.get_ft8_filter_start_text().as_str())?,
                end_utc: parse_optional_utc_datetime(ui.get_ft8_filter_end_text().as_str())?,
            };
            if matches!(
                (filter.minimum_snr_received_db, filter.maximum_snr_received_db),
                (Some(minimum), Some(maximum)) if minimum > maximum
            ) {
                return Err("Minimum SNR cannot exceed maximum SNR".into());
            }
            {
                let mut state = filter_state.borrow_mut();
                state.query = LogbookQuery::Ft8(filter);
                state.offset = 0;
            }
            refresh_qso_list(&ui, &filter_repository, &filter_state)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                ui.set_filters_applied(true);
                ui.set_filters_expanded(false);
                set_status(&ui, "FT8 filters applied", STATUS_SUCCESS);
            }
            Err(error) => set_status(
                &ui,
                format!("Could not filter FT8 QSOs: {error}"),
                STATUS_ERROR,
            ),
        }
    });

    let weak_ui = ui.as_weak();
    let clear_repository = Rc::clone(repository);
    let clear_state = Rc::clone(state);
    ui.on_clear_ft8_filter(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        clear_ft8_filter_fields(&ui);
        {
            let mut state = clear_state.borrow_mut();
            state.query = LogbookQuery::General(ui.get_search_text().to_string());
            state.offset = 0;
        }
        match refresh_qso_list(&ui, &clear_repository, &clear_state) {
            Ok(()) => {
                ui.set_filters_applied(false);
                ui.set_filters_expanded(false);
                set_status(&ui, "FT8 filters cleared", STATUS_INFO);
            }
            Err(error) => set_status(&ui, format!("Could not reload QSOs: {error}"), STATUS_ERROR),
        }
    });
}

fn clear_ft8_filter_fields(ui: &MainWindow) {
    ui.set_ft8_filter_callsign_text("".into());
    ui.set_ft8_filter_grid_text("".into());
    ui.set_ft8_filter_band_text("".into());
    ui.set_ft8_filter_min_snr_text("".into());
    ui.set_ft8_filter_max_snr_text("".into());
    ui.set_ft8_filter_start_text("".into());
    ui.set_ft8_filter_end_text("".into());
}

fn parse_optional_snr(input: &str) -> Result<Option<i16>, Box<dyn Error>> {
    if input.trim().is_empty() {
        return Ok(None);
    }
    let snr = input
        .trim()
        .parse::<i16>()
        .map_err(|_| "SNR filter must be an integer")?;
    if !(-50..=50).contains(&snr) {
        return Err("SNR filter must be between -50 and 50 dB".into());
    }
    Ok(Some(snr))
}

fn parse_optional_utc_datetime(input: &str) -> Result<Option<i64>, Box<dyn Error>> {
    if input.trim().is_empty() {
        Ok(None)
    } else {
        parse_utc_datetime(input).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_optional_dmr_filters() {
        assert_eq!(parse_optional_positive_u32("724", "TG").unwrap(), Some(724));
        assert_eq!(parse_optional_positive_u32("", "TG").unwrap(), None);
        assert!(parse_optional_positive_u32("0", "TG").is_err());
        assert_eq!(parse_optional_timeslot("2").unwrap(), Some(2));
        assert!(parse_optional_timeslot("3").is_err());
        assert_eq!(
            optional_filter_text(" network ").as_deref(),
            Some("network")
        );
        assert_eq!(parse_optional_snr("-18").unwrap(), Some(-18));
        assert!(parse_optional_snr("-60").is_err());
        assert_eq!(parse_optional_utc_datetime("").unwrap(), None);
    }
}
