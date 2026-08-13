use super::*;

pub(crate) fn connect_adif_handlers(
    ui: &MainWindow,
    repository: &Rc<QsoRepository>,
    pending_plan: &Rc<RefCell<Option<AdifImportPlan>>>,
    state: &SharedLogbookViewState,
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
                    actionable_error("Could not preview ADIF", error.as_ref()),
                    STATUS_ERROR,
                );
            }
        }
    });

    let weak_ui = ui.as_weak();
    let import_repository = Rc::clone(repository);
    let import_plan = Rc::clone(pending_plan);
    let import_state = Rc::clone(state);
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
            refresh_qso_list(&ui, &import_repository, &import_state)?;
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
                set_status(
                    &ui,
                    actionable_error("Could not import ADIF", error.as_ref()),
                    STATUS_ERROR,
                );
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
                set_status(
                    &ui,
                    actionable_error("Could not export ADIF", error.as_ref()),
                    STATUS_ERROR,
                );
            }
        }
    });
}
pub(super) fn write_new_file_atomically(
    path: &Path,
    contents: &[u8],
) -> Result<(), Box<dyn Error>> {
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
pub(crate) fn required_adif_path(input: &str) -> Result<&Path, Box<dyn Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_adif_file_paths() {
        assert!(required_adif_path("/tmp/log.adi").is_ok());
        assert!(required_adif_path("/tmp/log.ADIF").is_ok());
        assert!(required_adif_path("").is_err());
        assert!(required_adif_path("/tmp/log.txt").is_err());
    }
}
