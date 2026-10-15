# C2L Antivirus — interface Windows (protótipo)

O C2L Antivirus tem agora uma primeira interface gráfica construída com
**Rust + egui/eframe**. A interface executa o mesmo motor de hashing e de
comparação de indicadores utilizado no CLI e partilhado com o Android.

## Funcionalidades existentes

- Painel principal com estatísticas **da última análise**.
- Seleção de um ficheiro ou uma pasta através de uma caixa de diálogo.
- Análise de ficheiro SHA-256 e análise recursiva de uma pasta (limite de 1 000
  entradas; sem seguir links simbólicos).
- Seleção opcional de uma base de indicadores locais CSV, com comparação
  exata de SHA-256. O formato é documentado em
  [ANDROID-READINESS.md](../../docs/ANDROID-READINESS.md).
- Resultados com hashes, tamanho, erros e correspondências.
- Interface disponível enquanto a análise decorre (thread de trabalho).

**Atenção: protótipo, não um antivírus de proteção ativa.** Não existe
proteção em tempo real, bloqueio de ameaças, quarentena, limpeza, atualizações
automáticas nem garantia de segurança para ficheiros sem correspondência.
Os resultados mantêm-se apenas na memória enquanto a aplicação estiver aberta.

## Testar sem programar

1. Abrir o separador [Actions](https://github.com/Cabidellir/C2L-Antivirus/actions)
   e selecionar **Windows Desktop GUI**.
2. Abrir uma execução concluída com sucesso.
3. Descarregar o artefacto **c2l-antivirus-windows-prototype**.
4. Extrair o ZIP e executar \`c2l-desktop.exe\` num computador Windows de testes.

O binário não está assinado digitalmente e pode provocar alertas do Windows.
Verifique a origem e não contorne avisos de segurança se não confiar no
programa.

## Compilar localmente

No terminal PowerShell, na raiz do repositório:

\`\`\`powershell
cargo run --manifest-path apps/c2l-desktop/Cargo.toml
\`\`\`

Para gerar o executável otimizado:

\`\`\`powershell
cargo build --manifest-path apps/c2l-desktop/Cargo.toml --release
\`\`\`

O executável ficará em
\`apps/c2l-desktop/target/release/c2l-desktop.exe\`.

## Teste com ficheiro inofensivo

1. No ecrã **Analisar**, escolha
   \`tests/fixtures/c2l-demo.txt\`.
2. Escolha a base \`tests/fixtures/demo-indicators.csv\`.
3. Inicie a análise.
4. Deverá aparecer **Correspondência: C2L_DEMO_ONLY**.

A correspondência é intencional e refere-se a um ficheiro de demonstração
que contém apenas \`abc\`. Não é malware real.

## Próximas entregas propostas

1. Persistência local segura dos resultados e exportação JSON.
2. Indicadores autênticos com assinatura/verificação de origem, atualização
   e mecanismos de revogação.
3. Testes de robustez, de desempenho e de falsos positivos.
4. Melhorias da interface: progresso, cancelamento, acessibilidade e
   opções de análise configuráveis.
5. Arquitetura de monitorização em tempo real no Windows (processo/serviço
   separado, privilégios mínimos, revisão de segurança e testes).
6. Quarentena apenas após definição e teste de recuperação, integridade e
   auditoria.

O núcleo do scanner permanece independente da camada gráfica para continuar
a servir o Windows CLI e a aplicação Android.
