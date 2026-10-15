//! C2L Antivirus desktop prototype (Windows).
//! The program only reads selected paths. It does not offer real-time protection.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use c2l_detection::{Verdict, classify};
use c2l_indicators::IndicatorDb;
use c2l_scanner::{FileOutcome, ScanResult, scan_directory, scan_file};
use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

const MAX_FILES: usize = 1_000;
const BACKGROUND: Color32 = Color32::from_rgb(15, 22, 35);
const TEXT_MUTED: Color32 = Color32::from_rgb(161, 174, 194);
const ACCENT: Color32 = Color32::from_rgb(62, 194, 187);
const WARNING: Color32 = Color32::from_rgb(245, 172, 85);
const DANGER: Color32 = Color32::from_rgb(244, 118, 118);

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

#[derive(Debug)]
struct Record {
    path: PathBuf,
    size: Option<u64>,
    hash: Option<String>,
    verdict: String,
    matched: bool,
    error: bool,
}

#[derive(Debug, Default)]
struct ScanBatch {
    records: Vec<Record>,
    scanned: usize,
    matched: usize,
    errors: usize,
    truncated: bool,
}

fn make_record(result: ScanResult, indicators: Option<&IndicatorDb>) -> Record {
    let (verdict, matched) = match classify(&result.sha256, indicators) {
        Verdict::NotChecked => ("Não verificado (sem base de indicadores)".to_owned(), false),
        Verdict::NotListed => ("Sem correspondência; segurança não garantida".to_owned(), false),
        Verdict::Listed(name) => (format!("Correspondência: {name}"), true),
    };

    Record {
        path: result.path,
        size: Some(result.size_bytes),
        hash: Some(result.sha256),
        verdict,
        matched,
        error: false,
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
    }
}

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
    }
}

fn execute_scan(
    mode: ScanMode,
    path: PathBuf,
    indicator_file: Option<PathBuf>,
) -> Result<ScanBatch, String> {
    let db = indicator_file
        .as_ref()
        .map(|location| {
            IndicatorDb::load(location)
                .map_err(|error| format!("Não foi possível carregar a base: {error}"))
        })
        .transpose()?;

    match mode {
        ScanMode::File => {
            let scanned = scan_file(&path)
                .map_err(|error| format!("Não foi possível analisar o ficheiro: {error}"))?;
            Ok(summarize(vec![make_record(scanned, db.as_ref())], false))
        }
        ScanMode::Folder => {
            let directory = scan_directory(&path, MAX_FILES)
                .map_err(|error| format!("Não foi possível analisar a pasta: {error}"))?;
            let records = directory
                .outcomes
                .into_iter()
                .map(|outcome| match outcome {
                    FileOutcome::Scanned(result) => make_record(result, db.as_ref()),
                    FileOutcome::Failed { path, message } => error_record(path, message),
                })
                .collect();
            Ok(summarize(records, directory.truncated))
        }
    }
}

struct DesktopApp {
    page: Page,
    mode: ScanMode,
    selection: Option<PathBuf>,
    indicator_file: Option<PathBuf>,
    receiver: Option<Receiver<Result<ScanBatch, String>>>,
    last_report: Option<ScanBatch>,
    status: String,
    run_count: usize,
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
        let indicators = self.indicator_file.clone();
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.page = Page::Results;
        self.status = "Análise em curso. A janela continua disponível.".to_owned();

        thread::spawn(move || {
            let result = execute_scan(mode, path, indicators);
            let _ = sender.send(result);
        });
    }

    fn update_scan_state(&mut self) {
        let Some(receiver) = self.receiver.as_ref() else {
            return;
        };
        match receiver.try_recv() {
            Ok(Ok(report)) => {
                self.status = format!(
                    "Concluído: {} ficheiros analisados, {} correspondências, {} erros{}",
                    report.scanned,
                    report.matched,
                    report.errors,
                    if report.truncated { " (limite atingido)" } else { "" }
                );
                self.run_count += 1;
                self.last_report = Some(report);
                self.receiver = None;
            }
            Ok(Err(message)) => {
                self.status = message;
                self.receiver = None;
            }
            Err(TryRecvError::Disconnected) => {
                self.status = "O processo de análise terminou inesperadamente.".to_owned();
                self.receiver = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    fn nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.heading(RichText::new("C2L").size(30.0).color(ACCENT).strong());
        ui.label(RichText::new("ANTIVIRUS").color(TEXT_MUTED).size(12.0));
        ui.add_space(25.0);
        ui.separator();
        ui.add_space(8.0);

        for (page, name) in [
            (Page::Home, "Início"),
            (Page::Scan, "Analisar"),
            (Page::Results, "Resultados"),
            (Page::Settings, "Definições"),
        ] {
            if ui.selectable_label(self.page == page, name).clicked() {
                self.page = page;
            }
            ui.add_space(5.0);
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.label(RichText::new("Versão 0.1.0 · Protótipo").small().color(TEXT_MUTED));
            ui.label(RichText::new("Sem proteção em tempo real").small().color(WARNING));
        });
    }

    fn title(ui: &mut egui::Ui, heading: &str, subtitle: &str) {
        ui.add_space(12.0);
        ui.label(RichText::new(heading).size(28.0).strong());
        ui.label(RichText::new(subtitle).color(TEXT_MUTED));
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(14.0);
    }

    fn tile(ui: &mut egui::Ui, name: &str, value: &str, color: Color32) {
        egui::Frame::default()
            .fill(Color32::from_rgb(26, 38, 55))
            .inner_margin(16)
            .corner_radius(10)
            .show(ui, |ui| {
                ui.set_min_width(182.0);
                ui.label(RichText::new(name).color(TEXT_MUTED));
                ui.add_space(8.0);
                ui.label(RichText::new(value).size(27.0).color(color).strong());
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
                WARNING,
            );
            Self::tile(
                ui,
                "Erros de leitura",
                &report.map_or(0, |r| r.errors).to_string(),
                DANGER,
            );
        });

        ui.add_space(24.0);
        ui.heading("Estado do produto");
        ui.add_space(8.0);
        ui.label(RichText::new("Scanner manual: disponível").color(ACCENT));
        ui.label(RichText::new("Proteção em tempo real: não implementada").color(WARNING));
        ui.label(RichText::new("Quarentena e bloqueio: não implementados").color(WARNING));
        ui.label(RichText::new("Atualizações automáticas de assinaturas: não implementadas").color(WARNING));
        ui.add_space(12.0);

        if ui.button("Iniciar uma análise  →").clicked() {
            self.page = Page::Scan;
        }

        ui.add_space(24.0);
        ui.label(
            RichText::new("A ausência de uma correspondência SHA-256 não significa que o ficheiro esteja seguro.")
                .color(TEXT_MUTED),
        );
    }

    fn scan(&mut self, ui: &mut egui::Ui) {
        Self::title(
            ui,
            "Análise manual",
            "Escolhe um ficheiro ou uma pasta. Nenhum ficheiro será modificado.",
        );

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.mode, ScanMode::File, "Um ficheiro");
            ui.selectable_value(&mut self.mode, ScanMode::Folder, "Uma pasta");
        });
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            if ui.button("Procurar…").clicked() {
                let selected = match self.mode {
                    ScanMode::File => rfd::FileDialog::new().pick_file(),
                    ScanMode::Folder => rfd::FileDialog::new().pick_folder(),
                };
                if let Some(path) = selected {
                    self.selection = Some(path);
                }
            }

            ui.label(
                self.selection
                    .as_ref()
                    .map_or_else(|| "Nada selecionado".to_owned(), |p| p.display().to_string()),
            );
        });

        if self.mode == ScanMode::Folder {
            ui.label(
                RichText::new(format!("Limite por análise: {MAX_FILES} entradas. Ligações simbólicas são ignoradas."))
                    .small()
                    .color(TEXT_MUTED),
            );
        }

        ui.add_space(16.0);
        ui.label(RichText::new("Base de indicadores SHA-256 (opcional)").strong());
        ui.horizontal_wrapped(|ui| {
            ui.label(
                self.indicator_file
                    .as_ref()
                    .map_or("Nenhuma base carregada", |p| p.to_str().unwrap_or("Nome não representável")),
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
        let ready = self.selection.is_some() && self.receiver.is_none();
        if ui.add_enabled(ready, egui::Button::new("Iniciar análise")).clicked() {
            self.begin_scan();
        }
        if self.receiver.is_some() {
            ui.spinner();
        }
        ui.add_space(14.0);
        ui.label(
            RichText::new("O C2L calcula hashes e, quando existe uma base fornecida, procura correspondências. Não determina se os restantes ficheiros estão seguros.")
                .color(TEXT_MUTED),
        );
    }

    fn results(&self, ui: &mut egui::Ui) {
        Self::title(
            ui,
            "Resultados",
            "Resultados apenas da última análise executada nesta sessão.",
        );

        ui.label(&self.status);
        ui.add_space(12.0);

        if self.receiver.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("A calcular hashes…");
            });
            return;
        }

        let Some(report) = &self.last_report else {
            ui.label("Ainda não existem análises concluídas.");
            return;
        };

        if report.truncated {
            ui.colored_label(WARNING, format!("Análise parcial: atingido o limite de {MAX_FILES} entradas."));
        }

        ui.horizontal_wrapped(|ui| {
            Self::tile(ui, "Analisados", &report.scanned.to_string(), ACCENT);
            Self::tile(ui, "Correspondências", &report.matched.to_string(), WARNING);
            Self::tile(ui, "Erros", &report.errors.to_string(), DANGER);
        });
        ui.add_space(12.0);
        ui.separator();

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for record in &report.records {
                ui.add_space(5.0);
                ui.group(|ui| {
                    let color = if record.error {
                        DANGER
                    } else if record.matched {
                        WARNING
                    } else {
                        TEXT_MUTED
                    };
                    ui.label(RichText::new(&record.verdict).strong().color(color));
                    ui.label(record.path.display().to_string());
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

    fn settings(&self, ui: &mut egui::Ui) {
        Self::title(ui, "Definições", "Estado e limites da versão experimental.");
        ui.heading("Base de indicadores");
        ui.label(
            self.indicator_file
                .as_ref()
                .map_or_else(|| "Nenhuma base selecionada".to_owned(), |p| p.display().to_string()),
        );
        ui.add_space(16.0);
        ui.heading("Limites");
        ui.label(format!("Análise de pasta: máximo de {MAX_FILES} entradas registadas."));
        ui.label("Análise: apenas leitura, sem eliminação ou alteração de ficheiros.");
        ui.label("Resultados: apenas em memória, enquanto a aplicação estiver aberta.");
        ui.add_space(16.0);
        ui.heading("Funcionalidades futuras");
        ui.label("Proteção em tempo real, quarentena, atualizações assinadas e relatórios persistentes ainda não estão disponíveis.");
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_scan_state();
        if self.receiver.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(178.0)
            .show(ctx, |ui| self.nav(ui));

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("C2L Antivirus · análise experimental, não substitui um antivírus").color(TEXT_MUTED));
                ui.separator();
                ui.label(format!("Análises nesta sessão: {}", self.run_count));
            });
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(BACKGROUND).inner_margin(20))
            .show(ctx, |ui| match self.page {
                Page::Home => self.home(ui),
                Page::Scan => self.scan(ui),
                Page::Results => self.results(ui),
                Page::Settings => self.settings(ui),
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
            context.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(DesktopApp::default()))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn files_without_a_database_are_not_marked_clean() {
        let record = make_record(
            ScanResult {
                path: PathBuf::from("test.exe"),
                size_bytes: 3,
                sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
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
