//! C2L Antivirus desktop prototype (Windows).
//! The program only reads selected paths. It does not offer real-time protection.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use c2l_detection::{Verdict, classify};
use c2l_indicators::IndicatorDb;
use c2l_scanner::{FileOutcome, ScanResult, scan_directory_filtered, scan_file_cancellable};
use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread;
use std::time::Duration;

const MAX_SCAN_FILES: usize = 50_000;
const DEFAULT_SCAN_FILES: usize = 10_000;
const RESULT_PAGE_SIZE: usize = 100;
mod diagnostics;
mod history;
mod store;
mod theme;
use serde::{Deserialize, Serialize};
use theme::{ACCENT, GOLD, PURPLE, apply_theme, choice, danger, muted, warning};
const THEME_KEY: &str = "c2l.theme.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Home,
    Scan,
    Results,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScanMode {
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
    path: PathBuf,
    size: Option<u64>,
    hash: Option<String>,
    verdict: String,
    matched: bool,
    error: bool,
    #[serde(default)]
    diagnostic: Option<diagnostics::Diagnostic>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct ScanBatch {
    records: Vec<Record>,
    scanned: usize,
    matched: usize,
    errors: usize,
    truncated: bool,
    #[serde(default)]
    cancelled: bool,
    #[serde(default)]
    entry_limit: Option<usize>,
    #[serde(default)]
    interrupted: bool,
    #[serde(default)]
    skipped: usize,
    #[serde(default)]
    failure: Option<String>,
    #[serde(skip)]
    store_path: Option<PathBuf>,
    #[serde(skip)]
    error_causes: std::collections::BTreeMap<String, usize>,
}

fn make_record(result: ScanResult, indicators: Option<&IndicatorDb>) -> Record {
    let (verdict, matched) = match classify(&result.sha256, indicators) {
        Verdict::NotChecked => ("Não verificado (sem base de indicadores)".to_owned(), false),
        Verdict::NotListed => (
            "Sem correspondência; segurança não garantida".to_owned(),
            false,
        ),
        Verdict::Listed(name) => (format!("Correspondência: {name}"), true),
    };

    Record {
        path: result.path,
        size: Some(result.size_bytes),
        hash: Some(result.sha256),
        verdict,
        matched,
        error: false,
        diagnostic: None,
    }
}

fn error_record(path: PathBuf, message: String) -> Record {
    Record {
        path,
        size: None,
        hash: None,
        verdict: format!("Erro de leitura: {message}"),
        matched: false,
        error: true,
        diagnostic: None,
    }
}

fn io_error_record(path: PathBuf, error: std::io::Error) -> Record {
    let diagnostic = diagnostics::Diagnostic::from_error(&error);
    let mut record = error_record(path, error.to_string());
    record.diagnostic = Some(diagnostic);
    record
}

#[cfg(test)]
fn summarize(records: Vec<Record>, truncated: bool) -> ScanBatch {
    let scanned = records.iter().filter(|record| !record.error).count();
    let matched = records.iter().filter(|record| record.matched).count();
    let errors = records.iter().filter(|record| record.error).count();

    ScanBatch {
        records,
        scanned,
        matched,
        errors,
        truncated,
        ..Default::default()
    }
}

fn execute_persisted(
    mode: ScanMode,
    path: PathBuf,
    indicator_file: Option<PathBuf>,
    max_files: usize,
    cancel: &AtomicBool,
    processed: &AtomicUsize,
    dir: &std::path::Path,
) -> Result<(ScanBatch, PathBuf), String> {
    let db = indicator_file
        .as_ref()
        .map(|location| {
            IndicatorDb::load(location)
                .map_err(|e| format!("Não foi possível carregar a base: {e}"))
        })
        .transpose()?;
    let mut header = history::SavedScan {
        schema_version: 3,
        app_version: env!("CARGO_PKG_VERSION").into(),
        completed_unix_seconds: 0,
        target: path.clone(),
        mode: match mode {
            ScanMode::File => "file",
            ScanMode::Folder => "folder",
        }
        .into(),
        indicator_file,
        report: Some(ScanBatch {
            interrupted: true,
            entry_limit: (max_files != usize::MAX).then_some(max_files),
            ..Default::default()
        }),
        failure: None,
    };
    let mut writer = store::Writer::create(dir, &header)?;
    let active_report = std::fs::canonicalize(&writer.path).map_err(|e| e.to_string())?;
    let mut emit = |outcome| {
        let record = match outcome {
            FileOutcome::Scanned(result) => make_record(result, db.as_ref()),
            FileOutcome::Failed { path, error } => io_error_record(path, error),
        };
        writer.push(&record).map_err(std::io::Error::other)?;
        processed.fetch_add(1, Ordering::Relaxed);
        Ok(())
    };
    let result = match mode {
        ScanMode::Folder => scan_directory_filtered(
            &path,
            max_files,
            cancel,
            |candidate| !store::is_active_file(candidate, &active_report),
            &mut emit,
        ),
        ScanMode::File => {
            let outcome = match scan_file_cancellable(&path, cancel) {
                Ok(result) => Some(FileOutcome::Scanned(result)),
                Err(_) if cancel.load(Ordering::Relaxed) => None,
                Err(error) => Some(FileOutcome::Failed { path, error }),
            };
            match outcome {
                Some(outcome) => emit(outcome).map(|()| c2l_scanner::DirectorySummary {
                    processed: 1,
                    ..Default::default()
                }),
                None => Ok(c2l_scanner::DirectorySummary {
                    cancelled: true,
                    ..Default::default()
                }),
            }
        }
    };
    let report = header.report.as_mut().unwrap();
    match result {
        Ok(summary) => {
            report.interrupted = false;
            report.cancelled = summary.cancelled;
            report.truncated = summary.truncated;
            report.skipped = summary.skipped;
        }
        Err(error) => {
            report.failure = Some(error.to_string());
        }
    }
    header.completed_unix_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let saved = writer.finish(&header)?;
    let report = store::load(&saved)?.report.ok_or("Resumo em falta")?;
    Ok((report, saved))
}

struct FinishedScan {
    result: Result<ScanBatch, String>,
    saved: Result<PathBuf, String>,
}

struct DesktopApp {
    page: Page,
    mode: ScanMode,
    selection: Option<PathBuf>,
    indicator_file: Option<PathBuf>,
    receiver: Option<Receiver<FinishedScan>>,
    last_report: Option<ScanBatch>,
    status: String,
    run_count: usize,
    dark: bool,
    errors_only: bool,
    history_notice: String,
    scan_limit: usize,
    cancel: Arc<AtomicBool>,
    processed: Arc<AtomicUsize>,
    result_page: usize,
    cached_key: Option<(PathBuf, bool, usize)>,
    cached_records: Vec<Record>,
    page_error: Option<String>,
}

impl Default for DesktopApp {
    fn default() -> Self {
        Self {
            page: Page::Home,
            mode: ScanMode::File,
            selection: None,
            indicator_file: None,
            receiver: None,
            last_report: None,
            status: "Pronto para uma análise manual".to_owned(),
            run_count: 0,
            dark: true,
            errors_only: false,
            history_notice: String::new(),
            scan_limit: DEFAULT_SCAN_FILES,
            cancel: Arc::new(AtomicBool::new(false)),
            processed: Arc::new(AtomicUsize::new(0)),
            result_page: 0,
            cached_key: None,
            cached_records: Vec::new(),
            page_error: None,
        }
    }
}

impl DesktopApp {
    fn begin_scan(&mut self) {
        if self.receiver.is_some() {
            return;
        }
        let Some(path) = self.selection.clone() else {
            self.status = "Seleciona primeiro um ficheiro ou uma pasta.".to_owned();
            return;
        };

        let mode = self.mode;
        let max_files = self.scan_limit;
        self.cancel = Arc::new(AtomicBool::new(false));
        self.processed = Arc::new(AtomicUsize::new(0));
        let cancel = Arc::clone(&self.cancel);
        let processed = Arc::clone(&self.processed);
        let indicators = self.indicator_file.clone();
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.page = Page::Results;
        self.last_report = None;
        self.history_notice.clear();
        self.status = "Análise em curso. A janela continua disponível.".to_owned();

        thread::spawn(move || {
            let completed = history::directory().and_then(|dir| {
                execute_persisted(mode, path, indicators, max_files, &cancel, &processed, &dir)
            });
            let (result, saved) = match completed {
                Ok((report, path)) => (Ok(report), Ok(path)),
                Err(error) => (Err(format!("Análise interrompida: {error}")), Err(error)),
            };
            let _ = sender.send(FinishedScan { result, saved });
        });
    }

    fn update_scan_state(&mut self) {
        let Some(receiver) = self.receiver.as_ref() else {
            return;
        };
        match receiver.try_recv() {
            Ok(finished) => {
                self.history_notice = match finished.saved {
                    Ok(path) => format!("Relatório guardado localmente: {}", path.display()),
                    Err(error) => format!(
                        "Não foi possível guardar o relatório: {error}. Os lotes já confirmados podem ser consultados no histórico; a análise não foi concluída."
                    ),
                };
                self.accept_result(finished.result);
                self.run_count += 1;
                self.receiver = None;
            }
            Err(TryRecvError::Disconnected) => {
                self.status = "O processo de análise terminou inesperadamente; não foi confirmado qualquer relatório guardado.".to_owned();
                self.receiver = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    fn accept_result(&mut self, result: Result<ScanBatch, String>) {
        self.result_page = 0;
        self.cached_key = None;
        self.cached_records.clear();
        self.page_error = None;
        match result {
            Ok(report) => {
                self.status = format!(
                    "{}: {} ficheiros analisados, {} correspondências, {} erros",
                    if report.interrupted {
                        "Interrompida (cobertura parcial)"
                    } else if report.cancelled {
                        "Cancelada (cobertura parcial)"
                    } else if report.truncated || report.errors > 0 || report.skipped > 0 {
                        "Análise parcial"
                    } else {
                        "Concluído"
                    },
                    report.scanned,
                    report.matched,
                    report.errors
                );
                self.errors_only = report.errors > 0;
                self.last_report = Some(report);
            }
            Err(message) => {
                self.status = message;
                self.last_report = None;
            }
        }
    }

    fn restore_report(&mut self, path: PathBuf, scan: history::SavedScan) {
        self.accept_result(match scan.report {
            Some(report) => Ok(report),
            None => Err(scan
                .failure
                .unwrap_or_else(|| "Relatório sem resultado".into())),
        });
        self.history_notice = format!("Histórico local carregado: {}", path.display());
    }

    fn nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        egui::Frame::default()
            .fill(ACCENT)
            .stroke(egui::Stroke::new(2.0_f32, GOLD))
            .inner_margin(16)
            .corner_radius(10)
            .show(ui, |ui| {
                ui.set_min_width(150.0);
                ui.heading(RichText::new("C2L").size(30.0).color(PURPLE).strong());
                ui.label(
                    RichText::new("ANTIVIRUS")
                        .color(Color32::from_rgb(22, 25, 40))
                        .size(12.0),
                );
            });
        ui.add_space(25.0);
        ui.separator();
        ui.add_space(8.0);

        for (page, name) in [
            (Page::Home, "Início"),
            (Page::Scan, "Analisar"),
            (Page::Results, "Resultados"),
            (Page::Settings, "Definições"),
        ] {
            if ui
                .add_sized(
                    [ui.available_width(), 42.0],
                    egui::Button::new(RichText::new(name).color(Color32::WHITE))
                        .fill(PURPLE)
                        .selected(self.page == page),
                )
                .clicked()
            {
                self.page = page;
            }
            ui.add_space(5.0);
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.label(
                RichText::new(concat!(
                    "Versão ",
                    env!("CARGO_PKG_VERSION"),
                    " · Protótipo"
                ))
                .small()
                .color(muted(ui)),
            );
            ui.label(
                RichText::new("Sem proteção em tempo real")
                    .small()
                    .color(warning(ui)),
            );
        });
    }

    fn title(ui: &mut egui::Ui, heading: &str, subtitle: &str) {
        ui.add_space(12.0);
        ui.label(RichText::new(heading).size(28.0).strong());
        ui.label(RichText::new(subtitle).color(muted(ui)));
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(14.0);
    }

    fn tile(ui: &mut egui::Ui, name: &str, value: &str, color: Color32) {
        egui::Frame::default()
            .fill(ui.visuals().window_fill)
            .stroke(egui::Stroke::new(1.0_f32, ACCENT))
            .inner_margin(16)
            .corner_radius(10)
            .show(ui, |ui| {
                ui.set_min_width(160.0_f32.min(ui.available_width()));
                ui.label(RichText::new(name).color(muted(ui)));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(value)
                        .size(27.0)
                        .color(if color == ACCENT && !ui.visuals().dark_mode {
                            PURPLE
                        } else {
                            color
                        })
                        .strong(),
                );
            });
    }

    fn home(&mut self, ui: &mut egui::Ui) {
        Self::title(
            ui,
            "Visão geral",
            "Um painel de análises manuais; ainda não existe monitorização do sistema.",
        );
        ui.horizontal_wrapped(|ui| {
            let report = self.last_report.as_ref();
            Self::tile(
                ui,
                "Ficheiros na última análise",
                &report.map_or(0, |r| r.scanned).to_string(),
                ACCENT,
            );
            Self::tile(
                ui,
                "Indicadores encontrados",
                &report.map_or(0, |r| r.matched).to_string(),
                warning(ui),
            );
            Self::tile(
                ui,
                "Erros de leitura",
                &report.map_or(0, |r| r.errors).to_string(),
                danger(ui),
            );
        });

        ui.add_space(24.0);
        ui.heading("Estado do produto");
        ui.add_space(8.0);
        ui.label(
            RichText::new("Scanner manual: disponível").color(if ui.visuals().dark_mode {
                ACCENT
            } else {
                PURPLE
            }),
        );
        ui.label(RichText::new("Proteção em tempo real: não implementada").color(warning(ui)));
        ui.label(RichText::new("Quarentena e bloqueio: não implementados").color(warning(ui)));
        ui.label(
            RichText::new("Atualizações automáticas de assinaturas: não implementadas")
                .color(warning(ui)),
        );
        ui.add_space(12.0);

        if ui.button("Iniciar uma análise  →").clicked() {
            self.page = Page::Scan;
        }

        ui.add_space(24.0);
        ui.label(
            RichText::new("A ausência de uma correspondência SHA-256 não significa que o ficheiro esteja seguro.")
                .color(muted(ui)),
        );
    }

    fn scan(&mut self, ui: &mut egui::Ui) {
        Self::title(
            ui,
            "Análise manual",
            "Escolhe um ficheiro ou uma pasta. Nenhum ficheiro será modificado.",
        );

        ui.heading("1. Escolher o alvo");
        ui.horizontal_wrapped(|ui| {
            let previous = self.mode;
            choice(ui, &mut self.mode, ScanMode::File, "Um ficheiro");
            choice(ui, &mut self.mode, ScanMode::Folder, "Uma pasta");
            if previous != self.mode {
                self.selection = None;
            }
        });
        ui.add_space(14.0);

        ui.horizontal_wrapped(|ui| {
            if ui.button("Procurar…").clicked() {
                let selected = match self.mode {
                    ScanMode::File => rfd::FileDialog::new().pick_file(),
                    ScanMode::Folder => rfd::FileDialog::new().pick_folder(),
                };
                if let Some(path) = selected {
                    self.selection = Some(path);
                }
            }

            ui.label(self.selection.as_ref().map_or_else(
                || "Nada selecionado".to_owned(),
                |p| p.display().to_string(),
            ));
        });

        if self.mode == ScanMode::Folder {
            ui.label("Máximo de ficheiros/erros nesta análise:");
            ui.add_enabled_ui(self.receiver.is_none(), |ui| {
                ui.horizontal_wrapped(|ui| {
                    choice(ui, &mut self.scan_limit, 1_000, "1 000");
                    choice(ui, &mut self.scan_limit, 10_000, "10 000");
                    choice(ui, &mut self.scan_limit, MAX_SCAN_FILES, "50 000");
                    choice(
                        ui,
                        &mut self.scan_limit,
                        usize::MAX,
                        "Sem limite de entradas",
                    );
                });
            });
            ui.label(
                "Ligações simbólicas são ignoradas. Atingir o limite deixa a análise parcial.",
            );
        }

        ui.add_space(16.0);
        ui.label(RichText::new("2. Indicadores SHA-256 (CSV local, não autenticado)").strong());
        ui.horizontal_wrapped(|ui| {
            ui.label(
                self.indicator_file
                    .as_ref()
                    .map_or("Nenhuma base carregada", |p| {
                        p.to_str().unwrap_or("Nome não representável")
                    }),
            );

            if ui.button("Escolher CSV…").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Indicadores", &["csv", "txt"])
                    .pick_file()
            {
                self.indicator_file = Some(path);
            }
            if self.indicator_file.is_some() && ui.button("Remover").clicked() {
                self.indicator_file = None;
            }
        });

        ui.add_space(20.0);
        ui.heading("3. Executar em modo de leitura");
        let ready = self.selection.is_some() && self.receiver.is_none();
        if ui
            .add_enabled(ready, egui::Button::new("Iniciar análise"))
            .clicked()
        {
            self.begin_scan();
        }
        if self.receiver.is_some() {
            ui.spinner();
        }
        ui.add_space(14.0);
        ui.label(
            RichText::new("O C2L calcula hashes e, quando existe uma base fornecida, procura correspondências. Não determina se os restantes ficheiros estão seguros.")
                .color(muted(ui)),
        );
    }

    fn results(&mut self, ui: &mut egui::Ui) {
        Self::title(
            ui,
            "Resultados",
            "Último resultado ou relatório do histórico local.",
        );

        ui.label(&self.status);
        if !self.history_notice.is_empty() {
            ui.label(&self.history_notice);
        }
        if self.receiver.is_none() && ui.button("Abrir relatório guardado…").clicked() {
            let mut dialog =
                rfd::FileDialog::new().add_filter("Relatório C2L", &["json", "sqlite"]);
            if let Ok(dir) = history::directory() {
                dialog = dialog.set_directory(dir);
            }
            if let Some(path) = dialog.pick_file() {
                match history::load(&path) {
                    Ok(scan) => self.restore_report(path, scan),
                    Err(error) => {
                        self.history_notice = format!("Não foi possível abrir o relatório: {error}")
                    }
                }
            }
        }
        ui.add_space(12.0);

        if self.receiver.is_some() {
            ui.horizontal_wrapped(|ui| {
                ui.spinner();
                ui.label(format!(
                    "{} entradas processadas (ficheiros e erros)",
                    self.processed.load(Ordering::Relaxed)
                ));
            });
            if ui
                .add_enabled(
                    !self.cancel.load(Ordering::Relaxed),
                    egui::Button::new("Cancelar análise"),
                )
                .clicked()
            {
                self.cancel.store(true, Ordering::Relaxed);
            }
            ui.horizontal_wrapped(|ui| {
                if self.cancel.load(Ordering::Relaxed) {
                    ui.label("Cancelamento solicitado. A aguardar a operação de leitura atual e a gravação do relatório parcial…");
                } else {
                    ui.label("A calcular hashes; o total de ficheiros ainda não é conhecido.");
                }
            });
            return;
        }

        let Some(report) = &self.last_report else {
            ui.label("Ainda não existem análises concluídas.");
            return;
        };

        if let Some(failure) = &report.failure {
            ui.colored_label(danger(ui), format!("Falha: {failure}"));
        }
        if report.skipped > 0 {
            ui.label(format!("{} ficheiros do relatório ativo não analisados para evitar leitura da própria gravação.", report.skipped));
        }
        if report.interrupted {
            ui.label("Histórico parcial: apenas lotes confirmados. Não houve retoma automática da análise.");
        }
        if report.truncated {
            ui.colored_label(
                warning(ui),
                format!(
                    "Análise parcial: limite de {} entradas atingido.",
                    report.entry_limit.unwrap_or(1_000)
                ),
            );
        }

        ui.horizontal_wrapped(|ui| {
            Self::tile(ui, "Analisados", &report.scanned.to_string(), ACCENT);
            Self::tile(
                ui,
                "Correspondências",
                &report.matched.to_string(),
                warning(ui),
            );
            Self::tile(ui, "Erros", &report.errors.to_string(), danger(ui));
        });
        ui.add_space(12.0);
        let previous_filter = self.errors_only;
        ui.horizontal_wrapped(|ui| {
            choice(ui, &mut self.errors_only, false, "Todos os resultados");
            choice(ui, &mut self.errors_only, true, "Apenas erros");
        });
        if previous_filter != self.errors_only {
            self.result_page = 0;
        }
        let matching = if self.errors_only {
            report.errors
        } else {
            report.scanned + report.errors
        };
        let pages = matching.div_ceil(RESULT_PAGE_SIZE).max(1);
        self.result_page = self.result_page.min(pages - 1);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(self.result_page > 0, egui::Button::new("Anterior"))
                .clicked()
            {
                self.result_page -= 1;
            }
            ui.label(format!(
                "Página {} de {pages} · {matching} resultados",
                self.result_page + 1
            ));
            if ui
                .add_enabled(self.result_page + 1 < pages, egui::Button::new("Seguinte"))
                .clicked()
            {
                self.result_page += 1;
            }
        });
        if report.errors > 0 {
            let causes = if report.store_path.is_some() {
                report.error_causes.clone()
            } else {
                diagnostics::counts(&report.records)
                    .into_iter()
                    .map(|(k, v)| (k.to_owned(), v))
                    .collect()
            };
            for (label, count) in causes {
                ui.label(format!("{label}: {count}"));
            }
            ui.label("Erros de leitura não são deteções de malware. Consulte as mensagens abaixo antes de decidir uma correção.");
        }
        ui.separator();

        let records = if let Some(path) = &report.store_path {
            let key = (path.clone(), self.errors_only, self.result_page);
            if self.cached_key.as_ref() != Some(&key) {
                match store::page(path, self.errors_only, self.result_page) {
                    Ok(records) => {
                        self.cached_records = records;
                        self.page_error = None;
                    }
                    Err(error) => {
                        self.cached_records.clear();
                        self.page_error = Some(error);
                    }
                }
                self.cached_key = Some(key);
            }
            self.cached_records.clone()
        } else {
            report
                .records
                .iter()
                .filter(|r| !self.errors_only || r.error)
                .skip(self.result_page * RESULT_PAGE_SIZE)
                .take(RESULT_PAGE_SIZE)
                .cloned()
                .collect()
        };
        if let Some(error) = &self.page_error {
            ui.colored_label(danger(ui), format!("Erro ao ler página: {error}"));
        }
        egui::ScrollArea::vertical()
            .id_salt("result_records")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for record in &records {
                    ui.add_space(5.0);
                    ui.group(|ui| {
                        let color = if record.error {
                            danger(ui)
                        } else if record.matched {
                            warning(ui)
                        } else {
                            muted(ui)
                        };
                        ui.label(RichText::new(&record.verdict).strong().color(color));
                        ui.label(record.path.display().to_string());
                        if let Some(diagnostic) = &record.diagnostic {
                            ui.label(diagnostic.kind.advice());
                            if let Some(code) = diagnostic.os_code {
                                ui.label(format!("Código do sistema: {code}"));
                            }
                        } else if record.error {
                            ui.label("Registo antigo sem diagnóstico estruturado; conserve a mensagem original.");
                        }
                        if let Some(size) = record.size {
                            ui.label(format!("Tamanho: {size} bytes"));
                        }
                        if let Some(hash) = &record.hash {
                            ui.label(format!("SHA-256: {hash}"));
                        }
                    });
                }
            });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        Self::title(ui, "Definições", "Estado e limites da versão experimental.");
        ui.heading("Aparência");
        let previous = self.dark;
        ui.horizontal_wrapped(|ui| {
            choice(ui, &mut self.dark, false, "Claro");
            choice(ui, &mut self.dark, true, "Escuro");
        });
        if previous != self.dark {
            apply_theme(ui.ctx(), self.dark);
        }
        ui.label("O tema é guardado neste perfil de utilizador ao fechar a aplicação.");
        ui.add_space(16.0);
        ui.heading("Base de indicadores");
        ui.label(self.indicator_file.as_ref().map_or_else(
            || "Nenhuma base selecionada".to_owned(),
            |p| p.display().to_string(),
        ));
        ui.add_space(16.0);
        ui.heading("Limites");
        ui.label("Análise de pasta: limites opcionais; 10 000 por defeito. Histórico incremental com orçamento de 2 GiB por análise.");
        ui.label("Análise: apenas leitura, sem eliminação ou alteração de ficheiros.");
        ui.label("Resultados: gravação SQLite a cada 128 entradas; páginas de 100 registos. JSON antigo continua legível.");
        if let Ok(dir) = history::directory() {
            ui.label(format!("Pasta dos relatórios: {}", dir.display()));
        }
        ui.label("Os relatórios contêm caminhos e mensagens de erro, ficam neste perfil e não são enviados. São mantidos até os remover manualmente.");
        ui.label("O histórico serve para diagnóstico; não altera permissões, exclusões ou regras automaticamente.");
        ui.add_space(16.0);
        ui.heading("Funcionalidades futuras");
        ui.label("Proteção em tempo real, quarentena e atualizações assinadas ainda não estão disponíveis.");
    }
}

impl eframe::App for DesktopApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(
            THEME_KEY,
            if self.dark { "dark" } else { "light" }.to_owned(),
        );
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_scan_state();
        if self.receiver.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(210.0)
            .show(ctx, |ui| self.nav(ui));

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(
                        "C2L Antivirus · análise experimental, não substitui um antivírus",
                    )
                    .color(muted(ui)),
                );
                ui.separator();
                ui.label(format!("Análises nesta sessão: {}", self.run_count));
            });
        });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(20),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_max_width(1050.0);
                    match self.page {
                        Page::Home => self.home(ui),
                        Page::Scan => self.scan(ui),
                        Page::Results => self.results(ui),
                        Page::Settings => self.settings(ui),
                    }
                })
            });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("C2L Antivirus — Windows")
            .with_inner_size([1100.0, 760.0])
            .with_min_inner_size([780.0, 540.0]),
        ..Default::default()
    };

    eframe::run_native(
        "C2L Antivirus",
        options,
        Box::new(|context| {
            let dark = context
                .storage
                .and_then(|s| s.get_string(THEME_KEY))
                .as_deref()
                != Some("light");
            apply_theme(&context.egui_ctx, dark);
            let mut app = DesktopApp {
                dark,
                ..Default::default()
            };
            match history::directory().and_then(|dir| history::latest(&dir)) {
                Ok(Some((path, scan))) => app.restore_report(path, scan),
                Ok(None) => {}
                Err(error) => {
                    app.history_notice = format!("Não foi possível recuperar o histórico: {error}")
                }
            }
            Ok(Box::new(app))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn results_render_only_the_selected_page() {
        let ctx = egui::Context::default();
        let mut app = DesktopApp {
            result_page: 1,
            last_report: Some(summarize(
                (0..205)
                    .map(|i| error_record(format!("entry-{i}").into(), "error".into()))
                    .collect(),
                false,
            )),
            ..Default::default()
        };
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1100.0, 20000.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.results(ui));
            },
        );
        let labels: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.job.text.starts_with("entry-") => {
                    Some(text.galley.job.text.as_str())
                }
                _ => None,
            })
            .collect();
        assert_eq!(labels.len(), RESULT_PAGE_SIZE);
        assert_eq!(labels.first(), Some(&"entry-100"));
        assert_eq!(labels.last(), Some(&"entry-199"));
    }

    #[test]
    fn light_and_dark_settings_paint_readable_titles_and_both_theme_choices() {
        for dark in [false, true] {
            let ctx = egui::Context::default();
            apply_theme(&ctx, dark);
            let mut app = DesktopApp {
                dark,
                ..Default::default()
            };
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.settings(ui));
            });
            for (label, expected) in [
                (
                    "Definições",
                    if dark {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(22, 25, 40)
                    },
                ),
                ("Claro", if dark { Color32::WHITE } else { GOLD }),
                ("Escuro", if dark { GOLD } else { Color32::WHITE }),
            ] {
                let shape = output
                    .shapes
                    .iter()
                    .find(|shape| {
                        matches!(&shape.shape,
                    egui::epaint::Shape::Text(text) if text.galley.job.text == label)
                    })
                    .expect(label);
                let primitives = ctx.tessellate(vec![shape.clone()], output.pixels_per_point);
                let colors: Vec<_> = primitives
                    .iter()
                    .flat_map(|primitive| match &primitive.primitive {
                        egui::epaint::Primitive::Mesh(mesh) => {
                            mesh.vertices.iter().map(|v| v.color).collect()
                        }
                        _ => Vec::new(),
                    })
                    .collect();
                assert!(!colors.is_empty(), "{label}");
                assert!(
                    colors.iter().all(|color| *color == expected),
                    "{label}: {colors:?}"
                );
            }
        }
    }

    #[test]
    fn failed_scan_does_not_show_previous_success_and_partial_scan_selects_errors() {
        let mut app = DesktopApp::default();
        app.accept_result(Ok(summarize(
            vec![error_record("unreadable".into(), "denied".into())],
            false,
        )));
        assert!(app.errors_only);
        assert!(app.status.starts_with("Análise parcial"));
        app.accept_result(Err("invalid CSV".into()));
        assert!(app.last_report.is_none());
        assert_eq!(app.status, "invalid CSV");
    }

    #[test]
    fn theme_is_saved_without_scan_paths_or_reports() {
        #[derive(Default)]
        struct Storage(std::collections::HashMap<String, String>);
        impl eframe::Storage for Storage {
            fn get_string(&self, key: &str) -> Option<String> {
                self.0.get(key).cloned()
            }
            fn set_string(&mut self, key: &str, value: String) {
                self.0.insert(key.to_owned(), value);
            }
            fn flush(&mut self) {}
        }
        let mut storage = Storage::default();
        for dark in [false, true] {
            let mut app = DesktopApp {
                dark,
                selection: Some(PathBuf::from("private.txt")),
                ..Default::default()
            };
            eframe::App::save(&mut app, &mut storage);
            assert_eq!(
                eframe::Storage::get_string(&storage, THEME_KEY).as_deref(),
                Some(if dark { "dark" } else { "light" })
            );
            assert_eq!(storage.0.len(), 1);
        }
    }

    #[test]
    fn all_pages_produce_layout_in_both_themes_at_supported_sizes() {
        for dark in [false, true] {
            for size in [[780.0, 540.0], [1100.0, 760.0], [1920.0, 1080.0]] {
                for page in [Page::Home, Page::Scan, Page::Results, Page::Settings] {
                    let ctx = egui::Context::default();
                    apply_theme(&ctx, dark);
                    let mut app = DesktopApp {
                        dark,
                        page,
                        ..Default::default()
                    };
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(size[0], size[1]),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::SidePanel::left("nav")
                                .exact_width(210.0)
                                .show(ctx, |ui| app.nav(ui));
                            egui::CentralPanel::default().show(ctx, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| match page {
                                    Page::Home => app.home(ui),
                                    Page::Scan => app.scan(ui),
                                    Page::Results => app.results(ui),
                                    Page::Settings => app.settings(ui),
                                });
                            });
                        },
                    );
                    assert!(!output.shapes.is_empty());
                }
            }
        }
    }

    #[test]
    fn files_without_a_database_are_not_marked_clean() {
        let record = make_record(
            ScanResult {
                path: PathBuf::from("test.exe"),
                size_bytes: 3,
                sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                    .to_owned(),
            },
            None,
        );
        assert!(!record.matched);
        assert!(record.verdict.contains("Não verificado"));
    }

    #[test]
    fn synthetic_demo_indicator_is_identified() {
        let hash = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let db = IndicatorDb::parse(Cursor::new(format!("{hash},C2L_DEMO_ONLY\n"))).unwrap();
        let record = make_record(
            ScanResult {
                path: PathBuf::from("abc.txt"),
                size_bytes: 3,
                sha256: hash.to_owned(),
            },
            Some(&db),
        );
        assert!(record.matched);
        assert_eq!(record.verdict, "Correspondência: C2L_DEMO_ONLY");
    }
}
