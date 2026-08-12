use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmrCallType {
    Group,
    Private,
}

impl DmrCallType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Group => "group",
            Self::Private => "private",
        }
    }
}

impl FromStr for DmrCallType {
    type Err = DmrValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "group" => Ok(Self::Group),
            "private" => Ok(Self::Private),
            _ => Err(DmrValidationError::InvalidCallType),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmrAccessType {
    Repeater,
    Hotspot,
    Simplex,
}

impl DmrAccessType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Repeater => "repeater",
            Self::Hotspot => "hotspot",
            Self::Simplex => "simplex",
        }
    }
}

impl FromStr for DmrAccessType {
    type Err = DmrValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "repeater" => Ok(Self::Repeater),
            "hotspot" => Ok(Self::Hotspot),
            "simplex" => Ok(Self::Simplex),
            _ => Err(DmrValidationError::InvalidAccessType),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmrMetadata {
    pub remote_dmr_id: Option<u32>,
    pub local_dmr_id: Option<u32>,
    pub talkgroup: Option<u32>,
    pub timeslot: Option<u8>,
    pub color_code: Option<u8>,
    pub network: Option<String>,
    pub call_type: DmrCallType,
    pub access_type: DmrAccessType,
    pub repeater_callsign: Option<String>,
    pub hotspot: Option<String>,
    pub rx_frequency_hz: Option<i64>,
    pub tx_frequency_hz: Option<i64>,
    pub notes: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DmrMetadataInput {
    pub remote_dmr_id: String,
    pub local_dmr_id: String,
    pub talkgroup: String,
    pub timeslot: String,
    pub color_code: String,
    pub network: String,
    pub call_type: String,
    pub access_type: String,
    pub repeater_callsign: String,
    pub hotspot: String,
    pub rx_frequency_hz: Option<i64>,
    pub tx_frequency_hz: Option<i64>,
    pub notes: String,
}

impl DmrMetadata {
    pub fn from_input(input: DmrMetadataInput) -> Result<Self, DmrValidationError> {
        let call_type = input.call_type.parse()?;
        let access_type = input.access_type.parse()?;
        let repeater_callsign = optional_uppercase(input.repeater_callsign);
        let hotspot = optional_string(input.hotspot);

        if access_type == DmrAccessType::Repeater && repeater_callsign.is_none() {
            return Err(DmrValidationError::RepeaterCallsignRequired);
        }
        if access_type == DmrAccessType::Hotspot && hotspot.is_none() {
            return Err(DmrValidationError::HotspotRequired);
        }

        Ok(Self {
            remote_dmr_id: optional_positive_u32(
                &input.remote_dmr_id,
                DmrValidationError::InvalidDmrId,
            )?,
            local_dmr_id: optional_positive_u32(
                &input.local_dmr_id,
                DmrValidationError::InvalidDmrId,
            )?,
            talkgroup: optional_positive_u32(
                &input.talkgroup,
                DmrValidationError::InvalidTalkgroup,
            )?,
            timeslot: optional_ranged_u8(
                &input.timeslot,
                1,
                2,
                DmrValidationError::InvalidTimeslot,
            )?,
            color_code: optional_ranged_u8(
                &input.color_code,
                0,
                15,
                DmrValidationError::InvalidColorCode,
            )?,
            network: optional_string(input.network),
            call_type,
            access_type,
            repeater_callsign,
            hotspot,
            rx_frequency_hz: validate_frequency(input.rx_frequency_hz)?,
            tx_frequency_hz: validate_frequency(input.tx_frequency_hz)?,
            notes: input.notes.trim().to_owned(),
        })
    }
}

fn optional_positive_u32(
    value: &str,
    error: DmrValidationError,
) -> Result<Option<u32>, DmrValidationError> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    let number = value.trim().parse::<u32>().map_err(|_| error)?;
    if number == 0 {
        return Err(error);
    }
    Ok(Some(number))
}

fn optional_ranged_u8(
    value: &str,
    minimum: u8,
    maximum: u8,
    error: DmrValidationError,
) -> Result<Option<u8>, DmrValidationError> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    let number = value.trim().parse::<u8>().map_err(|_| error)?;
    if !(minimum..=maximum).contains(&number) {
        return Err(error);
    }
    Ok(Some(number))
}

fn validate_frequency(value: Option<i64>) -> Result<Option<i64>, DmrValidationError> {
    match value {
        Some(value) if value <= 0 => Err(DmrValidationError::InvalidFrequency),
        _ => Ok(value),
    }
}

fn optional_string(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn optional_uppercase(value: String) -> Option<String> {
    optional_string(value).map(|value| value.to_uppercase())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmrValidationError {
    InvalidDmrId,
    InvalidTalkgroup,
    InvalidTimeslot,
    InvalidColorCode,
    InvalidCallType,
    InvalidAccessType,
    RepeaterCallsignRequired,
    HotspotRequired,
    InvalidFrequency,
}

impl Display for DmrValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDmrId => formatter.write_str("DMR ID must be a positive integer"),
            Self::InvalidTalkgroup => formatter.write_str("Talkgroup must be a positive integer"),
            Self::InvalidTimeslot => formatter.write_str("Timeslot must be 1 or 2"),
            Self::InvalidColorCode => formatter.write_str("Color code must be between 0 and 15"),
            Self::InvalidCallType => formatter.write_str("Call type must be group or private"),
            Self::InvalidAccessType => {
                formatter.write_str("Access type must be repeater, hotspot or simplex")
            }
            Self::RepeaterCallsignRequired => {
                formatter.write_str("Repeater callsign is required for repeater access")
            }
            Self::HotspotRequired => formatter.write_str("Hotspot is required for hotspot access"),
            Self::InvalidFrequency => {
                formatter.write_str("DMR RX/TX frequency must be greater than zero")
            }
        }
    }
}

impl Error for DmrValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> DmrMetadataInput {
        DmrMetadataInput {
            remote_dmr_id: "7241234".into(),
            local_dmr_id: "7240001".into(),
            talkgroup: "724".into(),
            timeslot: "1".into(),
            color_code: "1".into(),
            network: "BrandMeister".into(),
            call_type: "group".into(),
            access_type: "repeater".into(),
            repeater_callsign: " py2xyz ".into(),
            ..Default::default()
        }
    }

    #[test]
    fn validates_and_normalizes_dmr_metadata() {
        let metadata = DmrMetadata::from_input(valid_input()).unwrap();

        assert_eq!(metadata.remote_dmr_id, Some(7_241_234));
        assert_eq!(metadata.talkgroup, Some(724));
        assert_eq!(metadata.timeslot, Some(1));
        assert_eq!(metadata.repeater_callsign.as_deref(), Some("PY2XYZ"));
        assert_eq!(metadata.network.as_deref(), Some("BrandMeister"));
    }

    #[test]
    fn rejects_invalid_dmr_ranges() {
        let mut timeslot = valid_input();
        timeslot.timeslot = "3".into();
        assert_eq!(
            DmrMetadata::from_input(timeslot),
            Err(DmrValidationError::InvalidTimeslot)
        );

        let mut color_code = valid_input();
        color_code.color_code = "16".into();
        assert_eq!(
            DmrMetadata::from_input(color_code),
            Err(DmrValidationError::InvalidColorCode)
        );
    }

    #[test]
    fn requires_access_specific_identification() {
        let mut repeater = valid_input();
        repeater.repeater_callsign.clear();
        assert_eq!(
            DmrMetadata::from_input(repeater),
            Err(DmrValidationError::RepeaterCallsignRequired)
        );

        let mut hotspot = valid_input();
        hotspot.access_type = "hotspot".into();
        hotspot.repeater_callsign.clear();
        assert_eq!(
            DmrMetadata::from_input(hotspot),
            Err(DmrValidationError::HotspotRequired)
        );
    }

    #[test]
    fn allows_open_network_names_and_simplex() {
        let mut input = valid_input();
        input.network = "Local experimental network".into();
        input.access_type = "simplex".into();
        input.repeater_callsign.clear();

        let metadata = DmrMetadata::from_input(input).unwrap();
        assert_eq!(metadata.access_type, DmrAccessType::Simplex);
        assert_eq!(
            metadata.network.as_deref(),
            Some("Local experimental network")
        );
    }
}
