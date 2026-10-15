# C2L Antivirus — interface Windows (protótipo)

O C2L Antivirus tem agora uma primeira interface gráfica construída com
**Rust + egui/eframe**. A interface executa o mesmo motor de hashing e de
comparação de indicadores utilizado no CLI e partilhado com o Android.

## Funcionalidades existentes

- Painel principal com estatísticas **da última análise**.
- Seleção de um ficheiro ou uma pasta através de uma caixa de diálogo.
- Análise de ficheiro SHA-256 e análise recursiva de uma pasta (limite de 1 000, 10 000 ou 50 000
  entradas configurável; sem seguir links simbólicos).
- Seleção opcional de uma base de indicadores locais CSV, com comparação
  exata de SHA-256. O formato é documentado em
  [ANDROID-READINESS.md](../../docs/ANDROID-READINESS.md).
- Resultados com hashes, tamanho, erros e correspondências.
- Interface disponível enquanto a análise decorre (thread de trabalho).

**Atenção: protótipo, não um antivírus de proteção ativa.** Não existe
proteção em tempo real, bloqueio de ameaças, quarentena, limpeza, atualizações
automáticas nem garantia de segurança para ficheiros sem correspondência.
Os resultados são guardados localmente no final de cada análise. Uma falha de gravação é apresentada na página Resultados.

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

1. Processamento/armazenamento por lotes para volumes inteiros e diagnóstico estruturado dos erros.
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

Cada análise terminada (incluindo falhas de leitura ou CSV inválido) grava um
JSON versionado em `%LOCALAPPDATA%/C2L-Antivirus/reports`. A gravação ocorre na
thread de trabalho, com ficheiro temporário, sincronização e publicação final.
A interface só confirma a gravação depois do sucesso. Interrupção abrupta antes
da conclusão pode deixar a análise sem relatório; ficheiros `.partial` são
ignorados. Não existe histórico retroativo de versões anteriores.

Ao reabrir, o último relatório é recuperado. **Resultados → Abrir relatório
guardado…** permite escolher anteriores. **Apenas erros** é selecionado por
defeito se a análise tiver erros; **Todos os resultados** mostra os restantes.
Erros e limite atingido produzem o estado **Análise parcial**. Caminhos,
mensagens originais de erro, contadores, hash quando disponível, instante UTC
em segundos Unix e versão da aplicação ficam no JSON. Relatórios inválidos,
com contadores inconsistentes ou maiores que 128 MiB são rejeitados.

Os dados ficam no perfil local, sem envio para servidores. Não são encriptados
nem assinados; herdam as permissões da pasta do perfil e são dados de diagnóstico,
não prova de auditoria contra adulteração. Conservação manual: relatórios não
são eliminados automaticamente. O histórico não concede permissões, não cria
exclusões e não muda o motor de deteção por si só.

Teste: analisar uma pasta, verificar o caminho guardado, fechar e reabrir; os
contadores e mensagens devem manter-se. Abrir um relatório anterior não altera
o alvo da próxima análise. A falha de uma análise nova não deve apresentar os
resultados da anterior como se fossem novos.

## Análises maiores — desktop 0.2.0

Em **Analisar → Uma pasta**, escolher **1 000**, **10 000** (por defeito) ou
**50 000** entradas. O limite inclui ficheiros e erros. A interface apresenta
contagem de entradas processadas; não mostra uma percentagem fictícia porque
não conhece previamente o total de ficheiros da árvore.

**Cancelar análise** pede paragem cooperativa entre blocos de 64 KiB e durante
a enumeração. Uma operação de I/O bloqueada precisa de retornar antes da
paragem: não há garantia de cancelamento imediato. Os resultados concluídos
antes da paragem são guardados com `cancelled: true`, mantendo cobertura parcial.

Resultados são mostrados em páginas de 100. O JSON v2 inclui o limite usado e
estado de cancelamento, e o leitor mantém compatibilidade com relatórios v1.
A gravação é feita em fluxo com orçamento de 128 MiB; falha de gravação é visível.
A análise ainda conserva os registos em memória, até 50 000. A enumeração usa
até 256 níveis de pastas; ultrapassagens são erros explícitos. Não é uma análise
ilimitada de todos os discos: gravação e consulta por lotes são o próximo passo.
O limite antigo do CLI permanece inalterado; estas opções são do desktop.

Validação: teste com 1 505 ficheiros de 3 bytes, limite de 1 000, limite exato,
cancelamento após três entradas e cancelamento durante hashing. Histórico com
50 000 registos sintéticos gravado/reaberto; paginação validada no egui. Isso
não constitui benchmark de 50 000 ficheiros reais nem de um disco inteiro.
