use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::domain::{
    CommonQsoFields, DmrMetadata, DmrMetadataInput, Ft8Metadata, Ft8MetadataInput, NewQso,
};

use super::{AdifField, AdifRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedQso {
    pub qso: NewQso,
    pub mode_metadata: ImportedModeMetadata,
    pub extra_fields: Vec<AdifField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportedModeMetadata {
    Dmr(DmrMetadata),
    Ft8(Ft8Metadata),
    Generic,
}

pub fn record_to_domain(record: &AdifRecord) -> Result<ImportedQso, AdifConversionError> {
    let callsign = required(record, "CALL")?;
    let mode = required(record, "MODE")?;
    let date = required(record, "QSO_DATE")?;
    let time = required(record, "TIME_ON")?;
    let frequency_hz = parse_frequency_hz(required(record, "FREQ")?)?;
    let datetime_start_utc = parse_datetime(date, time)?;

    let qso = NewQso::new(callsign, datetime_start_utc, frequency_hz, mode)
        .map_err(|error| AdifConversionError::new(error.to_string()))?
        .with_common_fields(CommonQsoFields {
            band_override: value(record, "BAND").unwrap_or_default().to_owned(),
            rst_sent: value(record, "RST_SENT").unwrap_or_default().to_owned(),
            rst_received: value(record, "RST_RCVD").unwrap_or_default().to_owned(),
            grid_locator: value(record, "GRIDSQUARE").unwrap_or_default().to_owned(),
            name: value(record, "NAME").unwrap_or_default().to_owned(),
            qth: value(record, "QTH").unwrap_or_default().to_owned(),
            notes: value(record, "COMMENT")
                .or_else(|| value(record, "NOTES"))
                .unwrap_or_default()
                .to_owned(),
        })
        .map_err(|error| AdifConversionError::new(error.to_string()))?;

    let mode_metadata = match qso.mode.as_str() {
        "DMR" => ImportedModeMetadata::Dmr(convert_dmr(record)?),
        "FT8" => ImportedModeMetadata::Ft8(convert_ft8(record)?),
        _ => ImportedModeMetadata::Generic,
    };
    let known = known_fields(&qso.mode);
    reject_duplicate_known_fields(record, &known)?;
    let extra_fields = record
        .fields
        .iter()
        .filter(|field| !known.contains(field.name.as_str()))
        .cloned()
        .collect();

    Ok(ImportedQso {
        qso,
        mode_metadata,
        extra_fields,
    })
}

pub fn domain_to_record(imported: &ImportedQso) -> Result<AdifRecord, AdifConversionError> {
    let datetime = chrono::DateTime::from_timestamp(imported.qso.datetime_start_utc, 0)
        .ok_or_else(|| AdifConversionError::new("QSO timestamp is out of range"))?;
    let mut fields = vec![
        field("CALL", &imported.qso.callsign),
        field("QSO_DATE", &datetime.format("%Y%m%d").to_string()),
        field("TIME_ON", &datetime.format("%H%M%S").to_string()),
        field("FREQ", &format_frequency_mhz(imported.qso.frequency_hz)),
        field("MODE", &imported.qso.mode),
    ];
    push_optional(&mut fields, "BAND", imported.qso.band.as_deref());
    push_optional(&mut fields, "RST_SENT", imported.qso.rst_sent.as_deref());
    push_optional(
        &mut fields,
        "RST_RCVD",
        imported.qso.rst_received.as_deref(),
    );
    push_optional(
        &mut fields,
        "GRIDSQUARE",
        imported.qso.grid_locator.as_deref(),
    );
    push_optional(&mut fields, "NAME", imported.qso.name.as_deref());
    push_optional(&mut fields, "QTH", imported.qso.qth.as_deref());
    if !imported.qso.notes.is_empty() {
        fields.push(field("COMMENT", &imported.qso.notes));
    }

    match &imported.mode_metadata {
        ImportedModeMetadata::Dmr(metadata) => append_dmr(&mut fields, metadata),
        ImportedModeMetadata::Ft8(metadata) => append_ft8(&mut fields, metadata),
        ImportedModeMetadata::Generic => {}
    }
    fields.extend(imported.extra_fields.iter().cloned());
    Ok(AdifRecord { fields })
}

fn convert_dmr(record: &AdifRecord) -> Result<DmrMetadata, AdifConversionError> {
    DmrMetadata::from_input(DmrMetadataInput {
        remote_dmr_id: string_value(record, "APP_DHRL_REMOTE_DMR_ID"),
        local_dmr_id: string_value(record, "APP_DHRL_LOCAL_DMR_ID"),
        talkgroup: value(record, "APP_DHRL_TALKGROUP")
            .or_else(|| value(record, "MY_SIG_INFO"))
            .unwrap_or_default()
            .to_owned(),
        timeslot: string_value(record, "APP_DHRL_TIMESLOT"),
        color_code: string_value(record, "APP_DHRL_COLOR_CODE"),
        network: value(record, "APP_DHRL_NETWORK")
            .or_else(|| value(record, "SIG"))
            .unwrap_or_default()
            .to_owned(),
        call_type: value(record, "APP_DHRL_CALL_TYPE")
            .unwrap_or("group")
            .to_owned(),
        access_type: value(record, "APP_DHRL_ACCESS_TYPE")
            .unwrap_or("simplex")
            .to_owned(),
        repeater_callsign: string_value(record, "APP_DHRL_REPEATER"),
        hotspot: string_value(record, "APP_DHRL_HOTSPOT"),
        notes: string_value(record, "APP_DHRL_DMR_NOTES"),
        ..Default::default()
    })
    .map_err(|error| AdifConversionError::new(error.to_string()))
}

fn convert_ft8(record: &AdifRecord) -> Result<Ft8Metadata, AdifConversionError> {
    Ft8Metadata::from_input(Ft8MetadataInput {
        snr_sent_db: string_value(record, "APP_DHRL_SNR_SENT"),
        snr_received_db: value(record, "SNR")
            .or_else(|| value(record, "APP_DHRL_SNR_RECEIVED"))
            .unwrap_or_default()
            .to_owned(),
        power_watts: string_value(record, "TX_PWR"),
        audio_frequency_hz: string_value(record, "APP_DHRL_AUDIO_FREQUENCY"),
        source_software: string_value(record, "APP_DHRL_SOURCE_SOFTWARE"),
        protocol: string_value(record, "APP_DHRL_PROTOCOL"),
        final_message: string_value(record, "APP_DHRL_FINAL_MESSAGE"),
    })
    .map_err(|error| AdifConversionError::new(error.to_string()))
}

fn append_dmr(fields: &mut Vec<AdifField>, metadata: &DmrMetadata) {
    push_number(fields, "APP_DHRL_REMOTE_DMR_ID", metadata.remote_dmr_id);
    push_number(fields, "APP_DHRL_LOCAL_DMR_ID", metadata.local_dmr_id);
    push_number(fields, "APP_DHRL_TALKGROUP", metadata.talkgroup);
    push_number(fields, "APP_DHRL_TIMESLOT", metadata.timeslot);
    push_number(fields, "APP_DHRL_COLOR_CODE", metadata.color_code);
    push_optional(fields, "APP_DHRL_NETWORK", metadata.network.as_deref());
    fields.push(field("APP_DHRL_CALL_TYPE", metadata.call_type.as_str()));
    fields.push(field("APP_DHRL_ACCESS_TYPE", metadata.access_type.as_str()));
    push_optional(
        fields,
        "APP_DHRL_REPEATER",
        metadata.repeater_callsign.as_deref(),
    );
    push_optional(fields, "APP_DHRL_HOTSPOT", metadata.hotspot.as_deref());
    if !metadata.notes.is_empty() {
        fields.push(field("APP_DHRL_DMR_NOTES", &metadata.notes));
    }
}

fn append_ft8(fields: &mut Vec<AdifField>, metadata: &Ft8Metadata) {
    push_number(fields, "APP_DHRL_SNR_SENT", metadata.snr_sent_db);
    push_number(fields, "SNR", metadata.snr_received_db);
    push_number(fields, "TX_PWR", metadata.power_watts);
    push_number(
        fields,
        "APP_DHRL_AUDIO_FREQUENCY",
        metadata.audio_frequency_hz,
    );
    push_optional(
        fields,
        "APP_DHRL_SOURCE_SOFTWARE",
        metadata.source_software.as_deref(),
    );
    push_optional(fields, "APP_DHRL_PROTOCOL", metadata.protocol.as_deref());
    push_optional(
        fields,
        "APP_DHRL_FINAL_MESSAGE",
        metadata.final_message.as_deref(),
    );
}

fn reject_duplicate_known_fields(
    record: &AdifRecord,
    known: &HashSet<&str>,
) -> Result<(), AdifConversionError> {
    let mut seen = HashSet::new();
    for field in &record.fields {
        if known.contains(field.name.as_str()) && !seen.insert(field.name.as_str()) {
            return Err(AdifConversionError::new(format!(
                "duplicate ADIF field {}",
                field.name
            )));
        }
    }
    Ok(())
}

fn known_fields(mode: &str) -> HashSet<&'static str> {
    let mut fields: HashSet<_> = [
        "CALL",
        "MODE",
        "QSO_DATE",
        "TIME_ON",
        "FREQ",
        "BAND",
        "RST_SENT",
        "RST_RCVD",
        "GRIDSQUARE",
        "NAME",
        "QTH",
        "COMMENT",
        "NOTES",
    ]
    .into_iter()
    .collect();
    if mode == "DMR" {
        fields.extend([
            "APP_DHRL_REMOTE_DMR_ID",
            "APP_DHRL_LOCAL_DMR_ID",
            "APP_DHRL_TALKGROUP",
            "MY_SIG_INFO",
            "APP_DHRL_TIMESLOT",
            "APP_DHRL_COLOR_CODE",
            "APP_DHRL_NETWORK",
            "SIG",
            "APP_DHRL_CALL_TYPE",
            "APP_DHRL_ACCESS_TYPE",
            "APP_DHRL_REPEATER",
            "APP_DHRL_HOTSPOT",
            "APP_DHRL_DMR_NOTES",
        ]);
    } else if mode == "FT8" {
        fields.extend([
            "APP_DHRL_SNR_SENT",
            "SNR",
            "APP_DHRL_SNR_RECEIVED",
            "TX_PWR",
            "APP_DHRL_AUDIO_FREQUENCY",
            "APP_DHRL_SOURCE_SOFTWARE",
            "APP_DHRL_PROTOCOL",
            "APP_DHRL_FINAL_MESSAGE",
        ]);
    }
    fields
}

fn required<'a>(record: &'a AdifRecord, name: &str) -> Result<&'a str, AdifConversionError> {
    value(record, name)
        .ok_or_else(|| AdifConversionError::new(format!("missing ADIF field {name}")))
}

fn value<'a>(record: &'a AdifRecord, name: &str) -> Option<&'a str> {
    record
        .get(name)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn string_value(record: &AdifRecord, name: &str) -> String {
    value(record, name).unwrap_or_default().to_owned()
}

fn parse_datetime(date: &str, time: &str) -> Result<i64, AdifConversionError> {
    let date = NaiveDate::parse_from_str(date, "%Y%m%d")
        .map_err(|_| AdifConversionError::new("QSO_DATE must use YYYYMMDD"))?;
    let format = match time.len() {
        4 => "%H%M",
        6 => "%H%M%S",
        _ => return Err(AdifConversionError::new("TIME_ON must use HHMM or HHMMSS")),
    };
    let time = NaiveTime::parse_from_str(time, format)
        .map_err(|_| AdifConversionError::new("TIME_ON is invalid"))?;
    Ok(NaiveDateTime::new(date, time).and_utc().timestamp())
}

fn parse_frequency_hz(value: &str) -> Result<i64, AdifConversionError> {
    let mut parts = value.trim().split('.');
    let whole = parts
        .next()
        .ok_or_else(|| AdifConversionError::new("FREQ is required"))?;
    let fraction = parts.next().unwrap_or("");
    if whole.is_empty() || parts.next().is_some() || fraction.len() > 6 {
        return Err(AdifConversionError::new(
            "FREQ must be MHz with up to 6 decimals",
        ));
    }
    let whole_hz = whole
        .parse::<i64>()
        .map_err(|_| AdifConversionError::new("FREQ is invalid"))?
        .checked_mul(1_000_000)
        .ok_or_else(|| AdifConversionError::new("FREQ is too large"))?;
    let fraction_hz = if fraction.is_empty() {
        0
    } else {
        format!("{fraction:0<6}")
            .parse::<i64>()
            .map_err(|_| AdifConversionError::new("FREQ is invalid"))?
    };
    whole_hz
        .checked_add(fraction_hz)
        .filter(|value| *value > 0)
        .ok_or_else(|| AdifConversionError::new("FREQ must be positive"))
}

fn format_frequency_mhz(frequency_hz: i64) -> String {
    format!(
        "{}.{:06}",
        frequency_hz / 1_000_000,
        frequency_hz.rem_euclid(1_000_000)
    )
}

fn field(name: &str, value: &str) -> AdifField {
    AdifField {
        name: name.to_owned(),
        value: value.to_owned(),
        data_type: None,
    }
}

fn push_optional(fields: &mut Vec<AdifField>, name: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        fields.push(field(name, value));
    }
}

fn push_number<T: ToString>(fields: &mut Vec<AdifField>, name: &str, value: Option<T>) {
    if let Some(value) = value {
        fields.push(field(name, &value.to_string()));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdifConversionError {
    pub message: String,
}

impl AdifConversionError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for AdifConversionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for AdifConversionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adif::{export, parse, AdifDocument};

    #[test]
    fn converts_common_and_ft8_fields_and_preserves_unknowns() {
        let document = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074\
             <MODE:3>FT8<GRIDSQUARE:6>GG66AA<SNR:3>-18<TX_PWR:2>25\
             <APP_VENDOR_FIELD:5:S>value<EOR>",
        )
        .unwrap();
        let imported = record_to_domain(&document.records[0]).unwrap();

        assert_eq!(imported.qso.frequency_hz, 14_074_000);
        assert_eq!(imported.qso.datetime_start_utc, 1_700_000_000);
        assert_eq!(imported.extra_fields[0].name, "APP_VENDOR_FIELD");
        assert!(matches!(
            imported.mode_metadata,
            ImportedModeMetadata::Ft8(_)
        ));

        let record = domain_to_record(&imported).unwrap();
        let reparsed = parse(&export(&AdifDocument {
            header: None,
            records: vec![record],
        }))
        .unwrap();
        assert_eq!(reparsed.records[0].get("APP_VENDOR_FIELD"), Some("value"));
    }

    #[test]
    fn converts_dmr_private_fields() {
        let record = parse(
            "<CALL:6>PU2XYZ<QSO_DATE:8>20231114<TIME_ON:4>2213<FREQ:7>438.500\
             <MODE:3>DMR<APP_DHRL_TALKGROUP:3>724<APP_DHRL_TIMESLOT:1>1\
             <APP_DHRL_COLOR_CODE:1>1<APP_DHRL_CALL_TYPE:5>group\
             <APP_DHRL_ACCESS_TYPE:8>repeater<APP_DHRL_REPEATER:6>PY2XYZ<EOR>",
        )
        .unwrap()
        .records
        .remove(0);
        let imported = record_to_domain(&record).unwrap();

        let ImportedModeMetadata::Dmr(metadata) = imported.mode_metadata else {
            panic!("expected DMR metadata");
        };
        assert_eq!(metadata.talkgroup, Some(724));
        assert_eq!(metadata.repeater_callsign.as_deref(), Some("PY2XYZ"));
    }

    #[test]
    fn rejects_duplicate_known_fields_without_rejecting_repeated_unknown_fields() {
        let duplicate_call = parse(
            "<CALL:6>PY2ABC<CALL:6>PU2XYZ<QSO_DATE:8>20231114<TIME_ON:6>221320\
             <FREQ:6>14.074<MODE:3>FT8<EOR>",
        )
        .unwrap();
        let error = record_to_domain(&duplicate_call.records[0])
            .unwrap_err()
            .to_string();
        assert!(error.contains("duplicate ADIF field CALL"));

        let repeated_unknown = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074\
             <MODE:3>FT8<APP_VENDOR_FIELD:3>one<APP_VENDOR_FIELD:3>two<EOR>",
        )
        .unwrap();
        let imported = record_to_domain(&repeated_unknown.records[0]).unwrap();
        assert_eq!(imported.extra_fields.len(), 2);
    }

    #[test]
    fn rejects_missing_and_invalid_required_fields() {
        let missing = parse("<CALL:6>PU2XYZ<EOR>").unwrap();
        assert!(record_to_domain(&missing.records[0])
            .unwrap_err()
            .message
            .contains("MODE"));

        let invalid =
            parse("<CALL:6>PU2XYZ<MODE:3>FT8<QSO_DATE:8>20231340<TIME_ON:4>2500<FREQ:4>zero<EOR>")
                .unwrap();
        assert!(record_to_domain(&invalid.records[0]).is_err());
    }
}
