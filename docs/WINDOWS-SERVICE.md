# Serviço Windows experimental — 0.7.0

## O que esta entrega faz

O mesmo executável tem uma entrada SCM `--service`, em processo independente da
GUI. Um adaptador Windows trata início, Stop/Shutdown e estado do serviço.
O worker observa uma pasta local configurada, executa ClamAV com bases oficiais
e guarda relatórios SQLite. O serviço pode iniciar com o Windows e continuar
sem a interface, depois de instalado pelo administrador.

Isto é análise contínua **pós-alteração**, não bloqueio antes da execução, proteção
integral do computador nem integração com Windows Security Center. Nenhum ficheiro
é removido pelo serviço. A quarentena continua manual, pelo perfil do utilizador.

## Limites explícitos

- Uma pasta por instalação. Análise inicial, agrupamento de alterações, mínimo
  de 30 segundos entre inícios e reconciliação periódica a cada 15 minutos.
  Alterações de ficheiros são agrupadas em lotes deduplicados de até 128 caminhos,
  com limite adicional do tamanho da linha de comando. Pastas, remoções, eventos
  ambíguos, perdas ou lotes acima do limite exigem reconciliação integral.
  Não há garantia de latência de deteção.
- Fila de eventos limitada a 1024. Overflow obriga a nova análise integral;
  falha do observador termina com erro, permitindo recuperação pelo SCM.
- Leituras negadas, ficheiros demasiado grandes, conteúdo cifrado e falhas do
  motor mantêm estado degradado; não geram exclusões automáticas.
- Avisos pela área de notificações com a GUI fechada requerem agente por
  utilizador, sessão iniciada e permissões de leitura dos dados do serviço.
  O Windows pode silenciá-los; os relatórios continuam a ser a fonte de auditoria.
- Bases do serviço separadas das bases da GUI. Atualização automática a cada
  seis horas, após a primeira tentativa de análise. Bases com mais de sete
  dias continuam a causar aviso/estado degradado.
- Relatórios não são apagados automaticamente. Acima de 512 MiB já existentes
  ou 1500 entradas a próxima análise é suspensa. Uma análise em curso pode
  exceder esse limiar; aplicam-se também os limites do adaptador ClamAV.
- Cancelamento do motor é cooperativo, com terminação do processo e Job Object.
  O subprocesso de identificação do motor tem timeout de dez segundos.
- Não foram implementados minifilter, USN journal, autoproteção nem atualização
  automática do executável do serviço. Binário experimental ainda sem assinatura Authenticode.

## Instalar (administrador / UAC)

Escolher uma pasta dedicada; para o primeiro teste, preferir uma pasta de testes
em `C:\Users\Public`. Não indicar a raiz do disco ou a pasta do Windows.

1. Atualizar as bases oficiais na GUI.
2. Identificar a geração ativa no diretório de bases do motor. A configuração
   do serviço recebe uma cópia de `main`, `daily` e `bytecode` dessa geração.
3. Abrir PowerShell **como administrador** e executar, ajustando os caminhos:

```powershell
.\scripts\install-monitor-service.ps1 `
  -Executable 'C:\caminho\c2l-desktop.exe' `
  -WatchPath 'C:\Users\Public\C2L-Testes' `
  -Database 'C:\Users\utilizador\AppData\Local\C2L-Antivirus\engine\generation-...' `
  -EngineArchive 'C:\Users\utilizador\AppData\Local\C2L-Antivirus\engine\clamav-1.5.4.win.x64.zip' `
  -GrantReadAccess
```

`-GrantReadAccess` acrescenta apenas leitura/execução ao SID específico do serviço
na pasta escolhida e nos filhos que herdem permissões. Não altera permissões dos
volumes nem contorna ACLs protegidas. Omitir a opção para conservar as ACLs e
usar só acessos já existentes. Pastas ancestrais inacessíveis podem impedir o
arranque; escolher outra pasta ou rever os acessos de forma específica.

Conta: `NT AUTHORITY\LocalService`, com SID `NT SERVICE\C2LMonitor`. Configuração,
DLLs e executável ficam em `%ProgramFiles%\C2L-Monitor`, com escrita só para
administradores/SYSTEM; o serviço recebe leitura/execução. Dados em
`%ProgramData%\C2L-Monitor` permitem escrita ao SID do serviço e leitura ao
utilizador que instalou. Propriedade dos ficheiros fica em Administradores.
O arquivo oficial do motor é verificado por SHA-256 antes e depois da cópia.
Não há pipe nem endpoint que aceite comandos/caminhos da GUI.

Instalações existentes não são sobrescritas. Falhas deixam os artefactos para
inspeção e podem deixar um serviço registado mas parado; nunca tratar isso como
proteção ativa. `-RequireSigned` permite exigir assinatura de código válida.

## Verificar

Abrir Monitorização na GUI. O painel combina consulta SCM com heartbeat de até
15 segundos. Um estado antigo ou apenas SCM Running não confirma cobertura.
Confirmar análise inicial concluída, pasta e idade das bases. Criar um texto
inofensivo e verificar nova análise; fechar a GUI, repetir e reabrir. Testar
Stop/Start no gestor Serviços e, num ambiente de teste, reiniciar o Windows.

Testes automatizados locais cobrem estado desatualizado, falha de configuração,
alvos proibidos e o worker com ClamAV real: análise inicial, alteração com fixture
inofensiva oficial, deteção guardada, paragem e reinício com histórico. Esses testes
não equivalem ao teste de SCM, UAC, ACLs de LocalService ou arranque após reboot.
O teste real do serviço está em `scripts/test-monitor-service.ps1` e requer
Windows elevado, Python e ambiente descartável sem instalação C2LMonitor.

## Gestão

```powershell
Stop-Service C2LMonitor
Start-Service C2LMonitor
.\scripts\uninstall-monitor-service.ps1 -RemoveReadGrant
```

A remoção preserva binários, bases e relatórios. Não apagar diretórios do serviço
manualmente enquanto estiver ativo. As bases iniciais em Program Files são
preservadas como recuperação. As atualizações automáticas usam gerações em
ProgramData/C2L-Monitor/definitions, verificadas pelo motor oficial antes da
publicação transacional. Não se concede escrita ao serviço em Program Files.

## Referências de implementação

- [Microsoft: LocalService](https://learn.microsoft.com/en-us/windows/win32/services/localservice-account)
- [Microsoft: segurança dos serviços](https://learn.microsoft.com/en-us/windows/win32/services/service-security-and-access-rights)
- [windows-service: dispatcher](https://docs.rs/windows-service/0.8.1/windows_service/service_dispatcher/fn.start.html)

## Validação desta entrega

Em 20/10/2026, o worker foi executado com ClamAV real e a amostra inofensiva
oficial do fornecedor. A aceitação confirmou análise limpa, alteração posterior,
deteção persistida, paragem, reinício com histórico e cancelamento ativo.
São testes funcionais, não uma avaliação de eficácia comercial.

A aceitação elevada em 20/10/2026 confirmou instalação SCM sob LocalService,
análise inicial limpa, deteção da fixture sem GUI, paragem, reinício e histórico.
O serviço tem apenas leitura/execução nos binários e escrita nos próprios dados;
a propriedade dos ficheiros instalados é do grupo Administradores.
**Arranque após reboot continua pendente**; não foi reiniciado o computador.

## Próximas capacidades

Validar arranque após reinício do Windows e logon; ampliar testes de eficácia,
latência e consumo; avaliar bloqueio pré-execução como projeto separado.

## Ativação após aceitação

`activate-monitor-service.ps1` executa a aceitação numa pasta dedicada, preserva
os relatórios de teste num diretório separado e passa para `-WatchPath` apenas
após sucesso. Aceita os mesmos caminhos de motor/base/fixture do teste e
`-PythonExecutable` para indicar Python explicitamente. Não altera o executável.

Se o perfil tiver ACL privada, pode ser necessário `-TraversalPaths` com o
caminho exato do perfil. `grant-monitor-traversal.ps1` concede somente
ReadAttributes/Traverse nesse antecessor, **sem herança, listagem ou leitura de
conteúdo**. Recusa raízes de volume, UNC e caminhos fora dos antecessores do alvo.
As regras são registadas para remoção por `uninstall-monitor-service.ps1
-RemoveReadGrant`. Nenhuma regra autoriza escrita na pasta observada.

Em 22/10/2026, a instalação em Downloads mantinha SCM Running, heartbeat recente
e 231 análises concluídas. O relatório inspecionado apresentava limites de tamanho
e tempo, arquivo cifrado e falha de leitura, sem deteções classificadas pelo
adaptador. Isto confirma funcionamento contínuo, mas cobertura incompleta.
O teste `test-directory-dacl.ps1` confirmou concessão sem herança, preservação das
ACLs dos filhos e reposição das permissões originais, tanto com herança protegida
como não protegida. A comparação inclui todos os bytes das ACEs ordenadas,
proprietário, grupo e proteção de herança; o indicador AUTO_INHERITED normalizado
pelo Windows não é confundido com uma alteração das permissões.

## Análise incremental — 0.7.0

O serviço analisa os caminhos alterados num único processo do motor e conserva
eventos recebidos durante a análise para o lote seguinte. O arranque e a
reconciliação periódica continuam integrais. Falhas persistentes de leitura ou
limites aguardam a reconciliação periódica, em vez de repetir imediatamente
toda a pasta. Um lote limpo não apaga uma lacuna de cobertura anterior.
Os relatórios incrementais usam o modo `clamav-changes`; o alvo é a pasta
autorizada, mas a contagem refere-se apenas ao lote. A monitorização da GUI
continua a usar análises integrais nesta versão.

Validação 0.6.1: 40 testes passaram e oito integrações ficaram opt-in. A integração
real executada separadamente confirmou uma única amostra nova analisada no lote
incremental, deteção persistida, reinício com histórico e cancelamento ativo.
Clippy com avisos como erros e build release passaram. A atualização local do
serviço preservou ACLs, configuração, relatórios e o executável anterior; o hash
instalado corresponde ao pacote e o heartbeat foi confirmado após reiniciar.
A análise integral inicial após essa atualização ainda estava em curso na verificação.

## Bases automáticas e notificações — 0.7.0

`automatic_updates` é verdadeiro por omissão; o instalador aceita
`-DisableAutomaticUpdates` para testes isolados. O prazo de seis horas é guardado
antes de iniciar a atualização, impedindo ciclos de rede após reinícios. FreshClam
preserva o estado de espera imposto pelo fornecedor. As bases oficiais são
carregadas com `--official-db-only=yes` antes de publicar; versões diárias
inferiores à atual são recusadas. Cancelamento, falha de rede ou validação mantêm
a referência anterior. A publicação atual e anterior é transacional em SQLite.

O serviço não analisa e atualiza em simultâneo. O heartbeat continua ativo e os
eventos aguardam na fila limitada. O download tem limite de dez minutos e a
validação três minutos; a paragem cancela os subprocessos. Após sucesso pede-se
reconciliação integral. A retenção remove apenas gerações internas inativas,
conservando a ativa, a anterior publicada e o snapshot instalado. Isto não apaga
relatórios nem ficheiros do utilizador. Um candidato falhado é conservado até à
próxima manutenção; os logs das gerações eliminadas não são um arquivo permanente.

Ativar os avisos na sessão do utilizador, sem elevação:

```powershell
.\scripts\install-notifications.ps1
# Desativar apenas os avisos; a análise continua:
.\scripts\install-notifications.ps1 -Disable
```

O agente usa a API Windows Shell_NotifyIcon, consulta apenas o estado do serviço
e guarda o cursor de avisos no perfil do utilizador. É iniciado agora e pelo HKCU
Run no próximo logon. Um relatório só gera um aviso por perfil; novos relatórios
são agrupados com intervalo mínimo de quinze minutos. Uma deteção repetida numa
reanálise pode originar novo aviso após esse intervalo. Não são mostrados nomes
de ficheiros no aviso. Não há garantia de apresentação, entrega ou leitura pelo
utilizador; modo silencioso, políticas e sessão encerrada podem impedi-la. Abra
Monitorização para os detalhes e recuperação manual. O ícone de área de notificação
é informativo e não inicia quarentena nem comandos administrativos.

`c2l-desktop.exe --notification-test` submete um aviso claramente identificado como
teste inofensivo e termina após dez segundos. Executar antes de ativar o agente
normal, porque o perfil permite apenas uma instância. A API aceitar o aviso não
prova que foi visto. Não é um teste EICAR nem mede eficácia antivírus.

Referências: [FreshClam](https://docs.clamav.net/manual/Usage/SignatureManagement.html),
[notificações Windows](https://learn.microsoft.com/en-us/windows/win32/shell/notification-area).

Validação 0.7.0 em 23/10/2026: 44 testes normais passaram; nove integrações são
opt-in. Foram executadas separadamente a atualização real numa raiz isolada
(base 28161, candidato corrompido recusado sem mudar a ativa) e a integração de
análise incremental/deteção/reinício/cancelamento. Clippy, release e integridade
do pacote passaram. A API Windows aceitou o aviso de teste; a apresentação visual
não foi confirmada. O agente foi ativado no perfil do utilizador e configurado
para logon. A substituição do serviço instalado ficou pendente porque o pedido
UAC foi cancelado; isto não é uma validação sob LocalService da versão 0.7.0.

`scripts/test-service-updates.ps1` executa um worker isolado com atualização
automática real e verifica publicação, reconciliação posterior e ausência de
atualizações duplicadas no intervalo. Preserva a pasta temporária para auditoria;
não instala serviços nem altera a pasta observada pelo serviço em utilização.

A aceitação automática isolada confirmou duas análises sem erros, publicação
automática da base 28161 e análise posterior com essa base. A instalação
LocalService existente continuou na 0.6.1 enquanto o UAC estava pendente/cancelado.

Em nova tentativa autorizada em 23/10/2026, o UAC foi aceite e o serviço 0.7.0
foi instalado com hash correspondente ao pacote, ACLs preservadas e cópia do
executável anterior. Sob LocalService, a primeira atualização automática validou
e publicou a base diária 28162 (cerca de duas horas de idade) e iniciou a
reconciliação com a nova base. Heartbeat recente e agente de notificações ativo
foram confirmados. Reboot/logon e apresentação visual do aviso continuam por validar.
