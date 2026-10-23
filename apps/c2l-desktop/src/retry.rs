//! Explicit, file-only retries. Original report is never updated.
use crate::{ScanBatch, io_error_record, make_record, store};
use c2l_indicators::IndicatorDb;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn run(
    source: &Path,
    directory: &Path,
    cancel: &AtomicBool,
    processed: &AtomicUsize,
) -> Result<(ScanBatch, PathBuf), String> {
    let source = std::fs::canonicalize(source).map_err(|e| e.to_string())?;
    let mut header = store::load(&source)?;
    let source_report = header.report.as_ref().ok_or("Relatório sem resumo")?;
    if source_report.engine.is_some() {
        return Err(
            "Repita a análise no modo Antimalware; este atalho é apenas para diagnóstico SHA-256"
                .into(),
        );
    }
    if source_report.interrupted {
        return Err("Este relatório está incompleto. Repita a análise da pasta para confirmar a cobertura antes de reanalisar apenas erros.".into());
    }
    let errors = source_report.errors;
    if errors == 0 {
        return Err("Este relatório não contém erros".into());
    }
    let indicators = header
        .indicator_file
        .as_ref()
        .map(IndicatorDb::load)
        .transpose()
        .map_err(|e| format!("Não foi possível carregar a base original: {e}"))?;
    header.app_version = env!("CARGO_PKG_VERSION").into();
    header.mode = "retry_files".into();
    header.completed_unix_seconds = 0;
    header.failure = None;
    header.report = Some(ScanBatch {
        interrupted: true,
        retry_of: Some(source.clone()),
        ..Default::default()
    });
    let mut writer = store::Writer::create(directory, &header)?;
    let result = (|| -> Result<(), String> {
        for index in 0..errors.div_ceil(crate::RESULT_PAGE_SIZE) {
            if cancel.load(Ordering::Relaxed) {
                return Ok(());
            }
            let page = store::page(&source, true, index)?;
            if page.is_empty() {
                return Err("O relatório de origem mudou durante a reanálise".into());
            }
            for old in page {
                if cancel.load(Ordering::Relaxed) {
                    return Ok(());
                }
                let record = match c2l_scanner::scan_file_cancellable(&old.path, cancel) {
                    Ok(result) => make_record(result, indicators.as_ref()),
                    Err(_) if cancel.load(Ordering::Relaxed) => return Ok(()),
                    Err(error) => io_error_record(old.path, error),
                };
                writer.push(&record)?;
                processed.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    })();
    let report = header.report.as_mut().unwrap();
    report.interrupted = result.is_err();
    report.failure = result.err();
    report.cancelled = cancel.load(Ordering::Relaxed);
    header.completed_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let saved = writer.finish(&header)?;
    Ok((store::load(&saved)?.report.ok_or("Resumo em falta")?, saved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history;
    #[test]
    fn retries_only_failed_paths_and_preserves_original_even_when_cancelled() {
        let temp = std::fs::canonicalize(std::env::temp_dir()).unwrap();
        let dir = temp.join(format!(
            "c2l-retry-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let fixed = dir.join("fixed.txt");
        std::fs::write(&fixed, b"abc").unwrap();
        let folder = dir.join("folder");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("must-not-scan.txt"), b"abc").unwrap();
        let header = history::SavedScan {
            schema_version: 3,
            app_version: "0.3.0".into(),
            completed_unix_seconds: 1,
            target: dir.clone(),
            mode: "folder".into(),
            indicator_file: None,
            report: Some(ScanBatch::default()),
            failure: None,
        };
        let mut writer = store::Writer::create(&dir, &header).unwrap();
        for path in [fixed, dir.join("missing"), folder] {
            writer
                .push(&crate::error_record(path, "previous error".into()))
                .unwrap();
        }
        writer
            .push(&make_record(
                c2l_scanner::ScanResult {
                    path: dir.join("already-scanned"),
                    size_bytes: 3,
                    sha256: "0".repeat(64),
                },
                None,
            ))
            .unwrap();
        for i in 0..100 {
            writer
                .push(&crate::error_record(
                    dir.join(format!("missing-{i}")),
                    "previous error".into(),
                ))
                .unwrap();
        }
        let source = writer.finish(&header).unwrap();
        let original = std::fs::read(&source).unwrap();
        let (report, saved) =
            run(&source, &dir, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
        assert_ne!(source, saved);
        assert_eq!(report.scanned, 1);
        assert_eq!(report.errors, 102);
        assert_eq!(store::page(&saved, false, 1).unwrap().len(), 3);
        assert!(!report.interrupted);
        assert!(report.retry_of.is_some());
        assert_eq!(std::fs::read(&source).unwrap(), original);
        let (stopped, _) =
            run(&source, &dir, &AtomicBool::new(true), &AtomicUsize::new(0)).unwrap();
        assert!(stopped.cancelled);
        assert_eq!(stopped.scanned + stopped.errors, 0);
        assert_eq!(std::fs::read(&source).unwrap(), original);
        let resolved = std::fs::canonicalize(&dir).unwrap();
        assert_eq!(resolved.parent(), Some(temp.as_path()));
        std::fs::remove_dir_all(resolved).unwrap();
    }
}
