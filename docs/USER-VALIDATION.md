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


## Relatórios locais revistos em 16 de outubro de 2026

Foram lidos quatro relatórios, sem publicar caminhos individuais nem o conteúdo
privado dos ficheiros. As causas abaixo provêm dos códigos presentes nas mensagens
antigas; os novos relatórios 0.2.1 passam a guardar campos estruturados.

| Versão | Analisados | Erros | Acesso negado (5) | Em uso (32) | Cobertura |
|---|---:|---:|---:|---:|---|
| 0.1.0 | 991 | 9 | 5 | 4 | Limite de 1 000 |
| 0.2.0 | 26 281 | 16 | 13 | 3 | Cancelada |
| 0.2.0 | 9 984 | 16 | 13 | 3 | Limite de 10 000 |
| 0.2.0 | 49 983 | 17 | 14 | 3 | Limite de 50 000 |

Foi encontrado um relatório persistido com os mesmos contadores da captura
inicial. Isto complementa a evidência anteriormente disponível apenas nas imagens.
As contagens não são comparáveis como taxa de regressão: as análises cobriram
quantidades diferentes e ficheiros podem mudar entre execuções.

A versão 0.2.1 distingue acesso negado, ficheiro em uso, caminho desaparecido,
alteração durante a leitura e outros erros; conserva a mensagem e o código do SO.
Relatórios antigos continuam legíveis e não são reescritos nem classificados
silenciosamente por texto. Não foram criadas exclusões nem alteradas permissões.
O teste Windows abre um ficheiro temporário com partilha bloqueada e confirma
que o erro 32 atravessa o scanner intacto.

O scanner agora emite resultados individualmente; a GUI ainda os acumula para
o histórico JSON e mantém o limite de 50 000. A próxima etapa é armazenamento
incremental com paginação a partir do disco, recuperação de análise interrompida
e medições de recursos antes de oferecer análise de volumes sem este limite.


## Aceitação de escala — desktop 0.3.0

Em 16 de outubro de 2026, o teste de integração criou e analisou 50 129 ficheiros
reais de três bytes, com histórico incremental e consulta da página após a entrada
50 000. Resultado: 50 129 analisados, zero erros, sem truncagem ou cancelamento;
nenhum vetor de resultados completos retido no resumo.

- Tempo da análise/gravação/reabertura: 12,174 s (build final de testes debug, rusqlite 0.40.2).
- Banco final: 18 706 432 bytes.
- Pico observado do working set do processo de teste: 14 225 408 bytes
  (aprox. 13,6 MiB), observado a cada 200 ms pelo processo controlador.
- Teste completo incluindo criação e limpeza: 35,75 s.

Estes números são de um corpus pequeno por ficheiro e cache local; não medem
GUI, discos completos, ficheiros grandes, redes, eficácia antivírus ou cancelamento
sob I/O bloqueado. Não devem ser apresentados como desempenho garantido.

A suite também verifica 60 001 registos sintéticos paginados, filtro de erros,
recuperação depois de saída de processo sem executar destrutores, falta de espaço
simulada por quota SQLite, preservação de lotes confirmados e omissão contada do
próprio relatório ativo. A retoma da análise não está implementada: recuperam-se
os resultados confirmados e apresenta-se cobertura parcial.


## Análise real de 772 108 ficheiros e entrega 0.3.1

O relatório SQLite 0.3.0 consultado em modo só de leitura tem 774 614 registos:
772 108 ficheiros analisados e 2 506 erros, sem truncagem/cancelamento/interrupção.
Foram omitidos explicitamente dois ficheiros do relatório ativo. Base de indicadores
ausente: zero correspondências não mede eficácia de deteção. Banco: 322 244 608 bytes.

Erros: 1 375 acesso negado (5); 201 partilha (32) e 194 bloqueio de região (33);
657 fornecedor de nuvem não ativo (362); 74 inacessíveis ao sistema (1920);
3 alterações durante leitura e 2 caminhos não encontrados. Não publicamos caminhos
individuais nem deduzimos a causa concreta dos 74 casos apenas pelo código 1920.

A versão 0.3.1 apresenta os dois novos grupos também no relatório antigo, a partir
dos códigos estruturados e sem reescrever o original. Oferece reanálise explícita
dos caminhos com erro, com relatório separado. Não foi executada uma reanálise
automática dos dados pessoais nem alterada a sincronização do utilizador.

Referências primárias para a classificação:
- [Microsoft: ERROR_CLOUD_FILE_PROVIDER_NOT_RUNNING](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Foundation/constant.ERROR_CLOUD_FILE_PROVIDER_NOT_RUNNING.html)
- [Microsoft: ERROR_CANT_ACCESS_FILE (1920)](https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--1700-3999-)
