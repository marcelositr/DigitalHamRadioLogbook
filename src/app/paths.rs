use super::*;

pub(crate) fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    let config_home = match env::var_os("XDG_CONFIG_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => {
            let home = env::var_os("HOME").ok_or("HOME is not defined")?;
            PathBuf::from(home).join(".config")
        }
    };
    Ok(config_home.join("digital-ham-log/config.toml"))
}

pub(crate) fn database_path() -> Result<PathBuf, Box<dyn Error>> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => {
            let home = env::var_os("HOME").ok_or("HOME is not defined")?;
            PathBuf::from(home).join(".local/share")
        }
    };

    let application_directory = data_home.join("digital-ham-log");
    fs::create_dir_all(&application_directory)?;
    Ok(application_directory.join("logbook.sqlite3"))
}
