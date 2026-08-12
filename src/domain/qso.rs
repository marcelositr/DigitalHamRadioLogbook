use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qso {
    pub id: i64,
    pub callsign: String,
    pub datetime_start_utc: i64,
    pub datetime_end_utc: Option<i64>,
    pub frequency_hz: i64,
    pub band: Option<String>,
    pub mode: String,
    pub submode: Option<String>,
    pub rst_sent: Option<String>,
    pub rst_received: Option<String>,
    pub grid_locator: Option<String>,
    pub name: Option<String>,
    pub qth: Option<String>,
    pub notes: String,
    pub created_at_utc: i64,
    pub updated_at_utc: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewQso {
    pub callsign: String,
    pub datetime_start_utc: i64,
    pub frequency_hz: i64,
    pub band: Option<String>,
    pub mode: String,
    pub rst_sent: Option<String>,
    pub rst_received: Option<String>,
    pub grid_locator: Option<String>,
    pub name: Option<String>,
    pub qth: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommonQsoFields {
    pub band_override: String,
    pub rst_sent: String,
    pub rst_received: String,
    pub grid_locator: String,
    pub name: String,
    pub qth: String,
    pub notes: String,
}

impl NewQso {
    pub fn new(
        callsign: impl Into<String>,
        datetime_start_utc: i64,
        frequency_hz: i64,
        mode: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let callsign = callsign.into().trim().to_uppercase();
        let mode = mode.into().trim().to_uppercase();

        if callsign.is_empty() {
            return Err(ValidationError::EmptyCallsign);
        }
        if frequency_hz <= 0 {
            return Err(ValidationError::InvalidFrequency);
        }
        if mode.is_empty() {
            return Err(ValidationError::EmptyMode);
        }

        Ok(Self {
            callsign,
            datetime_start_utc,
            frequency_hz,
            band: derive_band(frequency_hz).map(str::to_owned),
            mode,
            rst_sent: None,
            rst_received: None,
            grid_locator: None,
            name: None,
            qth: None,
            notes: String::new(),
        })
    }

    pub fn with_common_fields(mut self, fields: CommonQsoFields) -> Result<Self, ValidationError> {
        self.band = optional_string(fields.band_override).or(self.band);
        self.rst_sent = optional_string(fields.rst_sent);
        self.rst_received = optional_string(fields.rst_received);
        self.grid_locator = normalize_grid(fields.grid_locator)?;
        self.name = optional_string(fields.name);
        self.qth = optional_string(fields.qth);
        self.notes = fields.notes.trim().to_owned();
        Ok(self)
    }
}

pub fn derive_band(frequency_hz: i64) -> Option<&'static str> {
    const BANDS: &[(i64, i64, &str)] = &[
        (1_800_000, 2_000_000, "160m"),
        (3_500_000, 4_000_000, "80m"),
        (5_250_000, 5_450_000, "60m"),
        (7_000_000, 7_300_000, "40m"),
        (10_100_000, 10_150_000, "30m"),
        (14_000_000, 14_350_000, "20m"),
        (18_068_000, 18_168_000, "17m"),
        (21_000_000, 21_450_000, "15m"),
        (24_890_000, 24_990_000, "12m"),
        (28_000_000, 29_700_000, "10m"),
        (50_000_000, 54_000_000, "6m"),
        (144_000_000, 148_000_000, "2m"),
        (420_000_000, 450_000_000, "70cm"),
        (1_240_000_000, 1_300_000_000, "23cm"),
    ];

    BANDS
        .iter()
        .find(|(start, end, _)| (*start..=*end).contains(&frequency_hz))
        .map(|(_, _, band)| *band)
}

fn optional_string(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn normalize_grid(value: String) -> Result<Option<String>, ValidationError> {
    let value = value.trim().to_uppercase();
    if value.is_empty() {
        return Ok(None);
    }

    let bytes = value.as_bytes();
    let valid_length = matches!(bytes.len(), 2 | 4 | 6 | 8);
    let valid = valid_length
        && matches!(bytes[0], b'A'..=b'R')
        && matches!(bytes[1], b'A'..=b'R')
        && (bytes.len() < 4 || (bytes[2].is_ascii_digit() && bytes[3].is_ascii_digit()))
        && (bytes.len() < 6
            || (matches!(bytes[4], b'A'..=b'X') && matches!(bytes[5], b'A'..=b'X')))
        && (bytes.len() < 8 || (bytes[6].is_ascii_digit() && bytes[7].is_ascii_digit()));

    if !valid {
        return Err(ValidationError::InvalidGridLocator);
    }
    Ok(Some(value))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    EmptyCallsign,
    InvalidFrequency,
    EmptyMode,
    InvalidGridLocator,
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCallsign => formatter.write_str("Callsign is required"),
            Self::InvalidFrequency => formatter.write_str("Frequency must be greater than zero"),
            Self::EmptyMode => formatter.write_str("Mode is required"),
            Self::InvalidGridLocator => formatter.write_str("Grid locator is invalid"),
        }
    }
}

impl Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_callsign_and_mode() {
        let qso = NewQso::new(" pu2xyz ", 1_700_000_000, 438_500_000, " dmr ").unwrap();

        assert_eq!(qso.callsign, "PU2XYZ");
        assert_eq!(qso.mode, "DMR");
        assert_eq!(qso.band.as_deref(), Some("70cm"));
    }

    #[test]
    fn derives_common_amateur_bands() {
        assert_eq!(derive_band(14_074_000), Some("20m"));
        assert_eq!(derive_band(145_500_000), Some("2m"));
        assert_eq!(derive_band(438_500_000), Some("70cm"));
        assert_eq!(derive_band(100_000_000), None);
    }

    #[test]
    fn normalizes_and_validates_optional_grid() {
        let qso = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR")
            .unwrap()
            .with_common_fields(CommonQsoFields {
                rst_sent: "59".into(),
                rst_received: "59".into(),
                grid_locator: " gg66aa ".into(),
                name: "Marcelo".into(),
                qth: "SP".into(),
                notes: "Test".into(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(qso.grid_locator.as_deref(), Some("GG66AA"));
        assert_eq!(qso.notes, "Test");

        let invalid = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR")
            .unwrap()
            .with_common_fields(CommonQsoFields {
                grid_locator: "ZZ99".into(),
                ..Default::default()
            });
        assert_eq!(invalid, Err(ValidationError::InvalidGridLocator));
    }

    #[test]
    fn rejects_invalid_required_fields() {
        assert_eq!(
            NewQso::new("", 1_700_000_000, 438_500_000, "DMR"),
            Err(ValidationError::EmptyCallsign)
        );
        assert_eq!(
            NewQso::new("PU2XYZ", 1_700_000_000, 0, "DMR"),
            Err(ValidationError::InvalidFrequency)
        );
        assert_eq!(
            NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, ""),
            Err(ValidationError::EmptyMode)
        );
    }
}
