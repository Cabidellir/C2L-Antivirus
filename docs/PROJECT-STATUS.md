# Avaliação do projeto após o desktop 0.3.1

Estado: protótipo funcional de scanner manual, em fase de engenharia e testes.
Não existe ainda evidência para o apresentar como proteção antivírus de produção.
Esta avaliação baseia-se no código e nas validações desta branch, não numa
percentagem arbitrária de conclusão ou em resultados de eficácia não medidos.

| Área | Evidência atual | Lacuna principal |
|---|---|---|
| Interface Windows | Rust/egui, temas, seleção, progresso por contagem, cancelamento, páginas e histórico local | Testes manuais de DPI/acessibilidade e usabilidade continuada |
| Scanner | SHA-256, árvore de pastas, erros individuais; emissão e gravação por lotes; contagem opcionalmente ilimitada | Retoma, cache e orçamento de tempo/I/O; testes em volumes representativos |
| Deteção | Correspondência exata com CSV local | Não autentica indicadores; não identifica variantes por conteúdo/comportamento; sem eficácia medida |
| Diagnóstico | SQLite incremental, leitura JSON legado, cobertura parcial, mensagens, categorias e códigos de erro | Comparação entre execuções e auditoria autenticada |
| Segurança Windows | GUI sem elevação, sem remediação | Adaptador Windows, serviço e eventos são estruturas ainda sem implementação |
| Quarentena/risco | Crates previstos | Não implementados; falta recuperação e validação contra perda de dados |
| Android | Núcleo partilhado e JNI; testes no host | Testes em dispositivo, restrições da plataforma e ciclo de distribuição |
| Entrega | Testes, Clippy, builds e workflows | Assinatura de código, atualizações autenticadas, política de lançamento e rollback |

## Ordem recomendada das próximas entregas

1. **Scanner confiável e escalável.** Banco local e páginas em disco implementados.
   Falta retoma da análise após interrupção (só recuperamos relatórios), seleção de volumes,
   cache invalidável e orçamentos de recursos. Confirmar estabilidade de identidade
   e conteúdo durante leitura. Medir CPU/RAM/throughput e latência de cancelamento
   em corpora representativos; a opção sem limite de contagem é experimental e
   mantém orçamento de 2 GiB por banco e o limite de profundidade.
2. **Fechar o ciclo dos erros reais.** Quatro relatórios já revistos, incluindo
   991/9 e 49 983/17; os erros observados são acesso negado e ficheiros em uso.
   Causas estruturadas e regressão de partilha Windows implementadas na 0.2.1.
   Falta comparação entre execuções com cobertura equivalente. Não silenciar
   erros nem ampliar permissões para melhorar os números.
3. **Base de deteção confiável.** Definir proveniência/licenciamento, assinar
   manifestos/pacotes, rejeitar corrupção/replay/downgrade, preservar última base
   válida e testar revogação. Mostrar versão e idade da base na GUI.
4. **Qualidade de deteção.** Corpus benigno e testes EICAR controlados, métricas
   de falsos positivos/negativos, depois heurísticas explicáveis. Um hash sem
   correspondência continua a não demonstrar segurança.
5. **Resposta segura.** Serviço mínimo, IPC autenticado/autorizado, quarentena
   transacional, restauração e testes de falha/crash. Só depois remediação em
   dados reais com consentimento específico.
6. **Monitorização e distribuição.** Observação Windows com tratamento de eventos
   perdidos, serviço saudável e cobertura visível; avaliar minifilter apenas para
   necessidade comprovada de bloqueio pré-acesso. Assinar binários/instalador e
   criar atualização verificável com rollback antes de distribuição alargada.

Os critérios técnicos detalhados estão em [WINDOWS-SECURITY-ROADMAP.md](WINDOWS-SECURITY-ROADMAP.md).
O histórico é evidência para correções controladas e testadas; ainda não existe
um mecanismo de autoajuste de segurança. Os próximos marcos devem ser aceites
pelos testes e medições, não apenas por novas opções na interface.


Entrega 0.3.0: emissão individual e SQLite em lotes de 128; a GUI conserva uma
página de 100 resultados. Recuperação de lotes confirmados após falha, sem retoma
automática; histórico antigo preservado. Relatório ativo omitido com contagem visível.
Verifica tamanho e data de modificação no mesmo identificador aberto antes/depois
do hash e descarta resultados quando deteta alterações. Esta verificação é de
melhor esforço: não é uma fotografia atómica, nem deteta todas as reescritas do
mesmo tamanho com metadados preservados. Retoma, cache e identidade robusta
continuam pendentes.


Entrega 0.3.1: categorias próprias para códigos Windows 362/1920, interpretação
visível dos códigos antigos sem alterar o histórico e reanálise explícita dos
caminhos com erro num relatório separado. Teste real do utilizador: 772 108
ficheiros analisados, 2 506 erros, sem base de indicadores. O próximo marco de
deteção é autenticar as bases; reanalisar erros não substitui esse trabalho.
