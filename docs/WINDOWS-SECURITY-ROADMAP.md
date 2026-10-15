# Windows: arquitetura e critérios de entrega

## Estado desta entrega

O desktop Rust/egui faz análises manuais em thread separada, calcula SHA-256
e compara um CSV local opcional. Não autentica esse CSV, não deteta malware
desconhecido, não bloqueia execuções, não monitoriza o sistema e não altera
ficheiros. Sem correspondência não significa seguro. A preferência de tema é
local ao utilizador; relatórios JSON de diagnóstico são guardados no perfil
local após cada análise, incluindo erros e cobertura parcial. Este histórico
não é uma auditoria autenticada nem implementa autoajustes do motor.

## Limites de confiança e privilégios

- **GUI sem elevação**: seleção de alvos, progresso, resultados e consentimento
  específico para uma ação. Não receber conteúdo remoto como instruções.
- **Motor portátil**: hashing, formatos de indicadores, decisões e relatórios
  versionados nos crates partilhados. Parsers de conteúdo não confiável em worker
  isolado, com limites de memória, tempo e expansão de arquivos comprimidos.
- **Serviço Windows separado, futuro**: SCM, identidade/SID de serviço e ACLs
  restritas; apenas os privilégios necessários para cada operação. Não usar
  LocalSystem por conveniência. Instalação/atualização do serviço exige UAC;
  abrir a GUI ou analisar ficheiros acessíveis ao utilizador não exige UAC.
- **IPC local versionado**: named pipe com DACL explícita, rejeição de clientes
  remotos, identidade/token do cliente verificados e autorização por comando.
  Pedidos com limites de tamanho, correlação e proteção contra repetição.
  Não aceitar comandos shell nem operações arbitrárias de escrita.
- **Remediação**: consentimento ligado ao ID da deteção, identidade do ficheiro,
  destino e ação exata. Revalidar identidade/hash pelo handle antes de alterar;
  rejeitar reparse points, mudanças de alvo e acessos não autorizados. A permissão
  de instalar o serviço não autoriza apagar ou editar todos os discos.

Não mudar ACLs de volumes, tomar posse de discos ou pedir escrita irrestrita
no instalador. Acesso negado deve produzir cobertura parcial explícita.

## Roadmap por dependências

| Marco | Entrega | Critério para aceitar |
|---|---|---|
| M1 — scanner robusto (parcial) | Já existe contagem de progresso, cancelamento cooperativo, limite de entradas configurável e JSON versionado. Falta processamento por lotes, retoma, limites de tempo/I/O e deteção de alterações durante hashing | Testes de acesso negado, ficheiro mutável, links/junctions, caminhos Unicode/longos, pastas enormes e cancelamento; todos os itens omitidos constam do relatório |
| M2 — confiança nas assinaturas | Manifesto assinado com chave pública embutida, versão monotónica, hash/tamanho do pacote, validade e versão mínima do motor; verificação antes de publicar snapshot imutável | Rejeitar assinatura inválida, pacote truncado, replay/downgrade e expirado; rotação/revogação de chaves testada; falha mantém última base válida e sinaliza idade |
| M3 — deteção explicável | Regras heurísticas versionadas sobre PE/metadados/conteúdo, separadas de correspondências exatas; evidências e confiança no resultado | Corpus benigno versionado, métricas de falsos positivos/negativos por regra; parser fuzzing; heurística isolada não remove ficheiros automaticamente |
| M4 — quarentena recuperável | Serviço/IPC revistos, cofre com ACL restrita, blobs não executáveis, metadados de origem/hash/ACL, transação e journal durável | Crash/power loss em cada fase, disco cheio, troca de alvo e restauração com colisão testados; nunca apagar original antes de cópia íntegra e commit; restauração autorizada não sobrescreve destino |
| M5 — monitorização pós-alteração | Serviço com ReadDirectoryChangesW, fila limitada, coalescing/debounce, cache por identidade+versão da base; USN em volumes compatíveis para reconciliação | Overflow, reinício, journal recriado, disco removível e eventos duplicados não desaparecem silenciosamente; rescan explícito de lacunas; GUI mostra cobertura real e saúde do serviço |
| M6 — bloqueio antes do acesso, se necessário | Avaliação de minifilter Filter Manager após medir limitações de M5; revisão dedicada do protocolo kernel/user-mode | Justificativa de necessidade, driver assinado e pacote validado, Driver Verifier/HLK aplicáveis, testes de stress/boot/rollback; orçamento de latência e política de timeout publicados |
| M7 — distribuição | Authenticode para executável/serviço/instalador, atualização autenticada e atómica, rollout gradual, recuperação | Upgrade/downgrade recusado, chave revogada, interrupção e rollback testados; consentimento UAC com editor identificado; nenhum pedido de desativar Defender/Secure Boot |

M1 e M2 precedem uso de deteções para remediação. M4 depende da revisão de
privilégios/IPC. M5 pode começar em modo de observação depois de M1/M2;
não equivale a impedir execução. M6 é opcional e exige avaliação própria.

## Monitorização Windows: o que cada fonte permite

`ReadDirectoryChangesW` notifica alterações em diretórios selecionados; eventos
podem perder-se quando o buffer transborda. O USN journal permite reconciliar
alterações nos volumes suportados, mas não é um mecanismo de bloqueio nem uma
prova de que todo o conteúdo foi analisado. Guardar journal ID/cursor e fazer
nova enumeração quando houver descontinuidade. Windows Event Log/ETW, quando
justificados, complementam auditoria/telemetria; não substituem controlo de I/O.

Um minifilter pode interpor operações de sistema de ficheiros antes do acesso,
mas introduz risco no kernel. Só considerar após demonstrar necessidade de
bloqueio, usando APIs documentadas do Filter Manager, sem hooks não documentados.
Não anunciar “proteção em tempo real” baseada num watcher. A GUI futura deve
distinguir observação pós-escrita, deteção e bloqueio, com cobertura e falhas.

## Quarentena, auditoria e falsos positivos

Cada ação deve registar data UTC, versão do motor/base/regra, evidência, identidade
do objeto, utilizador/consentimento, resultado e erro. Logs locais com ACL,
rotação, retenção configurável e exportação redigida; sem upload de ficheiros ou
caminhos pessoais por defeito. ACL não garante integridade contra administrador;
auditoria resistente a adulteração requer cadeia autenticada/âncora externa.

Quarentena deve remover execução/acesso ao blob e preservar dados necessários à
recuperação, sem executar o conteúdo. Encriptação não substitui ACL. Movimentos
entre volumes precisam de cópia verificada, commit e remoção autorizada; preservar
estado recuperável se a remoção falhar. Ficheiros protegidos/sistema ficam fora
da remediação automática até existir política específica. Restauração exige
reavaliação pela base atual, confirmação do destino e auditoria. Exceções devem
ser específicas, expiráveis e visíveis, nunca exclusões silenciosas de volumes.

## Performance e validação

Antes de M1, registar hardware/corpus e medir throughput, RAM máxima, CPU,
latência p50/p95 e tempo de cancelamento. Metas iniciais de engenharia (não
resultados medidos): fila até 10 000 itens com reconciliação em overflow,
workers limitados/configuráveis, cancelamento em até 2 s entre blocos de leitura,
nenhuma leitura de disco na thread GUI. Definir orçamento de latência para M6
com benchmarks antes de implementar bloqueio. Testar SSD/HDD, bateria, ficheiros
grandes, arquivos comprimidos e carga concorrente. Cache invalida quando objeto,
regra ou base muda. Cobertura parcial nunca aparece como análise completa.

Usar primeiro `tests/fixtures/c2l-demo.txt` e `demo-indicators.csv` (apenas `abc`).
EICAR é um teste inofensivo de resposta antivírus, não prova de eficácia. Em VM
de testes, obter apenas do site oficial; manter a proteção instalada ativa.
Se outro antivírus bloquear o download, registar o resultado, não o desativar.
No C2L atual, o teste só encontra correspondência se o hash exato estiver no CSV
selecionado; não existe deteção EICAR integrada. Nos marcos seguintes, testar
scan, evento, quarentena, recuperação e auditoria separadamente. Não executar
malware real nem armazenar amostras no repositório.

## Android continua portátil

Não mover APIs Windows para os crates partilhados. Serviço, IPC e eventual driver
ficam em adaptadores Windows. Android continua com Storage Access Framework/JNI
e URIs autorizados pelo utilizador; não prometer varrimento de outras apps nem
replicar privilégios Windows. Verificação de pacotes e schemas de resultados
podem ser partilhados; mecanismos de acesso e monitorização são da plataforma.

## Referências oficiais consultadas

- [Change journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [ReadDirectoryChangesW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw)
- [File system filter drivers](https://learn.microsoft.com/en-us/windows-hardware/drivers/ifs/about-file-system-filter-drivers)
- [Named pipe security](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights)
- [UAC](https://learn.microsoft.com/en-us/windows-server/security/user-account-control/how-user-account-control-works)
- [Driver signing](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/windows-driver-signing-tutorial)
- [EICAR](https://www.eicar.org/download-anti-malware-testfile/)
