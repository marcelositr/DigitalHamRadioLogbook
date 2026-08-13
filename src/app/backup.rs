use super::*;

pub(crate) fn connect_backup_handler(ui: &MainWindow, repository: &Rc<QsoRepository>) {
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
                    actionable_error("Could not create backup", error.as_ref()),
                    STATUS_ERROR,
                );
            }
        }
    });
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
