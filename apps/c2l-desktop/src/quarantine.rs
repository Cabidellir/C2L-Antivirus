//! User-consented quarantine. Encrypted copy is committed before handle-based removal.
//! SQLite state is deliberately conservative across interrupted operations.
use c2l_hashing::Sha256State;
use c2l_platform_windows as win;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
const MAX_FILE: u64 = 32 * 1024 * 1024;
type Result<T> = std::result::Result<T, String>;
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn digest(data: &[u8]) -> String {
    let mut hash = Sha256State::new();
    hash.update(data);
    hash.finish()
}
fn inside(path: &Path, root: &Path) -> bool {
    let path = path.to_string_lossy().replace('/', "\\").to_lowercase();
    let root = root
        .to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase();
    path == root || path.starts_with(&(root + "\\"))
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Consent {
    pub path: PathBuf,
    pub hash: String,
    pub size: u64,
    pub reason: String,
    pub at: u64,
    pub original_security: Vec<u8>,
    pub original_attributes: u32,
    pub original_modified: u64,
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: i64,
    pub source: String,
    pub state: String,
    pub hash: String,
}

pub fn directory() -> Result<PathBuf> {
    let base = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?)
        .join("C2L-Antivirus");
    fs::create_dir_all(&base).map_err(error)?;
    Ok(base.join("quarantine"))
}
fn connect(root: &Path) -> Result<Connection> {
    win::create_vault_dir(root).map_err(error)?;
    let db = Connection::open(root.join("vault.sqlite")).map_err(error)?;
    db.busy_timeout(std::time::Duration::from_secs(3))
        .map_err(error)?;
    db.execute_batch("PRAGMA synchronous=FULL; PRAGMA journal_mode=DELETE; PRAGMA max_page_count=262144;
        CREATE TABLE IF NOT EXISTS items(id INTEGER PRIMARY KEY, source TEXT NOT NULL, hash TEXT NOT NULL, state TEXT NOT NULL, blob BLOB NOT NULL);
        CREATE TABLE IF NOT EXISTS audit(id INTEGER PRIMARY KEY, item INTEGER, at INTEGER NOT NULL, action TEXT NOT NULL, detail TEXT NOT NULL);").map_err(error)?;
    Ok(db)
}
fn audit(db: &Connection, id: i64, action: &str, detail: &str) -> Result<()> {
    db.execute(
        "INSERT INTO audit(item,at,action,detail) VALUES (?1,?2,?3,?4)",
        params![id, now() as i64, action, detail],
    )
    .map_err(error)?;
    Ok(())
}
fn read(file: &mut std::fs::File) -> Result<Vec<u8>> {
    let size = file.metadata().map_err(error)?.len();
    if size > MAX_FILE {
        return Err("Quarentena limitada a 32 MiB por ficheiro nesta versão".into());
    }
    let mut bytes = Vec::with_capacity(size as usize);
    file.take(MAX_FILE + 1)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    if bytes.len() as u64 != size {
        return Err("Ficheiro mudou durante a leitura".into());
    }
    Ok(bytes)
}
pub fn prepare(path: &Path, reason: String) -> Result<Consent> {
    let path = std::path::absolute(path).map_err(error)?;
    let _parents = win::lock_parents(&path).map_err(error)?;
    let mut file = win::open_exclusive(&path, false).map_err(error)?;
    let bytes = read(&mut file)?;
    use std::os::windows::fs::MetadataExt;
    let metadata = file.metadata().map_err(error)?;
    Ok(Consent {
        path,
        hash: digest(&bytes),
        size: bytes.len() as u64,
        reason,
        at: now(),
        original_security: win::file_security(&file).map_err(error)?,
        original_attributes: metadata.file_attributes(),
        original_modified: metadata.last_write_time(),
    })
}
pub fn isolate(root: &Path, consent: &Consent) -> Result<i64> {
    isolate_at(root, consent, 0)
}
fn isolate_at(root: &Path, consent: &Consent, fault: u8) -> Result<i64> {
    if now().saturating_sub(consent.at) > 300 {
        return Err("Confirmação expirada; volte a selecionar o ficheiro".into());
    }
    let _parents = win::lock_parents(&consent.path).map_err(error)?;
    win::create_vault_dir(root).map_err(error)?;
    let _vault_parents = win::lock_parents(&root.join("vault.sqlite")).map_err(error)?;
    if inside(&consent.path, root) {
        return Err("O cofre não pode ser colocado em quarentena".into());
    }
    let mut file = win::open_exclusive(&consent.path, true).map_err(error)?;
    let bytes = read(&mut file)?;
    if digest(&bytes) != consent.hash || bytes.len() as u64 != consent.size {
        return Err("Conteúdo mudou desde a confirmação; original preservado".into());
    }
    let metadata = serde_json::to_vec(consent).map_err(error)?;
    if metadata.len() > 65532 {
        return Err("Metadados excedem o limite; original preservado".into());
    }
    let mut payload = (metadata.len() as u32).to_le_bytes().to_vec();
    payload.extend_from_slice(&metadata);
    payload.extend_from_slice(&bytes);
    let encrypted = win::protect(&payload, false).map_err(error)?;
    if win::protect(&encrypted, true).map_err(error)? != payload {
        return Err("Falha na verificação da cópia cifrada".into());
    }
    let mut db = connect(root)?;
    if fault == 5 {
        let pages: i64 = db
            .query_row("PRAGMA page_count", [], |r| r.get(0))
            .map_err(error)?;
        db.pragma_update(None, "max_page_count", pages)
            .map_err(error)?;
    }
    let tx = db.transaction().map_err(error)?;
    tx.execute(
        "INSERT INTO items(source,hash,state,blob) VALUES (?1,?2,'copy_saved',?3)",
        params![consent.path.to_string_lossy(), consent.hash, encrypted],
    )
    .map_err(error)?;
    let id = tx.last_insert_rowid();
    audit(
        &tx,
        id,
        "consent_and_copy",
        &serde_json::to_string(consent).map_err(error)?,
    )?;
    tx.commit().map_err(error)?;
    let stored: Vec<u8> = db
        .query_row("SELECT blob FROM items WHERE id=?1", [id], |r| r.get(0))
        .map_err(error)?;
    if win::protect(&stored, true).map_err(error)? != payload {
        return Err("Cópia persistida inválida; original preservado".into());
    }
    if fault == 3 {
        std::process::exit(87);
    }
    // Fault injection proves that a committed copy is sufficient for recovery.
    if fault == 1 {
        return Err("Falha simulada depois da cópia; original preservado".into());
    }
    if let Err(e) = win::delete_on_close(&file) {
        audit(&db, id, "removal_failed", &e.to_string())?;
        return Err(format!("Cópia guardada #{id}; original não removido: {e}"));
    }
    drop(file);
    if fault == 4 {
        std::process::exit(87);
    }
    if fault == 2 {
        return Err("Falha simulada depois da remoção; cópia recuperável".into());
    }
    let tx = db.transaction().map_err(error)?;
    tx.execute("UPDATE items SET state='isolated' WHERE id=?1", [id])
        .map_err(error)?;
    audit(
        &tx,
        id,
        "isolated",
        "Original removido pelo handle verificado",
    )?;
    tx.commit().map_err(error)?;
    Ok(id)
}
#[cfg(test)]
pub fn list(root: &Path) -> Result<Vec<Entry>> {
    list_page(root, 0)
}
pub fn list_page(root: &Path, page: usize) -> Result<Vec<Entry>> {
    let offset: i64 = page
        .checked_mul(100)
        .and_then(|v| i64::try_from(v).ok())
        .ok_or("Página inválida")?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    let db = connect(root)?;
    let mut query = db
        .prepare("SELECT id,source,state,hash FROM items ORDER BY id DESC LIMIT 100 OFFSET ?1")
        .map_err(error)?;
    query
        .query_map([offset], |r| {
            Ok(Entry {
                id: r.get(0)?,
                source: r.get(1)?,
                state: r.get(2)?,
                hash: r.get(3)?,
            })
        })
        .map_err(error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(error)
}
/// Re-scan decrypted bytes with the current official database before destination confirmation.
pub fn review_restore(root: &Path, id: i64) -> Result<String> {
    let engine = crate::antimalware::Engine::discover()?;
    let info = crate::signatures::read(&engine.database)?;
    if now().saturating_sub(info.built) > 7 * 86400 {
        return Err("Atualize as bases antes de reavaliar a recuperação".into());
    }
    let db = connect(root)?;
    let blob: Vec<u8> = db
        .query_row(
            "SELECT blob FROM items WHERE id=?1 AND length(blob)<=?2",
            params![id, (MAX_FILE + 65536) as i64],
            |r| r.get(0),
        )
        .map_err(error)?;
    let payload = win::protect(&blob, true).map_err(error)?;
    if payload.len() < 4 {
        return Err("Cópia inválida".into());
    }
    let len = u32::from_le_bytes(payload[..4].try_into().unwrap()) as usize;
    if len > 65532 || len + 4 > payload.len() {
        return Err("Metadados inválidos".into());
    }
    let report = crate::antimalware::scan_memory(
        &engine,
        &payload[4 + len..],
        &crate::history::directory()?,
    )?;
    if report.interrupted || report.cancelled || report.errors > 0 || report.scanned == 0 {
        audit(
            &db,
            id,
            "restore_review_failed",
            &format!("{:?}", report.failure),
        )?;
        return Err("Reavaliação inconclusiva; cópia mantida no cofre. Consulte Resultados/relatórios e resolva a falha antes de recuperar.".into());
    }
    let message = format!(
        "Reavaliação pela base {}: {} deteções; isto não garante segurança.",
        info.version, report.matched
    );
    audit(
        &db,
        id,
        "restore_review",
        &format!("{message}; relatório {:?}", report.store_path),
    )?;
    Ok(message)
}
/// Export only after explicit destination consent. Never overwrite; keep encrypted copy.
/// It is recovery, not an assertion that the restored content is safe.
pub fn restore(root: &Path, id: i64, destination: &Path) -> Result<()> {
    let destination = std::path::absolute(destination).map_err(error)?;
    if inside(&destination, root) {
        return Err("Escolha um destino fora do cofre".into());
    }
    let _parents = win::lock_parents(&destination).map_err(error)?;
    let db = connect(root)?;
    let (blob, expected): (Vec<u8>, String) = db
        .query_row(
            "SELECT blob,hash FROM items WHERE id=?1 AND length(blob)<=?2",
            params![id, (MAX_FILE + 65536) as i64],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(error)?;
    let payload = win::protect(&blob, true).map_err(error)?;
    if payload.len() < 4 {
        return Err("Cópia inválida".into());
    }
    let len = u32::from_le_bytes(payload[..4].try_into().unwrap()) as usize;
    if len > 65532 || len + 4 > payload.len() {
        return Err("Metadados inválidos".into());
    }
    let consent: Consent = serde_json::from_slice(&payload[4..4 + len]).map_err(error)?;
    let bytes = &payload[4 + len..];
    if digest(bytes) != expected || consent.hash != expected || consent.size != bytes.len() as u64 {
        return Err("Integridade da cópia não confirmada".into());
    }
    audit(&db, id, "restore_requested", &destination.to_string_lossy())?;
    use std::os::windows::fs::OpenOptionsExt;
    let mut output = OpenOptions::new()
        .share_mode(0)
        .read(true)
        .write(true)
        .create_new(true)
        .open(&destination)
        .map_err(error)?;
    if let Err(e) = output.write_all(bytes).and_then(|_| output.sync_all()) {
        audit(&db, id, "restore_incomplete", &e.to_string())?;
        return Err(format!("Destino parcial; cópia no cofre preservada: {e}"));
    }
    output.seek(SeekFrom::Start(0)).map_err(error)?;
    if c2l_hashing::sha256_reader_cancellable(&mut output, || false).map_err(error)? != expected {
        return Err("Verificação do destino falhou; cofre preservado".into());
    }
    audit(&db, id, "restored_copy", &destination.to_string_lossy())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vault_pagination_preserves_access_to_old_entries() {
        let root = sandbox();
        let vault = root.join("vault");
        let mut db = connect(&vault).unwrap();
        let tx = db.transaction().unwrap();
        for i in 0..101 {
            tx.execute(
                "INSERT INTO items(source,hash,state,blob) VALUES(?1,'test','copy_saved',X'00')",
                [format!("test-{i}")],
            )
            .unwrap();
        }
        tx.commit().unwrap();
        drop(db);
        assert_eq!(list_page(&vault, 0).unwrap().len(), 100);
        assert_eq!(list_page(&vault, 1).unwrap()[0].id, 1);
        assert!(list_page(&vault, usize::MAX).is_err());
        assert!(inside(
            &PathBuf::from("C:\\VAULT\\a.bin"),
            Path::new("c:\\vault")
        ));
        assert!(!inside(
            Path::new("c:\\vault-other\\a.bin"),
            Path::new("c:\\vault")
        ));
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "Requires real ClamAV and current databases for restore review"]
    fn real_restore_review() {
        let root = sandbox();
        let vault = root.join("vault");
        let source = root.join("source.txt");
        fs::write(&source, b"C2L harmless restore review").unwrap();
        let consent = prepare(&source, "Integration test".into()).unwrap();
        let id = isolate(&vault, &consent).unwrap();
        let message = review_restore(&vault, id).unwrap();
        assert!(message.contains("0 deteções"));
        restore(&vault, id, &source).unwrap();
        assert_eq!(fs::read(&source).unwrap(), b"C2L harmless restore review");
        println!("Real restore review passed: {message}");
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn crash_worker() {
        let Some(root) = std::env::var_os("C2L_VAULT_CRASH_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let phase = std::env::var("C2L_VAULT_CRASH_PHASE")
            .unwrap()
            .parse()
            .unwrap();
        let consent = prepare(&root.join("source.txt"), "Crash test consent".into()).unwrap();
        isolate_at(&root.join("vault"), &consent, phase).unwrap();
        panic!("Crash point was not reached");
    }
    #[test]
    fn actual_process_exit_preserves_recovery_before_and_after_removal() {
        for phase in [3, 4] {
            let root = sandbox();
            fs::write(root.join("source.txt"), b"recover after process exit").unwrap();
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "quarantine::tests::crash_worker", "--nocapture"])
                .env("C2L_VAULT_CRASH_ROOT", &root)
                .env("C2L_VAULT_CRASH_PHASE", phase.to_string())
                .status()
                .unwrap();
            assert_eq!(status.code(), Some(87));
            assert_eq!(root.join("source.txt").exists(), phase == 3);
            let entries = list(&root.join("vault")).unwrap();
            assert_eq!(entries.len(), 1);
            restore(
                &root.join("vault"),
                entries[0].id,
                &root.join("restored.txt"),
            )
            .unwrap();
            assert_eq!(
                fs::read(root.join("restored.txt")).unwrap(),
                b"recover after process exit"
            );
            assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn disk_budget_failure_preserves_original_and_expired_consent_is_refused() {
        let root = sandbox();
        let source = root.join("source.bin");
        fs::write(&source, vec![42; 65536]).unwrap();
        let mut consent = prepare(&source, "test".into()).unwrap();
        let result = isolate_at(&root.join("vault"), &consent, 5);
        assert!(result.is_err(), "SQLite quota should fail");
        assert!(source.exists());
        assert!(list(&root.join("vault")).unwrap().is_empty());
        consent.at = 1;
        assert!(isolate(&root.join("vault"), &consent).is_err());
        assert!(source.exists());
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
    fn sandbox() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "c2l-vault-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn quarantine_recovery_integrity_collisions_and_faults() {
        let root = sandbox();
        let vault = root.join("vault");
        win::create_vault_dir(&vault).unwrap();
        let source = root.join("document.txt");
        fs::write(&source, b"harmless document").unwrap();
        let consent = prepare(&source, "Test consent".into()).unwrap();
        fs::write(&source, b"changed").unwrap();
        assert!(isolate(&vault, &consent).is_err());
        assert!(source.exists());
        fs::write(&source, b"harmless document").unwrap();
        assert!(isolate_at(&vault, &consent, 1).is_err());
        assert!(source.exists());
        assert!(isolate_at(&vault, &consent, 2).is_err());
        assert!(!source.exists());
        let entries = list(&vault).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].state, "copy_saved");
        let restored = root.join("recovered.txt");
        restore(&vault, entries[0].id, &restored).unwrap();
        assert_eq!(fs::read(&restored).unwrap(), b"harmless document");
        assert!(restore(&vault, entries[0].id, &restored).is_err());
        let consent = prepare(&restored, "Test consent".into()).unwrap();
        let id = isolate(&vault, &consent).unwrap();
        assert!(!restored.exists());
        let db = connect(&vault).unwrap();
        db.execute("UPDATE items SET blob=X'00010203' WHERE id=?1", [id])
            .unwrap();
        assert!(restore(&vault, id, &restored).is_err());
        assert!(!restored.exists());
        drop(db);
        // Cleanup only this uniquely created temp child.
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn refuses_hardlinks_and_named_streams() {
        let root = sandbox();
        let file = root.join("a.txt");
        fs::write(&file, b"safe").unwrap();
        fs::hard_link(&file, root.join("b.txt")).unwrap();
        assert!(prepare(&file, "test".into()).is_err());
        let other = root.join("c.txt");
        fs::write(&other, b"safe").unwrap();
        fs::write(root.join("c.txt:extra"), b"stream").unwrap();
        assert!(prepare(&other, "test".into()).is_err());
        assert!(prepare(&root.join("c.txt:extra"), "test".into()).is_err());
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
}
