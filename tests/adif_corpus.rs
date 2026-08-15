use digital_ham_radio_logbook::adif::{parse, AdifDocument};

fn valid(name: &str) -> &'static str {
    match name {
        "minimal" => include_str!("fixtures/adif/valid/minimal.adi"),
        "header-only" => include_str!("fixtures/adif/valid/header-only.adi"),
        "generic" => include_str!("fixtures/adif/valid/generic.adi"),
        "dmr" => include_str!("fixtures/adif/valid/dmr.adi"),
        "ft8" => include_str!("fixtures/adif/valid/ft8.adi"),
        "mixed" => include_str!("fixtures/adif/valid/mixed.adi"),
        "unicode" => include_str!("fixtures/adif/valid/unicode.adi"),
        "unknown" => include_str!("fixtures/adif/valid/unknown.adi"),
        "private" => include_str!("fixtures/adif/valid/private.adi"),
        "duplicates" => include_str!("fixtures/adif/valid/duplicates.adi"),
        "multiple" => include_str!("fixtures/adif/valid/multiple.adi"),
        "long-notes" => include_str!("fixtures/adif/valid/long-notes.adi"),
        "mixed-case" => include_str!("fixtures/adif/valid/mixed-case.adi"),
        "whitespace" => include_str!("fixtures/adif/valid/whitespace.adi"),
        "optional-types" => include_str!("fixtures/adif/valid/optional-types.adi"),
        "crlf" => include_str!("fixtures/adif/valid/crlf.adi"),
        _ => panic!("unknown valid fixture: {name}"),
    }
}

fn invalid(name: &str) -> &'static str {
    match name {
        "truncated-tag" => include_str!("fixtures/adif/invalid/truncated-tag.adi"),
        "missing-length" => include_str!("fixtures/adif/invalid/missing-length.adi"),
        "invalid-length" => include_str!("fixtures/adif/invalid/invalid-length.adi"),
        "too-large" => include_str!("fixtures/adif/invalid/too-large.adi"),
        "missing-eor" => include_str!("fixtures/adif/invalid/missing-eor.adi"),
        "broken-header" => include_str!("fixtures/adif/invalid/broken-header.adi"),
        "truncated-record" => include_str!("fixtures/adif/invalid/truncated-record.adi"),
        "malformed-type" => include_str!("fixtures/adif/invalid/malformed-type.adi"),
        _ => panic!("unknown invalid fixture: {name}"),
    }
}

fn parsed(name: &str) -> AdifDocument {
    parse(valid(name)).unwrap_or_else(|error| panic!("valid fixture {name} failed: {error}"))
}

#[test]
fn parses_minimal_and_header_only_documents() {
    let minimal = parsed("minimal");
    assert!(minimal.header.is_none());
    assert_eq!(minimal.records.len(), 1);
    assert_eq!(minimal.records[0].get("ID"), Some("1"));

    let header_only = parsed("header-only");
    assert!(header_only.records.is_empty());
    let header = header_only.header.expect("header should be present");
    assert_eq!(header.get("ADIF_VER"), Some("3.1.4"));
    assert_eq!(header.get("PROGRAMID"), Some("Corpus Test"));
}

#[test]
fn distinguishes_generic_dmr_ft8_and_mixed_records() {
    assert_eq!(parsed("generic").records[0].get("MODE"), Some("MFSK"));

    let dmr = parsed("dmr");
    assert_eq!(dmr.records[0].get("MODE"), Some("DMR"));
    assert_eq!(dmr.records[0].get("APP_DHRL_TALKGROUP"), Some("999"));
    assert_eq!(dmr.records[0].get("APP_DHRL_TIMESLOT"), Some("2"));

    let ft8 = parsed("ft8");
    assert_eq!(ft8.records[0].get("MODE"), Some("FT8"));
    assert_eq!(ft8.records[0].get("SNR"), Some("-12"));
    assert_eq!(ft8.records[0].get("FREQ"), Some("14.074"));

    let mixed = parsed("mixed");
    let modes: Vec<_> = mixed
        .records
        .iter()
        .map(|record| record.get("MODE"))
        .collect();
    assert_eq!(modes, [Some("DMR"), Some("FT8"), Some("MFSK")]);
}

#[test]
fn preserves_unicode_unknown_private_duplicate_and_long_values() {
    let unicode = parsed("unicode");
    assert_eq!(unicode.records[0].get("NAME"), Some("Rádio"));
    assert_eq!(unicode.records[0].get("COMMENT"), Some("Olá"));

    assert_eq!(
        parsed("unknown").records[0].get("X_FUTURE_FIELD"),
        Some("enabled")
    );
    assert_eq!(
        parsed("private").records[0].get("APP_CORP_SAMPLE"),
        Some("opaque")
    );

    let duplicates = parsed("duplicates");
    let values: Vec<_> = duplicates.records[0]
        .fields
        .iter()
        .filter(|field| field.name == "TAG")
        .map(|field| field.value.as_str())
        .collect();
    assert_eq!(values, ["one", "two"]);

    assert_eq!(
        parsed("long-notes").records[0].get("NOTES"),
        Some("0123456789abcdef0123456789abcdef")
    );
}

#[test]
fn handles_multiple_records_case_whitespace_and_crlf() {
    let multiple = parsed("multiple");
    let ids: Vec<_> = multiple
        .records
        .iter()
        .map(|record| record.get("ID"))
        .collect();
    assert_eq!(ids, [Some("1"), Some("2"), Some("3")]);

    let mixed_case = parsed("mixed-case");
    assert_eq!(mixed_case.records[0].fields[0].name, "CALL");
    assert_eq!(mixed_case.records[0].get("call"), Some("TEST09"));

    let whitespace = parsed("whitespace");
    assert_eq!(whitespace.records[0].get("CALL"), Some("TEST10"));
    assert_eq!(whitespace.records[0].get("MODE"), Some("FT8"));

    let crlf = parsed("crlf");
    assert_eq!(
        crlf.header
            .as_ref()
            .and_then(|header| header.get("ADIF_VER")),
        Some("3.1.4")
    );
    assert_eq!(crlf.records[0].get("CALL"), Some("TEST12"));
}

#[test]
fn preserves_optional_field_types() {
    let document = parsed("optional-types");
    let fields = &document.records[0].fields;
    let types: Vec<_> = fields
        .iter()
        .map(|field| field.data_type.as_deref())
        .collect();
    assert_eq!(types, [Some("S"), Some("N"), Some("D"), Some("T")]);
    assert_eq!(document.records[0].get("QSO_DATE"), Some("20260102"));
}

#[test]
fn rejects_structurally_invalid_documents() {
    for name in [
        "truncated-tag",
        "missing-length",
        "invalid-length",
        "too-large",
        "missing-eor",
        "broken-header",
        "truncated-record",
    ] {
        assert!(
            parse(invalid(name)).is_err(),
            "invalid fixture {name} was accepted"
        );
    }
}

#[test]
fn rejects_malformed_type_descriptor() {
    // A type descriptor is a single ADIF type token. Accepting extra `:` components
    // silently turns malformed input into an invented type and weakens validation.
    assert!(
        parse(invalid("malformed-type")).is_err(),
        "malformed type descriptors should be rejected rather than preserved as opaque types"
    );
}
