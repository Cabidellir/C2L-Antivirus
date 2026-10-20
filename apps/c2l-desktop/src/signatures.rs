//! Generation-based database updates. Publish only a successfully loaded official snapshot.
use crate::antimalware::{Engine, hidden_command};
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    process::Stdio,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
#[derive(Debug, PartialEq)]
pub struct Info {
    pub version: u64,
    pub built: u64,
}
impl Info {
    pub fn parse(header: &[u8], now: u64) -> Result<Self> {
        let text = std::str::from_utf8(header).map_err(err)?;
        let fields: Vec<_> = text.trim().split(':').collect();
        if fields.len() != 9 || fields[0] != "ClamAV-VDB" {
            return Err("Cabeçalho de base inválido".into());
        }
        let version = fields[2].parse::<u64>().map_err(err)?;
        let built = fields[8].parse::<u64>().map_err(err)?;
        if version == 0
            || version > i64::MAX as u64
            || built == 0
            || built > now.saturating_add(86400)
        {
            return Err("Versão/data da base inválida ou relógio incorreto".into());
        }
        Ok(Self { version, built })
    }
    pub fn label(&self, now: u64) -> String {
        let hours = now.saturating_sub(self.built) / 3600;
        format!(
            "Base diária {} · idade {} h{} · autenticidade verificada pelo motor ao carregar",
            self.version,
            hours,
            if hours > 168 {
                " · DESATUALIZADA: atualizar antes de confiar na análise"
            } else {
                ""
            }
        )
    }
}
pub fn read(database: &Path) -> Result<Info> {
    let path = ["daily.cld", "daily.cvd"]
        .iter()
        .map(|n| database.join(n))
        .find(|p| p.is_file())
        .ok_or("Base diária ausente")?;
    let mut bytes = [0; 512];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(err)?;
    Info::parse(&bytes, now())
}
fn state(root: &Path) -> Result<Connection> {
    let db = Connection::open(root.join("signatures.sqlite")).map_err(err)?;
    db.execute_batch("PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS active(id INTEGER PRIMARY KEY CHECK(id=1), generation TEXT NOT NULL, version INTEGER NOT NULL);").map_err(err)?;
    Ok(db)
}
pub fn active(root: &Path) -> Result<PathBuf> {
    if !root.join("signatures.sqlite").exists() {
        return Ok(root.join("database"));
    }
    let db = state(root)?;
    let generation: Option<String> = db
        .query_row("SELECT generation FROM active WHERE id=1", [], |r| r.get(0))
        .optional()
        .map_err(err)?;
    match generation {
        None => Ok(root.join("database")),
        Some(name)
            if name.starts_with("generation-")
                && name
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c.is_ascii_lowercase()) =>
        {
            Ok(root.join(name))
        }
        _ => Err("Referência de base inválida".into()),
    }
}
fn run(command: &mut std::process::Command, timeout: Duration) -> Result<()> {
    let mut child = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(err)?;
    let _job = c2l_platform_windows::ChildJob::attach(&mut child).map_err(err)?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err(format!("Processo falhou: {status}"))
                };
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(err(e));
            }
            _ => {}
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Tempo limite; base anterior preservada".into());
        }
        thread::sleep(Duration::from_millis(100));
    }
}
fn publish(db: &Connection, name: &str, new: u64, previous: u64) -> Result<()> {
    if new < previous {
        return Err("Downgrade de base rejeitado".into());
    }
    db.execute("INSERT INTO active VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET generation=excluded.generation,version=excluded.version",params![name,new as i64]).map_err(err)?;
    Ok(())
}
pub fn update(engine: &Engine) -> Result<String> {
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(engine.root.join("update.lock"))
        .map_err(err)?;
    lock.try_lock()
        .map_err(|_| "Já existe outra atualização em curso".to_owned())?;
    let db = state(&engine.root)?;
    let previous = db
        .query_row("SELECT version FROM active WHERE id=1", [], |r| {
            r.get::<_, i64>(0)
        })
        .optional()
        .map_err(err)?
        .unwrap_or(0) as u64;
    let previous = previous.max(read(&active(&engine.root)?).map(|i| i.version).unwrap_or(0));
    // Keep old snapshots to protect running scans. No hidden deletion/retention policy.
    let count = fs::read_dir(&engine.root)
        .map_err(err)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("generation-"))
        .count();
    if count >= 10 {
        return Err("Existem 10 gerações de bases; manutenção de armazenamento necessária. Base atual preservada.".into());
    }
    let name = format!(
        "generation-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(err)?
            .as_nanos(),
        std::process::id()
    );
    let staging = engine.root.join(&name);
    fs::create_dir(&staging).map_err(err)?;
    let current = active(&engine.root)?;
    for base in [
        "main.cvd",
        "main.cld",
        "daily.cvd",
        "daily.cld",
        "bytecode.cvd",
        "bytecode.cld",
    ] {
        let source = current.join(base);
        if source.is_file() {
            fs::copy(source, staging.join(base)).map_err(err)?;
        }
    }
    let config = staging.join("freshclam.conf");
    fs::write(&config,format!("DatabaseDirectory \"{}\"\nDatabaseMirror database.clamav.net\nTestDatabases yes\nConnectTimeout 15\nReceiveTimeout 60\n",staging.display())).map_err(err)?;
    let log = staging.join("update.log");
    run(
        hidden_command(&engine.bin.join("freshclam.exe"))
            .arg(format!("--config-file={}", config.display()))
            .arg("--quiet")
            .arg(format!("--log={}", log.display())),
        Duration::from_secs(600),
    )
    .map_err(|e| format!("{e}; última base mantida. Log: {}", log.display()))?;
    let info = read(&staging)?;
    if now().saturating_sub(info.built) > 7 * 86400 {
        return Err("Atualização continua com mais de 7 dias; não publicada".into());
    }
    let probe = staging.join("validation.txt");
    fs::write(&probe, b"C2L harmless database validation").map_err(err)?;
    run(
        hidden_command(&engine.bin.join("clamscan.exe"))
            .arg("--official-db-only=yes")
            .arg(format!("--database={}", staging.display()))
            .arg("--")
            .arg(&probe),
        Duration::from_secs(180),
    )?;
    // Ensure database bytes are flushed before the transactional pointer switch.
    for entry in fs::read_dir(&staging).map_err(err)? {
        let path = entry.map_err(err)?.path();
        if path.is_file() {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .and_then(|f| f.sync_all())
                .map_err(err)?;
        }
    }
    publish(&db, &name, info.version, previous)?;
    Ok(format!(
        "Atualização validada e publicada. {}",
        info.label(now())
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Downloads official signature updates and validates a new generation"]
    fn real_update_publishes_valid_snapshot() {
        let engine = Engine::discover().unwrap();
        let old = engine.database.clone();
        let message = update(&engine).unwrap();
        let new = Engine::discover().unwrap();
        assert_ne!(new.database, old);
        assert!(old.exists());
        assert!(read(&new.database).unwrap().version >= read(&old).unwrap().version);
        println!("{message}; previous snapshot preserved");
    }
    #[test]
    fn rejects_malformed_future_and_warns_stale() {
        assert!(Info::parse(b"bad", 100).is_err());
        assert!(Info::parse(b"ClamAV-VDB:date:10:1:90:x:x:publisher:999999", 100).is_err());
        let i = Info::parse(b"ClamAV-VDB:date:10:1:90:x:x:publisher:1", 100).unwrap();
        assert!(i.label(9 * 86400).contains("DESATUALIZADA"));
    }
    #[test]
    fn rollback_rejected_and_old_pointer_survives() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE active(id INTEGER PRIMARY KEY,generation TEXT,version INTEGER)",
        )
        .unwrap();
        publish(&db, "generation-1", 20, 0).unwrap();
        assert!(publish(&db, "generation-2", 19, 20).is_err());
        assert_eq!(
            db.query_row("SELECT generation FROM active", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "generation-1"
        );
        publish(&db, "generation-3", 21, 20).unwrap();
    }
}
