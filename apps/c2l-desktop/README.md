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
