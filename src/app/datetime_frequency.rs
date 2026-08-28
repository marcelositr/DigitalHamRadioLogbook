use super::*;

pub(crate) fn current_utc_timestamp() -> Result<i64, Box<dyn Error>> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(i64::try_from(seconds)?)
}

pub(crate) fn parse_utc_datetime(input: &str) -> Result<i64, Box<dyn Error>> {
    let input = input.trim().strip_suffix(" UTC").unwrap_or(input.trim());
    let datetime = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| "Enter date/time as YYYY-MM-DD HH:MM:SS UTC")?;
    Ok(datetime.and_utc().timestamp())
}

pub(crate) fn format_utc_datetime(timestamp: i64) -> Result<String, Box<dyn Error>> {
    let datetime: DateTime<Utc> =
        DateTime::from_timestamp(timestamp, 0).ok_or("UTC timestamp is out of range")?;
    Ok(datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string())
}

pub(crate) fn parse_mhz_to_hz(input: &str) -> Result<i64, Box<dyn Error>> {
    let normalized = input.trim().replace(',', ".");
    if normalized.starts_with('-') {
        return Err("Frequency must be greater than zero".into());
    }
    let mut parts = normalized.split('.');
    let whole = parts.next().ok_or("Frequency is required")?;
    let fraction = parts.next().unwrap_or("");

    if whole.is_empty() || parts.next().is_some() || fraction.len() > 6 {
        return Err("Enter a valid frequency in MHz".into());
    }

    let whole_hz = whole
        .parse::<i64>()?
        .checked_mul(1_000_000)
        .ok_or("Frequency is too large")?;
    let fraction_hz = if fraction.is_empty() {
        0
    } else {
        format!("{fraction:0<6}").parse::<i64>()?
    };
    let frequency_hz = whole_hz
        .checked_add(fraction_hz)
        .ok_or("Frequency is too large")?;

    if frequency_hz <= 0 {
        return Err("Frequency must be greater than zero".into());
    }
    Ok(frequency_hz)
}

pub(crate) fn format_frequency(frequency_hz: i64) -> String {
    format!("{} MHz", format_frequency_input(frequency_hz))
}

pub(crate) fn format_frequency_input(frequency_hz: i64) -> String {
    let whole = frequency_hz / 1_000_000;
    let fraction = frequency_hz.rem_euclid(1_000_000);
    format!("{whole}.{fraction:06}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_utc_datetime() {
        let timestamp = parse_utc_datetime("2023-11-14 22:13:20 UTC").unwrap();
        assert_eq!(timestamp, 1_700_000_000);
        assert_eq!(
            format_utc_datetime(timestamp).unwrap(),
            "2023-11-14 22:13:20 UTC"
        );
        assert!(parse_utc_datetime("14/11/2023 22:13").is_err());
    }

    #[test]
    fn handles_utc_day_boundary_leap_day_and_invalid_dates() {
        let before_midnight = parse_utc_datetime("2028-02-28 23:59:59 UTC").unwrap();
        let leap_day = parse_utc_datetime("2028-02-29 00:00:00 UTC").unwrap();
        assert_eq!(leap_day - before_midnight, 1);
        assert_eq!(
            format_utc_datetime(leap_day).unwrap(),
            "2028-02-29 00:00:00 UTC"
        );
        assert!(parse_utc_datetime("2027-02-29 00:00:00 UTC").is_err());
        assert!(parse_utc_datetime("2028-02-30 00:00:00 UTC").is_err());
        assert!(parse_utc_datetime("2028-02-29 24:00:00 UTC").is_err());
    }

    #[test]
    fn parses_mhz_without_floating_point() {
        assert_eq!(parse_mhz_to_hz("438.500").unwrap(), 438_500_000);
        assert_eq!(parse_mhz_to_hz("14,074").unwrap(), 14_074_000);
        assert_eq!(parse_mhz_to_hz("145").unwrap(), 145_000_000);
    }

    #[test]
    fn rejects_invalid_frequency() {
        assert!(parse_mhz_to_hz("").is_err());
        assert!(parse_mhz_to_hz("145.1234567").is_err());
        assert!(parse_mhz_to_hz("145.5.1").is_err());
        assert!(parse_mhz_to_hz("-0.5").is_err());
        assert!(parse_mhz_to_hz("-145.5").is_err());
    }

    #[test]
    fn formats_hz_as_mhz() {
        assert_eq!(format_frequency(438_500_000), "438.500000 MHz");
        assert_eq!(format_frequency_input(14_074_000), "14.074000");
    }
}
