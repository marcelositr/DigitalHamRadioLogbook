use std::cell::{Cell, RefCell};
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, NaiveDateTime, Utc};
use digital_ham_radio_logbook::adif::{export as export_adif_text, parse as parse_adif};
use digital_ham_radio_logbook::config::{
    self, expand_url_template, AppConfig, DEFAULT_CALLSIGN_URL, DEFAULT_GRID_URL,
};
use digital_ham_radio_logbook::database::{
    AdifImportPlan, AdifImportReport, DmrFilter, Ft8Filter, QsoRepository,
};
use digital_ham_radio_logbook::domain::{
    CommonQsoFields, DmrMetadata, DmrMetadataInput, Ft8Metadata, Ft8MetadataInput, NewQso,
};
use digital_ham_radio_logbook::logging;
use rfd::FileDialog;
use slint::{CloseRequestResponse, ModelRc, SharedString, VecModel};

slint::include_modules!();

const STATUS_INFO: i32 = 0;
const STATUS_SUCCESS: i32 = 1;
const STATUS_WARNING: i32 = 2;
const STATUS_ERROR: i32 = 3;

fn set_status(ui: &MainWindow, text: impl Into<SharedString>, kind: i32) {
    ui.set_status_text(text.into());
    ui.set_status_kind(kind);
}

fn main() -> Result<(), Box<dyn Error>> {
    logging::info(concat!("starting version ", env!("CARGO_PKG_VERSION")));
    let database_path = database_path()?;
    let config_path = config_path()?;
    let app_config = Rc::new(RefCell::new(config::load(&config_path)?));
    let repository = Rc::new(QsoRepository::open(&database_path)?);
    logging::info("database opened and migrations completed");
    let ui = MainWindow::new()?;

    ui.set_local_callsign_text(app_config.borrow().station.callsign.clone().into());
    ui.set_callsign_url_text(
        app_config
            .borrow()
            .external_links
            .callsign_url
            .clone()
            .into(),
    );
    ui.set_grid_url_text(app_config.borrow().external_links.grid_url.clone().into());
    ui.set_active_page(app_config.borrow().operational.sanitized_active_page());
    ui.set_active_filter(app_config.borrow().operational.sanitized_active_filter());
    ui.set_filters_expanded(app_config.borrow().operational.filters_expanded);
    if app_config.borrow().station.callsign.is_empty() {
        set_status(&ui, "Configure the local station callsign", STATUS_WARNING);
    }
    ui.set_datetime_text(format_utc_datetime(current_utc_timestamp()?)?.into());
    refresh_qso_list(&ui, &repository, "")?;
    let editor_baseline = Rc::new(RefCell::new(editor_snapshot(&ui)));
    let pending_adif_plan = Rc::new(RefCell::new(None::<AdifImportPlan>));
    connect_station_config_handler(&ui, &app_config, config_path.clone());
    connect_mode_handler(&ui);
    connect_external_link_handlers(&ui, &app_config, config_path.clone());
    connect_save_handler(&ui, &repository);
    connect_search_handler(&ui, &repository);
    connect_dmr_filter_handlers(&ui, &repository);
    connect_ft8_filter_handlers(&ui, &repository);
    connect_delete_handler(&ui, &repository);
    connect_file_dialog_handlers(&ui, &app_config, config_path.clone());
    connect_adif_handlers(&ui, &repository, &pending_adif_plan);
    connect_backup_handler(&ui, &repository);
    connect_editor_navigation_handlers(&ui, &editor_baseline);
    connect_close_handlers(
        &ui,
        &app_config,
        config_path,
        &editor_baseline,
        &pending_adif_plan,
    );

    ui.run()?;
    logging::info("application stopped");
    Ok(())
}

fn connect_station_config_handler(
    ui: &MainWindow,
    app_config: &Rc<RefCell<AppConfig>>,
    config_path: PathBuf,
) {
    let weak_ui = ui.as_weak();
    let app_config = Rc::clone(app_config);
    ui.on_save_local_callsign(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let result = (|| -> Result<(), Box<dyn Error>> {
            let mut updated = app_config.borrow().clone();
            updated.set_callsign(ui.get_local_callsign_text().as_str())?;
            config::save(&config_path, &updated)?;
            logging::info("local station configuration saved");
            ui.set_local_callsign_text(updated.station.callsign.clone().into());
            *app_config.borrow_mut() = updated;
            Ok(())
        })();
        match result {
            Ok(()) => set_status(&ui, "Local station saved", STATUS_SUCCESS),
            Err(error) => {
                logging::error("failed to save local station configuration");
                set_status(
                    &ui,
                    format!("Could not save station: {error}"),
                    STATUS_ERROR,
                );
            }
        }
    });
}

fn connect_mode_handler(ui: &MainWindow) {
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

fn connect_external_link_handlers(
    ui: &MainWindow,
    app_config: &Rc<RefCell<AppConfig>>,
    config_path: PathBuf,
) {
    let weak_ui = ui.as_weak();
    let app_config_for_save = Rc::clone(app_config);
    let save_path = config_path.clone();
    ui.on_save_external_links(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let result = (|| -> Result<(), Box<dyn Error>> {
            let mut updated = app_config_for_save.borrow().clone();
            updated.set_external_links(
                ui.get_callsign_url_text().as_str(),
                ui.get_grid_url_text().as_str(),
            )?;
            config::save(&save_path, &updated)?;
            *app_config_for_save.borrow_mut() = updated;
            Ok(())
        })();
        match result {
            Ok(()) => set_status(&ui, "External lookup links saved", STATUS_SUCCESS),
            Err(error) => set_status(
                &ui,
                format!("Could not save lookup links: {error}"),
                STATUS_ERROR,
            ),
        }
    });

    let weak_ui = ui.as_weak();
    ui.on_restore_external_link_defaults(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        ui.set_callsign_url_text(DEFAULT_CALLSIGN_URL.into());
        ui.set_grid_url_text(DEFAULT_GRID_URL.into());
        set_status(
            &ui,
            "Default lookup links restored; save to persist them",
            STATUS_INFO,
        );
    });

    let weak_ui = ui.as_weak();
    let app_config_for_callsign = Rc::clone(app_config);
    ui.on_open_callsign_lookup(move |callsign| {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        open_lookup(
            &ui,
            &app_config_for_callsign.borrow().external_links.callsign_url,
            "{callsign}",
            callsign.as_str(),
            "callsign",
        );
    });

    let weak_ui = ui.as_weak();
    let app_config_for_grid = Rc::clone(app_config);
    ui.on_open_grid_lookup(move |grid| {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        open_lookup(
            &ui,
            &app_config_for_grid.borrow().external_links.grid_url,
            "{grid}",
            grid.as_str(),
            "grid",
        );
    });
}

fn open_lookup(
    ui: &MainWindow,
    template: &str,
    placeholder: &'static str,
    value: &str,
    label: &str,
) {
    let result = (|| -> Result<(), Box<dyn Error>> {
        let url = expand_url_template(template, placeholder, value)?;
        webbrowser::open(&url)?;
        Ok(())
    })();
    match result {
        Ok(()) => set_status(
            ui,
            format!("Opening {label} lookup in the default browser"),
            STATUS_INFO,
        ),
        Err(error) => set_status(
            ui,
            format!("Could not open {label} lookup: {error}"),
            STATUS_ERROR,
        ),
    }
}

fn connect_save_handler(ui: &MainWindow, repository: &Rc<QsoRepository>) {
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

fn connect_search_handler(ui: &MainWindow, repository: &Rc<QsoRepository>) {
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

fn connect_dmr_filter_handlers(ui: &MainWindow, repository: &Rc<QsoRepository>) {
    let weak_ui = ui.as_weak();
    let filter_repository = Rc::clone(repository);
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
            refresh_rows(
                &ui,
                filter_repository.search_dmr(&filter)?,
                &filter_repository,
            )?;
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
    ui.on_clear_dmr_filter(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        clear_dmr_filter_fields(&ui);
        match refresh_qso_list(&ui, &clear_repository, ui.get_search_text().as_str()) {
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

fn connect_ft8_filter_handlers(ui: &MainWindow, repository: &Rc<QsoRepository>) {
    let weak_ui = ui.as_weak();
    let filter_repository = Rc::clone(repository);
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
            refresh_rows(
                &ui,
                filter_repository.search_ft8(&filter)?,
                &filter_repository,
            )?;
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
    ui.on_clear_ft8_filter(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        clear_ft8_filter_fields(&ui);
        match refresh_qso_list(&ui, &clear_repository, ui.get_search_text().as_str()) {
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

fn connect_delete_handler(ui: &MainWindow, repository: &Rc<QsoRepository>) {
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

fn connect_file_dialog_handlers(
    ui: &MainWindow,
    app_config: &Rc<RefCell<AppConfig>>,
    config_path: PathBuf,
) {
    let weak_ui = ui.as_weak();
    let import_config = Rc::clone(app_config);
    let import_config_path = config_path.clone();
    ui.on_choose_adif_import(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let mut dialog = FileDialog::new()
            .set_title("Select an ADIF file to import")
            .add_filter("ADIF logbook", &["adi", "adif"]);
        if let Some(directory) = config::OperationalConfig::existing_directory(
            &import_config.borrow().operational.adif_import_directory,
        ) {
            dialog = dialog.set_directory(directory);
        }
        if let Some(path) = dialog.pick_file() {
            ui.set_adif_path_text(path.to_string_lossy().into_owned().into());
            if let Some(parent) = path.parent() {
                let mut updated = import_config.borrow().clone();
                updated.operational.adif_import_directory = parent.to_string_lossy().into_owned();
                if let Err(error) = config::save(&import_config_path, &updated) {
                    set_status(
                        &ui,
                        format!("ADIF selected, but could not remember folder: {error}"),
                        STATUS_WARNING,
                    );
                    return;
                }
                *import_config.borrow_mut() = updated;
            }
            set_status(&ui, "ADIF import file selected", STATUS_INFO);
        }
    });

    let weak_ui = ui.as_weak();
    let export_config = Rc::clone(app_config);
    let export_config_path = config_path.clone();
    ui.on_choose_adif_export(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let mut dialog = FileDialog::new()
            .set_title("Choose the ADIF export destination")
            .set_file_name(suggested_filename("logbook", "adi"))
            .add_filter("ADIF logbook", &["adi", "adif"]);
        if let Some(directory) = config::OperationalConfig::existing_directory(
            &export_config.borrow().operational.adif_export_directory,
        ) {
            dialog = dialog.set_directory(directory);
        }
        if let Some(path) = dialog.save_file() {
            ui.set_adif_path_text(path.to_string_lossy().into_owned().into());
            if let Some(parent) = path.parent() {
                let mut updated = export_config.borrow().clone();
                updated.operational.adif_export_directory = parent.to_string_lossy().into_owned();
                if let Err(error) = config::save(&export_config_path, &updated) {
                    set_status(
                        &ui,
                        format!("Destination selected, but could not remember folder: {error}"),
                        STATUS_WARNING,
                    );
                    return;
                }
                *export_config.borrow_mut() = updated;
            }
            set_status(&ui, "ADIF export destination selected", STATUS_INFO);
        }
    });

    let weak_ui = ui.as_weak();
    let backup_config = Rc::clone(app_config);
    ui.on_choose_backup_destination(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let mut dialog = FileDialog::new()
            .set_title("Choose the database backup destination")
            .set_file_name(suggested_filename("logbook-backup", "sqlite3"))
            .add_filter("SQLite database", &["sqlite3"]);
        if let Some(directory) = config::OperationalConfig::existing_directory(
            &backup_config.borrow().operational.backup_directory,
        ) {
            dialog = dialog.set_directory(directory);
        }
        if let Some(path) = dialog.save_file() {
            ui.set_backup_path_text(path.to_string_lossy().into_owned().into());
            if let Some(parent) = path.parent() {
                let mut updated = backup_config.borrow().clone();
                updated.operational.backup_directory = parent.to_string_lossy().into_owned();
                if let Err(error) = config::save(&config_path, &updated) {
                    set_status(
                        &ui,
                        format!("Destination selected, but could not remember folder: {error}"),
                        STATUS_WARNING,
                    );
                    return;
                }
                *backup_config.borrow_mut() = updated;
            }
            set_status(&ui, "Backup destination selected", STATUS_INFO);
        }
    });
}

fn suggested_filename(prefix: &str, extension: &str) -> String {
    let date = DateTime::<Utc>::from(SystemTime::now()).format("%Y-%m-%d");
    format!("{prefix}-{date}.{extension}")
}

fn connect_adif_handlers(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    pending_plan: &Rc<RefCell<Option<AdifImportPlan>>>,
) {
    let weak_ui = ui.as_weak();
    let preview_repository = Rc::clone(repository);
    let preview_plan = Rc::clone(pending_plan);
    ui.on_preview_adif(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let result = (|| -> Result<AdifImportPlan, Box<dyn Error>> {
            let path_text = ui.get_adif_path_text();
            let path = required_adif_path(path_text.as_str())?;
            let contents = fs::read_to_string(path)?;
            let document = parse_adif(&contents)?;
            preview_repository.prepare_adif_import(&document)
        })();
        match result {
            Ok(plan) => {
                let preview = plan.preview();
                let modes = preview
                    .modes
                    .iter()
                    .map(|(mode, count)| format!("{mode}: {count}"))
                    .collect::<Vec<_>>()
                    .join("  •  ");
                ui.set_adif_preview_total(preview.total as i32);
                ui.set_adif_preview_new(preview.new_qsos as i32);
                ui.set_adif_preview_duplicates(preview.duplicates as i32);
                ui.set_adif_preview_invalid(preview.invalid as i32);
                ui.set_adif_preview_modes(if modes.is_empty() {
                    "None".into()
                } else {
                    modes.into()
                });
                ui.set_adif_preview_visible(true);
                *preview_plan.borrow_mut() = Some(plan);
                set_status(
                    &ui,
                    "ADIF preview ready; review before importing",
                    STATUS_INFO,
                );
            }
            Err(error) => {
                *preview_plan.borrow_mut() = None;
                ui.set_adif_preview_visible(false);
                logging::error("ADIF preview failed");
                set_status(
                    &ui,
                    format!("Could not preview ADIF: {error}"),
                    STATUS_ERROR,
                );
            }
        }
    });

    let weak_ui = ui.as_weak();
    let import_repository = Rc::clone(repository);
    let import_plan = Rc::clone(pending_plan);
    ui.on_confirm_adif_import(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let result = (|| -> Result<AdifImportReport, Box<dyn Error>> {
            let plan = import_plan
                .borrow_mut()
                .take()
                .ok_or("Preview the ADIF file before importing")?;
            let report = import_repository.import_adif_plan(plan, current_utc_timestamp()?)?;
            logging::info(&format!(
                "ADIF import completed: {} imported, {} duplicate(s) skipped",
                report.imported, report.duplicates_skipped
            ));
            refresh_qso_list(&ui, &import_repository, ui.get_search_text().as_str())?;
            Ok(report)
        })();
        ui.set_adif_preview_visible(false);
        match result {
            Ok(report) => set_status(
                &ui,
                format!(
                    "Imported {} ADIF QSO(s); skipped {} duplicate(s)",
                    report.imported, report.duplicates_skipped
                ),
                STATUS_SUCCESS,
            ),
            Err(error) => {
                logging::error("ADIF import failed");
                set_status(&ui, format!("Could not import ADIF: {error}"), STATUS_ERROR);
            }
        }
    });

    let weak_ui = ui.as_weak();
    let cancel_plan = Rc::clone(pending_plan);
    ui.on_cancel_adif_import(move || {
        *cancel_plan.borrow_mut() = None;
        if let Some(ui) = weak_ui.upgrade() {
            ui.set_adif_preview_visible(false);
            set_status(&ui, "ADIF import canceled; no changes made", STATUS_INFO);
        }
    });

    let weak_ui = ui.as_weak();
    let invalidate_plan = Rc::clone(pending_plan);
    ui.on_invalidate_adif_preview(move || {
        *invalidate_plan.borrow_mut() = None;
        if let Some(ui) = weak_ui.upgrade() {
            ui.set_adif_preview_visible(false);
        }
    });

    let weak_ui = ui.as_weak();
    let export_repository = Rc::clone(repository);
    ui.on_export_adif(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let result = (|| -> Result<usize, Box<dyn Error>> {
            let path_text = ui.get_adif_path_text();
            let path = required_adif_path(path_text.as_str())?;
            let document = export_repository.export_adif()?;
            let count = document.records.len();
            let contents = export_adif_text(&document);
            write_new_file_atomically(path, contents.as_bytes())?;
            logging::info(&format!("ADIF export completed: {count} record(s)"));
            Ok(count)
        })();
        match result {
            Ok(count) => set_status(
                &ui,
                format!("Exported {count} QSO(s) to ADIF"),
                STATUS_SUCCESS,
            ),
            Err(error) => {
                logging::error("ADIF export failed");
                set_status(&ui, format!("Could not export ADIF: {error}"), STATUS_ERROR);
            }
        }
    });
}

fn connect_backup_handler(ui: &MainWindow, repository: &Rc<QsoRepository>) {
    let weak_ui = ui.as_weak();
    let repository = Rc::clone(repository);
    ui.on_backup_database(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let result = (|| -> Result<(), Box<dyn Error>> {
            let path_text = ui.get_backup_path_text();
            let path = required_backup_path(path_text.as_str())?;
            repository.backup_to(path)?;
            logging::info("database backup completed");
            Ok(())
        })();
        match result {
            Ok(()) => set_status(&ui, "Database backup created", STATUS_SUCCESS),
            Err(error) => {
                logging::error("database backup failed");
                set_status(
                    &ui,
                    format!("Could not create backup: {error}"),
                    STATUS_ERROR,
                );
            }
        }
    });
}

fn write_new_file_atomically(path: &Path, contents: &[u8]) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        return Err("destination already exists".into());
    }
    let parent = path.parent().ok_or("destination has no parent directory")?;
    if !parent.is_dir() {
        return Err("destination directory does not exist".into());
    }
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("destination filename is invalid")?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary =
        path.with_file_name(format!(".{filename}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
        if path.exists() {
            return Err("destination already exists".into());
        }
        fs::rename(&temporary, path)?;
        #[cfg(unix)]
        std::fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn required_backup_path(input: &str) -> Result<&Path, Box<dyn Error>> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Enter a backup file path".into());
    }
    let path = Path::new(input);
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("sqlite3") => Ok(path),
        _ => Err("Backup file path must end in .sqlite3".into()),
    }
}

fn required_adif_path(input: &str) -> Result<&Path, Box<dyn Error>> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Enter an ADIF file path".into());
    }
    let path = Path::new(input);
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension)
            if extension.eq_ignore_ascii_case("adi") || extension.eq_ignore_ascii_case("adif") =>
        {
            Ok(path)
        }
        _ => Err("ADIF file path must end in .adi or .adif".into()),
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
struct EditorSnapshot(Vec<String>);

fn has_unsaved_changes(current: &EditorSnapshot, baseline: &EditorSnapshot) -> bool {
    current != baseline
}

fn has_pending_exit_work(
    active_page: i32,
    current: &EditorSnapshot,
    baseline: &EditorSnapshot,
    adif_preview_pending: bool,
) -> bool {
    (active_page == 1 && has_unsaved_changes(current, baseline)) || adif_preview_pending
}

fn editor_snapshot(ui: &MainWindow) -> EditorSnapshot {
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

fn save_operational_preferences(
    ui: &MainWindow,
    app_config: &Rc<RefCell<AppConfig>>,
    config_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut updated = app_config.borrow().clone();
    updated.operational.active_page = ui.get_active_page();
    updated.operational.active_filter = ui.get_active_filter();
    updated.operational.filters_expanded = ui.get_filters_expanded();
    config::save(config_path, &updated)?;
    *app_config.borrow_mut() = updated;
    Ok(())
}

fn connect_close_handlers(
    ui: &MainWindow,
    app_config: &Rc<RefCell<AppConfig>>,
    config_path: PathBuf,
    baseline: &Rc<RefCell<EditorSnapshot>>,
    pending_adif_plan: &Rc<RefCell<Option<AdifImportPlan>>>,
) {
    let exit_authorized = Rc::new(Cell::new(false));

    let weak_ui = ui.as_weak();
    let close_config = Rc::clone(app_config);
    let close_path = config_path.clone();
    let close_baseline = Rc::clone(baseline);
    let close_plan = Rc::clone(pending_adif_plan);
    let close_authorized = Rc::clone(&exit_authorized);
    ui.window().on_close_requested(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return CloseRequestResponse::HideWindow;
        };
        if close_authorized.get() {
            return CloseRequestResponse::HideWindow;
        }
        if has_pending_exit_work(
            ui.get_active_page(),
            &editor_snapshot(&ui),
            &close_baseline.borrow(),
            close_plan.borrow().is_some(),
        ) {
            ui.set_exit_save_failed(false);
            ui.set_exit_error_text("".into());
            ui.set_exit_confirmation_visible(true);
            set_status(
                &ui,
                "Pending work needs confirmation before exit",
                STATUS_WARNING,
            );
            return CloseRequestResponse::KeepWindowShown;
        }
        match save_operational_preferences(&ui, &close_config, &close_path) {
            Ok(()) => CloseRequestResponse::HideWindow,
            Err(error) => {
                ui.set_exit_save_failed(true);
                ui.set_exit_error_text(error.to_string().into());
                ui.set_exit_confirmation_visible(true);
                set_status(&ui, "Could not save preferences before exit", STATUS_ERROR);
                CloseRequestResponse::KeepWindowShown
            }
        }
    });

    let weak_ui = ui.as_weak();
    ui.on_continue_working(move || {
        if let Some(ui) = weak_ui.upgrade() {
            ui.set_exit_confirmation_visible(false);
            ui.set_exit_save_failed(false);
            set_status(&ui, "Continuing work", STATUS_INFO);
        }
    });

    let weak_ui = ui.as_weak();
    let discard_config = Rc::clone(app_config);
    let discard_path = config_path.clone();
    let discard_authorized = Rc::clone(&exit_authorized);
    ui.on_discard_and_exit(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        match save_operational_preferences(&ui, &discard_config, &discard_path) {
            Ok(()) => {
                discard_authorized.set(true);
                let _ = ui.window().hide();
            }
            Err(error) => {
                ui.set_exit_save_failed(true);
                ui.set_exit_error_text(error.to_string().into());
                set_status(&ui, "Could not save preferences before exit", STATUS_ERROR);
            }
        }
    });

    let weak_ui = ui.as_weak();
    let retry_config = Rc::clone(app_config);
    let retry_path = config_path;
    let retry_authorized = Rc::clone(&exit_authorized);
    ui.on_retry_exit(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        match save_operational_preferences(&ui, &retry_config, &retry_path) {
            Ok(()) => {
                retry_authorized.set(true);
                let _ = ui.window().hide();
            }
            Err(error) => {
                ui.set_exit_error_text(error.to_string().into());
                set_status(&ui, "Could not save preferences before exit", STATUS_ERROR);
            }
        }
    });

    let weak_ui = ui.as_weak();
    let force_authorized = Rc::clone(&exit_authorized);
    ui.on_exit_without_saving(move || {
        if let Some(ui) = weak_ui.upgrade() {
            force_authorized.set(true);
            let _ = ui.window().hide();
        }
    });
}

fn connect_editor_navigation_handlers(ui: &MainWindow, baseline: &Rc<RefCell<EditorSnapshot>>) {
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

fn clear_editor(ui: &MainWindow) -> Result<(), Box<dyn Error>> {
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

fn refresh_qso_list(
    ui: &MainWindow,
    repository: &QsoRepository,
    query: &str,
) -> Result<(), Box<dyn Error>> {
    refresh_rows(ui, repository.search(query)?, repository)
}

fn refresh_rows(
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

fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    let config_home = match env::var_os("XDG_CONFIG_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => {
            let home = env::var_os("HOME").ok_or("HOME is not defined")?;
            PathBuf::from(home).join(".config")
        }
    };
    Ok(config_home.join("digital-ham-log/config.toml"))
}

fn database_path() -> Result<PathBuf, Box<dyn Error>> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => {
            let home = env::var_os("HOME").ok_or("HOME is not defined")?;
            PathBuf::from(home).join(".local/share")
        }
    };

    let application_directory = data_home.join("digital-ham-log");
    fs::create_dir_all(&application_directory)?;
    Ok(application_directory.join("logbook.sqlite3"))
}

fn current_utc_timestamp() -> Result<i64, Box<dyn Error>> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(i64::try_from(seconds)?)
}

fn parse_utc_datetime(input: &str) -> Result<i64, Box<dyn Error>> {
    let input = input.trim().strip_suffix(" UTC").unwrap_or(input.trim());
    let datetime = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| "Enter date/time as YYYY-MM-DD HH:MM:SS UTC")?;
    Ok(datetime.and_utc().timestamp())
}

fn format_utc_datetime(timestamp: i64) -> Result<String, Box<dyn Error>> {
    let datetime: DateTime<Utc> =
        DateTime::from_timestamp(timestamp, 0).ok_or("UTC timestamp is out of range")?;
    Ok(datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string())
}

fn parse_mhz_to_hz(input: &str) -> Result<i64, Box<dyn Error>> {
    let normalized = input.trim().replace(',', ".");
    let mut parts = normalized.split('.');
    let whole = parts.next().ok_or("Frequency is required")?;
    let fraction = parts.next().unwrap_or("");

    if whole.is_empty() || parts.next().is_some() || fraction.len() > 6 {
        return Err("Enter a valid frequency in MHz".into());
    }

    let whole_hz = whole
        .parse::<i64>()?
        .checked_mul(1_000_000)
        .ok_or("Frequency is too large")?;
    let fraction_hz = if fraction.is_empty() {
        0
    } else {
        format!("{fraction:0<6}").parse::<i64>()?
    };
    let frequency_hz = whole_hz
        .checked_add(fraction_hz)
        .ok_or("Frequency is too large")?;

    if frequency_hz <= 0 {
        return Err("Frequency must be greater than zero".into());
    }
    Ok(frequency_hz)
}

fn format_frequency(frequency_hz: i64) -> String {
    format!("{} MHz", format_frequency_input(frequency_hz))
}

fn format_frequency_input(frequency_hz: i64) -> String {
    let whole = frequency_hz / 1_000_000;
    let fraction = frequency_hz.rem_euclid(1_000_000);
    format!("{whole}.{fraction:06}")
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

    #[test]
    fn parses_and_formats_utc_datetime() {
        let timestamp = parse_utc_datetime("2023-11-14 22:13:20 UTC").unwrap();
        assert_eq!(timestamp, 1_700_000_000);
        assert_eq!(
            format_utc_datetime(timestamp).unwrap(),
            "2023-11-14 22:13:20 UTC"
        );
        assert!(parse_utc_datetime("14/11/2023 22:13").is_err());
    }

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

    #[test]
    fn validates_backup_file_paths() {
        assert!(required_backup_path("/tmp/logbook-backup.sqlite3").is_ok());
        assert!(required_backup_path("").is_err());
        assert!(required_backup_path("/tmp/logbook.db").is_err());
    }

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

    #[test]
    fn validates_adif_file_paths() {
        assert!(required_adif_path("/tmp/log.adi").is_ok());
        assert!(required_adif_path("/tmp/log.ADIF").is_ok());
        assert!(required_adif_path("").is_err());
        assert!(required_adif_path("/tmp/log.txt").is_err());
    }

    #[test]
    fn parses_mhz_without_floating_point() {
        assert_eq!(parse_mhz_to_hz("438.500").unwrap(), 438_500_000);
        assert_eq!(parse_mhz_to_hz("14,074").unwrap(), 14_074_000);
        assert_eq!(parse_mhz_to_hz("145").unwrap(), 145_000_000);
    }

    #[test]
    fn rejects_invalid_frequency() {
        assert!(parse_mhz_to_hz("").is_err());
        assert!(parse_mhz_to_hz("145.1234567").is_err());
        assert!(parse_mhz_to_hz("145.5.1").is_err());
    }

    #[test]
    fn formats_hz_as_mhz() {
        assert_eq!(format_frequency(438_500_000), "438.500000 MHz");
        assert_eq!(format_frequency_input(14_074_000), "14.074000");
    }
}
