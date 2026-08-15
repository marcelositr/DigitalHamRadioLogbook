use super::*;

pub(crate) fn connect_station_config_handler(
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
                logging::error(&format!(
                    "failed to save local station configuration: {error}"
                ));
                set_status(
                    &ui,
                    actionable_error("Could not save station", error.as_ref()),
                    STATUS_ERROR,
                );
            }
        }
    });
}

pub(crate) fn connect_external_link_handlers(
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
            Err(error) => {
                logging::error(&format!("failed to save external lookup links: {error}"));
                set_status(
                    &ui,
                    actionable_error("Could not save lookup links", error.as_ref()),
                    STATUS_ERROR,
                );
            }
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
            actionable_error(&format!("Could not open {label} lookup"), error.as_ref()),
            STATUS_ERROR,
        ),
    }
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

pub(crate) fn connect_close_handlers(
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
                logging::error(&format!("failed to save preferences before exit: {error}"));
                ui.set_exit_save_failed(true);
                ui.set_exit_error_text(
                    actionable_error("Could not save preferences", error.as_ref()).into(),
                );
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
                logging::error(&format!("failed to save preferences before exit: {error}"));
                ui.set_exit_save_failed(true);
                ui.set_exit_error_text(
                    actionable_error("Could not save preferences", error.as_ref()).into(),
                );
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
                logging::error(&format!("failed to save preferences before exit: {error}"));
                ui.set_exit_error_text(
                    actionable_error("Could not save preferences", error.as_ref()).into(),
                );
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
