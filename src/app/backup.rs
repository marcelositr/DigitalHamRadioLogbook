use super::*;

pub(crate) fn connect_backup_handler(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    database_path: PathBuf,
) {
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
                logging::error(&format!("database backup failed: {error}"));
                set_status(
                    &ui,
                    actionable_error("Could not create backup", error.as_ref()),
                    STATUS_ERROR,
                );
            }
        }
    });

    let weak_ui = ui.as_weak();
    ui.on_check_data_health(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let report = inspect_database(&database_path);
        logging::info(&format!("data health check completed: {:?}", report.status));
        present_health_report(&ui, "Active logbook health", &report);
    });

    let weak_ui = ui.as_weak();
    ui.on_verify_backup(move || {
        let Some(ui) = weak_ui.upgrade() else {
            return;
        };
        let Some(path) = FileDialog::new()
            .set_title("Select a database backup to verify")
            .add_filter("SQLite database", &["sqlite3", "sqlite", "db"])
            .pick_file()
        else {
            return;
        };
        let report = inspect_database(&path);
        logging::info(&format!(
            "backup verification completed: {:?}",
            report.status
        ));
        present_health_report(&ui, "Backup verification", &report);
    });
}

fn present_health_report(ui: &MainWindow, title: &str, report: &HealthReport) {
    let (summary, kind) = match report.status {
        HealthStatus::HealthyCurrent => (
            "All checks passed. No data was modified.",
            STATUS_SUCCESS,
        ),
        HealthStatus::HealthyMigratableOld => (
            "This is a valid backup from an older supported schema. It will be migrated when restored. No data was modified.",
            STATUS_WARNING,
        ),
        HealthStatus::FutureIncompatible => (
            "This database uses a newer unsupported schema. No data was modified.",
            STATUS_ERROR,
        ),
        HealthStatus::InvalidOrCorrupt => (
            "A database consistency problem was found. No data was modified.",
            STATUS_ERROR,
        ),
        HealthStatus::Unreadable => (
            "The selected database could not be read. No data was modified.",
            STATUS_ERROR,
        ),
    };
    ui.set_database_report_title(title.into());
    ui.set_database_report_text(format!("{summary}\n\n{}", report.diagnostic_text()).into());
    ui.set_database_report_kind(kind);
    ui.set_database_report_visible(true);
    set_status(ui, summary, kind);
}

pub(crate) fn required_backup_path(input: &str) -> Result<&Path, Box<dyn Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_backup_file_paths() {
        assert!(required_backup_path("/tmp/logbook-backup.sqlite3").is_ok());
        assert!(required_backup_path("").is_err());
        assert!(required_backup_path("/tmp/logbook.db").is_err());
    }
}
