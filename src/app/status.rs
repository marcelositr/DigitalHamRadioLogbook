use super::*;

pub(crate) const STATUS_INFO: i32 = 0;
pub(crate) const STATUS_SUCCESS: i32 = 1;
pub(crate) const STATUS_WARNING: i32 = 2;
pub(crate) const STATUS_ERROR: i32 = 3;

pub(crate) fn set_status(ui: &MainWindow, text: impl Into<SharedString>, kind: i32) {
    ui.set_status_text(text.into());
    ui.set_status_kind(kind);
}

pub(crate) fn actionable_error(context: &str, error: &(dyn Error + 'static)) -> String {
    let detail = error.to_string();
    let detail_lower = detail.to_ascii_lowercase();
    let io_kind = io_error_kind(error);

    let guidance = if detail_lower.contains("destination already exists")
        || io_kind == Some(ErrorKind::AlreadyExists)
    {
        "Choose a new filename; existing files are never overwritten."
    } else if detail_lower.contains("directory does not exist")
        || io_kind == Some(ErrorKind::NotFound)
    {
        "Select an existing file or folder and try again."
    } else if io_kind == Some(ErrorKind::PermissionDenied) {
        "Choose a location where your user has permission, then try again."
    } else if io_kind == Some(ErrorKind::ReadOnlyFilesystem) {
        "Choose a writable location, then try again."
    } else {
        ""
    };

    if guidance.is_empty() {
        format!("{context}: {detail}")
    } else {
        format!("{context}: {detail}. {guidance}")
    }
}

fn io_error_kind(mut error: &(dyn Error + 'static)) -> Option<ErrorKind> {
    loop {
        if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
            return Some(io_error.kind());
        }
        error = error.source()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_actionable_guidance_to_common_file_errors() {
        let exists = std::io::Error::from(ErrorKind::AlreadyExists);
        assert!(
            actionable_error("Could not export ADIF", &exists).contains("Choose a new filename")
        );
        let missing = std::io::Error::from(ErrorKind::NotFound);
        assert!(actionable_error("Could not preview ADIF", &missing)
            .contains("Select an existing file or folder"));
        let denied = std::io::Error::from(ErrorKind::PermissionDenied);
        assert!(actionable_error("Could not create backup", &denied)
            .contains("where your user has permission"));
        let validation = std::io::Error::new(ErrorKind::InvalidInput, "path must end in .adi");
        assert_eq!(
            actionable_error("Could not export ADIF", &validation),
            "Could not export ADIF: path must end in .adi"
        );
    }

    #[test]
    fn recognizes_existing_destination_messages_without_typed_io_errors() {
        let error = std::io::Error::other("destination already exists");
        assert!(actionable_error("Could not create backup", &error)
            .contains("existing files are never overwritten"));
    }
}
