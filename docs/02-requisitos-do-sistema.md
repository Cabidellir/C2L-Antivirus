# 02 — Requisitos do Sistema

**Projeto:** C2L Antivirus  
**Versão do documento:** 1.0  
**Data:** 08/10/2026  
**Status:** Aprovado  
**Documento:** 02 — Requisitos do Sistema

---

## 1. Objetivo

Este documento define os requisitos funcionais, não funcionais, de segurança e de evolução do C2L Antivirus.

O objetivo é estabelecer **o que o sistema deverá fazer e quais propriedades deverá possuir**, sem antecipar decisões de implementação que pertencem ao documento de arquitetura.

Os requisitos serão utilizados como referência para:

- definição da arquitetura;
- escolha das tecnologias;
- implementação;
- testes;
- validação das versões;
- avaliação de desempenho;
- evolução futura do produto.

---

## 2. Classificação dos requisitos

Cada requisito possui uma prioridade:

| Prioridade | Significado |
|---|---|
| **Obrigatório** | Necessário para a versão ou capacidade definida. |
| **Desejável** | Importante, mas pode ser implementado posteriormente. |
| **Futuro** | Previsto para versões posteriores. |
| **Fora do escopo** | Não faz parte dos objetivos atuais do projeto. |

A classificação poderá ser revista conforme o projeto evoluir.

---

# 3. Requisitos funcionais

## 3.1 Digitalização de ficheiros

### RF-001 — Seleção de ficheiros e diretórios
**Prioridade:** Obrigatório — V0.1

O sistema deverá permitir ao utilizador selecionar um ficheiro ou diretório para análise.

**Critérios de aceitação:**
- aceitar ficheiros individuais;
- aceitar diretórios;
- permitir análise recursiva de subdiretórios;
- informar os ficheiros analisados;
- tratar erros de acesso sem interromper toda a análise.

### RF-002 — Análise de ficheiros
**Prioridade:** Obrigatório — V0.1

O motor deverá analisar ficheiros e determinar se existem indicadores conhecidos de ameaça.

A análise deverá produzir um resultado classificável, mesmo quando não houver uma deteção positiva.

### RF-003 — Hash criptográfico
**Prioridade:** Obrigatório — V0.1

O sistema deverá calcular pelo menos um hash criptográfico seguro para cada ficheiro analisado.

O hash deverá poder ser utilizado para identificação e comparação com indicadores conhecidos.

**Implementação específica do algoritmo:** definida no documento de arquitetura.

### RF-004 — Base local de indicadores
**Prioridade:** Obrigatório — V0.1

O sistema deverá possuir uma base local de indicadores de ameaça.

Inicialmente, esta base poderá conter hashes e outros indicadores controlados pelo próprio projeto.

### RF-005 — Comparação com indicadores
**Prioridade:** Obrigatório — V0.1

O hash ou outros indicadores obtidos durante a análise deverão ser comparados com a base local.

Quando houver correspondência, o sistema deverá gerar uma deteção.

---

# 4. Classificação de risco

### RF-006 — Motor de risco
**Prioridade:** Obrigatório — V0.1

O sistema deverá transformar os resultados da análise em uma classificação de risco.

A classificação inicial deverá distinguir, no mínimo:

- **Seguro / sem evidências relevantes**
- **Suspeito**
- **Malicioso / deteção confirmada**

O modelo deverá permitir evolução futura para uma pontuação de risco mais sofisticada.

### RF-007 — Justificação da deteção
**Prioridade:** Obrigatório — V0.1

Sempre que possível, o sistema deverá informar quais evidências contribuíram para a classificação.

Exemplos:

- correspondência de hash;
- assinatura conhecida;
- indicador heurístico;
- reputação;
- comportamento.

O objetivo é evitar uma "caixa preta" de deteção sempre que tecnicamente possível.

---

# 5. Quarentena

### RF-008 — Isolamento de ficheiros
**Prioridade:** Obrigatório — V0.1

O sistema deverá permitir mover um ficheiro identificado como ameaça para uma área de quarentena controlada.

### RF-009 — Proteção da quarentena
**Prioridade:** Obrigatório — V0.1

Os ficheiros em quarentena não deverão permanecer em uma localização onde possam ser executados acidentalmente pelo utilizador ou por aplicações comuns.

### RF-010 — Restauração
**Prioridade:** Desejável — V0.2+

O sistema poderá permitir a restauração controlada de um ficheiro colocado em quarentena.

A restauração deverá exigir uma ação explícita do utilizador.

### RF-011 — Eliminação
**Prioridade:** Desejável — V0.2+

O sistema poderá permitir a eliminação definitiva de itens em quarentena.

---

# 6. Registo e observabilidade

### RF-012 — Registo de eventos
**Prioridade:** Obrigatório — V0.1

O sistema deverá registrar eventos relevantes, incluindo:

- início e fim de análises;
- ficheiros analisados;
- deteções;
- erros;
- operações de quarentena;
- alterações importantes de configuração.

### RF-013 — Identificação das análises
**Prioridade:** Obrigatório — V0.1

Cada execução de análise deverá possuir informações suficientes para reconstruir o resultado posteriormente.

### RF-014 — Relatório de análise
**Prioridade:** Obrigatório — V0.1

Ao finalizar uma análise, o sistema deverá apresentar pelo menos:

- quantidade de ficheiros analisados;
- quantidade de ficheiros com erro;
- quantidade de deteções;
- classificação final;
- tempo de execução.

---

# 7. Interface

### RF-015 — Interface de linha de comandos
**Prioridade:** Obrigatório — V0.1

O sistema deverá possuir uma interface CLI que permita executar as principais operações da primeira versão.

Exemplos:

- iniciar análise;
- indicar ficheiro ou diretório;
- visualizar resultado;
- consultar versão;
- consultar configuração básica.

A CLI também servirá como interface de testes e automação.

### RF-016 — Interface gráfica
**Prioridade:** Futuro — V0.7

O projeto deverá permitir a criação de uma interface gráfica para utilizadores finais.

A interface gráfica não deverá conter a lógica principal do motor de segurança.

---

# 8. Deteção avançada

### RF-017 — Assinaturas
**Prioridade:** Desejável — V0.2

O sistema deverá evoluir de uma simples comparação de hashes para mecanismos de assinatura capazes de identificar padrões conhecidos.

### RF-018 — Análise heurística
**Prioridade:** Desejável — V0.2

O sistema deverá possuir mecanismos heurísticos capazes de identificar características potencialmente maliciosas mesmo sem correspondência exata em uma base de hashes.

### RF-019 — Análise comportamental
**Prioridade:** Futuro — V0.5

O sistema deverá ser capaz de analisar comportamentos suspeitos de processos e aplicações.

Esta capacidade deverá ser implementada de acordo com os mecanismos de segurança disponíveis em cada sistema operativo.

---

# 9. Proteção em tempo real

### RF-020 — Monitorização em tempo real
**Prioridade:** Futuro — V0.4

O sistema deverá poder monitorizar eventos relevantes do sistema operativo e analisar ficheiros ou atividades potencialmente perigosas.

A implementação deverá utilizar componentes específicos de cada plataforma.

### RF-021 — Proteção contra alterações maliciosas
**Prioridade:** Futuro

Componentes críticos do antivírus deverão possuir mecanismos contra alteração, interrupção ou manipulação não autorizada.

---

# 10. Threat Intelligence e atualizações

### RF-022 — Atualização da base de indicadores
**Prioridade:** Futuro — V0.6

O sistema deverá permitir atualizar indicadores de ameaça sem exigir uma nova versão completa do programa.

### RF-023 — Integridade das atualizações
**Prioridade:** Futuro

As atualizações de definições e componentes deverão possuir mecanismos de autenticidade e integridade.

### RF-024 — Threat Intelligence remota
**Prioridade:** Futuro

O C2L poderá consultar serviços remotos de inteligência de ameaças para complementar a análise local.

Essa funcionalidade deverá respeitar requisitos de privacidade, disponibilidade e segurança.

---

# 11. Requisitos multiplataforma

### RF-025 — Núcleo independente da plataforma
**Prioridade:** Obrigatório — arquitetura

O núcleo lógico do C2L deverá ser projetado para minimizar dependências específicas de um único sistema operativo.

### RF-026 — Adaptadores de plataforma
**Prioridade:** Obrigatório — arquitetura

Funcionalidades dependentes do sistema operativo deverão ser isoladas através de componentes específicos da plataforma.

Plataformas previstas:

1. Windows — primeira plataforma;
2. Linux — segunda plataforma;
3. Android — evolução futura.

### RF-027 — Comportamento consistente
**Prioridade:** Desejável

O C2L deverá procurar manter conceitos e resultados consistentes entre plataformas, mesmo quando a implementação interna for diferente.

---

# 12. Requisitos não funcionais

## 12.1 Segurança

### RNF-001 — Princípio do menor privilégio
**Prioridade:** Obrigatório

Os componentes deverão executar com o menor nível de privilégio necessário para cumprir a sua função.

### RNF-002 — Separação de componentes
**Prioridade:** Obrigatório

Componentes com diferentes níveis de risco ou privilégio deverão ser separados quando tecnicamente justificável.

### RNF-003 — Integridade
**Prioridade:** Obrigatório

O sistema deverá possuir mecanismos para detectar corrupção ou alteração indevida de componentes críticos.

### RNF-004 — Fail-safe
**Prioridade:** Obrigatório

Falhas do motor de análise não deverão resultar, por padrão, em comportamentos que aumentem desnecessariamente o risco para o sistema.

---

## 12.2 Desempenho

### RNF-005 — Eficiência
**Prioridade:** Obrigatório

O motor deverá analisar grandes quantidades de ficheiros sem consumo desnecessário de recursos.

### RNF-006 — Medição de desempenho
**Prioridade:** Obrigatório

O projeto deverá possuir métricas que permitam avaliar:

- tempo por ficheiro;
- ficheiros por segundo;
- utilização de CPU;
- utilização de memória;
- impacto do armazenamento.

Não serão estabelecidos valores definitivos antes dos primeiros testes de referência.

---

## 12.3 Confiabilidade

### RNF-007 — Tolerância a erros
**Prioridade:** Obrigatório

Um erro individual de acesso, leitura ou análise não deverá interromper uma análise completa quando for possível continuar com segurança.

### RNF-008 — Resultados reproduzíveis
**Prioridade:** Obrigatório

Dadas as mesmas entradas, configuração e base de definições, o sistema deverá produzir resultados consistentes, salvo componentes explicitamente probabilísticos.

---

## 12.4 Manutenibilidade

### RNF-009 — Modularidade
**Prioridade:** Obrigatório

Os componentes deverão possuir responsabilidades bem definidas e baixo acoplamento.

### RNF-010 — Testabilidade
**Prioridade:** Obrigatório

Componentes críticos deverão poder ser testados de forma isolada.

### RNF-011 — Documentação técnica
**Prioridade:** Obrigatório

Decisões arquiteturais e comportamentos relevantes deverão ser documentados.

---

## 12.5 Portabilidade

### RNF-012 — Portabilidade do núcleo
**Prioridade:** Obrigatório

O núcleo deverá evitar dependências desnecessárias de APIs específicas do Windows.

### RNF-013 — Isolamento das APIs do sistema operativo
**Prioridade:** Obrigatório

Integrações específicas deverão permanecer nas camadas de plataforma.

---

## 12.6 Privacidade

### RNF-014 — Minimização de dados
**Prioridade:** Obrigatório

O C2L deverá coletar apenas os dados necessários para as funcionalidades implementadas.

### RNF-015 — Transparência
**Prioridade:** Obrigatório

Quando houver comunicação externa ou telemetria, o sistema deverá permitir identificar claramente:

- quais dados são enviados;
- para onde são enviados;
- por que são enviados.

### RNF-016 — Telemetria
**Prioridade:** Futuro

Qualquer sistema de telemetria deverá ser projetado considerando privacidade, segurança e possibilidade de configuração pelo utilizador.

---

# 13. Requisitos específicos da V0.1

A V0.1 será a primeira versão funcional do C2L e deverá permanecer deliberadamente limitada.

## Escopo obrigatório

A V0.1 deverá possuir:

- execução em Windows;
- CLI;
- análise manual de ficheiros;
- análise recursiva de diretórios;
- cálculo de hash;
- base local de indicadores;
- deteção por correspondência;
- classificação de risco inicial;
- quarentena;
- logs;
- relatório de análise;
- tratamento básico de erros;
- testes automatizados;
- documentação de utilização.

## Fora do escopo da V0.1

Não fazem parte da primeira versão:

- proteção em tempo real;
- análise comportamental avançada;
- driver/kernel;
- cloud;
- inteligência de ameaças externa;
- interface gráfica completa;
- proteção Android;
- proteção Linux em produção;
- atualização automática de definições;
- sistema comercial de licenciamento;
- mecanismos avançados anti-tamper.

A ausência dessas funcionalidades na V0.1 é deliberada e não representa abandono da visão de longo prazo.

---

# 14. Requisitos do laboratório de desenvolvimento

### RNF-017 — Isolamento de amostras
**Prioridade:** Obrigatório

Amostras potencialmente maliciosas utilizadas para pesquisa e testes deverão permanecer isoladas do ambiente de desenvolvimento principal.

### RNF-018 — Ambiente controlado
**Prioridade:** Obrigatório

Testes de malware real, quando necessários em fases futuras, deverão ser realizados em ambientes controlados, como máquinas virtuais ou laboratórios dedicados.

### RNF-019 — Separação entre desenvolvimento e teste
**Prioridade:** Obrigatório

O projeto deverá evitar que artefactos de teste potencialmente perigosos sejam incluídos acidentalmente nos binários, pacotes ou repositório principal.

---

# 15. Critérios de aceitação da V0.1

A V0.1 será considerada funcional quando:

1. o utilizador conseguir iniciar uma análise pela CLI;
2. um ficheiro ou diretório puder ser analisado;
3. o sistema calcular o identificador criptográfico do ficheiro;
4. o identificador puder ser comparado com a base local;
5. uma ameaça conhecida de teste puder ser identificada;
6. o resultado apresentar uma classificação compreensível;
7. uma deteção puder ser enviada para quarentena;
8. a operação ficar registrada em log;
9. erros individuais forem tratados sem interromper desnecessariamente a análise;
10. existirem testes automatizados para os componentes críticos;
11. a documentação permitir que outra pessoa instale e execute o projeto;
12. o projeto puder ser executado sem depender da presença do ambiente de desenvolvimento.

---

# 16. Riscos iniciais

| Risco | Impacto | Mitigação |
|---|---|---|
| Complexidade excessiva da arquitetura | Alto | Evolução incremental |
| Dependência prematura de uma plataforma | Alto | Separação core/platform |
| Falsos positivos | Alto | Testes e classificação de risco |
| Falsos negativos | Crítico | Evolução contínua da deteção |
| Corrupção da quarentena | Alto | Integridade e testes |
| Consumo excessivo de recursos | Médio | Benchmarks desde as primeiras versões |
| Alteração maliciosa do próprio antivírus | Crítico | Anti-tamper em fases futuras |
| Testes inseguros com malware | Crítico | Laboratório isolado |
| Base de ameaças desatualizada | Alto | Sistema de atualização futuro |
| Decisões tecnológicas prematuras | Médio | Arquitetura após requisitos |

---

# 17. Premissas

Este documento assume que:

- o C2L começará como projeto educacional/profissional;
- a primeira implementação será focada em Windows;
- Linux e Android permanecerão objetivos de evolução;
- a arquitetura será definida após os requisitos;
- a tecnologia principal ainda não está definitivamente escolhida;
- segurança terá prioridade sobre velocidade de implementação;
- o projeto deverá permanecer aberto à possibilidade de transformação em produto comercial.

---

# 18. Decisões ainda pendentes

Os seguintes pontos deverão ser definidos no documento de arquitetura:

- linguagem principal do núcleo;
- linguagem e tecnologia da CLI;
- tecnologia da futura interface gráfica;
- estrutura dos módulos;
- mecanismo de armazenamento dos indicadores;
- formato da base de definições;
- modelo de comunicação entre core e adapters;
- estratégia de empacotamento;
- sistema de atualização;
- estratégia de assinatura de código;
- mecanismos específicos de Windows;
- estratégia Linux;
- estratégia futura Android.

Essas decisões não serão tomadas neste documento para evitar acoplamento prematuro entre requisitos e implementação.

---

# 19. Relação com os próximos documentos

Os requisitos definidos aqui serão utilizados diretamente para produzir:

- **03 — Arquitetura:** transformação dos requisitos em componentes técnicos;
- **04 — Threat Model:** identificação das ameaças contra o C2L;
- **05 — Motor de Detecção:** definição dos mecanismos de análise;
- **06 — Segurança:** controles e princípios de proteção;
- **07 — Testes:** validação dos requisitos;
- **08 — Roadmap:** evolução dos requisitos por versão.

---

## 20. Conclusão

O C2L Antivirus deverá ser construído de forma incremental, mantendo uma separação clara entre **o que o produto precisa fazer**, **como será implementado** e **como será validado**.

A V0.1 terá como objetivo provar o núcleo fundamental do produto: **analisar, identificar, classificar, isolar e registrar ameaças de forma segura e reproduzível**.

As capacidades avançadas — proteção em tempo real, comportamento, threat intelligence, múltiplas plataformas e serviços cloud — serão adicionadas somente depois que a fundação demonstrar estabilidade, segurança e testabilidade.

Este documento constitui a referência oficial de requisitos para a próxima etapa de arquitetura do C2L Antivirus.
