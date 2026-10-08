# 08 — Roadmap de Desenvolvimento

**Projeto:** C2L Antivirus  
**Versão do documento:** 1.0  
**Data:** 08/10/2026  
**Status:** Aprovado

## 1. Objetivo

Este documento transforma a arquitetura, os requisitos, o modelo de ameaças, o motor de detecção, os controles de segurança e a estratégia de testes do C2L Antivirus em uma sequência de desenvolvimento executável.

O roadmap deve permitir que o projeto evolua de um MVP pequeno e verificável para uma plataforma de proteção de endpoints sem comprometer a arquitetura multiplataforma nem antecipar complexidade desnecessária.

O princípio central é:

> **Construir primeiro um núcleo pequeno, seguro, testável e mensurável; adicionar capacidade somente quando a camada anterior estiver comprovada.**

---

## 2. Princípios de evolução

O desenvolvimento seguirá estas regras:

1. **Core antes da interface.**
2. **Segurança e testes acompanham cada funcionalidade.**
3. **Detecção em camadas, sem depender de uma única técnica.**
4. **Conteúdo de detecção separado do código do motor.**
5. **Plataforma separada do core.**
6. **CLI antes de GUI.**
7. **Local antes de cloud.**
8. **Detecção determinística antes de ML.**
9. **Observabilidade antes de automação agressiva.**
10. **Nenhuma nova camada entra em produção sem critérios objetivos de aceitação.**
11. **Compatibilidade multiplataforma deve ser preservada desde a arquitetura, mesmo quando a implementação inicial for Windows.**
12. **Não construir complexidade comercial antes de existir um produto tecnicamente estável.**

---

## 3. Visão geral do roadmap

| Fase | Versão | Objetivo principal | Resultado |
|---|---|---|---|
| 0 | Pré-V0.1 | Fundação técnica | Repositório e decisões preparados |
| 1 | V0.1 | MVP de detecção local | Scanner funcional no Windows |
| 2 | V0.2 | Integridade e atualização | Conteúdo de detecção atualizável e protegido |
| 3 | V0.3 | Análise estática e heurística | Detecção além de hash |
| 4 | V0.4 | Telemetria e comportamento | Observação de processos e eventos |
| 5 | V0.5 | Resposta e recuperação | Primeiros recursos EDR |
| 6 | V0.x | Plataforma multiplataforma | Linux com core compartilhado |
| 7 | V1.0 | Produto de endpoint | Base madura para uso real controlado |
| 8 | Pós-V1.0 | Expansão | Android, cloud/TI, GUI avançada e recursos comerciais |

As versões são marcos técnicos, não promessas de calendário. Uma fase só termina quando seus critérios de saída forem cumpridos.

---

# 4. Fase 0 — Fundação

## Objetivo

Preparar a base de engenharia antes de implementar o scanner.

### Atividades

- Confirmar arquitetura modular.
- Definir contratos entre Core, Scanner, Detection Engine, Risk Engine, Quarantine e Platform Adapter.
- Escolher a linguagem principal do core.
- Definir estratégia de dependências.
- Definir estrutura de código.
- Configurar CI.
- Configurar lint/format/static analysis.
- Definir estratégia de versionamento.
- Definir formato inicial dos logs e eventos.
- Definir formato do indicador store.
- Criar testes básicos de infraestrutura.
- Definir política de build reproduzível.
- Definir estratégia inicial de release artifacts.
- Definir SBOM e provenance como requisitos futuros de release.

### Decisão obrigatória: linguagem

A linguagem de implementação ainda não está definitivamente escolhida.

A decisão deve considerar:

- segurança de memória;
- desempenho;
- acesso a APIs nativas;
- suporte a Windows;
- portabilidade para Linux;
- possibilidade de Android;
- ecossistema;
- manutenção de longo prazo;
- qualidade das bibliotecas;
- facilidade de testes;
- interoperabilidade com componentes futuros.

**Nenhuma escolha deve ser feita apenas por familiaridade.**

Rust permanece uma candidata forte para o core, mas a decisão deve ser registrada formalmente antes da implementação principal.

### Critério de saída

A Fase 0 termina quando:

- arquitetura estiver documentada;
- linguagem estiver decidida;
- estrutura do projeto estiver definida;
- CI executar build e testes;
- primeiro artefato reproduzível puder ser gerado;
- contratos internos principais estiverem documentados.

---

# 5. V0.1 — MVP Windows CLI

## Objetivo

Criar a primeira versão real e demonstrável do C2L.

A V0.1 deve provar que o C2L consegue:

> **receber um alvo → analisar → identificar uma ameaça conhecida → calcular risco → agir de forma segura → registrar o resultado.**

### Funcionalidades

- CLI;
- scan de arquivo;
- scan recursivo de diretório;
- coleta de metadados;
- cálculo de hash;
- indicador local;
- consulta ao indicador store;
- detecção por correspondência;
- classificação básica de risco;
- decisão de ação;
- quarentena;
- restauração controlada;
- logs estruturados;
- relatório de scan;
- tratamento de erros;
- testes automatizados.

### Fluxo

```
CLI
 ↓
Orchestrator
 ↓
Scanner
 ↓
Hashing
 ↓
Indicator Store
 ↓
Detection Engine
 ↓
Risk Engine
 ↓
Action
 ├── Allow
 ├── Report
 └── Quarantine
 ↓
Event / Log / Report
```

### Fora da V0.1

Não implementar ainda:

- proteção em tempo real;
- driver/kernel;
- EDR;
- cloud;
- ML;
- comportamento avançado;
- threat intelligence externa;
- GUI completa;
- Linux como produto;
- Android;
- atualizações automáticas de definições;
- infraestrutura comercial.

### Critérios de saída

A V0.1 só será considerada concluída quando:

- o scanner funcionar de forma reproduzível;
- hashes forem calculados corretamente;
- indicadores puderem ser consultados;
- uma detecção controlada for reproduzida;
- falso positivo básico puder ser testado;
- quarentena funcionar com segurança;
- logs forem suficientes para reconstruir a decisão;
- erros não forem tratados como "clean";
- testes obrigatórios passarem;
- não houver vulnerabilidade crítica conhecida;
- documentação de uso estiver disponível.

---

# 6. V0.2 — Conteúdo de Detecção e Atualização Segura

## Objetivo

Separar definitivamente o **motor** do **conteúdo de segurança**.

O C2L não deverá precisar de uma nova compilação do executável para cada novo indicador.

### Funcionalidades

- versão do detection content;
- schema versionado;
- validação de conteúdo;
- assinatura digital;
- verificação de autenticidade;
- proteção contra downgrade;
- atualização atômica;
- rollback;
- integridade do indicador store;
- configuração protegida;
- mecanismo inicial de atualização;
- auditoria das atualizações.

### Fluxo

```
Distribution
 ↓
Download
 ↓
Integrity
 ↓
Authenticity
 ↓
Compatibility
 ↓
Atomic Install
 ↓
Validation
 ↓
Activation
 ↓
Monitoring
```

### Critérios de saída

- conteúdo inválido nunca deve ser ativado;
- conteúdo não autenticado deve ser rejeitado;
- downgrade não autorizado deve ser rejeitado;
- interrupção durante atualização não pode deixar o sistema em estado inconsistente;
- rollback deve ser testado;
- atualização deve possuir rastreabilidade.

---

# 7. V0.3 — Análise Estática e Heurística

## Objetivo

Reduzir a dependência de hashes e começar a identificar arquivos desconhecidos por características suspeitas.

### Funcionalidades previstas

- identificação de tipo de arquivo;
- metadados;
- análise estrutural;
- assinaturas/padrões;
- regras heurísticas;
- score por evidência;
- explicabilidade da decisão;
- limites de recursos;
- parser defensivo;
- fuzzing dos componentes que processam conteúdo não confiável.

### Arquitetura

```
File
 ├── Hash
 ├── Metadata
 ├── Structure
 ├── Signature
 └── Heuristics
       ↓
 Evidence
       ↓
 Risk Engine
       ↓
 Decision
```

### Regra importante

A heurística não deve transformar qualquer característica incomum em malware.

O motor deve trabalhar com **evidências combinadas, pesos e contexto**, reduzindo falsos positivos.

### Critérios de saída

- cada regra possuir testes;
- resultados serem explicáveis;
- limites de CPU/memória/tempo existirem;
- entradas malformadas não provocarem crash;
- fuzzing mínimo estar integrado ao processo de qualidade;
- métricas de FP/FN serem acompanhadas.

---

# 8. V0.4 — Telemetria e Detecção Comportamental

## Objetivo

Passar de "o arquivo parece suspeito" para:

> **"o que está acontecendo no sistema?"**

### Funcionalidades previstas

- eventos de processo;
- criação/terminação de processos;
- relações pai/filho;
- acesso a arquivos;
- alterações relevantes;
- carregamento de módulos;
- indicadores de comportamento;
- correlação temporal;
- regras comportamentais;
- início da matriz ATT&CK:
  `Technique → Indicator → Telemetry → Detection → Response`.

### Prioridade

Começar por comportamentos de alto valor defensivo, por exemplo:

- tentativa de desativar mecanismos de segurança;
- comportamentos compatíveis com ransomware;
- execução anômala de processos;
- técnicas de injeção;
- execução suspeita de scripts;
- abuso de mecanismos legítimos do sistema.

### Critério de segurança

A observação deve ser implementada antes de respostas automáticas agressivas.

Primeiro:

**Observe → Detect → Explain**

Depois:

**Contain → Remediate**

---

# 9. V0.5 — Resposta, Contenção e Recuperação

## Objetivo

Evoluir para um modelo inicial de EDR.

### Funcionalidades

- bloqueio controlado;
- encerramento de processo quando justificável;
- isolamento de artefatos;
- contenção;
- resposta baseada em risco;
- correlação de eventos;
- attack-chain detection;
- remediation;
- recuperação;
- rollback de alterações quando tecnicamente seguro;
- proteção inicial contra interferência no próprio C2L.

### Modelo

```
Telemetry
 ↓
Detection
 ↓
Correlation
 ↓
Risk
 ↓
Response
 ↓
Containment
 ↓
Remediation
 ↓
Recovery
```

### Regra

Automação de resposta deve ser graduada:

1. registrar;
2. alertar;
3. bloquear ação específica;
4. colocar artefato em quarentena;
5. conter processo;
6. isolar sistema, quando houver infraestrutura para isso.

Nenhuma ação de alto impacto deve ser introduzida sem testes de falso positivo e recuperação.

---

# 10. Evolução Multiplataforma

## 10.1 Linux

Linux deverá ser o primeiro grande teste da arquitetura multiplataforma.

O objetivo não é copiar a implementação Windows, mas reutilizar:

- Core;
- Detection Engine;
- Risk Engine;
- Indicator Store;
- modelos de evento;
- políticas;
- testes;
- contratos.

E substituir apenas as camadas dependentes do sistema:

- scanner adapter;
- process/event telemetry;
- filesystem operations;
- service lifecycle;
- quarantine implementation;
- privileged operations.

### Critério para iniciar Linux

Linux só deve avançar para implementação significativa quando:

- core estiver suficientemente estável;
- contratos de plataforma estiverem claros;
- testes do core estiverem maduros;
- abstrações não estiverem escondendo diferenças importantes entre sistemas.

---

## 10.2 Android

Android será uma etapa posterior.

A arquitetura deverá considerar desde cedo que Android possui:

- modelo de permissões diferente;
- sandbox por aplicação;
- limitações de acesso;
- ciclo de vida próprio;
- APIs específicas;
- restrições diferentes para monitorização contínua.

O objetivo não será simplesmente portar o agente Windows/Linux.

Android deverá possuir uma estratégia específica de integração com o core compartilhado.

---

# 11. V1.0 — Plataforma de Endpoint

A V1.0 deverá representar uma mudança de categoria:

De:

> **projeto experimental de antivírus**

Para:

> **plataforma de proteção de endpoint tecnicamente estruturada.**

### Requisitos esperados

- core estável;
- scanner robusto;
- múltiplas camadas de detecção;
- conteúdo atualizável e assinado;
- proteção do próprio produto;
- telemetria;
- detecção comportamental;
- resposta controlada;
- quarentena;
- recuperação;
- observabilidade;
- testes automatizados;
- CI/CD;
- documentação;
- processo de release;
- SBOM;
- provenance;
- gestão de vulnerabilidades;
- matriz ATT&CK documentada;
- métricas de qualidade;
- suporte inicial a mais de uma plataforma, quando tecnicamente comprovado.

A V1.0 não deve ser definida apenas pelo número de funcionalidades, mas pela **maturidade e previsibilidade do sistema**.

---

# 12. Cloud e Threat Intelligence

Cloud e threat intelligence externa serão introduzidos depois que o endpoint local estiver funcional.

### Motivação

Evitar que a primeira arquitetura fique dependente de:

- servidor próprio;
- conectividade permanente;
- custos de infraestrutura;
- APIs externas;
- latência;
- privacidade de dados;
- disponibilidade de serviços.

### Evolução

```
Local Detection
      ↓
Optional Reputation
      ↓
Threat Intelligence
      ↓
Cloud Correlation
      ↓
Fleet Intelligence
```

O endpoint deverá continuar possuindo capacidade defensiva mesmo quando estiver offline.

---

# 13. GUI

A interface gráfica não será prioridade inicial.

Ordem recomendada:

```
Core
 ↓
CLI
 ↓
API/Contracts
 ↓
Observability
 ↓
GUI
```

A GUI deverá consumir funcionalidades já existentes no core, evitando colocar lógica de segurança dentro da interface.

---

# 14. ML e IA

Machine Learning não será utilizado apenas porque produtos comerciais utilizam ML.

Antes de introduzir modelos, o C2L deverá possuir:

- dataset confiável;
- definição clara de labels;
- pipeline de treinamento;
- validação;
- métricas;
- controle de versões;
- avaliação contra drift;
- proteção contra manipulação;
- explicabilidade adequada;
- estratégia de atualização;
- fallback seguro.

A primeira versão do C2L deverá priorizar mecanismos determinísticos e explicáveis.

---

# 15. CI/CD e Release Engineering

Desde a primeira versão funcional:

```
Commit
 ↓
Build
 ↓
Format
 ↓
Lint / Static Analysis
 ↓
Unit Tests
 ↓
Component Tests
 ↓
Integration Tests
 ↓
Security Tests
 ↓
Artifact
 ↓
Signing
 ↓
Release
```

Evoluções posteriores:

- SBOM;
- provenance;
- assinatura de artefatos;
- reprodutibilidade;
- verificação de dependências;
- vulnerability scanning;
- release channels;
- rollback;
- atualização segura.

---

# 16. Priorização por risco

Quando houver conflito entre funcionalidades, a prioridade deverá ser:

1. segurança do próprio C2L;
2. integridade das atualizações;
3. confiabilidade do scanner;
4. proteção contra corrupção/entrada malformada;
5. detecção;
6. quarentena e recuperação;
7. observabilidade;
8. performance;
9. interface;
10. recursos comerciais.

Uma funcionalidade nova não deve ser priorizada simplesmente por ser visualmente mais atraente.

---

# 17. O que deliberadamente não fazer cedo

Para evitar desperdício e complexidade prematura, o C2L não deverá começar por:

- GUI sofisticada;
- cloud complexa;
- ML próprio;
- infraestrutura de contas;
- sistema de licenciamento;
- marketplace;
- driver/kernel;
- integração com dezenas de fontes externas;
- suporte simultâneo completo a Windows/Linux/Android;
- automação agressiva de resposta;
- coleta massiva de dados.

Esses itens podem fazer parte do produto futuro, mas não devem impedir a validação do núcleo.

---

# 18. Gates de evolução

Cada versão deverá possuir um gate objetivo.

## Gate A — Fundação

**Pergunta:** conseguimos construir e testar o core de forma reproduzível?

## Gate B — V0.1

**Pergunta:** conseguimos detectar e isolar uma ameaça conhecida de forma segura?

## Gate C — V0.2

**Pergunta:** conseguimos atualizar o conhecimento de detecção sem comprometer a integridade?

## Gate D — V0.3

**Pergunta:** conseguimos detectar arquivos desconhecidos com evidências explicáveis?

## Gate E — V0.4

**Pergunta:** conseguimos identificar comportamento suspeito no sistema?

## Gate F — V0.5

**Pergunta:** conseguimos responder e recuperar sem criar risco desproporcional?

## Gate G — V1.0

**Pergunta:** o sistema é suficientemente previsível, testado, observável e seguro para ser tratado como produto?

---

# 19. Indicadores de progresso

O progresso do projeto não será medido somente por funcionalidades concluídas.

Também serão acompanhados:

- cobertura de testes;
- bugs críticos;
- vulnerabilidades;
- FP/FN;
- tempo de scan;
- consumo de CPU;
- consumo de memória;
- tempo de atualização;
- sucesso/falha de rollback;
- cobertura ATT&CK;
- tempo de detecção;
- tempo de contenção;
- tempo de recuperação;
- número de regressões;
- estabilidade por versão.

---

# 20. Dependências entre fases

```
Fundação
   ↓
V0.1 Scanner + Hash
   ↓
V0.2 Content + Update
   ↓
V0.3 Static + Heuristic
   ↓
V0.4 Telemetry + Behavior
   ↓
V0.5 Response + Recovery
   ↓
V1.0 Endpoint Platform
   ↓
Cloud / Threat Intel / GUI / Android / Commercial
```

Algumas atividades são transversais a todas as fases:

- segurança;
- testes;
- documentação;
- CI/CD;
- observabilidade;
- gestão de dependências;
- revisão arquitetural.

---

# 21. Critério geral para adicionar uma nova funcionalidade

Uma nova funcionalidade só deverá entrar no roadmap de implementação quando houver resposta clara para:

1. Qual ameaça ou problema ela resolve?
2. Em qual camada arquitetural ela pertence?
3. Quais são suas dependências?
4. Qual é o novo ataque que ela pode introduzir?
5. Como será testada?
6. Como será revertida?
7. Como será observada?
8. Como funcionará offline?
9. Como afetará Windows, Linux e Android?
10. Qual é o critério objetivo para considerá-la concluída?

Se essas respostas não existirem, a funcionalidade permanece em investigação.

---

# 22. Resultado esperado do roadmap

O roadmap estabelece uma estratégia deliberadamente incremental:

**V0.1** prova o núcleo.

**V0.2** torna o conhecimento atualizável e confiável.

**V0.3** amplia a detecção.

**V0.4** entende comportamento.

**V0.5** responde e recupera.

**V1.0** consolida a plataforma.

Depois disso, o C2L poderá avançar para threat intelligence, cloud, Linux em maior profundidade, Android, GUI avançada e eventual produto comercial.

O objetivo não é construir rapidamente "um antivírus cheio de funções".

O objetivo é construir uma base de segurança que possa **crescer sem precisar ser reescrita quando o projeto ficar sério**.
