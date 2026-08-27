use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rusqlite::params;

use crate::domain::NewQso;

use super::{DmrFilter, DstarFilter, Ft8Filter, QsoRepository, QsoSelection, YsfFilter};
use crate::database::{inspect_database, HealthStatus};

#[test]
#[ignore = "manual deterministic large-volume benchmark"]
fn benchmarks_deterministic_large_database() -> Result<(), Box<dyn Error>> {
    let count = std::env::var("DHRL_STRESS_QSOS")
        .unwrap_or_else(|_| "1000".to_owned())
        .parse::<usize>()?;
    if count == 0 {
        return Err("DHRL_STRESS_QSOS must be greater than zero".into());
    }

    let directory = temporary_directory(count);
    fs::create_dir_all(&directory)?;
    let database_path = directory.join("stress.sqlite3");
    let backup_path = directory.join("backup.sqlite3");
    let result = run_benchmark(count, &database_path, &backup_path);
    let _ = fs::remove_dir_all(&directory);
    result
}

fn run_benchmark(
    count: usize,
    database_path: &PathBuf,
    backup_path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let repository = QsoRepository::open(database_path)?;
    let generation = measure(|| seed_database(&repository, count))?;
    drop(repository);

    let started = Instant::now();
    let repository = QsoRepository::open(database_path)?;
    let opening = started.elapsed();
    print_query_plans(&repository)?;
    benchmark_identity_lookup(&repository, count)?;
    let health = measure(|| Ok::<_, Box<dyn Error>>(inspect_database(database_path)))?;
    if health.1.status != HealthStatus::HealthyCurrent || health.1.qso_count != Some(count as u64) {
        return Err("health check did not report the generated database as healthy".into());
    }

    let first_page = measure(|| repository.search_page("", 0, 100))?;
    let middle_offset = count.saturating_div(2).saturating_sub(50);
    let middle_page = measure(|| repository.search_page("", middle_offset, 100))?;
    let final_offset = count.saturating_sub(100);
    let final_page = measure(|| repository.search_page("", final_offset, 100))?;
    let callsign = measure(|| repository.search_page("PY00042", 0, 100))?;
    let mode = measure(|| repository.search_page("DMR", 0, 100))?;

    let dmr_id = measure(|| {
        repository.search_dmr_page(
            &DmrFilter {
                dmr_id: Some(1_000_040),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let dmr_tg = measure(|| {
        repository.search_dmr_page(
            &DmrFilter {
                talkgroup: Some(724),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let dmr_network = measure(|| {
        repository.search_dmr_page(
            &DmrFilter {
                network: Some("Network-2".into()),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let dmr_repeater = measure(|| {
        repository.search_dmr_page(
            &DmrFilter {
                repeater: Some("RPT000".into()),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let dmr_timeslot = measure(|| {
        repository.search_dmr_page(
            &DmrFilter {
                timeslot: Some(1),
                ..Default::default()
            },
            0,
            100,
        )
    })?;

    let dstar_route = measure(|| {
        repository.search_dstar_page(
            &DstarFilter {
                reflector: Some("REF000".into()),
                module: Some("A".into()),
                rpt1: Some("RPT000 B".into()),
            },
            0,
            100,
        )
    })?;

    let ysf_room = measure(|| {
        repository.search_ysf_page(
            &YsfFilter {
                room: Some("BRAZIL".into()),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let ysf_node = measure(|| {
        repository.search_ysf_page(
            &YsfFilter {
                wires_x_node: Some("PY2YSF-ND".into()),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let ysf_dg_id = measure(|| {
        repository.search_ysf_page(
            &YsfFilter {
                dg_id: Some(10),
                ..Default::default()
            },
            0,
            100,
        )
    })?;

    let ft8_callsign = measure(|| {
        repository.search_ft8_page(
            &Ft8Filter {
                callsign: Some("PY00041".into()),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let ft8_grid = measure(|| {
        repository.search_ft8_page(
            &Ft8Filter {
                grid: Some("GG41".into()),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let ft8_band = measure(|| {
        repository.search_ft8_page(
            &Ft8Filter {
                band: Some("20m".into()),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let ft8_period = measure(|| {
        repository.search_ft8_page(
            &Ft8Filter {
                start_utc: Some(1_700_000_000 + count.saturating_div(3) as i64),
                end_utc: Some(1_700_000_000 + (count.saturating_mul(2).saturating_div(3)) as i64),
                ..Default::default()
            },
            0,
            100,
        )
    })?;
    let ft8_snr = measure(|| {
        repository.search_ft8_page(
            &Ft8Filter {
                minimum_snr_received_db: Some(-15),
                maximum_snr_received_db: Some(-5),
                ..Default::default()
            },
            0,
            100,
        )
    })?;

    let backup = measure(|| repository.backup_to(backup_path))?;
    let backup_repository = QsoRepository::open(backup_path)?;
    backup_repository.verify_integrity()?;
    if backup_repository.search_page("", 0, 1)?.total != count {
        return Err("backup record count differs from source".into());
    }
    drop(backup_repository);

    let filtered_small =
        measure(|| repository.export_adif_selection(&QsoSelection::General("PY00042".into())))?;
    let filtered_large =
        measure(|| repository.export_adif_selection(&QsoSelection::General("DMR".into())))?;
    let export = measure(|| repository.export_adif())?;
    if export.1.records.len() != count {
        return Err("ADIF export record count differs from source".into());
    }
    if filtered_small.1.records.is_empty() || filtered_large.1.records.is_empty() {
        return Err("filtered ADIF export returned no generated records".into());
    }
    let serialize = measure(|| Ok::<_, Box<dyn Error>>(crate::adif::export(&export.1)))?;

    println!("\nDHRL_STRESS_RESULT volume={count}");
    print_metric("generate", generation.0);
    print_metric("open_and_verify", opening);
    print_metric("health_check_read_only", health.0);
    print_metric("first_page", first_page.0);
    print_metric("middle_page", middle_page.0);
    print_metric("final_page", final_page.0);
    print_metric("search_callsign", callsign.0);
    print_metric("search_mode", mode.0);
    print_metric("dmr_id", dmr_id.0);
    print_metric("dmr_talkgroup", dmr_tg.0);
    print_metric("dmr_network", dmr_network.0);
    print_metric("dmr_repeater", dmr_repeater.0);
    print_metric("dmr_timeslot", dmr_timeslot.0);
    print_metric("dstar_reflector_module_rpt1", dstar_route.0);
    print_metric("ysf_room", ysf_room.0);
    print_metric("ysf_node", ysf_node.0);
    print_metric("ysf_dg_id", ysf_dg_id.0);
    print_metric("ft8_callsign", ft8_callsign.0);
    print_metric("ft8_grid", ft8_grid.0);
    print_metric("ft8_band", ft8_band.0);
    print_metric("ft8_period", ft8_period.0);
    print_metric("ft8_snr", ft8_snr.0);
    print_metric("backup", backup.0);
    print_metric("export_adif_filtered_small", filtered_small.0);
    print_metric("export_adif_filtered_large", filtered_large.0);
    print_metric("export_adif_domain", export.0);
    print_metric("serialize_adif", serialize.0);
    println!("database_bytes={}", fs::metadata(database_path)?.len());
    println!("backup_bytes={}", fs::metadata(backup_path)?.len());
    println!("adif_bytes={}", serialize.1.len());
    Ok(())
}

fn print_query_plans(repository: &QsoRepository) -> rusqlite::Result<()> {
    println!("DHRL_QUERY_PLAN first_page");
    print_query_plan(
        repository,
        "EXPLAIN QUERY PLAN
         SELECT id FROM qsos
         ORDER BY datetime_start_utc DESC, id DESC LIMIT 100 OFFSET 0",
    )?;
    println!("DHRL_QUERY_PLAN identity_create");
    print_query_plan(
        repository,
        "EXPLAIN QUERY PLAN
         SELECT id FROM qsos
         WHERE callsign = 'PY00042' COLLATE NOCASE
           AND datetime_start_utc = 1700000042
           AND frequency_hz = 145500000
           AND mode = 'SSB' COLLATE NOCASE
         LIMIT 1",
    )?;
    println!("DHRL_QUERY_PLAN identity_edit");
    print_query_plan(
        repository,
        "EXPLAIN QUERY PLAN
         SELECT id FROM qsos
         WHERE callsign = 'PY00042' COLLATE NOCASE
           AND datetime_start_utc = 1700000042
           AND frequency_hz = 145500000
           AND mode = 'SSB' COLLATE NOCASE
           AND id <> 43
         LIMIT 1",
    )?;
    println!("DHRL_QUERY_PLAN callsign_substring");
    print_query_plan(
        repository,
        "EXPLAIN QUERY PLAN
         SELECT id FROM qsos
         WHERE callsign LIKE '%PY00042%' COLLATE NOCASE
         ORDER BY datetime_start_utc DESC, id DESC LIMIT 100",
    )?;
    println!("DHRL_QUERY_PLAN dmr_talkgroup");
    print_query_plan(
        repository,
        "EXPLAIN QUERY PLAN
         SELECT q.id FROM qsos q
         JOIN dmr_metadata d ON d.qso_id = q.id
         WHERE d.talkgroup = 724
         ORDER BY q.datetime_start_utc DESC, q.id DESC LIMIT 100",
    )?;
    println!("DHRL_QUERY_PLAN ysf_room_node_dg_id");
    print_query_plan(
        repository,
        "EXPLAIN QUERY PLAN
         SELECT q.id FROM qsos q
         JOIN ysf_metadata y ON y.qso_id = q.id
         WHERE y.room LIKE '%BRAZIL%' COLLATE NOCASE
           AND y.wires_x_node LIKE '%PY2YSF-ND%' COLLATE NOCASE
           AND (y.tx_dg_id = 10 OR y.rx_dg_id = 10)
         ORDER BY q.datetime_start_utc DESC, q.id DESC LIMIT 100",
    )?;
    println!("DHRL_QUERY_PLAN dstar_route");
    print_query_plan(
        repository,
        "EXPLAIN QUERY PLAN
         SELECT q.id FROM qsos q
         JOIN dstar_metadata ds ON ds.qso_id = q.id
         WHERE ds.reflector LIKE '%REF000%' COLLATE NOCASE
           AND ds.module LIKE '%A%' COLLATE NOCASE
           AND ds.rpt1 LIKE '%RPT000 B%' COLLATE NOCASE
         ORDER BY q.datetime_start_utc DESC, q.id DESC LIMIT 100",
    )?;
    println!("DHRL_QUERY_PLAN ft8_snr");
    print_query_plan(
        repository,
        "EXPLAIN QUERY PLAN
         SELECT q.id FROM qsos q
         JOIN ft8_metadata f ON f.qso_id = q.id
         WHERE f.snr_received_db BETWEEN -15 AND -5
         ORDER BY q.datetime_start_utc DESC, q.id DESC LIMIT 100",
    )
}

fn print_query_plan(repository: &QsoRepository, sql: &str) -> rusqlite::Result<()> {
    let mut statement = repository.connection.prepare(sql)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(3))?;
    for detail in rows {
        println!("  {}", detail?);
    }
    Ok(())
}

fn seed_database(repository: &QsoRepository, count: usize) -> rusqlite::Result<()> {
    let transaction = repository.connection.unchecked_transaction()?;
    {
        let mut qso_statement = transaction.prepare_cached(
            "INSERT INTO qsos (
                callsign, datetime_start_utc, frequency_hz, band, mode,
                rst_sent, rst_received, grid_locator, name, qth, notes,
                created_at_utc, updated_at_utc
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
        )?;
        let mut route_statement = transaction.prepare_cached(
            "INSERT INTO digital_routes(qso_id, access_type, network, repeater_callsign, hotspot)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        let mut dmr_statement = transaction.prepare_cached(
            "INSERT INTO dmr_metadata(
                qso_id, remote_dmr_id, local_dmr_id, talkgroup, timeslot,
                color_code, call_type, notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'group', ?7)",
        )?;
        let mut ft8_statement = transaction.prepare_cached(
            "INSERT INTO ft8_metadata(
                qso_id, snr_sent_db, snr_received_db, power_watts,
                audio_frequency_hz, source_software, protocol, final_message
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'WSJT-X', 'FT8', 'RR73')",
        )?;
        let mut dstar_statement = transaction.prepare_cached(
            "INSERT INTO dstar_metadata(
                qso_id, reflector, module, mycall, urcall, rpt1, rpt2, notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        let mut ysf_statement = transaction.prepare_cached(
            "INSERT INTO ysf_metadata(
                qso_id, room, wires_x_node, repeater, network, access_type,
                tx_dg_id, rx_dg_id, notes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;

        for index in 0..count {
            let mode_index = index / 5;
            let mode = match index % 5 {
                0 => "DMR",
                1 => "FT8",
                2 => "SSB",
                3 => "DSTAR",
                _ => "C4FM",
            };
            let band = match mode {
                "DMR" | "DSTAR" | "C4FM" => "70cm",
                "FT8" => "20m",
                _ => "2m",
            };
            let frequency_hz = match band {
                "70cm" => 438_500_000,
                "20m" => 14_074_000,
                "2m" => 145_500_000,
                _ => 7_074_000,
            };
            qso_statement.execute(params![
                format!("PY{:05}", index % 10_000),
                1_700_000_000 + index as i64,
                frequency_hz,
                band,
                mode,
                "59",
                "59",
                format!("GG{:02}", index % 100),
                format!("Operator {}", index % 50),
                format!("QTH {}", index % 25),
                format!("Deterministic stress record {index}"),
                1_700_000_000 + index as i64,
            ])?;
            let qso_id = transaction.last_insert_rowid();
            if mode == "DMR" {
                route_statement.execute(params![
                    qso_id,
                    if index % 2 == 0 {
                        "repeater"
                    } else {
                        "hotspot"
                    },
                    format!("Network-{}", mode_index % 5),
                    format!("RPT{:03}", mode_index % 20),
                    format!("HOT{:03}", mode_index % 20),
                ])?;
                dmr_statement.execute(params![
                    qso_id,
                    1_000_000 + index as i64,
                    2_000_000 + index as i64,
                    [724, 9, 91, 3100][mode_index % 4],
                    1 + mode_index % 2,
                    mode_index % 16,
                    format!("DMR stress {index}"),
                ])?;
            } else if mode == "FT8" {
                ft8_statement.execute(params![
                    qso_id,
                    -20 + (index % 41) as i64,
                    -20 + ((index * 7) % 41) as i64,
                    5 + index % 95,
                    300 + index % 2_700,
                ])?;
            } else if mode == "DSTAR" {
                dstar_statement.execute(params![
                    qso_id,
                    format!(
                        "REF{:03} {}",
                        mode_index % 5,
                        ['A', 'B', 'C'][mode_index % 3]
                    ),
                    ['A', 'B', 'C'][mode_index % 3].to_string(),
                    format!("PY{:05} G", index % 10_000),
                    if mode_index % 2 == 0 {
                        "CQCQCQ"
                    } else {
                        "REFLINK"
                    },
                    format!("RPT{:03} B", mode_index % 20),
                    format!("RPT{:03} G", (mode_index + 1) % 20),
                    format!("D-STAR stress {index}"),
                ])?;
            } else if mode == "C4FM" {
                let rooms = ["BRAZIL", "America-Link", "EUROPE", "PARROT", "LOCAL-SP"];
                let access_types = ["repeater", "hotspot", "simplex"];
                ysf_statement.execute(params![
                    qso_id,
                    rooms[mode_index % rooms.len()],
                    format!("PY2YSF-ND{:02}", mode_index % 12),
                    format!("PY2RPT-{:02}", mode_index % 20),
                    if mode_index % 2 == 0 {
                        "WIRES-X"
                    } else {
                        "YSF Reflector"
                    },
                    access_types[mode_index % access_types.len()],
                    [10, 0, 1, 32, 99][mode_index % 5],
                    [10, 0, 2, 32, 99][mode_index % 5],
                    format!("YSF/C4FM stress {index}"),
                ])?;
            }
        }
    }
    transaction.commit()
}

fn benchmark_identity_lookup(
    repository: &QsoRepository,
    count: usize,
) -> Result<(), Box<dyn Error>> {
    let iterations = std::env::var("DHRL_STRESS_IDENTITY_ITERATIONS")
        .unwrap_or_else(|_| "200".to_owned())
        .parse::<usize>()?;
    if iterations == 0 {
        return Err("DHRL_STRESS_IDENTITY_ITERATIONS must be greater than zero".into());
    }

    let index = count / 2;
    let mode = match index % 5 {
        0 => "DMR",
        1 => "FT8",
        2 => "SSB",
        3 => "DSTAR",
        _ => "C4FM",
    };
    let frequency_hz = match mode {
        "DMR" | "DSTAR" | "C4FM" => 438_500_000,
        "FT8" => 14_074_000,
        _ => 145_500_000,
    };
    let hit = NewQso::new(
        format!("PY{:05}", index % 10_000),
        1_700_000_000 + index as i64,
        frequency_hz,
        mode,
    )?;
    let miss = NewQso::new(
        "ZZ0MISS",
        hit.datetime_start_utc,
        hit.frequency_hz,
        &hit.mode,
    )?;
    let hit_id = index as i64 + 1;

    println!("DHRL_IDENTITY_RESULT iterations={iterations}");
    measure_repeated("identity_hit", iterations, || {
        repository.find_qso_identity_match(&hit, None)
    })?;
    measure_repeated("identity_miss", iterations, || {
        repository.find_qso_identity_match(&miss, None)
    })?;
    measure_repeated("identity_self", iterations, || {
        repository.find_qso_identity_match(&hit, Some(hit_id))
    })?;

    let duplicate_id = repository.insert(&hit, 1_800_000_000)?;
    measure_repeated("identity_collision", iterations, || {
        repository.find_qso_identity_match(&hit, Some(hit_id))
    })?;
    repository.delete(duplicate_id)?;
    Ok(())
}

fn measure_repeated<T>(
    label: &str,
    iterations: usize,
    mut operation: impl FnMut() -> rusqlite::Result<T>,
) -> rusqlite::Result<()> {
    let started = Instant::now();
    for _ in 0..iterations {
        operation()?;
    }
    let total = started.elapsed();
    println!(
        "{label}_total_ms={:.3} {label}_avg_ms={:.6}",
        total.as_secs_f64() * 1_000.0,
        total.as_secs_f64() * 1_000.0 / iterations as f64
    );
    Ok(())
}

fn measure<T, E>(operation: impl FnOnce() -> Result<T, E>) -> Result<(Duration, T), E> {
    let started = Instant::now();
    operation().map(|value| (started.elapsed(), value))
}

fn print_metric(label: &str, duration: Duration) {
    println!("{label}_ms={:.3}", duration.as_secs_f64() * 1_000.0);
}

fn temporary_directory(count: usize) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "dhrl-stress-{}-{count}-{nonce}",
        std::process::id()
    ))
}
