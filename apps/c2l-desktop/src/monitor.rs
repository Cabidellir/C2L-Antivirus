//! Opt-in post-change observation while the GUI is alive. No pre-execution blocking.
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};
pub struct Monitor {
    _watcher: RecommendedWatcher,
    events: Receiver<notify::Result<notify::Event>>,
    overflow: Arc<AtomicBool>,
    pub root: PathBuf,
    pub notice: String,
    dirty: bool,
    dirty_since: Instant,
    pub gaps: usize,
    last_event: Instant,
    last_scan: Option<Instant>,
}
impl Monitor {
    pub fn start(root: &Path) -> Result<Self, String> {
        let root = std::path::absolute(root).map_err(|e| e.to_string())?;
        let _guards =
            c2l_platform_windows::lock_parents(&root.join("probe")).map_err(|e| e.to_string())?;
        if !root.is_dir() {
            return Err("Selecione uma pasta existente".into());
        }
        let app = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?)
            .join("C2L-Antivirus");
        let a = app.to_string_lossy().to_lowercase();
        let b = root
            .to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .to_lowercase();
        if a == b || a.starts_with(&(b.clone() + "\\")) || b.starts_with(&(a + "\\")) {
            return Err("Escolha uma pasta dedicada fora dos dados C2L, por exemplo Downloads. Não monitorize o disco inteiro.".into());
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
            _watcher: watcher,
            events,
            overflow,
            root,
            notice: "Observação iniciada; análise inicial pendente".into(),
            dirty: true,
            dirty_since: Instant::now(),
            gaps: 0,
            last_event: Instant::now(),
            last_scan: None,
        })
    }
    pub fn poll(&mut self) -> bool {
        if self.overflow.swap(false, Ordering::Relaxed) {
            self.gaps += 1;
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
    pub fn started_scan(&mut self) {
        self.dirty = false;
        self.last_scan = Some(Instant::now());
        self.notice="Análise pós-alteração iniciada; consulte Resultados. Novas alterações ficam pendentes.".into();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
