//! Stable causes for new reports; never infer policy from localized error text.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: Cause,
    pub os_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    AccessDenied,
    InUse,
    NotFound,
    Changed,
    Other,
}

impl Diagnostic {
    pub fn from_error(error: &io::Error) -> Self {
        let kind = if error
            .get_ref()
            .is_some_and(|e| e.is::<c2l_scanner::FileChanged>())
        {
            Cause::Changed
        } else if cfg!(windows) && matches!(error.raw_os_error(), Some(32 | 33)) {
            Cause::InUse
        } else {
            match error.kind() {
                io::ErrorKind::PermissionDenied => Cause::AccessDenied,
                io::ErrorKind::NotFound => Cause::NotFound,
                _ => Cause::Other,
            }
        };
        Self {
            kind,
            os_code: error.raw_os_error(),
        }
    }
}

impl Cause {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AccessDenied => "Acesso negado",
            Self::InUse => "Ficheiro em uso",
            Self::NotFound => "Ficheiro não encontrado",
            Self::Changed => "Ficheiro alterado durante a leitura",
            Self::Other => "Outros erros",
        }
    }
    pub fn advice(&self) -> &'static str {
        match self {
            Self::AccessDenied => {
                "Cobertura incompleta: o perfil atual não tem acesso. Não foram alteradas permissões nem criada uma exclusão."
            }
            Self::InUse => {
                "Repita a análise mais tarde. Não é necessário terminar processos do sistema."
            }
            Self::NotFound => {
                "O caminho deixou de estar disponível. Verifique a localização antes de repetir."
            }
            Self::Changed => {
                "A impressão digital foi descartada. Repita quando o ficheiro estiver estável."
            }
            Self::Other => {
                "Conserve a mensagem e o código para investigação; este erro não prova infeção."
            }
        }
    }
}

pub fn counts(records: &[crate::Record]) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for record in records.iter().filter(|r| r.error) {
        let label = record
            .diagnostic
            .as_ref()
            .map_or("Sem classificação (relatório antigo)", |d| {
                d.kind.label()
            });
        *counts.entry(label).or_default() += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_codes_and_does_not_classify_localized_text() {
        assert_eq!(
            Diagnostic::from_error(&io::Error::new(
                io::ErrorKind::PermissionDenied,
                "qualquer idioma"
            ))
            .kind,
            Cause::AccessDenied
        );
        assert_eq!(
            Diagnostic::from_error(&io::Error::other("access denied (os error 5)")).kind,
            Cause::Other
        );
        assert_eq!(
            Diagnostic::from_error(&io::Error::other(c2l_scanner::FileChanged)).kind,
            Cause::Changed
        );
        #[cfg(windows)]
        for code in [32, 33] {
            let diagnostic = Diagnostic::from_error(&io::Error::from_raw_os_error(code));
            assert_eq!(diagnostic.kind, Cause::InUse);
            assert_eq!(diagnostic.os_code, Some(code));
        }
    }
    #[test]
    fn legacy_errors_remain_unclassified() {
        let old = crate::error_record("example".into(), "os error 5".into());
        let new = crate::io_error_record(
            "example".into(),
            io::Error::new(io::ErrorKind::NotFound, "gone"),
        );
        let records = vec![old, new];
        let loaded: Vec<crate::Record> =
            serde_json::from_str(&serde_json::to_string(&records).unwrap()).unwrap();
        assert_eq!(counts(&loaded)["Sem classificação (relatório antigo)"], 1);
        assert_eq!(counts(&loaded)["Ficheiro não encontrado"], 1);
    }
}
