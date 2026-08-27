use std::collections::{BTreeMap, HashSet};
use std::error::Error;

use rusqlite::{params, params_from_iter, Connection, Result, Transaction};

use crate::adif::{
    domain_to_record, record_to_domain, AdifDocument, AdifField, AdifRecord, ImportedQso,
};
use crate::domain::{ModeMetadata, NewQso, Qso};

use super::queries::{selection_sql, SELECTION_METADATA_JOINS};
use super::{
    insert_dmr_metadata, insert_dstar_metadata, insert_ft8_metadata, insert_qso,
    insert_ysf_metadata, AdifImportPlan, AdifImportPreview, AdifImportReport, QsoIdentity,
    QsoRepository, QsoSelection,
};

impl QsoRepository {
    pub fn export_adif(&self) -> std::result::Result<AdifDocument, Box<dyn Error>> {
        self.export_adif_selection(&QsoSelection::All)
    }

    pub fn export_adif_selection(
        &self,
        selection: &QsoSelection,
    ) -> std::result::Result<AdifDocument, Box<dyn Error>> {
        let items = self.selection_items(selection)?;
        let mut extra_fields = selected_adif_extra_fields(&self.connection, selection)?;
        let mut records = Vec::with_capacity(items.len());
        for item in items {
            let qso_id = item.qso.id;
            let imported = ImportedQso {
                qso: new_qso_from_stored(&item.qso),
                mode_metadata: item.metadata,
                extra_fields: extra_fields.remove(&qso_id).unwrap_or_default(),
            };
            records.push(domain_to_record(&imported)?);
        }

        Ok(AdifDocument {
            header: Some(AdifRecord {
                fields: vec![
                    AdifField {
                        name: "ADIF_VER".into(),
                        value: "3.1.4".into(),
                        data_type: None,
                    },
                    AdifField {
                        name: "PROGRAMID".into(),
                        value: "Digital Ham Radio Logbook".into(),
                        data_type: None,
                    },
                    AdifField {
                        name: "PROGRAMVERSION".into(),
                        value: env!("CARGO_PKG_VERSION").into(),
                        data_type: None,
                    },
                ],
            }),
            records,
        })
    }

    pub fn prepare_adif_import(
        &self,
        document: &AdifDocument,
    ) -> std::result::Result<AdifImportPlan, Box<dyn Error>> {
        let mut identities = existing_qso_identities(&self.connection)?;
        let mut preview = AdifImportPreview {
            total: document.records.len(),
            ..Default::default()
        };
        let mut qsos = Vec::new();

        for (index, record) in document.records.iter().enumerate() {
            let imported_qso = match record_to_domain(record) {
                Ok(imported_qso) => imported_qso,
                Err(error) => {
                    preview.invalid += 1;
                    if preview.invalid_details.len() < 20 {
                        preview
                            .invalid_details
                            .push(format!("Record {} — {error}", index + 1));
                    }
                    continue;
                }
            };
            *preview
                .modes
                .entry(imported_qso.qso.mode.clone())
                .or_default() += 1;
            *preview
                .bands
                .entry(
                    imported_qso
                        .qso
                        .band
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_owned()),
                )
                .or_default() += 1;
            preview.earliest_utc = Some(
                preview
                    .earliest_utc
                    .map_or(imported_qso.qso.datetime_start_utc, |current| {
                        current.min(imported_qso.qso.datetime_start_utc)
                    }),
            );
            preview.latest_utc = Some(
                preview
                    .latest_utc
                    .map_or(imported_qso.qso.datetime_start_utc, |current| {
                        current.max(imported_qso.qso.datetime_start_utc)
                    }),
            );
            if !identities.insert(QsoIdentity::from(&imported_qso.qso)) {
                preview.duplicates += 1;
                continue;
            }
            preview.new_qsos += 1;
            qsos.push(imported_qso);
        }

        if preview.invalid > preview.invalid_details.len() {
            preview.invalid_details.push(format!(
                "… {} additional invalid record(s) omitted",
                preview.invalid - preview.invalid_details.len()
            ));
        }

        Ok(AdifImportPlan { preview, qsos })
    }

    pub fn import_adif_plan(
        &self,
        plan: AdifImportPlan,
        now_utc: i64,
    ) -> std::result::Result<AdifImportReport, Box<dyn Error>> {
        let transaction = self.connection.unchecked_transaction()?;
        let mut identities = existing_qso_identities(&transaction)?;
        let mut imported = 0;
        let mut duplicates_skipped = plan.preview.duplicates;
        for imported_qso in &plan.qsos {
            if !identities.insert(QsoIdentity::from(&imported_qso.qso)) {
                duplicates_skipped += 1;
                continue;
            }
            let qso_id = insert_qso(&transaction, &imported_qso.qso, now_utc)?;
            match &imported_qso.mode_metadata {
                ModeMetadata::Dmr(metadata) => {
                    insert_dmr_metadata(&transaction, qso_id, metadata)?;
                }
                ModeMetadata::Dstar(metadata) => {
                    insert_dstar_metadata(&transaction, qso_id, metadata)?;
                }
                ModeMetadata::Ft8(metadata) => {
                    insert_ft8_metadata(&transaction, qso_id, metadata)?;
                }
                ModeMetadata::Ysf(metadata) => {
                    insert_ysf_metadata(&transaction, qso_id, metadata)?;
                }
                ModeMetadata::Generic => {}
            }
            insert_adif_extra_fields(&transaction, qso_id, &imported_qso.extra_fields)?;
            imported += 1;
        }
        transaction.commit()?;
        Ok(AdifImportReport {
            imported,
            duplicates_skipped,
        })
    }

    pub fn import_adif(
        &self,
        document: &AdifDocument,
        now_utc: i64,
    ) -> std::result::Result<AdifImportReport, Box<dyn Error>> {
        for (index, record) in document.records.iter().enumerate() {
            record_to_domain(record)
                .map_err(|error| format!("ADIF record {}: {error}", index + 1))?;
        }
        let plan = self.prepare_adif_import(document)?;
        self.import_adif_plan(plan, now_utc)
    }

    pub fn get_adif_extra_fields(&self, qso_id: i64) -> Result<Vec<AdifField>> {
        let mut statement = self.connection.prepare(
            "SELECT name, value, data_type
             FROM adif_extra_fields
             WHERE qso_id = ?1
             ORDER BY field_order",
        )?;
        let rows = statement.query_map(params![qso_id], |row| {
            Ok(AdifField {
                name: row.get(0)?,
                value: row.get(1)?,
                data_type: row.get(2)?,
            })
        })?;
        rows.collect()
    }
}

fn selected_adif_extra_fields(
    connection: &Connection,
    selection: &QsoSelection,
) -> Result<BTreeMap<i64, Vec<AdifField>>> {
    let selection = selection_sql(selection);
    let mut statement = connection.prepare(&format!(
        "SELECT e.qso_id, e.name, e.value, e.data_type
         FROM adif_extra_fields e
         JOIN qsos q ON q.id = e.qso_id
         {SELECTION_METADATA_JOINS}
         WHERE {}
         ORDER BY e.qso_id, e.field_order",
        selection.predicate
    ))?;
    let rows = statement.query_map(params_from_iter(selection.parameters.iter()), |row| {
        Ok((
            row.get::<_, i64>(0)?,
            AdifField {
                name: row.get(1)?,
                value: row.get(2)?,
                data_type: row.get(3)?,
            },
        ))
    })?;
    let mut fields = BTreeMap::<i64, Vec<AdifField>>::new();
    for row in rows {
        let (qso_id, field) = row?;
        fields.entry(qso_id).or_default().push(field);
    }
    Ok(fields)
}

fn existing_qso_identities(connection: &Connection) -> Result<HashSet<QsoIdentity>> {
    let mut statement =
        connection.prepare("SELECT callsign, datetime_start_utc, frequency_hz, mode FROM qsos")?;
    let rows = statement.query_map([], |row| {
        let callsign: String = row.get(0)?;
        let mode: String = row.get(3)?;
        Ok(QsoIdentity {
            callsign: callsign.trim().to_uppercase(),
            datetime_start_utc: row.get(1)?,
            frequency_hz: row.get(2)?,
            mode: mode.trim().to_uppercase(),
        })
    })?;
    rows.collect()
}

fn new_qso_from_stored(qso: &Qso) -> NewQso {
    NewQso {
        callsign: qso.callsign.clone(),
        datetime_start_utc: qso.datetime_start_utc,
        frequency_hz: qso.frequency_hz,
        band: qso.band.clone(),
        mode: qso.mode.clone(),
        rst_sent: qso.rst_sent.clone(),
        rst_received: qso.rst_received.clone(),
        grid_locator: qso.grid_locator.clone(),
        name: qso.name.clone(),
        qth: qso.qth.clone(),
        notes: qso.notes.clone(),
    }
}

pub(super) fn reconcile_adif_extra_fields(
    transaction: &Transaction<'_>,
    qso_id: i64,
    destination_mode: &str,
) -> Result<()> {
    for name in ImportedQso::known_field_names(destination_mode) {
        transaction.execute(
            "DELETE FROM adif_extra_fields
             WHERE qso_id = ?1 AND name = ?2 COLLATE NOCASE",
            params![qso_id, name],
        )?;
    }
    Ok(())
}

fn insert_adif_extra_fields(
    transaction: &Transaction<'_>,
    qso_id: i64,
    fields: &[AdifField],
) -> Result<()> {
    for (index, field) in fields.iter().enumerate() {
        transaction.execute(
            "INSERT INTO adif_extra_fields(qso_id, field_order, name, value, data_type)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                qso_id,
                index as i64,
                field.name,
                field.value,
                field.data_type
            ],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adif::parse;
    use crate::domain::{DStarMetadata, DStarMetadataInput};

    #[test]
    fn filtered_export_includes_all_matches_across_pages_and_only_selected_extras() {
        let repository = QsoRepository::in_memory().unwrap();
        let mut selected_id = None;
        for index in 0..100 {
            let callsign = if index < 17 {
                format!("FILTER{index:03}")
            } else {
                format!("OTHER{index:03}")
            };
            let qso = NewQso::new(&callsign, 1_700_000_000 + index, 145_500_000, "FM").unwrap();
            let id = repository.insert(&qso, 1_700_001_000).unwrap();
            if index == 0 {
                selected_id = Some(id);
            }
        }
        repository
            .connection
            .execute(
                "INSERT INTO adif_extra_fields(qso_id, field_order, name, value, data_type)
                 VALUES (?1, 0, 'APP_VENDOR_PRIVATE', 'preserved', 'S')",
                [selected_id.unwrap()],
            )
            .unwrap();

        let filtered = repository
            .export_adif_selection(&QsoSelection::General("FILTER".into()))
            .unwrap();
        assert_eq!(filtered.records.len(), 17);
        assert!(filtered
            .records
            .iter()
            .any(|record| record.get("APP_VENDOR_PRIVATE") == Some("preserved")));
        assert_eq!(repository.export_adif().unwrap().records.len(), 100);

        let many = QsoRepository::in_memory().unwrap();
        for index in 0..350 {
            let qso = NewQso::new(
                format!("MATCH{index:03}"),
                1_700_000_000 + index,
                145_500_000,
                "FM",
            )
            .unwrap();
            many.insert(&qso, 1_700_001_000).unwrap();
        }
        assert_eq!(
            many.export_adif_selection(&QsoSelection::General("MATCH".into()))
                .unwrap()
                .records
                .len(),
            350
        );
    }

    #[test]
    fn filtered_export_preserves_each_specialized_mode_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        for fixture in [
            "<CALL:6>PY2DMR<QSO_DATE:8>20260815<TIME_ON:6>120000<FREQ:7>438.500<MODE:3>DMR<APP_DHRL_CALL_TYPE:5>group<APP_DHRL_ACCESS_TYPE:7>simplex<APP_DHRL_TALKGROUP:3>724<EOR>",
            "<CALL:6>PY2FT8<QSO_DATE:8>20260815<TIME_ON:6>120100<FREQ:6>14.074<MODE:3>FT8<SNR:3>-12<EOR>",
            include_str!("../../../tests/fixtures/adif/valid/dstar-full.adi"),
            include_str!("../../../tests/fixtures/adif/valid/ysf-full.adi"),
        ] {
            repository
                .import_adif(&parse(fixture).unwrap(), 1_700_000_100)
                .unwrap();
        }

        let selections = [
            (
                QsoSelection::Dmr(super::super::DmrFilter::default()),
                "APP_DHRL_TALKGROUP",
            ),
            (QsoSelection::Ft8(super::super::Ft8Filter::default()), "SNR"),
            (
                QsoSelection::Dstar(super::super::DstarFilter::default()),
                "APP_DHRL_DSTAR_REFLECTOR",
            ),
            (
                QsoSelection::Ysf(super::super::YsfFilter::default()),
                "APP_DHRL_YSF_ROOM",
            ),
        ];
        for (selection, metadata_field) in selections {
            let exported = repository.export_adif_selection(&selection).unwrap();
            assert_eq!(exported.records.len(), 1, "{selection:?}");
            let record = &exported.records[0];
            assert!(record.get("MODE").is_some());
            assert!(record.get(metadata_field).is_some(), "{metadata_field}");
        }
    }

    #[test]
    fn imports_exports_and_reimports_dstar_with_sqlite_metadata_and_unknowns() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(include_str!(
            "../../../tests/fixtures/adif/valid/dstar-full.adi"
        ))
        .unwrap();

        let report = repository.import_adif(&document, 1_700_000_100).unwrap();
        assert_eq!(report.imported, 1);
        let qso = repository.list().unwrap().remove(0);
        assert_eq!(qso.mode, "DSTAR");
        let metadata = repository.get_dstar_metadata(qso.id).unwrap().unwrap();
        assert_eq!(metadata.reflector.as_deref(), Some("REF001 C"));
        assert_eq!(metadata.mycall.as_deref(), Some("PY2ABC G"));
        assert_eq!(
            repository.get_adif_extra_fields(qso.id).unwrap(),
            vec![AdifField {
                name: "APP_VENDOR_DSTAR".into(),
                value: "opaque".into(),
                data_type: Some("S".into()),
            }]
        );

        let exported = repository.export_adif().unwrap();
        let record = &exported.records[0];
        assert_eq!(record.get("MODE"), Some("DIGITALVOICE"));
        assert_eq!(record.get("SUBMODE"), Some("DSTAR"));
        assert_eq!(record.get("PROP_MODE"), Some("RPT"));
        assert_eq!(record.get("APP_VENDOR_DSTAR"), Some("opaque"));

        let restored = QsoRepository::in_memory().unwrap();
        restored.import_adif(&exported, 1_700_000_200).unwrap();
        let restored_qso = restored.list().unwrap().remove(0);
        assert_eq!(
            restored.get_dstar_metadata(restored_qso.id).unwrap(),
            Some(metadata)
        );
        assert_eq!(
            restored.get_adif_extra_fields(restored_qso.id).unwrap(),
            repository.get_adif_extra_fields(qso.id).unwrap()
        );
    }

    #[test]
    fn imports_exports_and_reimports_ysf_with_sqlite_metadata_and_unknowns() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(include_str!(
            "../../../tests/fixtures/adif/valid/ysf-full.adi"
        ))
        .unwrap();

        let report = repository.import_adif(&document, 1_700_000_100).unwrap();
        assert_eq!(report.imported, 1);
        let qso = repository.list().unwrap().remove(0);
        assert_eq!(qso.mode, "C4FM");
        let metadata = repository.get_ysf_metadata(qso.id).unwrap().unwrap();
        assert_eq!(metadata.room.as_deref(), Some("America-Link"));
        assert_eq!(metadata.tx_dg_id, Some(1));
        assert_eq!(
            repository.get_adif_extra_fields(qso.id).unwrap(),
            vec![AdifField {
                name: "APP_VENDOR_YSF".into(),
                value: "opaque".into(),
                data_type: Some("S".into()),
            }]
        );

        let exported = repository.export_adif().unwrap();
        let record = &exported.records[0];
        assert_eq!(record.get("MODE"), Some("DIGITALVOICE"));
        assert_eq!(record.get("SUBMODE"), Some("C4FM"));
        assert_eq!(record.get("APP_DHRL_YSF_TX_DG_ID"), Some("01"));
        assert_eq!(record.get("PROP_MODE"), Some("RPT"));
        assert_eq!(record.get("APP_VENDOR_YSF"), Some("opaque"));

        let restored = QsoRepository::in_memory().unwrap();
        restored.import_adif(&exported, 1_700_000_200).unwrap();
        let restored_qso = restored.list().unwrap().remove(0);
        assert_eq!(
            restored.get_ysf_metadata(restored_qso.id).unwrap(),
            Some(metadata)
        );
        assert_eq!(
            restored.get_adif_extra_fields(restored_qso.id).unwrap(),
            repository.get_adif_extra_fields(qso.id).unwrap()
        );
    }

    #[test]
    fn deduplicates_historical_and_canonical_ysf_as_one_domain_mode() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2YSF<QSO_DATE:8>20260815<TIME_ON:6>130000<FREQ:7>145.562<MODE:4>C4FM<EOR>\
             <CALL:6>PY2YSF<QSO_DATE:8>20260815<TIME_ON:6>130000<FREQ:7>145.562\
             <MODE:12>DIGITALVOICE<SUBMODE:4>C4FM<EOR>",
        )
        .unwrap();

        let report = repository.import_adif(&document, 1_700_000_100).unwrap();
        assert_eq!(report.imported, 1);
        assert_eq!(report.duplicates_skipped, 1);
    }

    #[test]
    fn editing_imported_qso_reconciles_extras_known_by_destination_mode() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2DST<QSO_DATE:8>20260815<TIME_ON:6>120000<FREQ:7>145.670\
             <MODE:2>FM<APP_DHRL_DSTAR_RPT1:8:S>OLD1   B\
             <APP_VENDOR_FIELD:3:N>one<APP_VENDOR_FIELD:3:S>two<EOR>",
        )
        .unwrap();
        repository.import_adif(&document, 1).unwrap();
        let imported = repository.list().unwrap().remove(0);

        let dstar_qso = NewQso::new(
            &imported.callsign,
            imported.datetime_start_utc,
            imported.frequency_hz,
            "DSTAR",
        )
        .unwrap();
        let metadata = DStarMetadata::from_input(DStarMetadataInput {
            rpt1: "NEW1 B".into(),
            ..Default::default()
        })
        .unwrap();
        assert!(repository
            .update_dstar(imported.id, &dstar_qso, &metadata, 2)
            .unwrap());

        assert_eq!(
            repository.get_adif_extra_fields(imported.id).unwrap(),
            vec![
                AdifField {
                    name: "APP_VENDOR_FIELD".into(),
                    value: "one".into(),
                    data_type: Some("N".into()),
                },
                AdifField {
                    name: "APP_VENDOR_FIELD".into(),
                    value: "two".into(),
                    data_type: Some("S".into()),
                },
            ]
        );
        let exported = repository.export_adif().unwrap();
        assert_eq!(
            exported.records[0]
                .fields
                .iter()
                .filter(|field| field.name == "APP_DHRL_DSTAR_RPT1")
                .count(),
            1
        );
        assert_eq!(
            exported.records[0].get("APP_DHRL_DSTAR_RPT1"),
            Some("NEW1 B")
        );

        let restored = QsoRepository::in_memory().unwrap();
        restored.import_adif(&exported, 3).unwrap();
        let restored_qso = restored.list().unwrap().remove(0);
        assert_eq!(
            restored
                .get_dstar_metadata(restored_qso.id)
                .unwrap()
                .unwrap()
                .rpt1
                .as_deref(),
            Some("NEW1 B")
        );
    }

    #[test]
    fn export_rejects_multiple_specialized_metadata_rows() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2FT8<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<EOR>",
        )
        .unwrap();
        repository.import_adif(&document, 1_700_000_100).unwrap();
        let qso_id = repository.list().unwrap()[0].id;
        repository
            .connection
            .execute("INSERT INTO dstar_metadata(qso_id) VALUES (?1)", [qso_id])
            .unwrap();

        assert!(repository.export_adif().is_err());
    }

    #[test]
    fn deduplicates_historical_and_canonical_dstar_as_one_domain_mode() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2DST<QSO_DATE:8>20260815<TIME_ON:6>120000<FREQ:7>145.670<MODE:5>DSTAR<EOR>\
             <CALL:6>PY2DST<QSO_DATE:8>20260815<TIME_ON:6>120000<FREQ:7>145.670\
             <MODE:12>DIGITALVOICE<SUBMODE:5>DSTAR<EOR>",
        )
        .unwrap();

        let report = repository.import_adif(&document, 1_700_000_100).unwrap();
        assert_eq!(report.imported, 1);
        assert_eq!(report.duplicates_skipped, 1);
        assert_eq!(repository.list().unwrap().len(), 1);
    }
}
