//! Continuous post-change worker. No file removal, command IPC or execution blocking.
use crate::{ScanBatch, antimalware, monitor::Monitor, service_updates, signatures};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub watch_root: PathBuf,
    #[serde(default = "enabled")]
    pub automatic_updates: bool,
}
fn enabled() -> bool {
    true
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Status {
    pub heartbeat: u64,
    #[serde(default)]
    pub update_next: u64,
    #[serde(default)]
    pub update_success: u64,
    #[serde(default)]
    pub update_notice: String,
    pub phase: String,
    pub notice: String,
    pub root: PathBuf,
    pub scans: u64,
    pub detections: usize,
    pub errors: usize,
    pub gaps: usize,
    pub database: String,
    pub last_report: Option<PathBuf>,
    #[serde(default)]
    pub last_detection_report: Option<PathBuf>,
}
impl Status {
    pub fn fresh(&self, now: u64) -> bool {
        self.heartbeat <= now && now - self.heartbeat <= 15
    }
}
pub fn data_dir() -> Result<PathBuf, String> {
    Ok(
        PathBuf::from(std::env::var_os("PROGRAMDATA").ok_or("PROGRAMDATA ausente")?)
            .join("C2L-Monitor"),
    )
}
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn open_status(dir: &Path) -> Result<Connection, String> {
    let db = Connection::open(dir.join("status.sqlite")).map_err(error)?;
    db.busy_timeout(Duration::from_secs(2)).map_err(error)?;
    db.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS status (id INTEGER PRIMARY KEY CHECK(id=1), json TEXT NOT NULL);").map_err(error)?;
    Ok(db)
}
fn publish(db: &Connection, status: &mut Status) -> Result<(), String> {
    status.heartbeat = signatures::now();
    db.execute(
        "INSERT INTO status VALUES (1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",
        [serde_json::to_string(status).map_err(error)?],
    )
    .map_err(error)?;
    Ok(())
}
pub fn read_status(dir: &Path) -> Result<Status, String> {
    let db =
        Connection::open_with_flags(dir.join("status.sqlite"), OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(error)?;
    db.busy_timeout(Duration::from_millis(50)).map_err(error)?;
    let json: String = db
        .query_row(
            "SELECT json FROM status WHERE id=1 AND length(json)<65536",
            [],
            |row| row.get(0),
        )
        .map_err(error)?;
    serde_json::from_str(&json).map_err(error)
}
fn load_config(install: &Path, data: &Path) -> Result<Config, String> {
    let path = install.join("service.json");
    if fs::metadata(&path).map_err(error)?.len() > 16384 {
        return Err("Configuração demasiado grande".into());
    }
    let config: Config = serde_json::from_str(
        fs::read_to_string(path)
            .map_err(error)?
            .trim_start_matches('\u{feff}'),
    )
    .map_err(error)?;
    let root = &config.watch_root;
    let text = root.to_string_lossy();
    if !root.is_absolute()
        || text.starts_with("\\\\")
        || text.contains('"')
        || root.parent().is_none()
        || root.components().count() < 3
    {
        return Err(
            "É necessária uma pasta local específica; raiz de volume e UNC não são aceites".into(),
        );
    }
    // Also rejects reparse points in parents. The watcher rechecks availability at runtime.
    let watch = Monitor::start_excluding(root, &[install.into(), data.into()])?;
    drop(watch);
    Ok(config)
}
fn within_budget(dir: &Path) -> Result<(), String> {
    let mut bytes = 0u64;
    let mut count = 0;
    for entry in fs::read_dir(dir).map_err(error)? {
        let meta = entry.map_err(error)?.metadata().map_err(error)?;
        bytes = bytes.saturating_add(meta.len());
        count += 1;
        if bytes > 512 * 1024 * 1024 || count > 1500 {
            return Err(
                "Limite de relatórios atingido; análise suspensa até manutenção pelo administrador"
                    .into(),
            );
        }
    }
    Ok(())
}
pub fn service_main(stop: Arc<AtomicBool>) -> Result<(), String> {
    let install = std::env::current_exe()
        .map_err(error)?
        .parent()
        .ok_or("Diretório de instalação ausente")?
        .to_path_buf();
    run(&install, &data_dir()?, stop, None)
}

/// Same worker used by SCM and a bounded foreground integration harness.
pub fn run(
    install: &Path,
    data: &Path,
    stop: Arc<AtomicBool>,
    duration: Option<Duration>,
) -> Result<(), String> {
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(data.join("worker.lock"))
        .map_err(error)?;
    lock.try_lock()
        .map_err(|_| "Já existe um worker ativo neste diretório".to_owned())?;
    let db = open_status(data)?;
    let mut state = read_status(data).unwrap_or_default();
    state.phase = "a iniciar".into();
    publish(&db, &mut state)?;
    let outcome = run_loop(install, data, &stop, duration, &db, &mut state);
    stop.store(true, Ordering::Release);
    state.phase = if outcome.is_ok() { "parado" } else { "falha" }.into();
    if let Err(e) = &outcome {
        state.notice = e.clone();
    }
    publish(&db, &mut state)?;
    outcome
}
fn run_loop(
    install: &Path,
    data: &Path,
    stop: &Arc<AtomicBool>,
    duration: Option<Duration>,
    db: &Connection,
    state: &mut Status,
) -> Result<(), String> {
    let config = load_config(install, data)?;
    state.root = config.watch_root.clone();
    let reports = data.join("reports");
    fs::create_dir_all(&reports).map_err(error)?;
    let _root_guards =
        c2l_platform_windows::lock_parents(&config.watch_root.join("probe")).map_err(error)?;
    let mut watch = Monitor::start_excluding(&config.watch_root, &[install.into(), data.into()])?;
    let start = Instant::now();
    let mut heartbeat = Instant::now() - Duration::from_secs(5);
    let mut periodic = Instant::now();
    let mut task: Option<thread::JoinHandle<Result<(ScanBatch, PathBuf), String>>> = None;
    let mut last_good = false;
    let mut full_scan = true;
    let mut full_good = false;
    let mut initial_attempted = false;
    let mut updater: Option<thread::JoinHandle<Result<String, String>>> = None;
    let result = (|| {
        while !stop.load(Ordering::Acquire) && duration.is_none_or(|d| start.elapsed() < d) {
            let due = watch.poll();
            state.gaps = watch.gaps;
            if watch.failed {
                return Err(watch.notice.clone());
            }
            if task.as_ref().is_some_and(|t| t.is_finished()) {
                let result = task
                    .take()
                    .unwrap()
                    .join()
                    .map_err(|_| "Worker de análise terminou inesperadamente".to_owned())?;
                match result {
                    Ok((report, path)) => {
                        state.scans += 1;
                        state.detections += report.matched;
                        state.errors = report.errors;
                        if report.matched > 0 {
                            state.last_detection_report = Some(path.clone());
                        }
                        state.last_report = Some(path);
                        last_good = !report.interrupted
                            && !report.cancelled
                            && report.errors == 0
                            && !report.truncated;
                        if full_scan || !last_good {
                            full_good = last_good;
                        }
                        last_good &= full_good;
                        state.notice = report.failure.unwrap_or_else(|| {
                            format!(
                                "Última análise: {} ficheiros; {} deteções; {} erros",
                                report.engine_scanned.unwrap_or(report.scanned),
                                report.matched,
                                report.errors
                            )
                        });
                        if !full_scan {
                            state.notice = format!(
                                "Ficheiros alterados: {}{}",
                                state.notice,
                                if full_good {
                                    ""
                                } else {
                                    " · cobertura anterior incompleta; aguarda reconciliação integral"
                                }
                            );
                        }
                    }
                    Err(e) => {
                        full_good = false;
                        last_good = false;
                        state.notice = e;
                    }
                }
                // Persistent read/size failures wait for periodic reconciliation.
                // Immediate retries previously kept scanning the entire folder in a loop.
            }
            if updater.as_ref().is_some_and(|t| t.is_finished()) {
                match updater
                    .take()
                    .unwrap()
                    .join()
                    .map_err(|_| "Atualizador terminou inesperadamente")?
                {
                    Ok(message) => {
                        state.update_success = signatures::now();
                        state.update_notice = message;
                        watch.request_rescan();
                    }
                    Err(message) => {
                        state.update_notice =
                            format!("Falha na atualização; base anterior mantida: {message}");
                    }
                }
            }
            if config.automatic_updates
                && updater.is_none()
                && task.is_none()
                && initial_attempted
                && signatures::now() >= state.update_next
            {
                // Persist before spawning: restarts cannot bypass the six-hour schedule.
                state.update_next = signatures::now().saturating_add(6 * 3600);
                state.update_notice =
                    "Atualização em curso; base anterior preservada até validação".into();
                publish(db, state)?;
                let i = install.to_path_buf();
                let d = data.to_path_buf();
                let cancel = stop.clone();
                updater = Some(thread::spawn(move || {
                    service_updates::update(&i, &d, &cancel)
                }));
            }
            if updater.is_some() {
                state.phase = "a atualizar bases".into();
            }
            if task.is_none() && updater.is_none() && due {
                initial_attempted = true;
                let changed = watch.take_targets();
                full_scan = changed.is_none();
                match within_budget(&reports)
                    .and_then(|_| service_updates::engine(install, data))
                    .and_then(|e| signatures::read(&e.database).map(|info| (e, info)))
                {
                    Ok((e, info)) => {
                        state.database = info.label(signatures::now());
                        let root = config.watch_root.clone();
                        let output = reports.clone();
                        let cancel = stop.clone();
                        task = Some(thread::spawn(move || {
                            if let Some(paths) = changed {
                                antimalware::scan_changed(&e, &root, &paths, &output, &cancel)
                            } else {
                                antimalware::scan(&e, &root, &output, &cancel, &AtomicUsize::new(0))
                            }
                        }));
                        state.phase = "a analisar".into();
                    }
                    Err(e) => {
                        state.phase = "degradado".into();
                        full_good = false;
                        state.notice = e;
                        last_good = false;
                        // Retry at the next periodic reconciliation or filesystem event.
                    }
                }
            }
            if task.is_none() && updater.is_none() {
                let stale = service_updates::engine(install, data)
                    .and_then(|e| signatures::read(&e.database))
                    .map_or(true, |i| {
                        signatures::now().saturating_sub(i.built) > 7 * 86400
                    });
                state.phase = if !last_good || stale {
                    "degradado"
                } else if watch.has_pending() {
                    "análise pendente"
                } else {
                    "a observar"
                }
                .into();
                if stale {
                    state.database="Bases ausentes, inválidas ou com mais de sete dias; consulte o estado das atualizações".into();
                }
            }
            if periodic.elapsed() > Duration::from_secs(900) {
                watch.request_rescan();
                periodic = Instant::now();
            }
            if !config.watch_root.is_dir() {
                state.phase = "degradado".into();
                state.notice =
                    "Pasta indisponível: reinicie o serviço após recuperar o acesso".into();
                publish(db, state)?;
                return Err(state.notice.clone());
            }
            if heartbeat.elapsed() >= Duration::from_secs(3) {
                publish(db, state)?;
                heartbeat = Instant::now();
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    })();
    // Stop and join on every exit path, including status/disk failures.
    stop.store(true, Ordering::Release);
    if let Some(task) = task {
        let _ = task.join();
    }
    if let Some(updater) = updater {
        let _ = updater.join();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heartbeat_never_claims_liveness_for_stale_or_future_data() {
        let s = Status {
            heartbeat: 100,
            ..Default::default()
        };
        assert!(s.fresh(110));
        assert!(!s.fresh(116));
        assert!(!s.fresh(99));
    }
    #[test]
    fn failure_persists_and_does_not_report_observation() {
        let root = std::env::temp_dir().join(format!("c2l-service-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let result = run(
            &root,
            &root,
            Arc::new(AtomicBool::new(false)),
            Some(Duration::from_secs(1)),
        );
        assert!(result.is_err());
        assert_eq!(read_status(&root).unwrap().phase, "falha");
        fs::remove_dir_all(root).unwrap();
    }
}
#[derive(Default)]
pub struct Panel {
    checked: Option<Instant>,
    status: Option<Result<Status, String>>,
    running: bool,
    scm_error: Option<String>,
}
impl Panel {
    pub fn show(&mut self, ui: &mut eframe::egui::Ui) -> Option<PathBuf> {
        if self
            .checked
            .is_none_or(|t| t.elapsed() >= Duration::from_secs(2))
        {
            match c2l_platform_windows::service_host::is_running() {
                Ok(running) => {
                    self.running = running;
                    self.scm_error = None;
                }
                Err(e) => {
                    self.running = false;
                    self.scm_error = Some(e);
                }
            }
            self.status = Some(data_dir().and_then(|d| read_status(&d)));
            self.checked = Some(Instant::now());
        }
        ui.ctx().request_repaint_after(Duration::from_secs(2));
        ui.heading("Serviço Windows · análise contínua");
        ui.label("Funciona com a janela fechada, quando instalado. Analisa após alterações; não bloqueia a execução nem coloca ficheiros automaticamente em quarentena.");
        if !self.running {
            ui.colored_label(crate::theme::warning(ui), "Serviço parado, não instalado ou estado indisponível. Sem cobertura contínua confirmada.");
            if let Some(e) = &self.scm_error {
                ui.collapsing("Detalhes do serviço", |ui| {
                    ui.label(e);
                });
            }
        }
        match &self.status {
            Some(Ok(s)) => {
                let fresh = s.fresh(signatures::now());
                if !fresh {
                    ui.colored_label(
                        crate::theme::danger(ui),
                        "Estado desatualizado: não é possível confirmar o funcionamento do motor.",
                    );
                }
                ui.label(format!(
                    "Último estado comunicado: {}{}",
                    s.phase,
                    if self.running && fresh {
                        " · comunicação recente"
                    } else {
                        " · não confirmado"
                    }
                ));
                ui.label(format!("Pasta configurada: {}", s.root.display()));
                ui.label(&s.notice);
                ui.label(&s.database);
                ui.label(&s.update_notice);
                if s.update_success > 0 {
                    ui.label(format!(
                        "Última atualização validada há {} minutos",
                        signatures::now().saturating_sub(s.update_success) / 60
                    ));
                }
                ui.label(format!("Histórico do serviço: {} análises · {} deteções · {} erros na última análise · {} lacunas de eventos",s.scans,s.detections,s.errors,s.gaps));
                if s.detections > 0 {
                    ui.colored_label(crate::theme::danger(ui),"Há deteções nos relatórios do serviço. Reveja-as antes de decidir a quarentena.");
                }
                if let Some(path) = &s.last_detection_report
                    && ui.button("Abrir relatório com deteções").clicked()
                {
                    return Some(path.clone());
                }
                if let Some(path) = &s.last_report
                    && ui.button("Abrir último relatório do serviço").clicked()
                {
                    return Some(path.clone());
                }
            }
            _ => {
                ui.label("Relatório de estado ainda indisponível para este perfil.");
            }
        }
        ui.collapsing("Instalação e limites",|ui|{
            ui.label("Instalação pelo administrador com scripts/install-monitor-service.ps1: indicar executável, pasta observada e base oficial. UAC é necessário apenas na instalação/gestão do serviço.");
            ui.label("Conta LocalService, leitura apenas na pasta autorizada. Uma pasta por instalação; análise inicial, após alterações (mínimo 30 s) e reconciliação a cada 15 min. Bases oficiais atualizadas automaticamente a cada seis horas, com validação e conservação da geração anterior.");
            ui.label("Deteções e erros ficam guardados mesmo com a GUI fechada. O agente de notificações do utilizador pode avisar com esta janela fechada. Requer sessão iniciada e ativação por scripts/install-notifications.ps1; o Windows pode silenciar os avisos.");
        });
        None
    }
}
#[cfg(test)]
mod integration {
    use super::*;
    use crate::antimalware::Engine;
    #[test]
    #[ignore = "Requires installed official ClamAV and harmless vendor fixture; tests the worker, not SCM/UAC"]
    fn real_continuous_worker_detects_changes_and_recovers_history() {
        let source = Engine::discover().unwrap();
        let fixture = source.root.join("validation/clam-test.bin");
        assert!(fixture.is_file(), "Prepare official harmless fixture first");
        let root = std::env::temp_dir().join(format!(
            "c2l-continuous-{}-{}",
            std::process::id(),
            signatures::now()
        ));
        let install = root.join("install");
        let data = root.join("data");
        let watch = root.join("watch");
        for dir in [
            &install,
            &data,
            &watch,
            &install.join("engine"),
            &install.join("database"),
        ] {
            fs::create_dir_all(dir).unwrap();
        }
        fn copy_tree(from: &Path, to: &Path) {
            fs::create_dir_all(to).unwrap();
            for entry in fs::read_dir(from).unwrap() {
                let entry = entry.unwrap();
                let dest = to.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    copy_tree(&entry.path(), &dest);
                } else {
                    fs::copy(entry.path(), dest).unwrap();
                }
            }
        }
        copy_tree(&source.bin, &install.join("engine"));
        for entry in fs::read_dir(&source.database).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                fs::copy(
                    entry.path(),
                    install.join("database").join(entry.file_name()),
                )
                .unwrap();
            }
        }
        fs::write(
            install.join("service.json"),
            serde_json::to_vec(&Config {
                watch_root: watch.clone(),
                automatic_updates: false,
            })
            .unwrap(),
        )
        .unwrap();
        fs::write(
            watch.join("clean.txt"),
            b"C2L benign continuous worker integration test",
        )
        .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let i = install.clone();
        let d = data.clone();
        let task = thread::spawn(move || run(&i, &d, signal, Some(Duration::from_secs(120))));
        let wait = |predicate: &dyn Fn(&Status) -> bool, seconds: u64| {
            let deadline = Instant::now() + Duration::from_secs(seconds);
            while Instant::now() < deadline {
                if let Ok(s) = read_status(&data)
                    && predicate(&s)
                {
                    return true;
                }
                thread::sleep(Duration::from_millis(200));
            }
            false
        };
        let clean = wait(
            &|s| s.scans >= 1 && s.detections == 0 && s.phase == "a observar",
            45,
        );
        fs::copy(fixture, watch.join("vendor-test.bin")).unwrap();
        let detected = wait(&|s| s.detections > 0 && s.scans >= 2, 55);
        stop.store(true, Ordering::Release);
        assert!(task.join().unwrap().is_ok());
        assert!(clean, "initial clean scan not confirmed");
        assert!(detected, "post-change vendor fixture not detected");
        let previous = read_status(&data).unwrap();
        assert_eq!(previous.phase, "parado");
        let saved = crate::store::load(previous.last_detection_report.as_ref().unwrap()).unwrap();
        assert_eq!(saved.mode, "clamav-changes");
        let report = saved.report.unwrap();
        assert!(report.matched > 0);
        assert_eq!(
            report.engine_scanned,
            Some(1),
            "unchanged clean file must not be rescanned"
        );
        // Restart with the same state; initial reconciliation and historical detection survive.
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let i = install.clone();
        let d = data.clone();
        let task = thread::spawn(move || run(&i, &d, signal, Some(Duration::from_secs(60))));
        let recovered = wait(
            &|s| s.scans > previous.scans && s.detections >= previous.detections,
            40,
        );
        stop.store(true, Ordering::Release);
        assert!(task.join().unwrap().is_ok());
        assert!(recovered);
        // Stop while an engine scan is active, with no GUI involved.
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let i = install.clone();
        let d = data.clone();
        let task = thread::spawn(move || run(&i, &d, signal, Some(Duration::from_secs(60))));
        let active = wait(&|s| s.phase == "a analisar", 10);
        thread::sleep(Duration::from_secs(1));
        let stopping = Instant::now();
        stop.store(true, Ordering::Release);
        assert!(task.join().unwrap().is_ok());
        assert!(active);
        assert!(
            stopping.elapsed() < Duration::from_secs(15),
            "active cancellation exceeded 15 seconds"
        );
        let state = read_status(&data).unwrap();
        println!(
            "Worker acceptance: scans={}, cumulative detections={}, last report={:?}; restart and active cancellation passed",
            state.scans, state.detections, state.last_report
        );
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_overlapping_and_remote_targets() {
        let root = std::env::temp_dir().join(format!("c2l-config-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        for path in [
            root.clone(),
            PathBuf::from("C:\\"),
            PathBuf::from("\\\\server\\share"),
            PathBuf::from("relative"),
        ] {
            fs::write(
                root.join("service.json"),
                serde_json::to_vec(&Config {
                    watch_root: path,
                    automatic_updates: false,
                })
                .unwrap(),
            )
            .unwrap();
            assert!(load_config(&root, &root).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }
}
