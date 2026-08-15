use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DStarMetadata {
    pub reflector: Option<String>,
    pub module: Option<String>,
    pub mycall: Option<String>,
    pub urcall: Option<String>,
    pub rpt1: Option<String>,
    pub rpt2: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DStarMetadataInput {
    pub reflector: String,
    pub module: String,
    pub mycall: String,
    pub urcall: String,
    pub rpt1: String,
    pub rpt2: String,
    pub notes: String,
}

impl DStarMetadata {
    pub fn from_input(input: DStarMetadataInput) -> Result<Self, DStarValidationError> {
        Ok(Self {
            reflector: optional_routing_value(
                input.reflector,
                DStarValidationError::InvalidReflector,
            )?,
            module: optional_module(input.module)?,
            mycall: optional_routing_value(input.mycall, DStarValidationError::InvalidMycall)?,
            urcall: optional_routing_value(input.urcall, DStarValidationError::InvalidUrcall)?,
            rpt1: optional_routing_value(input.rpt1, DStarValidationError::InvalidRpt1)?,
            rpt2: optional_routing_value(input.rpt2, DStarValidationError::InvalidRpt2)?,
            notes: input.notes.trim().to_owned(),
        })
    }
}

fn optional_routing_value(
    value: String,
    error: DStarValidationError,
) -> Result<Option<String>, DStarValidationError> {
    if value.chars().any(char::is_control) {
        return Err(error);
    }

    let value = value.trim();
    Ok((!value.is_empty()).then(|| value.to_uppercase()))
}

fn optional_module(value: String) -> Result<Option<String>, DStarValidationError> {
    if value.chars().any(char::is_control) {
        return Err(DStarValidationError::InvalidModule);
    }

    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }

    if value.len() != 1 || !value.as_bytes()[0].is_ascii_alphanumeric() {
        return Err(DStarValidationError::InvalidModule);
    }

    Ok(Some(value.to_ascii_uppercase()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DStarValidationError {
    InvalidReflector,
    InvalidModule,
    InvalidMycall,
    InvalidUrcall,
    InvalidRpt1,
    InvalidRpt2,
}

impl Display for DStarValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidReflector => {
                formatter.write_str("D-Star reflector must not contain control characters")
            }
            Self::InvalidModule => formatter
                .write_str("D-Star module must be empty or one ASCII alphanumeric character"),
            Self::InvalidMycall => {
                formatter.write_str("D-Star MYCALL must not contain control characters")
            }
            Self::InvalidUrcall => {
                formatter.write_str("D-Star URCALL must not contain control characters")
            }
            Self::InvalidRpt1 => {
                formatter.write_str("D-Star RPT1 must not contain control characters")
            }
            Self::InvalidRpt2 => {
                formatter.write_str("D-Star RPT2 must not contain control characters")
            }
        }
    }
}

impl Error for DStarValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_dstar_metadata_conservatively() {
        let metadata = DStarMetadata::from_input(DStarMetadataInput {
            reflector: "  ref001 c  ".into(),
            module: " b ".into(),
            mycall: "  py2abc g  ".into(),
            urcall: "  cqcqcq  ".into(),
            rpt1: "  py2xyz b  ".into(),
            rpt2: "  py2xyz g  ".into(),
            notes: "  contact through local repeater  ".into(),
        })
        .unwrap();

        assert_eq!(metadata.reflector.as_deref(), Some("REF001 C"));
        assert_eq!(metadata.module.as_deref(), Some("B"));
        assert_eq!(metadata.mycall.as_deref(), Some("PY2ABC G"));
        assert_eq!(metadata.urcall.as_deref(), Some("CQCQCQ"));
        assert_eq!(metadata.rpt1.as_deref(), Some("PY2XYZ B"));
        assert_eq!(metadata.rpt2.as_deref(), Some("PY2XYZ G"));
        assert_eq!(metadata.notes, "contact through local repeater");
    }

    #[test]
    fn permits_empty_optional_fields() {
        let metadata = DStarMetadata::from_input(DStarMetadataInput {
            reflector: "   ".into(),
            module: " ".into(),
            mycall: "".into(),
            urcall: "   ".into(),
            rpt1: String::new(),
            rpt2: " ".into(),
            notes: "   ".into(),
        })
        .unwrap();

        assert_eq!(metadata.reflector, None);
        assert_eq!(metadata.module, None);
        assert_eq!(metadata.mycall, None);
        assert_eq!(metadata.urcall, None);
        assert_eq!(metadata.rpt1, None);
        assert_eq!(metadata.rpt2, None);
        assert!(metadata.notes.is_empty());
    }

    #[test]
    fn permits_open_reflector_and_routing_values() {
        let metadata = DStarMetadata::from_input(DStarMetadataInput {
            reflector: "experimental reflector 42".into(),
            module: "7".into(),
            mycall: "operator suffix".into(),
            urcall: "custom route".into(),
            rpt1: "local node a".into(),
            rpt2: "gateway route z".into(),
            notes: String::new(),
        })
        .unwrap();

        assert_eq!(
            metadata.reflector.as_deref(),
            Some("EXPERIMENTAL REFLECTOR 42")
        );
        assert_eq!(metadata.module.as_deref(), Some("7"));
        assert_eq!(metadata.mycall.as_deref(), Some("OPERATOR SUFFIX"));
        assert_eq!(metadata.urcall.as_deref(), Some("CUSTOM ROUTE"));
    }

    #[test]
    fn rejects_structurally_invalid_modules() {
        for module in ["AB", "é", "-"] {
            let input = DStarMetadataInput {
                module: module.into(),
                ..Default::default()
            };

            assert_eq!(
                DStarMetadata::from_input(input),
                Err(DStarValidationError::InvalidModule)
            );
        }
    }

    #[test]
    fn rejects_control_characters_in_routing_and_callsign_fields() {
        let invalid_values = [
            ("reflector", DStarValidationError::InvalidReflector),
            ("mycall", DStarValidationError::InvalidMycall),
            ("urcall", DStarValidationError::InvalidUrcall),
            ("rpt1", DStarValidationError::InvalidRpt1),
            ("rpt2", DStarValidationError::InvalidRpt2),
        ];

        for (field, expected_error) in invalid_values {
            let mut input = DStarMetadataInput::default();
            match field {
                "reflector" => input.reflector = "REF001\nC".into(),
                "mycall" => input.mycall = "PY2ABC\tG".into(),
                "urcall" => input.urcall = "CQCQ\rCQ".into(),
                "rpt1" => input.rpt1 = "PY2\0XYZ".into(),
                "rpt2" => input.rpt2 = "PY2XYZ\u{7f}".into(),
                _ => unreachable!(),
            }

            assert_eq!(DStarMetadata::from_input(input), Err(expected_error));
        }

        let module = DStarMetadataInput {
            module: "A\n".into(),
            ..Default::default()
        };
        assert_eq!(
            DStarMetadata::from_input(module),
            Err(DStarValidationError::InvalidModule)
        );
    }
}
