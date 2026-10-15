# Observações do teste Windows — 15 de outubro de 2026

Evidência: duas capturas fornecidas pelo utilizador, da versão desktop 0.1.0.

- Tema claro: opção Escuro e títulos brancos sobre superfície clara.
- Primeira análise: 991 ficheiros analisados, 0 correspondências, 9 erros.
- Limite de 1 000 entradas atingido: análise parcial.
- Registos visíveis indicam ausência de base de indicadores. Zero
  correspondências não constitui resultado de segurança.
- As capturas não mostram as mensagens dos nove erros nem todos os alvos;
  não é possível atribuir causas ou recuperar esses detalhes a partir delas.
- A versão testada mantinha o relatório apenas em memória. Este documento
  conserva o resumo observado; não é um relatório produzido pelo scanner.

Correções nesta branch: opções com fundo explícito, contraste dos títulos,
teste da cor dos glifos efetivamente produzidos pelo egui, histórico local JSON
e filtro de erros. Os próximos relatórios permitem investigar os erros reais.

## Prioridades decorrentes do teste

1. Repetir a análise com histórico ativo e consultar as nove mensagens, caso
   voltem a ocorrer. Distinguir acesso negado, ficheiro em uso/desaparecido,
   falha de I/O e erro de implementação com evidência; não inferir a causa do
   contador. Adicionar regressões reproduzíveis para erros do scanner.
2. Progresso/cancelamento e limites configuráveis com orçamento de recursos.
   Não remover simplesmente o limite de 1 000 para varrer discos inteiros.
3. Assinaturas autenticadas, atualização verificável e testes de falsos positivos.
4. Heurísticas explicáveis, quarentena com recuperação e consentimento por ação.
5. Serviço Windows separado e monitorização com cobertura mensurável; avaliar
   minifilter apenas se o bloqueio pré-acesso justificar o risco adicional.

Autoajustes futuros precisam de métricas, versões, testes e rollback. Histórico
de erros não deve criar exclusões, ampliar privilégios ou suprimir avisos
automaticamente. Primeiro corrigir causas confirmadas e medir a regressão.
