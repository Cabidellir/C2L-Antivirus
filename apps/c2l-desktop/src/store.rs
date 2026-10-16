//! Incremental local reports. Transactions preserve only committed batches on interruption.
use crate::{Record, history::SavedScan};
use rusqlite::{Connection, OpenFlags, params};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const APPLICATION_ID: i64 = 0x43324c33;
const BATCH: usize = 128;
const MAX_RECORD: usize = 1024 * 1024;

pub struct Writer {
    connection: Connection,
    pub path: PathBuf,
    pending: usize,
}

impl Writer {
    pub fn create(dir: &Path, header: &SavedScan) -> Result<Self, String> {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let path = dir.join(format!("scan-{stamp:020}-{}.sqlite", std::process::id()));
        // Reserve the filename; never overwrite a previous scan.
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        let connection = Connection::open(&path).map_err(|e| e.to_string())?;
        connection.execute_batch("PRAGMA page_size=4096; PRAGMA max_page_count=524288;
            PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA cache_size=-4096;
            BEGIN IMMEDIATE;
            CREATE TABLE header (id INTEGER PRIMARY KEY CHECK(id=1), json TEXT NOT NULL);
            CREATE TABLE records (id INTEGER PRIMARY KEY, is_error INTEGER NOT NULL, matched INTEGER NOT NULL, cause TEXT NOT NULL, json TEXT NOT NULL);
            CREATE INDEX errors ON records(is_error,id);
            PRAGMA application_id=0x43324c33; PRAGMA user_version=3;").map_err(|e| e.to_string())?;
        connection
            .execute(
                "INSERT INTO header VALUES (1,?1)",
                [serde_json::to_string(header).map_err(|e| e.to_string())?],
            )
            .map_err(|e| e.to_string())?;
        connection
            .execute_batch("COMMIT; BEGIN IMMEDIATE")
            .map_err(|e| e.to_string())?;
        Ok(Self {
            connection,
            path,
            pending: 0,
        })
    }

    pub fn push(&mut self, record: &Record) -> Result<(), String> {
        let json = serde_json::to_string(record).map_err(|e| e.to_string())?;
        if json.len() > MAX_RECORD {
            return Err("Registo excede 1 MiB".into());
        }
        let cause = record
            .diagnostic
            .as_ref()
            .map_or("Sem classificação (relatório antigo)", |d| {
                d.kind.label()
            });
        self.connection
            .execute(
                "INSERT INTO records(is_error,matched,cause,json) VALUES (?1,?2,?3,?4)",
                params![record.error, record.matched, cause, json],
            )
            .map_err(|e| e.to_string())?;
        self.pending += 1;
        if self.pending == BATCH {
            self.connection
                .execute_batch("COMMIT; BEGIN IMMEDIATE")
                .map_err(|e| e.to_string())?;
            self.pending = 0;
        }
        Ok(())
    }

    pub fn finish(self, header: &SavedScan) -> Result<PathBuf, String> {
        self.connection
            .execute(
                "UPDATE header SET json=?1 WHERE id=1",
                [serde_json::to_string(header).map_err(|e| e.to_string())?],
            )
            .map_err(|e| e.to_string())?;
        self.connection
            .execute_batch("COMMIT")
            .map_err(|e| e.to_string())?;
        Ok(self.path)
    }
}

/// Only the active database and its exact SQLite sidecar names are excluded.
pub fn is_active_file(candidate: &Path, active: &Path) -> bool {
    let Some(name) = active.file_name() else {
        return false;
    };
    let names = [
        name.to_os_string(),
        format!("{}-journal", name.to_string_lossy()).into(),
        format!("{}-wal", name.to_string_lossy()).into(),
        format!("{}-shm", name.to_string_lossy()).into(),
    ];
    if !candidate
        .file_name()
        .is_some_and(|n| names.iter().any(|own| own == n))
    {
        return false;
    }
    candidate
        .parent()
        .and_then(|p| fs::canonicalize(p).ok())
        .as_deref()
        == active.parent()
}

/// SQLite needs write access to roll back a hot journal after an abrupt exit.
/// Only repair a recognized C2L database inside our own report directory.
pub fn recover_local(path: &Path, directory: &Path) -> Result<(), String> {
    use std::io::Read;
    let actual = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let own_dir = match fs::canonicalize(directory) {
        Ok(dir) => dir,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if actual.parent() != Some(own_dir.as_path()) {
        return Ok(());
    }
    let mut header = [0_u8; 100];
    fs::File::open(&actual)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_err(|e| e.to_string())?;
    if &header[..16] != b"SQLite format 3\0"
        || u32::from_be_bytes(header[68..72].try_into().unwrap()) != APPLICATION_ID as u32
    {
        return Err("Não é um relatório C2L reconhecido".into());
    }
    let connection = Connection::open_with_flags(&actual, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|e| e.to_string())?;
    connection
        .execute_batch("PRAGMA trusted_schema=OFF; PRAGMA cache_size=-4096;")
        .map_err(|e| e.to_string())?;
    connection
        .query_row("SELECT count(*) FROM header", [], |r| r.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn count(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<usize> {
    let value: i64 = row.get(index)?;
    usize::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}

fn open(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| e.to_string())?;
    connection
        .execute_batch("PRAGMA trusted_schema=OFF; PRAGMA query_only=ON; PRAGMA cache_size=-4096;")
        .map_err(|e| e.to_string())?;
    let id: i64 = connection
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if id != APPLICATION_ID || version != 3 {
        return Err("Base de relatório incompatível".into());
    }
    Ok(connection)
}

pub fn load(path: &Path) -> Result<SavedScan, String> {
    let connection = open(path)?;
    let json: String = connection
        .query_row(
            "SELECT substr(json,1,1048577) FROM header WHERE id=1",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if json.len() > MAX_RECORD {
        return Err("Cabeçalho demasiado grande".into());
    }
    let mut header: SavedScan = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    if header.schema_version != 3 {
        return Err("Cabeçalho incompatível".into());
    }
    let mut report = header.report.take().ok_or("Resumo em falta")?;
    let (total, errors, matched): (usize, usize, usize) = connection
        .query_row(
            "SELECT count(*),coalesce(sum(is_error),0),coalesce(sum(matched),0) FROM records",
            [],
            |r| Ok((count(r, 0)?, count(r, 1)?, count(r, 2)?)),
        )
        .map_err(|e| e.to_string())?;
    if errors > total || matched > total - errors {
        return Err("Contadores inválidos".into());
    }
    report.scanned = total - errors;
    report.errors = errors;
    report.matched = matched;
    report.records.clear();
    report.error_causes = causes(path)?;
    report.store_path = Some(path.to_path_buf());
    header.report = Some(report);
    Ok(header)
}

pub fn page(path: &Path, errors_only: bool, page: usize) -> Result<Vec<Record>, String> {
    let connection = open(path)?;
    let query = if errors_only {
        "SELECT substr(json,1,1048577) FROM records WHERE is_error=1 ORDER BY id LIMIT ?1 OFFSET ?2"
    } else {
        "SELECT substr(json,1,1048577) FROM records ORDER BY id LIMIT ?1 OFFSET ?2"
    };
    let offset = page
        .checked_mul(crate::RESULT_PAGE_SIZE)
        .ok_or("Página inválida")?;
    let mut statement = connection.prepare(query).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(
            params![
                crate::RESULT_PAGE_SIZE as i64,
                i64::try_from(offset).map_err(|e| e.to_string())?
            ],
            |r| r.get::<_, String>(0),
        )
        .map_err(|e| e.to_string())?;
    rows.map(|row| {
        let json = row.map_err(|e| e.to_string())?;
        if json.len() > MAX_RECORD {
            return Err("Registo demasiado grande".into());
        }
        serde_json::from_str(&json).map_err(|e| e.to_string())
    })
    .collect()
}

pub fn causes(path: &Path) -> Result<BTreeMap<String, usize>, String> {
    let connection = open(path)?;
    let mut statement = connection
        .prepare("SELECT substr(json,1,1048577) FROM records WHERE is_error=1")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut counts = BTreeMap::new();
    for row in rows {
        let json = row.map_err(|e| e.to_string())?;
        if json.len() > MAX_RECORD {
            return Err("Registo demasiado grande".into());
        }
        let record: Record = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let label = record
            .diagnostic
            .as_ref()
            .map_or("Sem classificação (relatório antigo)", |d| {
                d.effective_cause().label()
            });
        *counts.entry(label.to_owned()).or_default() += 1;
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScanBatch, ScanMode};
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    struct Sandbox(PathBuf);
    impl Sandbox {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "c2l-store-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&dir).unwrap();
            Self(dir)
        }
    }
    impl Drop for Sandbox {
        fn drop(&mut self) {
            let root = fs::canonicalize(&self.0).unwrap();
            assert_eq!(
                root.parent(),
                Some(fs::canonicalize(std::env::temp_dir()).unwrap().as_path())
            );
            fs::remove_dir_all(root).unwrap();
        }
    }
    fn header() -> SavedScan {
        SavedScan {
            schema_version: 3,
            app_version: "test".into(),
            completed_unix_seconds: 0,
            target: "synthetic".into(),
            mode: "folder".into(),
            indicator_file: None,
            report: Some(ScanBatch {
                interrupted: true,
                ..Default::default()
            }),
            failure: None,
        }
    }
    #[test]
    fn large_report_is_paged_from_disk_and_preserves_error_filter() {
        let temp = Sandbox::new();
        let mut writer = Writer::create(&temp.0, &header()).unwrap();
        for i in 0..60_001 {
            let mut record = crate::error_record(format!("item-{i}").into(), "synthetic".into());
            record.error = i % 1000 == 0;
            writer.push(&record).unwrap();
        }
        let mut summary = header();
        summary.report.as_mut().unwrap().interrupted = false;
        let path = writer.finish(&summary).unwrap();
        let report = load(&path).unwrap().report.unwrap();
        assert!(report.records.is_empty());
        assert_eq!(report.scanned, 59_940);
        assert_eq!(report.errors, 61);
        assert!(!report.interrupted);
        let last = page(&path, false, 600).unwrap();
        assert_eq!(last.len(), 1);
        assert_eq!(last[0].path, PathBuf::from("item-60000"));
        assert_eq!(page(&path, true, 0).unwrap().len(), 61);
        assert!(page(&path, true, 1).unwrap().is_empty());
        assert_eq!(report.error_causes.values().sum::<usize>(), 61);
        assert!(page(&path, false, usize::MAX).is_err());
    }
    #[test]
    fn interrupted_writer_preserves_only_committed_batches() {
        let temp = Sandbox::new();
        let mut writer = Writer::create(&temp.0, &header()).unwrap();
        let path = writer.path.clone();
        for _ in 0..BATCH + 3 {
            writer
                .push(&crate::error_record("test".into(), "denied".into()))
                .unwrap();
        }
        drop(writer);
        let report = load(&path).unwrap().report.unwrap();
        assert!(report.interrupted);
        assert_eq!(report.errors, BATCH);
    }
    #[test]
    fn storage_failure_keeps_committed_prefix_without_completion_marker() {
        let temp = Sandbox::new();
        let mut writer = Writer::create(&temp.0, &header()).unwrap();
        let path = writer.path.clone();
        for _ in 0..BATCH {
            writer
                .push(&crate::error_record("test".into(), "denied".into()))
                .unwrap();
        }
        writer
            .connection
            .execute_batch("PRAGMA max_page_count=16")
            .unwrap();
        let large = crate::error_record("test".into(), "x".repeat(200_000));
        assert!(writer.push(&large).is_err());
        drop(writer);
        let report = load(&path).unwrap().report.unwrap();
        assert!(report.interrupted);
        assert_eq!(report.errors, BATCH);
    }
    #[test]
    fn scanner_records_limits_cancellation_and_individual_errors() {
        let temp = Sandbox::new();
        let input = temp.0.join("input");
        fs::create_dir(&input).unwrap();
        for i in 0..7 {
            fs::write(input.join(format!("{i}.txt")), b"abc").unwrap();
        }
        let reports = temp.0.join("reports");
        let progress = AtomicUsize::new(0);
        let (report, _) = crate::execute_persisted(
            ScanMode::Folder,
            input.clone(),
            None,
            3,
            &AtomicBool::new(false),
            &progress,
            &reports,
        )
        .unwrap();
        assert_eq!(report.scanned, 3);
        assert!(report.truncated);
        let (report, _) = crate::execute_persisted(
            ScanMode::Folder,
            input,
            None,
            usize::MAX,
            &AtomicBool::new(true),
            &progress,
            &reports,
        )
        .unwrap();
        assert!(report.cancelled);
        assert!(!report.interrupted);
        let (report, path) = crate::execute_persisted(
            ScanMode::File,
            temp.0.join("missing"),
            None,
            usize::MAX,
            &AtomicBool::new(false),
            &progress,
            &reports,
        )
        .unwrap();
        assert_eq!(report.errors, 1);
        assert_eq!(
            page(&path, true, 0).unwrap()[0]
                .diagnostic
                .as_ref()
                .unwrap()
                .kind,
            crate::diagnostics::Cause::NotFound
        );
    }
    #[test]
    fn crash_child() {
        let Some(dir) = std::env::var_os("C2L_TEST_CRASH_DIR") else {
            return;
        };
        let mut writer = Writer::create(Path::new(&dir), &header()).unwrap();
        for _ in 0..BATCH {
            writer
                .push(&crate::error_record("test".into(), "denied".into()))
                .unwrap();
        }
        writer
            .connection
            .execute_batch("PRAGMA cache_size=1")
            .unwrap();
        for _ in 0..3 {
            writer
                .push(&crate::error_record("pending".into(), "x".repeat(20_000)))
                .unwrap();
        }
        std::process::exit(0); // Deliberately skip destructors to leave a hot journal.
    }
    #[test]
    fn recovers_committed_batches_after_process_exit() {
        let temp = Sandbox::new();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "store::tests::crash_child"])
            .env("C2L_TEST_CRASH_DIR", &temp.0)
            .status()
            .unwrap();
        assert!(status.success());
        let path = fs::read_dir(&temp.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|e| e == "sqlite"))
            .unwrap();
        recover_local(&path, &temp.0).unwrap();
        let report = load(&path).unwrap().report.unwrap();
        assert!(report.interrupted);
        assert_eq!(report.errors, BATCH);
    }
    #[test]
    fn active_report_is_excluded_and_counted() {
        let temp = Sandbox::new();
        fs::write(temp.0.join("file.txt"), b"abc").unwrap();
        let (report, _) = crate::execute_persisted(
            ScanMode::Folder,
            temp.0.clone(),
            None,
            usize::MAX,
            &AtomicBool::new(false),
            &AtomicUsize::new(0),
            &temp.0,
        )
        .unwrap();
        assert_eq!(report.scanned, 1);
        assert_eq!(report.errors, 0);
        assert!(report.skipped >= 1);
        assert!(!report.truncated);
    }
    #[test]
    #[ignore = "Creates 50129 real files for the scale acceptance check"]
    fn real_scan_above_fifty_thousand() {
        let temp = Sandbox::new();
        let input = temp.0.join("input");
        fs::create_dir(&input).unwrap();
        for i in 0..50_129 {
            fs::write(input.join(format!("{i}.txt")), b"abc").unwrap();
        }
        let start = std::time::Instant::now();
        let (report, path) = crate::execute_persisted(
            ScanMode::Folder,
            input,
            None,
            usize::MAX,
            &AtomicBool::new(false),
            &AtomicUsize::new(0),
            &temp.0.join("reports"),
        )
        .unwrap();
        println!(
            "SCALE: files={} scan_seconds={:.3} database_bytes={} retained_records={}",
            report.scanned,
            start.elapsed().as_secs_f64(),
            fs::metadata(&path).unwrap().len(),
            report.records.len()
        );
        assert_eq!(report.scanned, 50_129);
        assert_eq!(report.errors, 0);
        assert!(!report.truncated && !report.cancelled && !report.interrupted);
        assert!(report.records.is_empty());
        assert_eq!(page(&path, false, 501).unwrap().len(), 29);
    }

    #[test]
    fn rejects_foreign_or_corrupt_database() {
        let temp = Sandbox::new();
        let path = temp.0.join("foreign.sqlite");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch("CREATE TABLE unrelated(id INTEGER)")
            .unwrap();
        drop(connection);
        assert!(load(&path).is_err());
        fs::write(&path, b"not sqlite").unwrap();
        assert!(load(&path).is_err());
    }
}
