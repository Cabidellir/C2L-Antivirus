# Serviço Windows experimental — 0.6.0

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
  Cada análise percorre a pasta toda: apropriado apenas para pastas pequenas
  nesta etapa. Não há garantia de latência de deteção.
- Fila de eventos limitada a 1024. Overflow obriga a nova análise integral;
  falha do observador termina com erro, permitindo recuperação pelo SCM.
- Leituras negadas, ficheiros demasiado grandes, conteúdo cifrado e falhas do
  motor mantêm estado degradado; não geram exclusões automáticas.
- Sem alertas toast com a GUI fechada. As deteções são persistidas e aparecem
  no painel Monitorização, com acesso ao último relatório que teve deteções.
- Snapshot separado das bases. Atualizar as bases na GUI não atualiza este
  snapshot. Bases com mais de sete dias causam aviso/estado degradado.
- Relatórios não são apagados automaticamente. Acima de 512 MiB já existentes
  ou 1500 entradas a próxima análise é suspensa. Uma análise em curso pode
  exceder esse limiar; aplicam-se também os limites do adaptador ClamAV.
- Cancelamento do motor é cooperativo, com terminação do processo e Job Object.
  O subprocesso de identificação do motor tem timeout de dez segundos.
- Não foram implementados minifilter, USN journal, autoproteção nem atualização
  automática do serviço. Binário experimental ainda sem assinatura Authenticode.

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
manualmente enquanto estiver ativo. Antes de atualizar a base do serviço, parar
o serviço, preservar a cópia anterior, copiar uma geração oficial completa para
`database` mantendo as ACLs e reiniciar; confirmar a nova versão e uma análise
concluída. Se falhar, restaurar a cópia anterior. A publicação automática e
transacional de snapshots para o serviço é um marco seguinte.

## Referências de implementação

- [Microsoft: LocalService](https://learn.microsoft.com/en-us/windows/win32/services/localservice-account)
- [Microsoft: segurança dos serviços](https://learn.microsoft.com/en-us/windows/win32/services/service-security-and-access-rights)
- [windows-service: dispatcher](https://docs.rs/windows-service/0.8.1/windows_service/service_dispatcher/fn.start.html)

## Validação desta entrega

Em 20/10/2026, o worker foi executado com ClamAV real e a amostra inofensiva
oficial do fornecedor. A aceitação confirmou análise limpa, alteração posterior,
deteção persistida, paragem, reinício com histórico e cancelamento ativo.
São testes funcionais, não uma avaliação de eficácia comercial.

O ambiente local não dispõe de token elevado: instalação SCM, ACLs efetivas sob
LocalService, recuperação automática pelo SCM e arranque após reboot ficam
**pendentes de aceitação num Windows elevado**. Não estão declarados aprovados.

## Próximas capacidades

Validar SCM/UAC e reinício real; integrar atualização autenticada do snapshot;
substituir reanálise integral por fila de ficheiros com reconciliação; adicionar
notificações por sessão; avaliar bloqueio pré-execução como projeto separado.
