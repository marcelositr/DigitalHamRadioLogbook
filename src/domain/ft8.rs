use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ft8Metadata {
    pub snr_sent_db: Option<i16>,
    pub snr_received_db: Option<i16>,
    pub power_watts: Option<u32>,
    pub audio_frequency_hz: Option<u32>,
    pub source_software: Option<String>,
    pub protocol: Option<String>,
    pub final_message: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ft8MetadataInput {
    pub snr_sent_db: String,
    pub snr_received_db: String,
    pub power_watts: String,
    pub audio_frequency_hz: String,
    pub source_software: String,
    pub protocol: String,
    pub final_message: String,
}

impl Ft8Metadata {
    pub fn from_input(input: Ft8MetadataInput) -> Result<Self, Ft8ValidationError> {
        Ok(Self {
            snr_sent_db: optional_snr(&input.snr_sent_db)?,
            snr_received_db: optional_snr(&input.snr_received_db)?,
            power_watts: optional_positive_u32(
                &input.power_watts,
                Ft8ValidationError::InvalidPower,
            )?,
            audio_frequency_hz: optional_positive_u32(
                &input.audio_frequency_hz,
                Ft8ValidationError::InvalidAudioFrequency,
            )?,
            source_software: optional_string(input.source_software),
            protocol: optional_string(input.protocol),
            final_message: optional_string(input.final_message),
        })
    }
}

fn optional_snr(value: &str) -> Result<Option<i16>, Ft8ValidationError> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    let snr = value
        .trim()
        .parse::<i16>()
        .map_err(|_| Ft8ValidationError::InvalidSnr)?;
    if !(-50..=50).contains(&snr) {
        return Err(Ft8ValidationError::InvalidSnr);
    }
    Ok(Some(snr))
}

fn optional_positive_u32(
    value: &str,
    error: Ft8ValidationError,
) -> Result<Option<u32>, Ft8ValidationError> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    let value = value.trim().parse::<u32>().map_err(|_| error)?;
    if value == 0 {
        return Err(error);
    }
    Ok(Some(value))
}

fn optional_string(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ft8ValidationError {
    InvalidSnr,
    InvalidPower,
    InvalidAudioFrequency,
}

impl Display for Ft8ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSnr => formatter.write_str("FT8 SNR must be between -50 and 50 dB"),
            Self::InvalidPower => formatter.write_str("FT8 power must be a positive integer"),
            Self::InvalidAudioFrequency => {
                formatter.write_str("FT8 audio frequency must be a positive integer")
            }
        }
    }
}

impl Error for Ft8ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_normalizes_ft8_metadata() {
        let metadata = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_sent_db: "-12".into(),
            snr_received_db: "-18".into(),
            power_watts: "25".into(),
            audio_frequency_hz: "1500".into(),
            source_software: " WSJT-X ".into(),
            protocol: " FT8 ".into(),
            final_message: " RR73 ".into(),
        })
        .unwrap();

        assert_eq!(metadata.snr_sent_db, Some(-12));
        assert_eq!(metadata.power_watts, Some(25));
        assert_eq!(metadata.source_software.as_deref(), Some("WSJT-X"));
        assert_eq!(metadata.final_message.as_deref(), Some("RR73"));
    }

    #[test]
    fn permits_empty_optional_fields() {
        let metadata = Ft8Metadata::from_input(Ft8MetadataInput::default()).unwrap();
        assert_eq!(metadata.snr_received_db, None);
        assert_eq!(metadata.power_watts, None);
        assert_eq!(metadata.protocol, None);
    }

    #[test]
    fn rejects_invalid_numeric_fields() {
        let invalid_snr = Ft8MetadataInput {
            snr_received_db: "-60".into(),
            ..Default::default()
        };
        assert_eq!(
            Ft8Metadata::from_input(invalid_snr),
            Err(Ft8ValidationError::InvalidSnr)
        );

        let invalid_power = Ft8MetadataInput {
            power_watts: "0".into(),
            ..Default::default()
        };
        assert_eq!(
            Ft8Metadata::from_input(invalid_power),
            Err(Ft8ValidationError::InvalidPower)
        );
    }
}
