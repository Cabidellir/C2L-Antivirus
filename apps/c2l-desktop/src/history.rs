//! Local diagnostic reports only. Reports never drive permission changes or remediation.
use crate::ScanBatch;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_REPORT_BYTES: u64 = 16 * 1024 * 1024;

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

pub fn save(dir: &Path, scan: &SavedScan) -> Result<PathBuf, String> {
    let bytes = serde_json::to_vec_pretty(scan).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_REPORT_BYTES {
        return Err("Relatório excede o limite local de 16 MiB.".to_owned());
    }
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
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn load(path: &Path) -> Result<SavedScan, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_REPORT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_REPORT_BYTES {
        return Err("Relatório excede 16 MiB.".to_owned());
    }
    let scan: SavedScan = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if scan.schema_version != 1 || scan.report.is_some() == scan.failure.is_some() {
        return Err("Versão ou resultado do relatório inválido.".to_owned());
    }
    if let Some(report) = &scan.report
        && (report.records.len() > 2_000
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
            && name.ends_with(".json")
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
