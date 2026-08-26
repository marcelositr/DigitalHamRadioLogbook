use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum YsfAccessType {
    #[default]
    Simplex,
    Repeater,
    Hotspot,
}

impl YsfAccessType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Simplex => "simplex",
            Self::Repeater => "repeater",
            Self::Hotspot => "hotspot",
        }
    }
}

impl FromStr for YsfAccessType {
    type Err = YsfValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.chars().any(char::is_control) {
            return Err(YsfValidationError::InvalidAccessType);
        }

        match value.trim().to_ascii_lowercase().as_str() {
            "" | "simplex" => Ok(Self::Simplex),
            "repeater" => Ok(Self::Repeater),
            "hotspot" => Ok(Self::Hotspot),
            _ => Err(YsfValidationError::InvalidAccessType),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YsfMetadata {
    pub room: Option<String>,
    pub wires_x_node: Option<String>,
    pub repeater: Option<String>,
    pub network: Option<String>,
    pub access_type: YsfAccessType,
    pub tx_dg_id: Option<u8>,
    pub rx_dg_id: Option<u8>,
    pub notes: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct YsfMetadataInput {
    pub room: String,
    pub wires_x_node: String,
    pub repeater: String,
    pub network: String,
    pub access_type: String,
    pub tx_dg_id: String,
    pub rx_dg_id: String,
    pub notes: String,
}

impl YsfMetadata {
    /// Trims all text, preserves the spelling/case of open YSF names, and
    /// uppercases only the repeater callsign/routing identifier.
    pub fn from_input(input: YsfMetadataInput) -> Result<Self, YsfValidationError> {
        let access_type = input.access_type.parse()?;
        let repeater = optional_text(input.repeater, YsfValidationError::InvalidRepeater, true)?;

        if access_type == YsfAccessType::Repeater && repeater.is_none() {
            return Err(YsfValidationError::RepeaterRequired);
        }

        Ok(Self {
            room: optional_text(input.room, YsfValidationError::InvalidRoom, false)?,
            wires_x_node: optional_text(
                input.wires_x_node,
                YsfValidationError::InvalidWiresXNode,
                false,
            )?,
            repeater,
            network: optional_text(input.network, YsfValidationError::InvalidNetwork, false)?,
            access_type,
            tx_dg_id: optional_dg_id(&input.tx_dg_id, YsfValidationError::InvalidTxDgId)?,
            rx_dg_id: optional_dg_id(&input.rx_dg_id, YsfValidationError::InvalidRxDgId)?,
            notes: required_text(input.notes, YsfValidationError::InvalidNotes)?,
        })
    }
}

fn optional_text(
    value: String,
    error: YsfValidationError,
    uppercase: bool,
) -> Result<Option<String>, YsfValidationError> {
    let value = required_text(value, error)?;
    if value.is_empty() {
        return Ok(None);
    }

    Ok(Some(if uppercase {
        value.to_uppercase()
    } else {
        value
    }))
}

fn required_text(value: String, error: YsfValidationError) -> Result<String, YsfValidationError> {
    if value.chars().any(char::is_control) {
        return Err(error);
    }
    Ok(value.trim().to_owned())
}

fn optional_dg_id(
    value: &str,
    error: YsfValidationError,
) -> Result<Option<u8>, YsfValidationError> {
    if value.chars().any(char::is_control) {
        return Err(error);
    }

    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(error);
    }

    let id = value.parse::<u8>().map_err(|_| error)?;
    if id > 99 {
        return Err(error);
    }
    Ok(Some(id))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YsfValidationError {
    InvalidRoom,
    InvalidWiresXNode,
    InvalidRepeater,
    InvalidNetwork,
    InvalidAccessType,
    InvalidTxDgId,
    InvalidRxDgId,
    InvalidNotes,
    RepeaterRequired,
}

impl Display for YsfValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRoom => {
                formatter.write_str("YSF room must not contain control characters")
            }
            Self::InvalidWiresXNode => {
                formatter.write_str("YSF Wires-X node must not contain control characters")
            }
            Self::InvalidRepeater => {
                formatter.write_str("YSF repeater must not contain control characters")
            }
            Self::InvalidNetwork => {
                formatter.write_str("YSF network must not contain control characters")
            }
            Self::InvalidAccessType => {
                formatter.write_str("YSF access type must be simplex, repeater or hotspot")
            }
            Self::InvalidTxDgId => formatter.write_str("YSF TX DG-ID must be between 00 and 99"),
            Self::InvalidRxDgId => formatter.write_str("YSF RX DG-ID must be between 00 and 99"),
            Self::InvalidNotes => {
                formatter.write_str("YSF notes must not contain control characters")
            }
            Self::RepeaterRequired => {
                formatter.write_str("YSF repeater is required for repeater access")
            }
        }
    }
}

impl Error for YsfValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_text_and_access_type() {
        let metadata = YsfMetadata::from_input(YsfMetadataInput {
            room: "  America-Link  ".into(),
            wires_x_node: "  Local Node  ".into(),
            repeater: "  py2xyz-rpt  ".into(),
            network: "  Yaesu System Fusion  ".into(),
            access_type: "  RePeAtEr  ".into(),
            tx_dg_id: " 01 ".into(),
            rx_dg_id: "99".into(),
            notes: "  clear audio  ".into(),
        })
        .unwrap();

        assert_eq!(metadata.room.as_deref(), Some("America-Link"));
        assert_eq!(metadata.wires_x_node.as_deref(), Some("Local Node"));
        assert_eq!(metadata.repeater.as_deref(), Some("PY2XYZ-RPT"));
        assert_eq!(metadata.network.as_deref(), Some("Yaesu System Fusion"));
        assert_eq!(metadata.access_type, YsfAccessType::Repeater);
        assert_eq!(metadata.tx_dg_id, Some(1));
        assert_eq!(metadata.rx_dg_id, Some(99));
        assert_eq!(metadata.notes, "clear audio");
    }

    #[test]
    fn permits_empty_optional_fields_and_defaults_to_simplex() {
        let metadata = YsfMetadata::from_input(YsfMetadataInput::default()).unwrap();

        assert_eq!(metadata.room, None);
        assert_eq!(metadata.wires_x_node, None);
        assert_eq!(metadata.repeater, None);
        assert_eq!(metadata.network, None);
        assert_eq!(metadata.access_type, YsfAccessType::Simplex);
        assert_eq!(metadata.tx_dg_id, None);
        assert_eq!(metadata.rx_dg_id, None);
        assert!(metadata.notes.is_empty());
    }

    #[test]
    fn accepts_dg_id_limits_zero_and_leading_zeroes() {
        for (value, expected) in [("0", 0), ("00", 0), ("01", 1), ("99", 99)] {
            let metadata = YsfMetadata::from_input(YsfMetadataInput {
                tx_dg_id: value.into(),
                rx_dg_id: value.into(),
                ..Default::default()
            })
            .unwrap();
            assert_eq!(metadata.tx_dg_id, Some(expected));
            assert_eq!(metadata.rx_dg_id, Some(expected));
        }

        for value in ["100", "-1", "+1", "1.0", "A1"] {
            let input = YsfMetadataInput {
                tx_dg_id: value.into(),
                ..Default::default()
            };
            assert_eq!(
                YsfMetadata::from_input(input),
                Err(YsfValidationError::InvalidTxDgId)
            );
        }
    }

    #[test]
    fn parses_access_type_case_insensitively_and_requires_repeater_conditionally() {
        for (value, expected) in [
            ("SIMPLEX", YsfAccessType::Simplex),
            ("Repeater", YsfAccessType::Repeater),
            ("hotSPOT", YsfAccessType::Hotspot),
        ] {
            assert_eq!(value.parse::<YsfAccessType>().unwrap(), expected);
        }

        let repeater = YsfMetadataInput {
            access_type: "repeater".into(),
            ..Default::default()
        };
        assert_eq!(
            YsfMetadata::from_input(repeater),
            Err(YsfValidationError::RepeaterRequired)
        );

        let hotspot = YsfMetadataInput {
            access_type: "hotspot".into(),
            ..Default::default()
        };
        assert!(YsfMetadata::from_input(hotspot).is_ok());
    }

    #[test]
    fn rejects_control_characters_in_every_string_field() {
        for field in 0..8 {
            let mut input = YsfMetadataInput::default();
            let expected = match field {
                0 => {
                    input.room = "room\nname".into();
                    YsfValidationError::InvalidRoom
                }
                1 => {
                    input.wires_x_node = "node\tname".into();
                    YsfValidationError::InvalidWiresXNode
                }
                2 => {
                    input.repeater = "PY2\rXYZ".into();
                    YsfValidationError::InvalidRepeater
                }
                3 => {
                    input.network = "YSF\0net".into();
                    YsfValidationError::InvalidNetwork
                }
                4 => {
                    input.access_type = "simplex\n".into();
                    YsfValidationError::InvalidAccessType
                }
                5 => {
                    input.tx_dg_id = "0\t1".into();
                    YsfValidationError::InvalidTxDgId
                }
                6 => {
                    input.rx_dg_id = "9\r9".into();
                    YsfValidationError::InvalidRxDgId
                }
                _ => {
                    input.notes = "note\u{7f}".into();
                    YsfValidationError::InvalidNotes
                }
            };
            assert_eq!(YsfMetadata::from_input(input), Err(expected));
        }
    }
}
