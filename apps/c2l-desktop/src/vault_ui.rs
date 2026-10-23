//! Quarantine actions run away from the GUI thread, with a file-specific confirmation.
use crate::{
    quarantine::{self, Consent, Entry},
    theme::danger,
};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
};
enum Reply {
    Prepared(Consent),
    Refreshed(Vec<Entry>),
    Done(String),
    Reviewed(i64, PathBuf, String),
}
#[derive(Default)]
pub struct VaultUi {
    pending: Option<Consent>,
    restore: Option<(i64, PathBuf, std::time::Instant)>,
    entries: Vec<Entry>,
    page: usize,
    notice: String,
    worker: Option<Receiver<Result<Reply, String>>>,
}
impl VaultUi {
    fn task(&mut self, action: impl FnOnce() -> Result<Reply, String> + Send + 'static) {
        let (sender, receiver) = mpsc::channel();
        self.worker = Some(receiver);
        thread::spawn(move || {
            let _ = sender.send(action());
        });
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if let Some(receiver) = &self.worker {
            match receiver.try_recv() {
                Ok(reply) => {
                    self.worker = None;
                    match reply {
                        Ok(Reply::Prepared(c)) => self.pending = Some(c),
                        Ok(Reply::Refreshed(entries)) => self.entries = entries,
                        Ok(Reply::Done(message)) => self.notice = message,
                        Ok(Reply::Reviewed(id, path, message)) => {
                            self.restore = Some((id, path, std::time::Instant::now()));
                            self.notice = message;
                        }
                        Err(e) => self.notice = e,
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.worker = None;
                    self.notice =
                        "Operação interrompida; atualize a lista para verificar o estado".into();
                }
                _ => {
                    ui.spinner();
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
        }
        ui.heading("Quarentena manual · experimental");
        ui.label("Cofre cifrado para este utilizador e computador. Até 32 MiB por ficheiro. Não há remoção automática. A restauração mantém a cópia no cofre e herda as permissões do destino; não declara o conteúdo seguro.");
        ui.label("Ligações, streams adicionais e ficheiros EFS são recusados. Guarde uma cópia de segurança antes de usar esta versão experimental com dados importantes.");
        ui.add_enabled_ui(self.worker.is_none(),|ui| {
            if ui.button("Selecionar ficheiro para isolar…").clicked()
                && let Some(path)=rfd::FileDialog::new().pick_file() {
                self.pending=None;
                self.task(move||quarantine::prepare(&path,"Seleção manual e confirmação específica na GUI".into()).map(Reply::Prepared));
            }
            if let Some(consent)=self.pending.clone() {
                ui.separator();ui.label(format!("Isolar: {}",consent.path.display()));ui.label(format!("{} bytes · SHA-256: {}",consent.size,consent.hash));
                ui.colored_label(danger(ui),"Confirmar retira este ficheiro da localização original após guardar e verificar a cópia cifrada. Esta seleção manual não prova que exista malware.");
                if ui.button("Confirmar isolamento deste ficheiro").clicked() {
                    self.pending=None;self.task(move||{let id=quarantine::isolate(&quarantine::directory()?,&consent)?;Ok(Reply::Done(format!("Ficheiro isolado #{id}. Atualize a lista.")))});
                }
                if ui.button("Cancelar seleção").clicked(){self.pending=None;}
            }
            if ui.button("Atualizar lista do cofre").clicked() {let page=self.page;self.task(move||quarantine::list_page(&quarantine::directory()?,page).map(Reply::Refreshed));}
            ui.horizontal(|ui|{
                if ui.add_enabled(self.page>0,egui::Button::new("Página anterior")).clicked(){self.page-=1;let page=self.page;self.task(move||quarantine::list_page(&quarantine::directory()?,page).map(Reply::Refreshed));}
                ui.label(format!("Página {}",self.page+1));
                if ui.add_enabled(self.entries.len()==100,egui::Button::new("Página seguinte")).clicked(){self.page+=1;let page=self.page;self.task(move||quarantine::list_page(&quarantine::directory()?,page).map(Reply::Refreshed));}
            });
            ui.label("Até 100 itens por página. As cópias recuperáveis continuam disponíveis mesmo quando a remoção do original não pôde ser confirmada.");
            for entry in self.entries.clone() {
                ui.group(|ui|{
                    ui.label(format!("#{} · {} · {}",entry.id,if entry.state=="isolated" {"Isolado"} else {"Cópia guardada; remoção não confirmada"},entry.source));ui.label(format!("SHA-256: {}",entry.hash));
                    if ui.button(format!("Recuperar cópia #{}…",entry.id)).clicked()
                        && let Some(destination)=rfd::FileDialog::new().set_file_name("recuperado.bin").save_file() {self.restore=None;self.notice="A reavaliar a cópia com as bases atuais…".into();self.task(move||{let message=quarantine::review_restore(&quarantine::directory()?,entry.id)?;Ok(Reply::Reviewed(entry.id,destination,message))});}
                });
            }
            if let Some((id,destination,reviewed))=self.restore.clone() {
                ui.colored_label(danger(ui),format!("Recuperar #{id} para {}? O ficheiro pode conter malware. Não será executado nem considerado seguro; analise-o antes de o utilizar.",destination.display()));
                if ui.button("Confirmar recuperação neste destino").clicked() {
                    self.restore=None;self.task(move||{if reviewed.elapsed()>std::time::Duration::from_secs(300){return Err("Confirmação expirada; repita a reavaliação".into());}quarantine::restore(&quarantine::directory()?,id,&destination)?;Ok(Reply::Done("Cópia recuperada; original cifrado mantido. Analise o destino antes de usar.".into()))});
                }
                if ui.button("Cancelar recuperação").clicked(){self.restore=None;}
            }
        });
        ui.separator();
        ui.label(&self.notice);
    }
}
