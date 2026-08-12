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
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationConfig {
    #[serde(default)]
    pub callsign: String,
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
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCallsign => formatter.write_str("Local station callsign is required"),
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
    fn saves_and_loads_configuration_atomically() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("dhrl-config-test-{suffix}"));
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
