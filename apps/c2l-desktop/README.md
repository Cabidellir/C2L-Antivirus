# C2L Antivirus — desktop 0.4.0

O modo principal passa a **Antimalware · ClamAV**, para análise manual de ficheiros
ou pastas com bases oficiais. SHA-256/CSV permanece como modo de diagnóstico.
Não há proteção em tempo real, bloqueio ou quarentena nesta versão.

## Preparar e testar

1. No Windows x64, execute `scripts/setup-clamav.ps1` na raiz do repositório.
   Instala o pacote oficial 1.5.4 no perfil local, verifica o SHA-256 fixado e
   atualiza as bases. Não pede elevação nem muda o antivírus existente.
2. Abra o desktop e escolha **Analisar → Antimalware · ClamAV**.
3. Use **Atualizar bases oficiais**, selecione um ficheiro/pasta e inicie a análise.
4. Confira o resumo: deteções, erros, cancelamento e motor/base utilizados.

O instalador é separado do executável; o artefacto da GUI não contém motor nem
bases. Nesta máquina de desenvolvimento já foram instalados. A atualização é
manual e verifica as bases com FreshClam; não há agendamento ou atualização do
binário ClamAV nesta versão.

Resultados ClamAV guardam apenas deteções/avisos, além do total de ficheiros do
resumo do motor. Não existe um hash ou registo individual para cada ficheiro sem
alerta. Os logs `.clamav.log` e `.clamav.stderr` ficam ao lado do SQLite.

Limites fixados: 100 MiB por ficheiro, 400 MiB de conteúdo por arquivo, recursão
interna 30. Limites excedidos e conteúdo encriptado são classificados como
cobertura incompleta, não como prova de malware. Links simbólicos não são seguidos;
pastas do motor e dos relatórios são excluídas e esse âmbito é indicado na GUI.
Outros limites internos do ClamAV continuam aplicáveis. A contagem durante a
análise é de alertas/avisos; não é percentagem de ficheiros processados.

Cancelar termina apenas o processo filho ClamAV desta análise. Resultados já
recebidos são conservados e o estado fica parcial. Relatórios limitados a 2 GiB,
log principal a 256 MiB e diagnóstico a 16 MiB (verificados durante execução).
Uma falha do motor nunca é convertida num resultado sem ameaças.

## Validação

`cargo test --manifest-path apps/c2l-desktop/Cargo.toml --locked` executa a suite
habitual. Com motor/bases locais, executar também:

```powershell
cargo test --manifest-path apps/c2l-desktop/Cargo.toml --locked real_engine_clean_limits_cancellation_and_eicar_outcome -- --ignored --nocapture
```

O teste imprime o resultado EICAR separadamente: pode ser impedido pela proteção
residente. Nesse caso, apenas a propagação do erro é validada; não se declara uma
deteção EICAR aprovada. Nunca desative a proteção residente para esse teste.

Consultar [benchmark e prioridades](../../docs/MARKET-BENCHMARK.md) e
[estado do projeto](../../docs/PROJECT-STATUS.md).

---

## Histórico das versões anteriores (0.1–0.3)

# C2L Antivirus — interface Windows (protótipo)

O C2L Antivirus tem agora uma primeira interface gráfica construída com
**Rust + egui/eframe**. A interface executa o mesmo motor de hashing e de
comparação de indicadores utilizado no CLI e partilhado com o Android.

## Funcionalidades existentes

- Painel principal com estatísticas **da última análise**.
- Seleção de um ficheiro ou uma pasta através de uma caixa de diálogo.
- Análise de ficheiro SHA-256 e análise recursiva de uma pasta (limites de 1 000, 10 000 ou 50 000
  entradas, ou sem limite de contagem; sem seguir links simbólicos).
- Seleção opcional de uma base de indicadores locais CSV, com comparação
  exata de SHA-256. O formato é documentado em
  [ANDROID-READINESS.md](../../docs/ANDROID-READINESS.md).
- Resultados com hashes, tamanho, erros e correspondências.
- Interface disponível enquanto a análise decorre (thread de trabalho).

**Atenção: protótipo, não um antivírus de proteção ativa.** Não existe
proteção em tempo real, bloqueio de ameaças, quarentena, limpeza, atualizações
automáticas nem garantia de segurança para ficheiros sem correspondência.
Os resultados são guardados localmente em lotes de 128 entradas durante a análise. Uma falha de gravação é apresentada na página Resultados.

## Testar sem programar

1. Abrir o separador [Actions](https://github.com/Cabidellir/C2L-Antivirus/actions)
   e selecionar **Windows Desktop GUI**.
2. Abrir uma execução concluída com sucesso.
3. Descarregar o artefacto **c2l-antivirus-windows-prototype**.
4. Extrair o ZIP e executar `c2l-desktop.exe` num computador Windows de testes.

O binário não está assinado digitalmente e pode provocar alertas do Windows.
Verifique a origem e não contorne avisos de segurança se não confiar no
programa.

## Compilar localmente

No terminal PowerShell, na raiz do repositório:

```powershell
cargo run --manifest-path apps/c2l-desktop/Cargo.toml
```

Para gerar o executável otimizado:

```powershell
cargo build --manifest-path apps/c2l-desktop/Cargo.toml --release
```

O executável ficará em
`apps/c2l-desktop/target/release/c2l-desktop.exe`.

## Teste com ficheiro inofensivo

1. No ecrã **Analisar**, escolha
   `tests/fixtures/c2l-demo.txt`.
2. Escolha a base `tests/fixtures/demo-indicators.csv`.
3. Inicie a análise.
4. Deverá aparecer **Correspondência: C2L_DEMO_ONLY**.

A correspondência é intencional e refere-se a um ficheiro de demonstração
que contém apenas `abc`. Não é malware real.

## Próximas entregas propostas

1. Retoma da análise, medições em volumes reais, cache invalidável e orçamentos de CPU/I/O.
2. Indicadores autênticos com assinatura/verificação de origem, atualização
   e mecanismos de revogação.
3. Testes de robustez, de desempenho e de falsos positivos.
4. Acessibilidade, seleção de volumes e visualização de cobertura.
5. Arquitetura de monitorização em tempo real no Windows (processo/serviço
   separado, privilégios mínimos, revisão de segurança e testes).
6. Quarentena apenas após definição e teste de recuperação, integridade e
   auditoria.

O núcleo do scanner permanece independente da camada gráfica para continuar
a servir o Windows CLI e a aplicação Android.

## Paleta e teste de aceitação desta entrega

- `#00BFFF`: marca/superfície principal e contornos dos cartões.
- `#FFD700`: contorno da marca e destaques no tema escuro.
- `#663399`: botões (texto branco) e tipografia de destaque no tema claro.
- Texto pequeno usa pares com contraste mínimo 4,5:1; a marca roxa de 30 pt
  sobre azul usa o limiar 3:1 para texto grande. Amarelo não é texto sobre branco.

Em **Definições → Aparência**, escolher Claro/Escuro, fechar normalmente e
reabrir: o tema deve manter-se no mesmo perfil Windows. A preferência é guardada pelo eframe. Os relatórios são guardados separadamente,
incluindo o alvo, o caminho do CSV escolhido e os erros de leitura.
A persistência usa o armazenamento local do eframe; falhas de escrita ou saída
forçada podem impedir a gravação. Não exige permissões de administrador.

Verificar manualmente as quatro páginas a 780×540, 1100×760 e maximizado,
incluindo escala Windows 125%/150%: navegação legível, controlos acessíveis por
scroll, caminhos longos com quebra e avisos de funcionalidades ausentes.
Testar troca ficheiro/pasta (limpa o alvo), cancelamento da caixa de seleção,
análise sem CSV, com CSV demo e com CSV inválido. Sem CSV deve indicar **Não
verificado**, nunca “seguro”. Durante análise a GUI deve continuar a responder.
Testes egui sem janela cobrem a construção das páginas, não substituem esta
inspeção manual de pixels, foco e diálogos nativos.

Consultar [arquitetura e marcos de segurança](../../docs/WINDOWS-SECURITY-ROADMAP.md)
para assinaturas autenticadas, serviço Windows, quarentena, EICAR e distribuição.

## Histórico de diagnóstico

A versão 0.3.0 grava SQLite v3 em `%LOCALAPPDATA%/C2L-Antivirus/reports`,
com transações de 128 registos. Cancelamento normal confirma também o lote final.
Uma saída abrupta conserva os lotes confirmados, podendo perder até 127 registos
pendentes. Ao abrir um relatório local reconhecido, o SQLite recupera o journal
quando necessário. O estado permanece **Interrompida (cobertura parcial)**;
recuperar o relatório não retoma a análise. Não copie apenas o `.sqlite` de uma
análise ativa: o journal pode ser necessário para recuperar a transação.

A interface consulta 100 resultados de cada vez no disco e guarda só essa página.
O resumo e as causas são agregados no banco, sem carregar todos os registos.
Ao reabrir, recupera o último relatório. **Resultados → Abrir relatório guardado…**
aceita SQLite e JSON legado v1/v2 (este último continua limitado a 128 MiB/50 000
registos). O histórico antigo não é convertido nem apagado.

Falhas na base CSV antes do início não criam um relatório; são apresentadas na GUI.
Falhas de leitura são registos. Falhas de armazenamento interrompem a análise,
mostram erro e preservam o que foi confirmado, sem afirmar conclusão. O relatório
ativo e os seus ficheiros auxiliares SQLite são omitidos da análise e contados
explicitamente. Não há exclusões automáticas por acesso negado.

Os dados ficam no perfil local, sem envio para servidores. Não são encriptados
nem assinados; herdam as permissões da pasta do perfil e são dados de diagnóstico,
não prova de auditoria contra adulteração. Conservação manual: relatórios não
são eliminados automaticamente. O histórico não concede permissões nem muda o motor de deteção por si só.

Teste: analisar uma pasta, verificar o caminho guardado, fechar e reabrir; os
contadores e mensagens devem manter-se. Abrir um relatório anterior não altera
o alvo da próxima análise. A falha de uma análise nova não deve apresentar os
resultados da anterior como se fossem novos.

## Análises maiores — desktop 0.3.0

Em **Analisar → Uma pasta**, escolher **Sem limite de entradas** para ultrapassar
50 000. O valor por defeito continua em 10 000. Não significa recursos infinitos:
o banco tem orçamento de 2 GiB (ficheiros temporários de journal podem ocupar
espaço adicional), cache SQLite de 4 MiB por ligação, registos até 1 MiB e árvore
até 256 níveis. Falta de espaço interrompe com cobertura parcial. O limite é
por análise, não uma quota total do histórico; não existe remoção automática.

O progresso conta entradas processadas, podendo incluir até 127 ainda pendentes
na transação. Não há percentagem sem conhecer o total. **Cancelar análise** é
cooperativo entre blocos de hashing de 64 KiB e durante a enumeração; uma chamada
de I/O bloqueada tem de retornar. Não há garantia de cancelamento imediato.

Os diagnósticos incluem categoria, código do SO e orientação. Hashes são
rejeitados quando se deteta alteração de tamanho/data durante leitura; não é
uma fotografia atómica nem deteta todas as mutações com metadados preservados.

### Validação de escala

O teste opcional cria e analisa 50 129 ficheiros reais de três bytes, consulta
uma página após a entrada 50 000 e confirma ausência de truncagem/erros:

```powershell
cargo test --manifest-path apps/c2l-desktop/Cargo.toml --locked real_scan_above_fifty_thousand -- --ignored --nocapture
```

A suite habitual valida 60 001 registos sintéticos, recuperação após saída de
processo sem destrutores, falta de espaço simulada via quota SQLite, cancelamento,
limites, filtros e compatibilidade JSON. Um corpus de ficheiros pequenos não
representa o desempenho de um disco real nem mede eficácia de deteção.


## Desktop 0.3.1 — diagnóstico e reanálise de erros

Os códigos Windows 362 (fornecedor de nuvem indisponível) e 1920 (inacessível ao
sistema, causa concreta ainda desconhecida) têm categorias e orientação próprias.
Relatórios antigos com código estruturado são interpretados na apresentação,
sem atualizar os seus ficheiros; mensagens sem código não são classificadas por
suposição. A interface explica esta interpretação.

Em Resultados, **Reanalisar caminhos com erro** fica disponível para relatórios
SQLite com erros. Cria um novo histórico ligado ao original, lê apenas aqueles
caminhos e não percorre pastas. Um caminho que seja pasta continua com erro;
para explorar os seus filhos é necessário selecionar a pasta numa nova análise.
Relatórios com estado interrompido são rejeitados nesta operação; cancelamentos
normais com lotes confirmados podem ser reanalisados. Isto não retoma uma análise
nem atualiza os contadores da execução original.

A reanálise usa a base no caminho registado no relatório, se existir; o conteúdo
dessa base pode ter mudado e continua não autenticado. Se não puder ser carregada,
a operação falha explicitamente. Sem base, continua a mostrar não verificado.
Ficheiros de nuvem podem necessitar de sincronização/transferência pelo fornecedor
quando lidos. O C2L não inicia aplicações, altera configurações, termina processos,
concede permissões, cria exclusões por erro ou executa remediação.

Validação: regressão dos dois códigos, preservação byte a byte do relatório de
origem em reanálise/cancelamento, um ficheiro que voltou a estar acessível, um
caminho ausente e uma pasta que não deve ser percorrida; paginação e histórico
continuam cobertos pela suite desktop.
