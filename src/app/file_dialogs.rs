use super::*;

pub(crate) fn connect_file_dialog_handlers(
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

pub(crate) fn suggested_filename(prefix: &str, extension: &str) -> String {
    let date = DateTime::<Utc>::from(SystemTime::now()).format("%Y-%m-%d");
    format!("{prefix}-{date}.{extension}")
}
