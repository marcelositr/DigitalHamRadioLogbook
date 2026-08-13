use std::cell::RefCell;
use std::env;
use std::error::Error;

use std::rc::Rc;

use digital_ham_radio_logbook::config;
use digital_ham_radio_logbook::database::{AdifImportPlan, QsoRepository};
use digital_ham_radio_logbook::logging;

slint::include_modules!();

mod app;

use app::adif::connect_adif_handlers;
use app::backup::connect_backup_handler;
use app::datetime_frequency::{current_utc_timestamp, format_utc_datetime};
use app::editor_navigation::{connect_editor_navigation_handlers, editor_snapshot};
use app::file_dialogs::connect_file_dialog_handlers;
use app::filters::{connect_dmr_filter_handlers, connect_ft8_filter_handlers};
use app::paths::{config_path, database_path};
use app::qso_editor::{connect_mode_handler, connect_save_handler};
use app::qso_list::{connect_delete_handler, connect_search_handler, refresh_qso_list};
use app::settings_close::{
    connect_close_handlers, connect_external_link_handlers, connect_station_config_handler,
};
use app::status::{set_status, STATUS_WARNING};

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
