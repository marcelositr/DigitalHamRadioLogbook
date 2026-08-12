use std::time::{SystemTime, UNIX_EPOCH};

pub fn info(message: &str) {
    write("INFO", message);
}

pub fn error(message: &str) {
    write("ERROR", message);
}

fn write(level: &str, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    eprintln!("{timestamp} {level} {message}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logger_accepts_operational_messages() {
        info("test startup");
        error("test error");
    }
}
