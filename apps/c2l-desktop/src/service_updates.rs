//! Service-owned signature generations, with bounded retention between scans.
use crate::{antimalware::Engine, signatures};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub fn engine(install: &Path, data: &Path) -> Result<Engine, String> {
    let root = data.join("definitions");
    let database = if root.join("signatures.sqlite").exists() {
        let active = signatures::active(&root)?;
        if active == root.join("database") {
            install.join("database")
        } else {
            active
        }
    } else {
        install.join("database")
    };
    Ok(Engine {
        bin: install.join("engine"),
        root,
        database,
    })
}
// Called only with no scanner/updater running. Never remove the active generation
// or the immediately preceding generation; retain the installed recovery snapshot.
fn retain(root: &Path, active: &Path) -> Result<(), String> {
    let canonical = root.canonicalize().map_err(|e| e.to_string())?;
    let mut generations: Vec<PathBuf> = fs::read_dir(root)
        .map_err(|e| e.to_string())?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    generations.retain(|p| {
        p.file_name().and_then(|s| s.to_str()).is_some_and(|s| {
            s.starts_with("generation-")
                && s.bytes()
                    .all(|c| c.is_ascii_digit() || c == b'-' || c.is_ascii_lowercase())
        })
    });
    generations.sort();
    let preceding = if root.join("signatures.sqlite").exists() {
        let db = rusqlite::Connection::open_with_flags(
            root.join("signatures.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| e.to_string())?;
        db.query_row("SELECT generation FROM previous WHERE id=1", [], |r| {
            r.get::<_, String>(0)
        })
        .ok()
        .map(|name| root.join(name))
    } else {
        None
    };
    for path in generations {
        if path == active || preceding.as_ref() == Some(&path) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0
            || path.canonicalize().map_err(|e| e.to_string())?.parent() != Some(canonical.as_path())
        {
            return Err("Geração com caminho inesperado; manutenção recusada".into());
        }
        // Reject reparse points anywhere before recursive cleanup of owned data.
        fn plain(path: &Path) -> Result<(), String> {
            for e in fs::read_dir(path).map_err(|e| e.to_string())? {
                let e = e.map_err(|e| e.to_string())?;
                let m = fs::symlink_metadata(e.path()).map_err(|e| e.to_string())?;
                if m.file_attributes() & 0x400 != 0 {
                    return Err("Reparse point em geração".into());
                }
                if m.is_dir() {
                    plain(&e.path())?;
                }
            }
            Ok(())
        }
        plain(&path)?;
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn update(install: &Path, data: &Path, cancel: &AtomicBool) -> Result<String, String> {
    let engine = engine(install, data)?;
    fs::create_dir_all(&engine.root).map_err(|e| e.to_string())?;
    retain(&engine.root, &engine.database)?;
    signatures::update_cancellable(&engine, cancel)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_first_update_uses_installed_recovery_snapshot() {
        let root = std::env::temp_dir().join(format!("c2l-fallback-{}", std::process::id()));
        fs::create_dir_all(root.join("definitions")).unwrap();
        let db = rusqlite::Connection::open(root.join("definitions/signatures.sqlite")).unwrap();
        db.execute_batch(
            "CREATE TABLE active(id INTEGER PRIMARY KEY,generation TEXT,version INTEGER);",
        )
        .unwrap();
        drop(db);
        assert_eq!(
            engine(Path::new("installed"), &root).unwrap().database,
            Path::new("installed/database")
        );
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn retention_preserves_active_previous_and_unrelated_data() {
        let root = std::env::temp_dir().join(format!("c2l-retention-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        for name in ["generation-1", "generation-2", "generation-3", "unrelated"] {
            fs::create_dir(root.join(name)).unwrap();
        }
        let db = rusqlite::Connection::open(root.join("signatures.sqlite")).unwrap();
        db.execute_batch("CREATE TABLE previous(id INTEGER PRIMARY KEY,generation TEXT); INSERT INTO previous VALUES(1,'generation-1');").unwrap();
        drop(db);
        retain(&root, &root.join("generation-2")).unwrap();
        assert!(root.join("generation-1").exists());
        assert!(root.join("generation-2").exists());
        assert!(!root.join("generation-3").exists());
        assert!(root.join("unrelated").exists());
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
}
