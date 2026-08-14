use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub station: StationConfig,
    #[serde(default)]
    pub external_links: ExternalLinksConfig,
    #[serde(default)]
    pub operational: OperationalConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalConfig {
    #[serde(default)]
    pub active_page: i32,
    #[serde(default)]
    pub active_filter: i32,
    #[serde(default)]
    pub filters_expanded: bool,
    #[serde(default)]
    pub adif_import_directory: String,
    #[serde(default)]
    pub adif_export_directory: String,
    #[serde(default)]
    pub backup_directory: String,
}

impl OperationalConfig {
    pub fn sanitized_active_page(&self) -> i32 {
        if (0..=3).contains(&self.active_page) {
            self.active_page
        } else {
            0
        }
    }

    pub fn sanitized_active_filter(&self) -> i32 {
        if (0..=2).contains(&self.active_filter) {
            self.active_filter
        } else {
            0
        }
    }

    pub fn existing_directory(value: &str) -> Option<PathBuf> {
        let path = PathBuf::from(value);
        path.is_dir().then_some(path)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationConfig {
    #[serde(default)]
    pub callsign: String,
}

pub const DEFAULT_CALLSIGN_URL: &str = "https://www.qrz.com/db/{callsign}";
pub const DEFAULT_GRID_URL: &str = "https://www.levinecentral.com/ham/grid_square.php?Grid={grid}";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalLinksConfig {
    #[serde(default = "default_callsign_url")]
    pub callsign_url: String,
    #[serde(default = "default_grid_url")]
    pub grid_url: String,
}

impl Default for ExternalLinksConfig {
    fn default() -> Self {
        Self {
            callsign_url: DEFAULT_CALLSIGN_URL.into(),
            grid_url: DEFAULT_GRID_URL.into(),
        }
    }
}

fn default_callsign_url() -> String {
    DEFAULT_CALLSIGN_URL.into()
}

fn default_grid_url() -> String {
    DEFAULT_GRID_URL.into()
}

impl AppConfig {
    pub fn set_callsign(&mut self, callsign: impl Into<String>) -> Result<(), ConfigError> {
        let callsign = callsign.into().trim().to_uppercase();
        if callsign.is_empty() {
            return Err(ConfigError::EmptyCallsign);
        }
        self.station.callsign = callsign;
        Ok(())
    }

    pub fn set_external_links(
        &mut self,
        callsign_url: impl Into<String>,
        grid_url: impl Into<String>,
    ) -> Result<(), ConfigError> {
        let callsign_url = callsign_url.into().trim().to_owned();
        let grid_url = grid_url.into().trim().to_owned();
        validate_url_template(&callsign_url, "{callsign}")?;
        validate_url_template(&grid_url, "{grid}")?;
        self.external_links = ExternalLinksConfig {
            callsign_url,
            grid_url,
        };
        Ok(())
    }
}

pub fn expand_url_template(
    template: &str,
    placeholder: &'static str,
    value: &str,
) -> Result<String, ConfigError> {
    validate_url_template(template, placeholder)?;
    Ok(template.replace(placeholder, &urlencoding::encode(value)))
}

fn validate_url_template(template: &str, placeholder: &'static str) -> Result<(), ConfigError> {
    if !(template.starts_with("https://") || template.starts_with("http://")) {
        return Err(ConfigError::InvalidUrlScheme);
    }
    if !template.contains(placeholder) {
        return Err(ConfigError::MissingPlaceholder(placeholder));
    }
    Ok(())
}

pub fn load(path: &Path) -> Result<AppConfig, Box<dyn Error>> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(toml::from_str(&contents)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
        Err(error) => Err(error.into()),
    }
}

pub fn save(path: &Path, config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let parent = path
        .parent()
        .ok_or("configuration path has no parent directory")?;
    fs::create_dir_all(parent)?;
    let temporary_path = temporary_path(path)?;
    let contents = toml::to_string_pretty(config)?;

    let result = (|| -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)?;
        set_private_file_permissions(&file)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary_path, path)?;
        sync_parent_directory(parent)?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn temporary_path(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("configuration filename is invalid")?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce)))
}

#[cfg(unix)]
fn set_private_file_permissions(file: &std::fs::File) -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(0o600);
    file.set_permissions(permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_file_permissions(_file: &std::fs::File) -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> Result<(), Box<dyn Error>> {
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    EmptyCallsign,
    InvalidUrlScheme,
    MissingPlaceholder(&'static str),
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCallsign => formatter.write_str("Local station callsign is required"),
            Self::InvalidUrlScheme => {
                formatter.write_str("Lookup URL must start with http:// or https://")
            }
            Self::MissingPlaceholder(placeholder) => {
                write!(formatter, "Lookup URL must contain {placeholder}")
            }
        }
    }
}

impl Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn normalizes_and_validates_callsign() {
        let mut config = AppConfig::default();
        config.set_callsign(" pu2xyz ").unwrap();
        assert_eq!(config.station.callsign, "PU2XYZ");
        assert_eq!(config.set_callsign(""), Err(ConfigError::EmptyCallsign));
    }

    #[test]
    fn validates_and_expands_external_link_templates() {
        let mut config = AppConfig::default();
        config
            .set_external_links(
                "https://example.com/call/{callsign}",
                "https://example.com/grid?q={grid}",
            )
            .unwrap();
        assert_eq!(
            expand_url_template(&config.external_links.callsign_url, "{callsign}", "PU2/ABC")
                .unwrap(),
            "https://example.com/call/PU2%2FABC"
        );
        assert_eq!(
            config.set_external_links("file:///{callsign}", DEFAULT_GRID_URL),
            Err(ConfigError::InvalidUrlScheme)
        );
        assert_eq!(
            config.set_external_links("https://example.com/call", DEFAULT_GRID_URL),
            Err(ConfigError::MissingPlaceholder("{callsign}"))
        );
    }

    #[test]
    fn old_configuration_uses_new_section_defaults() {
        let config: AppConfig = toml::from_str("[station]\ncallsign = 'PY2ABC'\n").unwrap();
        assert_eq!(config.external_links, ExternalLinksConfig::default());
        assert_eq!(config.operational, OperationalConfig::default());
    }

    #[test]
    fn sanitizes_operational_navigation_values_and_directories() {
        let mut operational = OperationalConfig {
            active_page: 99,
            active_filter: -1,
            ..Default::default()
        };
        assert_eq!(operational.sanitized_active_page(), 0);
        assert_eq!(operational.sanitized_active_filter(), 0);
        assert!(OperationalConfig::existing_directory("/path/that/does/not/exist").is_none());

        operational.active_page = 3;
        operational.active_filter = 2;
        assert_eq!(operational.sanitized_active_page(), 3);
        assert_eq!(operational.sanitized_active_filter(), 2);
    }

    #[test]
    fn saves_and_loads_configuration_atomically() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("dhrl-config-test-{suffix}"));
        let path = directory.join("config.toml");
        let mut config = AppConfig::default();
        config.set_callsign("PY2ABC").unwrap();
        config.operational = OperationalConfig {
            active_page: 2,
            active_filter: 1,
            filters_expanded: true,
            adif_import_directory: "/tmp/import".into(),
            adif_export_directory: "/tmp/export".into(),
            backup_directory: "/tmp/backup".into(),
        };

        save(&path, &config).unwrap();
        assert_eq!(load(&path).unwrap(), config);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn invalid_and_truncated_configuration_is_rejected_without_modification() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("dhrl-invalid-config-{suffix}"));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("config.toml");

        for contents in ["[station", "[station]\ncallsign = 42"] {
            fs::write(&path, contents).unwrap();
            assert!(load(&path).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), contents);
        }
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn saved_configuration_uses_private_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("dhrl-config-mode-{suffix}"));
        let path = directory.join("config.toml");
        save(&path, &AppConfig::default()).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn saves_configuration_in_a_unicode_path_with_spaces() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("Configuração Rádio 日本 {suffix}"));
        let path = directory.join("config.toml");
        let mut config = AppConfig::default();
        config.set_callsign("PY2ABC").unwrap();

        save(&path, &config).unwrap();
        assert_eq!(load(&path).unwrap(), config);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn missing_configuration_uses_defaults() {
        let path = std::env::temp_dir().join("dhrl-nonexistent-config-test.toml");
        let _ = fs::remove_file(&path);
        assert_eq!(load(&path).unwrap(), AppConfig::default());
    }
}
