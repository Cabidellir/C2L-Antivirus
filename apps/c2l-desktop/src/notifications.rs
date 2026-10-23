//! A read-only user-session companion, independent of the GUI and service privileges.
use crate::{background, signatures};
use c2l_platform_windows::notification::Tray;
use rusqlite::{Connection, OptionalExtension};
use std::{fs, path::PathBuf, thread, time::Duration};
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn candidate(
    status: &background::Status,
    previous: &str,
    last_sent: u64,
    now: u64,
) -> Option<String> {
    let path = status
        .last_detection_report
        .as_ref()?
        .to_string_lossy()
        .into_owned();
    if status.fresh(now)
        && status.detections > 0
        && path != previous
        && now.saturating_sub(last_sent) >= 900
    {
        Some(path)
    } else {
        None
    }
}
pub fn run(test: bool) -> Result<(), String> {
    let root = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?)
        .join("C2L-Antivirus");
    fs::create_dir_all(&root).map_err(err)?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join("notifications.lock"))
        .map_err(err)?;
    lock.try_lock()
        .map_err(|_| "Já existe um agente de notificações".to_owned())?;
    let mut tray = Tray::new().map_err(err)?;
    if test {
        tray.notify(
            "C2L — teste de notificações",
            "Teste inofensivo: o canal de avisos do Windows está disponível. Não é uma deteção.",
        )
        .map_err(err)?;
        for _ in 0..100 {
            tray.pump();
            thread::sleep(Duration::from_millis(100));
        }
        return Ok(());
    }
    let db = Connection::open(root.join("notifications.sqlite")).map_err(err)?;
    db.execute_batch("PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS cursor(id INTEGER PRIMARY KEY CHECK(id=1), report TEXT NOT NULL, sent INTEGER NOT NULL);").map_err(err)?;
    let (mut previous, mut last_sent): (String, u64) = db
        .query_row("SELECT report,sent FROM cursor WHERE id=1", [], |r| {
            Ok((r.get(0)?, r.get::<_, i64>(1)?.max(0) as u64))
        })
        .optional()
        .map_err(err)?
        .unwrap_or_default();
    let data = background::data_dir()?;
    while !root.join("notifications.disabled").exists() {
        tray.pump();
        if c2l_platform_windows::service_host::is_running().unwrap_or(false)
            && let Ok(status) = background::read_status(&data)
            && let Some(report) = candidate(&status, &previous, last_sent, signatures::now())
        {
            match tray.notify("C2L — deteção registada", "A análise automática registou uma deteção. Abra C2L > Monitorização > relatório com deteções. Nenhum ficheiro foi removido automaticamente.") {
                Ok(()) => {
                    last_sent = signatures::now(); previous = report;
                    db.execute("INSERT INTO cursor VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET report=excluded.report,sent=excluded.sent", rusqlite::params![previous,last_sent as i64]).map_err(err)?;
                }
                Err(e) => { fs::write(root.join("notifications-error.txt"), e.to_string()).map_err(err)?; }
            }
        }
        for _ in 0..30 {
            tray.pump();
            thread::sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notifications_require_fresh_detection_and_deduplicate_with_cooldown() {
        let mut s = background::Status {
            heartbeat: 2000,
            detections: 1,
            last_detection_report: Some("report.sqlite".into()),
            ..Default::default()
        };
        assert!(candidate(&s, "", 0, 2001).is_some());
        assert!(candidate(&s, "report.sqlite", 0, 2001).is_none());
        assert!(candidate(&s, "", 1900, 2001).is_none());
        assert!(candidate(&s, "", 0, 2016).is_none());
        s.detections = 0;
        assert!(candidate(&s, "", 0, 2001).is_none());
    }
}
