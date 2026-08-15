use super::*;

pub(crate) fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    let config_home = xdg_home("XDG_CONFIG_HOME", ".config")?;
    Ok(config_home.join("digital-ham-log/config.toml"))
}

pub(crate) fn database_path() -> Result<PathBuf, Box<dyn Error>> {
    let data_home = xdg_home("XDG_DATA_HOME", ".local/share")?;
    let application_directory = data_home.join("digital-ham-log");
    fs::create_dir_all(&application_directory)?;
    Ok(application_directory.join("logbook.sqlite3"))
}

fn xdg_home(variable: &str, fallback: &str) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(path) = env::var_os(variable).filter(|path| !path.is_empty()) {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Ok(path);
        }
    }

    let home = env::var_os("HOME").ok_or("HOME is not defined")?;
    let home = PathBuf::from(home);
    if !home.is_absolute() {
        return Err("HOME must be an absolute path".into());
    }
    Ok(home.join(fallback))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};

    static ENVIRONMENT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvironmentGuard {
        values: Vec<(&'static str, Option<OsString>)>,
    }

    impl EnvironmentGuard {
        fn set(values: &[(&'static str, Option<&std::ffi::OsStr>)]) -> Self {
            let previous = values
                .iter()
                .map(|(name, _)| (*name, env::var_os(name)))
                .collect();
            for (name, value) in values {
                match value {
                    Some(value) => env::set_var(name, value),
                    None => env::remove_var(name),
                }
            }
            Self { values: previous }
        }
    }

    impl Drop for EnvironmentGuard {
        fn drop(&mut self) {
            for (name, value) in &self.values {
                match value {
                    Some(value) => env::set_var(name, value),
                    None => env::remove_var(name),
                }
            }
        }
    }

    #[test]
    fn uses_absolute_xdg_paths_with_spaces_and_unicode() {
        let _lock = ENVIRONMENT_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        let root = std::env::temp_dir().join("DHRL Configuração Rádio 日本");
        let config = root.join("config home");
        let data = root.join("data home");
        let _guard = EnvironmentGuard::set(&[
            ("XDG_CONFIG_HOME", Some(config.as_os_str())),
            ("XDG_DATA_HOME", Some(data.as_os_str())),
            ("HOME", None),
        ]);

        assert_eq!(
            config_path().unwrap(),
            config.join("digital-ham-log/config.toml")
        );
        assert_eq!(
            database_path().unwrap(),
            data.join("digital-ham-log/logbook.sqlite3")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ignores_relative_xdg_paths_and_uses_absolute_home_fallbacks() {
        let _lock = ENVIRONMENT_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        let home = std::env::temp_dir().join("dhrl-xdg-fallback");
        let _guard = EnvironmentGuard::set(&[
            (
                "XDG_CONFIG_HOME",
                Some(std::ffi::OsStr::new("relative-config")),
            ),
            ("XDG_DATA_HOME", Some(std::ffi::OsStr::new("relative-data"))),
            ("HOME", Some(home.as_os_str())),
        ]);

        assert_eq!(
            config_path().unwrap(),
            home.join(".config/digital-ham-log/config.toml")
        );
        assert_eq!(
            database_path().unwrap(),
            home.join(".local/share/digital-ham-log/logbook.sqlite3")
        );
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn rejects_missing_empty_or_relative_home_without_absolute_xdg_paths() {
        let _lock = ENVIRONMENT_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        for home in [
            None,
            Some(std::ffi::OsStr::new("")),
            Some(std::ffi::OsStr::new("relative")),
        ] {
            let _guard = EnvironmentGuard::set(&[
                ("XDG_CONFIG_HOME", None),
                ("XDG_DATA_HOME", None),
                ("HOME", home),
            ]);
            assert!(config_path().is_err());
            assert!(database_path().is_err());
        }
    }

    #[test]
    fn database_path_rejects_a_file_where_the_application_directory_should_be() {
        let _lock = ENVIRONMENT_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        let root = std::env::temp_dir().join(format!("dhrl-xdg-file-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("digital-ham-log"), b"not a directory").unwrap();
        let _guard =
            EnvironmentGuard::set(&[("XDG_DATA_HOME", Some(root.as_os_str())), ("HOME", None)]);

        assert!(database_path().is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
