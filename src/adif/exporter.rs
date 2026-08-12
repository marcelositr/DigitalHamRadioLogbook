use super::{AdifDocument, AdifField, AdifRecord};

pub fn export(document: &AdifDocument) -> String {
    let mut output = String::new();
    if let Some(header) = &document.header {
        write_record_fields(&mut output, header);
        output.push_str("<EOH>\n");
    }
    for record in &document.records {
        write_record_fields(&mut output, record);
        output.push_str("<EOR>\n");
    }
    output
}

fn write_record_fields(output: &mut String, record: &AdifRecord) {
    for field in &record.fields {
        write_field(output, field);
    }
}

fn write_field(output: &mut String, field: &AdifField) {
    output.push('<');
    output.push_str(&field.name.to_ascii_uppercase());
    output.push(':');
    output.push_str(&field.value.len().to_string());
    if let Some(data_type) = &field.data_type {
        output.push(':');
        output.push_str(&data_type.to_ascii_uppercase());
    }
    output.push('>');
    output.push_str(&field.value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adif::parse;

    #[test]
    fn exports_deterministically_in_field_and_record_order() {
        let document = AdifDocument {
            header: Some(AdifRecord {
                fields: vec![field("ADIF_VER", "3.1.4", Some("S"))],
            }),
            records: vec![AdifRecord {
                fields: vec![field("CALL", "PU2XYZ", None), field("MODE", "DMR", None)],
            }],
        };

        assert_eq!(
            export(&document),
            "<ADIF_VER:5:S>3.1.4<EOH>\n<CALL:6>PU2XYZ<MODE:3>DMR<EOR>\n"
        );
    }

    #[test]
    fn round_trip_preserves_unknown_and_utf8_fields() {
        let original = AdifDocument {
            header: None,
            records: vec![AdifRecord {
                fields: vec![
                    field("CALL", "PU2XYZ", None),
                    field("NAME", "João", Some("S")),
                    field("APP_DIGITAL_ROUTE", "Hotspot → TG 724", Some("S")),
                ],
            }],
        };

        let encoded = export(&original);
        let decoded = parse(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    fn field(name: &str, value: &str, data_type: Option<&str>) -> AdifField {
        AdifField {
            name: name.into(),
            value: value.into(),
            data_type: data_type.map(str::to_owned),
        }
    }
}
