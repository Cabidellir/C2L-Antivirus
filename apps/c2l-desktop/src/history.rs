//! Local diagnostic reports only. Reports never drive permission changes or remediation.
use crate::ScanBatch;
use serde::{Deserialize, Serialize};
use std::fs;
#[cfg(test)]
use std::fs::OpenOptions;
use std::io::Read;
#[cfg(test)]
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_REPORT_BYTES: u64 = 128 * 1024 * 1024;

#[cfg(test)]
struct LimitedWriter<W> {
    inner: W,
    remaining: u64,
}
#[cfg(test)]
impl<W: Write> Write for LimitedWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() as u64 > self.remaining {
            return Err(io::Error::other("Relatório excede 128 MiB"));
        }
        let count = self.inner.write(bytes)?;
        self.remaining -= count as u64;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedScan {
    pub schema_version: u32,
    pub app_version: String,
    pub completed_unix_seconds: u64,
    pub target: PathBuf,
    pub mode: String,
    pub indicator_file: Option<PathBuf>,
    pub report: Option<ScanBatch>,
    pub failure: Option<String>,
}

pub fn directory() -> Result<PathBuf, String> {
    std::env::var_os("LOCALAPPDATA")
        .filter(|value| !value.is_empty())
        .map(|base| PathBuf::from(base).join("C2L-Antivirus").join("reports"))
        .ok_or_else(|| "LOCALAPPDATA indisponível; histórico não gravado.".to_owned())
}

#[cfg(test)]
pub fn save(dir: &Path, scan: &SavedScan) -> Result<PathBuf, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    let name = format!("scan-{:020}-{}.json", stamp.as_nanos(), std::process::id());
    let path = dir.join(name);
    let temporary = path.with_extension("partial");
    // A partially written report never becomes visible to the history reader.
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    {
        let mut writer = LimitedWriter {
            inner: BufWriter::new(&mut file),
            remaining: MAX_REPORT_BYTES,
        };
        serde_json::to_writer_pretty(&mut writer, scan).map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
    }
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn load(path: &Path) -> Result<SavedScan, String> {
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("sqlite"))
    {
        if let Ok(dir) = directory() {
            crate::store::recover_local(path, &dir)?;
        }
        return crate::store::load(path);
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_REPORT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_REPORT_BYTES {
        return Err("Relatório excede 128 MiB.".to_owned());
    }
    let scan: SavedScan = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if !matches!(scan.schema_version, 1 | 2) || scan.report.is_some() == scan.failure.is_some() {
        return Err("Versão ou resultado do relatório inválido.".to_owned());
    }
    if let Some(report) = &scan.report
        && (report.records.len() > crate::MAX_SCAN_FILES
            || report.scanned != report.records.iter().filter(|r| !r.error).count()
            || report.errors != report.records.iter().filter(|r| r.error).count()
            || report.matched != report.records.iter().filter(|r| r.matched).count())
    {
        return Err("Contadores do relatório inconsistentes.".to_owned());
    }
    Ok(scan)
}

pub fn latest(dir: &Path) -> Result<Option<(PathBuf, SavedScan)>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("scan-")
            && (name.ends_with(".json") || name.ends_with(".sqlite"))
            && entry.file_type().map_err(|e| e.to_string())?.is_file()
        {
            paths.push(entry.path());
        }
    }
    paths.sort();
    paths
        .pop()
        .map(|path| load(&path).map(|scan| (path, scan)))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error_record, summarize};

    #[test]
    fn fifty_thousand_records_and_cancelled_state_survive_disk_roundtrip() {
        let temp = fs::canonicalize(std::env::temp_dir()).unwrap();
        let dir = temp.join(format!(
            "c2l-history-large-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut report = summarize(
            (0..crate::MAX_SCAN_FILES)
                .map(|i| error_record(format!("sample-{i}").into(), "synthetic read error".into()))
                .collect(),
            false,
        );
        report.cancelled = true;
        report.entry_limit = Some(crate::MAX_SCAN_FILES);
        let scan = SavedScan {
            schema_version: 2,
            app_version: "test".into(),
            completed_unix_seconds: 123,
            target: "synthetic".into(),
            mode: "folder".into(),
            indicator_file: None,
            report: Some(report),
            failure: None,
        };
        let path = save(&dir, &scan).unwrap();
        let restored = load(&path).unwrap().report.unwrap();
        assert_eq!(restored.records.len(), crate::MAX_SCAN_FILES);
        assert_eq!(restored.errors, crate::MAX_SCAN_FILES);
        assert!(restored.cancelled);
        assert_eq!(restored.entry_limit, Some(crate::MAX_SCAN_FILES));
        let resolved = fs::canonicalize(&dir).unwrap();
        assert_eq!(resolved.parent(), Some(temp.as_path()));
        fs::remove_dir_all(resolved).unwrap();
    }

    #[test]
    fn writer_enforces_budget_before_allocating_an_entire_report() {
        let mut writer = LimitedWriter {
            inner: Vec::new(),
            remaining: 3,
        };
        writer.write_all(b"abc").unwrap();
        assert!(writer.write_all(b"d").is_err());
        assert_eq!(writer.inner, b"abc");
    }

    #[test]
    fn legacy_reports_default_to_not_cancelled_and_unknown_limit() {
        let report: ScanBatch = serde_json::from_str(
            r#"{"records":[],"scanned":0,"matched":0,"errors":0,"truncated":true}"#,
        )
        .unwrap();
        assert!(!report.cancelled);
        assert_eq!(report.entry_limit, None);
    }

    #[test]
    fn reports_survive_reload_and_partial_files_are_ignored() {
        let dir = std::env::temp_dir().join(format!(
            "c2l-history-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let scan = SavedScan {
            schema_version: 1,
            app_version: "test".into(),
            completed_unix_seconds: 123,
            target: PathBuf::from("alvo"),
            mode: "folder".into(),
            indicator_file: None,
            report: Some(summarize(
                vec![error_record(
                    PathBuf::from("ficheiro"),
                    "access denied (os error 5)".into(),
                )],
                true,
            )),
            failure: None,
        };
        let path = save(&dir, &scan).unwrap();
        fs::write(dir.join("scan-99999999999999999999.partial"), b"incomplete").unwrap();
        let (restored_path, restored) = latest(&dir).unwrap().unwrap();
        assert_eq!(restored_path, path);
        let report = restored.report.unwrap();
        assert_eq!(report.errors, 1);
        assert!(report.truncated);
        assert!(report.records[0].verdict.contains("os error 5"));
        fs::write(&path, b"broken").unwrap();
        assert!(latest(&dir).is_err());
        // Only this test's uniquely created directory is removed.
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reports_reject_inconsistent_counts_and_write_failures_are_returned() {
        let dir = std::env::temp_dir().join(format!(
            "c2l-history-failure-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let scan = SavedScan {
            schema_version: 1,
            app_version: "test".into(),
            completed_unix_seconds: 123,
            target: "missing".into(),
            mode: "file".into(),
            indicator_file: None,
            report: None,
            failure: Some("not found".into()),
        };
        let path = save(&dir, &scan).unwrap();
        assert_eq!(load(&path).unwrap().failure.as_deref(), Some("not found"));
        assert!(save(&path, &scan).is_err());
        let mut invalid = scan;
        invalid.failure = None;
        invalid.report = Some(ScanBatch {
            errors: 9,
            ..Default::default()
        });
        fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(load(&path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
