use super::{DStarMetadata, DmrMetadata, Ft8Metadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeMetadata {
    Generic,
    Dmr(DmrMetadata),
    Ft8(Ft8Metadata),
    Dstar(DStarMetadata),
}

impl ModeMetadata {
    pub fn is_compatible_with(&self, mode: &str) -> bool {
        match mode {
            "DMR" => matches!(self, Self::Dmr(_)),
            "FT8" => matches!(self, Self::Ft8(_)),
            "DSTAR" => matches!(self, Self::Dstar(_)),
            _ => matches!(self, Self::Generic),
        }
    }

    pub fn expected_mode(&self) -> Option<&'static str> {
        match self {
            Self::Generic => None,
            Self::Dmr(_) => Some("DMR"),
            Self::Ft8(_) => Some("FT8"),
            Self::Dstar(_) => Some("DSTAR"),
        }
    }
}
