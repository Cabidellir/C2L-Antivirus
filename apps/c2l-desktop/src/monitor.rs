//! Opt-in post-change observation while the GUI is alive. No pre-execution blocking.
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};
#[derive(Default)]
struct Pending {
    full: bool,
    paths: BTreeSet<PathBuf>,
}
impl Pending {
    fn full(&mut self) {
        self.full = true;
        self.paths.clear();
    }
    fn event(&mut self, root: &Path, e: &notify::Event) {
        if self.full {
            return;
        }
        if e.need_rescan()
            || e.paths.is_empty()
            || !matches!(
                e.kind,
                notify::EventKind::Create(_) | notify::EventKind::Modify(_)
            )
        {
            self.full();
            return;
        }
        for path in &e.paths {
            if !path.starts_with(root)
                || path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
                || !path.is_file()
            {
                self.full();
                return;
            }
            self.paths.insert(path.clone());
        }
        // Keep the Windows command line and memory bounded.
        if self.paths.len() > 128
            || self
                .paths
                .iter()
                .map(|p| p.as_os_str().len() + 3)
                .sum::<usize>()
                > 12000
        {
            self.full();
        }
    }
}
pub struct Monitor {
    pending: Pending,
    _watcher: RecommendedWatcher,
    events: Receiver<notify::Result<notify::Event>>,
    overflow: Arc<AtomicBool>,
    pub root: PathBuf,
    pub notice: String,
    dirty: bool,
    dirty_since: Instant,
    pub gaps: usize,
    pub failed: bool,
    last_event: Instant,
    last_scan: Option<Instant>,
}
impl Monitor {
    pub fn start(root: &Path) -> Result<Self, String> {
        let app = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?)
            .join("C2L-Antivirus");
        Self::start_excluding(root, &[app])
    }
    pub fn start_excluding(root: &Path, excluded: &[PathBuf]) -> Result<Self, String> {
        let root = std::path::absolute(root).map_err(|e| e.to_string())?;
        let _guards =
            c2l_platform_windows::lock_parents(&root.join("probe")).map_err(|e| e.to_string())?;
        if !root.is_dir() {
            return Err("Selecione uma pasta existente".into());
        }
        let b = root
            .to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .to_lowercase();
        for path in excluded {
            let a = path
                .to_string_lossy()
                .trim_end_matches(['\\', '/'])
                .to_lowercase();
            if a == b || a.starts_with(&(b.clone() + "\\")) || b.starts_with(&(a + "\\")) {
                return Err("A pasta observada não pode conter dados ou executáveis C2L, nem estar dentro deles.".into());
            }
        }
        let (sender, events) = mpsc::sync_channel(1024);
        let overflow = Arc::new(AtomicBool::new(false));
        let lost = overflow.clone();
        let mut watcher = notify::recommended_watcher(move |event| {
            if sender.try_send(event).is_err() {
                lost.store(true, Ordering::Relaxed);
            }
        })
        .map_err(|e| e.to_string())?;
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            pending: Pending {
                full: true,
                paths: BTreeSet::new(),
            },
            _watcher: watcher,
            events,
            overflow,
            root,
            notice: "Observação iniciada; análise inicial pendente".into(),
            dirty: true,
            dirty_since: Instant::now(),
            gaps: 0,
            failed: false,
            last_event: Instant::now(),
            last_scan: None,
        })
    }
    pub fn poll(&mut self) -> bool {
        if self.overflow.swap(false, Ordering::Relaxed) {
            self.gaps += 1;
            self.pending.full();
            if !self.dirty {
                self.dirty_since = Instant::now();
            }
            self.dirty = true;
            self.notice =
                "Fila excedida: cobertura interrompida; reanálise integral da pasta pendente"
                    .into();
        }
        for _ in 0..1024 {
            let Ok(event) = self.events.try_recv() else {
                break;
            };
            match event {
                Ok(e) if !matches!(e.kind, notify::EventKind::Access(_)) => {
                    self.pending.event(&self.root, &e);
                    if !self.dirty {
                        self.dirty_since = Instant::now();
                    }
                    self.dirty = true;
                    self.last_event = Instant::now();
                    if e.need_rescan() {
                        self.gaps += 1;
                    }
                    self.notice = if e.need_rescan() {
                        "Eventos perdidos: reanálise integral pendente"
                    } else {
                        "Alterações recebidas; análise pendente"
                    }
                    .into();
                }
                Err(e) => {
                    self.failed = true;
                    self.gaps += 1;
                    if !self.dirty {
                        self.dirty_since = Instant::now();
                    }
                    self.dirty = true;
                    self.notice =
                        format!("Falha de observação: {e}; cobertura parcial, reanálise pendente");
                }
                _ => {}
            }
        }
        self.dirty
            && (self.last_event.elapsed() >= Duration::from_secs(2)
                || self.last_scan.is_none()
                || self.dirty_since.elapsed() >= Duration::from_secs(10))
            && self
                .last_scan
                .is_none_or(|t| t.elapsed() >= Duration::from_secs(30))
    }
    pub fn has_pending(&self) -> bool {
        self.dirty
    }
    pub fn request_rescan(&mut self) {
        self.pending.full();
        if !self.dirty {
            self.dirty_since = Instant::now();
        }
        self.dirty = true;
    }
    /// None denotes full reconciliation; Some contains the bounded changed paths.
    pub fn take_targets(&mut self) -> Option<Vec<PathBuf>> {
        let pending = std::mem::take(&mut self.pending);
        self.started_scan();
        if pending.full {
            None
        } else {
            Some(pending.paths.into_iter().collect())
        }
    }
    pub fn started_scan(&mut self) {
        self.pending = Pending::default();
        self.dirty = false;
        self.last_scan = Some(Instant::now());
        self.notice="Análise pós-alteração iniciada; consulte Resultados. Novas alterações ficam pendentes.".into();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_paths_are_deduplicated_and_ambiguous_events_reconcile() {
        let root = std::env::temp_dir().join(format!("c2l-pending-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("changed.txt");
        std::fs::write(&file, b"benign").unwrap();
        let event = notify::Event::new(notify::EventKind::Create(notify::event::CreateKind::File))
            .add_path(file.clone());
        let mut pending = Pending::default();
        pending.event(&root, &event);
        pending.event(&root, &event);
        assert!(!pending.full);
        assert_eq!(pending.paths.len(), 1);
        let taken = std::mem::take(&mut pending);
        pending.event(&root, &event);
        assert_eq!(
            pending.paths, taken.paths,
            "events during a scan remain pending"
        );
        pending.event(&root, &notify::Event::new(notify::EventKind::Any));
        assert!(pending.full);
        assert!(pending.paths.is_empty());
        let mut pending = Pending::default();
        pending.event(
            &root,
            &notify::Event::new(notify::EventKind::Create(notify::event::CreateKind::Folder))
                .add_path(root.clone()),
        );
        assert!(pending.full);
        let mut pending = Pending::default();
        pending.event(
            &root,
            &notify::Event::new(notify::EventKind::Modify(notify::event::ModifyKind::Any))
                .add_path(root.join("../outside")),
        );
        assert!(pending.full);
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn native_event_and_overflow_require_rescan() {
        let root = std::env::temp_dir().join(format!("c2l-watch-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let mut watch = Monitor::start(&root).unwrap();
        assert!(watch.poll());
        watch.started_scan();
        std::fs::write(root.join("change.txt"), b"benign").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && !watch.dirty {
            std::thread::sleep(Duration::from_millis(50));
            watch.poll();
        }
        assert!(watch.dirty, "native change was not observed");
        watch.dirty = false;
        watch.overflow.store(true, Ordering::Relaxed);
        watch.poll();
        assert!(watch.dirty);
        assert!(watch.gaps > 0);
        watch.last_scan = Some(Instant::now() - Duration::from_secs(35));
        watch.last_event = Instant::now();
        watch.dirty_since = Instant::now() - Duration::from_secs(11);
        assert!(
            watch.poll(),
            "continuous writes must not starve scanning indefinitely"
        );
        drop(watch);
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn refuses_own_data_and_drive_ancestor() {
        let local = PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap());
        assert!(Monitor::start(&local).is_err());
        assert!(Monitor::start(local.ancestors().last().unwrap()).is_err());
    }
}
