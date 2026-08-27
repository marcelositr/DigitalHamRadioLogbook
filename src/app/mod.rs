use std::cell::{Cell, RefCell};
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, NaiveDateTime, Utc};
use digital_ham_radio_logbook::adif::{export as export_adif_text, parse as parse_adif};
use digital_ham_radio_logbook::config::{
    self, expand_url_template, AppConfig, DEFAULT_CALLSIGN_URL, DEFAULT_GRID_URL,
};
use digital_ham_radio_logbook::database::{
    inspect_database, AdifImportPlan, AdifImportReport, DmrFilter, DstarFilter, Ft8Filter,
    HealthReport, HealthStatus, QsoRepository, YsfFilter,
};
use digital_ham_radio_logbook::domain::{
    CommonQsoFields, DStarMetadata, DStarMetadataInput, DmrMetadata, DmrMetadataInput, Ft8Metadata,
    Ft8MetadataInput, NewQso, YsfMetadata, YsfMetadataInput,
};
use digital_ham_radio_logbook::logging;
use rfd::FileDialog;
use slint::{CloseRequestResponse, ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{MainWindow, QsoRow};

pub(crate) mod adif;
pub(crate) mod backup;
pub(crate) mod datetime_frequency;
pub(crate) mod editor_navigation;
pub(crate) mod file_dialogs;
pub(crate) mod filters;
pub(crate) mod paths;
pub(crate) mod qso_editor;
pub(crate) mod qso_list;
pub(crate) mod settings_close;
pub(crate) mod status;

use datetime_frequency::{
    current_utc_timestamp, format_frequency, format_frequency_input, format_utc_datetime,
    parse_mhz_to_hz, parse_utc_datetime,
};
use editor_navigation::{editor_snapshot, has_pending_exit_work, EditorSnapshot};

use qso_editor::clear_editor;
use qso_list::{refresh_qso_list, LogbookQuery, SharedLogbookViewState};
use status::{
    actionable_error, set_status, STATUS_ERROR, STATUS_INFO, STATUS_SUCCESS, STATUS_WARNING,
};
