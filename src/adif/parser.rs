use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdifDocument {
    pub header: Option<AdifRecord>,
    pub records: Vec<AdifRecord>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdifRecord {
    pub fields: Vec<AdifField>,
}

impl AdifRecord {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| field.name.eq_ignore_ascii_case(name))
            .map(|field| field.value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdifField {
    pub name: String,
    pub value: String,
    pub data_type: Option<String>,
}

pub fn parse(input: &str) -> Result<AdifDocument, AdifError> {
    let content = input.strip_prefix('\u{feff}').unwrap_or(input);
    if !content.trim().is_empty() && !content.contains('<') {
        return Err(AdifError::new(
            input.len() - content.len(),
            "non-empty ADIF input contains no field tags",
        ));
    }

    let mut parser = Parser::new(input);
    let mut document = AdifDocument::default();
    let mut current = AdifRecord::default();
    let mut saw_end_of_header = false;
    let mut saw_record = false;

    while let Some(token) = parser.next_token()? {
        match token {
            Token::Field(field) => current.fields.push(field),
            Token::EndOfHeader => {
                if saw_record {
                    return Err(parser.error("<EOH> marker cannot appear after ADIF records"));
                }
                if saw_end_of_header {
                    return Err(parser.error("duplicate <EOH> marker"));
                }
                document.header = Some(std::mem::take(&mut current));
                saw_end_of_header = true;
            }
            Token::EndOfRecord => {
                if current.fields.is_empty() {
                    return Err(parser.error("empty ADIF record"));
                }
                document.records.push(std::mem::take(&mut current));
                saw_record = true;
            }
        }
    }

    if !current.fields.is_empty() {
        return Err(parser.error("ADIF record is missing <EOR>"));
    }
    Ok(document)
}

struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let start = if input.starts_with('\u{feff}') {
            '\u{feff}'.len_utf8()
        } else {
            0
        };
        Self {
            input,
            position: input[start..]
                .find('<')
                .map_or(input.len(), |position| start + position),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, AdifError> {
        self.skip_whitespace();
        if self.position == self.input.len() {
            return Ok(None);
        }
        if self.current_byte() != Some(b'<') {
            return Err(self.error("expected an ADIF field tag"));
        }

        let tag_start = self.position;
        let relative_end = self.input[self.position..]
            .find('>')
            .ok_or_else(|| self.error("unterminated ADIF field tag"))?;
        let tag_end = self.position + relative_end;
        let descriptor = &self.input[self.position + 1..tag_end];
        self.position = tag_end + 1;

        if descriptor.eq_ignore_ascii_case("EOH") {
            return Ok(Some(Token::EndOfHeader));
        }
        if descriptor.eq_ignore_ascii_case("EOR") {
            return Ok(Some(Token::EndOfRecord));
        }

        let mut parts = descriptor.splitn(3, ':');
        let name = parts.next().unwrap_or_default();
        let length_text = parts
            .next()
            .ok_or_else(|| AdifError::new(tag_start, "field tag must contain a value length"))?;
        let data_type = parts.next();
        if name.is_empty() {
            return Err(AdifError::new(tag_start, "field name cannot be empty"));
        }
        if name
            .chars()
            .any(|character| character.is_whitespace() || matches!(character, '<' | '>' | ':'))
        {
            return Err(AdifError::new(
                tag_start,
                "field name contains whitespace or structural characters",
            ));
        }
        if let Some(data_type) = data_type {
            if data_type.is_empty()
                || data_type.chars().any(|character| {
                    character.is_whitespace() || matches!(character, '<' | '>' | ':')
                })
            {
                return Err(AdifError::new(tag_start, "invalid field data type"));
            }
        }
        let length = length_text
            .trim()
            .parse::<usize>()
            .map_err(|_| AdifError::new(tag_start, "field length must be an integer"))?;
        let value_end = self
            .position
            .checked_add(length)
            .ok_or_else(|| AdifError::new(tag_start, "field length is too large"))?;
        if value_end > self.input.len() || !self.input.is_char_boundary(value_end) {
            return Err(AdifError::new(
                tag_start,
                "field value is shorter than its declared length",
            ));
        }
        let value = self.input[self.position..value_end].to_owned();
        self.position = value_end;

        Ok(Some(Token::Field(AdifField {
            name: name.to_ascii_uppercase(),
            value,
            data_type: data_type.map(|value| value.to_ascii_uppercase()),
        })))
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.input[self.position..].chars().next() {
            if !character.is_whitespace() {
                break;
            }
            self.position += character.len_utf8();
        }
    }

    fn current_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.position).copied()
    }

    fn error(&self, message: impl Into<String>) -> AdifError {
        AdifError::new(self.position, message)
    }
}

enum Token {
    Field(AdifField),
    EndOfHeader,
    EndOfRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdifError {
    pub position: usize,
    pub message: String,
}

impl AdifError {
    fn new(position: usize, message: impl Into<String>) -> Self {
        Self {
            position,
            message: message.into(),
        }
    }
}

impl Display for AdifError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "ADIF error at byte {}: {}",
            self.position, self.message
        )
    }
}

impl Error for AdifError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_empty_whitespace_and_header_only_documents_without_records() {
        for input in [
            "",
            "  \n\t",
            "\u{feff}",
            "\u{feff} \r\n",
            "<ADIF_VER:5>3.1.4<EOH>",
        ] {
            let document = parse(input).unwrap();
            assert!(document.records.is_empty());
        }
    }

    #[test]
    fn rejects_truncated_tags_values_and_utf8_byte_boundaries() {
        for input in [
            "<CALL:6",
            "<CALL:6>PU2",
            "<NAME:3>João<EOR>",
            "<CALL:999999999999999999999999>PU2XYZ<EOR>",
        ] {
            assert!(parse(input).is_err(), "input should be rejected: {input}");
        }
    }

    #[test]
    fn parses_header_and_multiple_records_case_insensitively() {
        let document = parse(
            "Generated by test\n<ADIF_VER:5>3.1.4<EOH>\n\
             <CALL:6>PU2XYZ<MODE:3>DMR<EOR>\n\
             <call:6>PY2ABC<mode:3>FT8<eor>",
        )
        .unwrap();

        assert_eq!(document.header.unwrap().get("adif_ver"), Some("3.1.4"));
        assert_eq!(document.records.len(), 2);
        assert_eq!(document.records[0].get("CALL"), Some("PU2XYZ"));
        assert_eq!(document.records[1].get("MODE"), Some("FT8"));
    }

    #[test]
    fn preserves_unknown_fields_and_data_types() {
        let document = parse("<CALL:6:S>PU2XYZ<APP_DIGITAL_ROUTE:8:S>TG 724 X<EOR>").unwrap();
        let record = &document.records[0];

        assert_eq!(record.fields[0].data_type.as_deref(), Some("S"));
        assert_eq!(record.get("APP_DIGITAL_ROUTE"), Some("TG 724 X"));
    }

    #[test]
    fn accepts_utf8_values_using_adif_byte_lengths() {
        let document = parse("<NAME:5>João<EOR>").unwrap();
        assert_eq!(document.records[0].get("NAME"), Some("João"));
    }

    #[test]
    fn accepts_bom_preamble_and_crlf_without_changing_byte_lengths() {
        let document = parse("\u{feff}Generated by test\r\n<NAME:5>João<EOR>\r\n").unwrap();
        assert_eq!(document.records[0].get("NAME"), Some("João"));
    }

    #[test]
    fn rejects_non_empty_input_without_any_tag() {
        let error = parse("Generated by test\r\n").unwrap_err();
        assert!(error.message.contains("no field tags"));
    }

    #[test]
    fn distinguishes_duplicate_eoh_from_eoh_after_records() {
        let duplicate = parse("<EOH><EOH>").unwrap_err();
        assert!(duplicate.message.contains("duplicate"));

        let after_record = parse("<CALL:6>PU2XYZ<EOR><EOH>").unwrap_err();
        assert!(after_record.message.contains("after ADIF records"));
        assert!(!after_record.message.contains("duplicate"));
    }

    #[test]
    fn validates_names_and_data_types_without_rejecting_extensions() {
        let document = parse("<APP.Custom-Field_2:1:X9>v<EOR>").unwrap();
        assert_eq!(document.records[0].get("APP.Custom-Field_2"), Some("v"));
        assert_eq!(
            document.records[0].fields[0].data_type.as_deref(),
            Some("X9")
        );

        for input in [
            "<BAD NAME:1>S>x<EOR>",
            "< BAD:1>S>x<EOR>",
            "<NAME:1:>x<EOR>",
            "<NAME:1:X Y>x<EOR>",
            "<NAME:1:X:Y>x<EOR>",
            "<NAME:1:X<Y>x<EOR>",
        ] {
            assert!(parse(input).is_err(), "input should be rejected: {input}");
        }
    }

    #[test]
    fn reports_malformed_tags_and_missing_record_end() {
        let malformed = parse("<CALL:X>PU2XYZ<EOR>").unwrap_err();
        assert!(malformed.message.contains("length"));

        let short = parse("<CALL:30>PU2XYZ<EOR>").unwrap_err();
        assert!(short.message.contains("shorter"));

        let missing_eor = parse("<CALL:6>PU2XYZ").unwrap_err();
        assert!(missing_eor.message.contains("<EOR>"));
    }
}
