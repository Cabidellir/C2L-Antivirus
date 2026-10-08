# 03 — Arquitetura

**Projeto:** C2L Antivirus  
**Versão do documento:** 1.0  
**Data:** 08/10/2026  
**Status:** Aprovado  
**Documento:** 03 — Arquitetura

---

## 1. Objetivo

Este documento define a arquitetura técnica de referência do C2L Antivirus.

A arquitetura deve permitir que o projeto:

- comece de forma simples e controlada;
- tenha uma V0.1 funcional no Windows;
- preserve a possibilidade de evolução para Linux e Android;
- mantenha o núcleo de segurança independente do sistema operativo;
- permita substituir componentes sem reescrever todo o produto;
- facilite testes, auditoria e manutenção;
- suporte uma futura evolução para um produto comercial.

A arquitetura apresentada neste documento é uma arquitetura de referência. Detalhes de implementação poderão ser refinados durante o desenvolvimento, desde que as decisões relevantes sejam documentadas.

---

# 2. Princípios arquiteturais

## 2.1 Security by Design

A segurança deverá ser considerada desde o desenho dos componentes, e não adicionada apenas depois da implementação.

## 2.2 Separação de responsabilidades

Cada componente deverá possuir uma responsabilidade clara.

O mecanismo de análise não deverá ser responsável pela interface do utilizador, por exemplo.

## 2.3 Core independente da plataforma

A lógica central do C2L deverá evitar dependências diretas de APIs específicas do Windows, Linux ou Android.

## 2.4 Adapters para funcionalidades específicas

Funcionalidades que dependam do sistema operativo deverão ser expostas ao núcleo através de interfaces bem definidas e implementadas por adaptadores específicos.

## 2.5 Privilégio mínimo

Componentes que não necessitem de privilégios elevados não deverão executá-los.

## 2.6 Testabilidade

Os componentes críticos deverão poder ser testados isoladamente.

## 2.7 Observabilidade

As operações relevantes deverão produzir eventos e informações suficientes para diagnóstico, auditoria e análise de desempenho.

## 2.8 Evolução incremental

A arquitetura deverá permitir que funcionalidades futuras sejam adicionadas sem exigir uma reconstrução completa do projeto.

---

# 3. Visão geral

A arquitetura inicial será organizada em camadas:

```
┌───────────────────────────────────────────────┐
│              Interface / Aplicação            │
│                 CLI / futura UI               │
├───────────────────────────────────────────────┤
│              Orquestração / API               │
├───────────────────────────────────────────────┤
│                 C2L Core                      │
│                                               │
│  Scanner │ Detection │ Risk │ Quarantine     │
│  Hashing │ Indicators │ Events │ Reporting    │
├───────────────────────────────────────────────┤
│          Platform Abstraction Layer           │
├───────────────────────────────────────────────┤
│      Windows Adapter / Linux / Android        │
└───────────────────────────────────────────────┘
```

O fluxo lógico principal será:

```
Utilizador
    │
    ▼
Interface
    │
    ▼
Orquestrador
    │
    ▼
Scanner
    │
    ├──► Hashing
    │
    ├──► Indicators
    │
    ├──► Detection Engine
    │
    └──► Risk Engine
              │
              ▼
        Resultado da análise
              │
       ┌──────┴──────┐
       ▼             ▼
   Quarantine      Report
       │             │
       └──────┬──────┘
              ▼
            Events
              │
              ▼
             Log
```

---

# 4. Componentes principais

## 4.1 C2L Core

O Core será o núcleo lógico do antivírus.

Deverá conter regras e processos que possam ser compartilhados entre plataformas.

Responsabilidades:

- coordenação da análise;
- cálculo e utilização de hashes;
- consulta a indicadores;
- deteção;
- classificação de risco;
- geração de resultados;
- eventos;
- regras de negócio;
- orquestração da quarentena;
- geração de relatórios.

O Core não deverá conhecer detalhes específicos de APIs do Windows ou Linux.

---

# 5. Scanner

O Scanner será responsável por localizar e fornecer os objetos que deverão ser analisados.

Responsabilidades:

- receber um ficheiro;
- receber um diretório;
- percorrer diretórios recursivamente;
- identificar ficheiros acessíveis;
- encaminhar os ficheiros para análise;
- informar erros de acesso;
- contabilizar objetos processados.

O Scanner não deverá decidir sozinho se um ficheiro é malicioso.

Essa responsabilidade pertence ao Detection Engine e ao Risk Engine.

---

# 6. Hashing

O componente de Hashing será responsável pela identificação criptográfica dos ficheiros.

Responsabilidades:

- ler o conteúdo do ficheiro;
- calcular o hash configurado;
- devolver o resultado ao Core;
- tratar erros de leitura.

O componente deverá ser projetado para permitir a utilização futura de outros algoritmos quando necessário.

---

# 7. Indicator Store

O Indicator Store será responsável pelo armazenamento e consulta dos indicadores conhecidos.

Na V0.1, deverá suportar pelo menos:

- hashes;
- identificação do indicador;
- classificação;
- origem;
- versão;
- metadados mínimos.

Exemplo conceitual:

```
Indicator
 ├── ID
 ├── Type
 ├── Value
 ├── Classification
 ├── Source
 └── Version
```

A tecnologia de armazenamento ainda não será definida neste documento.

---

# 8. Detection Engine

O Detection Engine será responsável por determinar se existem evidências de uma ameaça.

Na V0.1, o mecanismo será simples:

```
Hash do ficheiro
       │
       ▼
Indicator Store
       │
       ├── Correspondência ──► Deteção
       │
       └── Sem correspondência ──► Sem deteção
```

Nas versões seguintes, o componente deverá poder incorporar:

- assinaturas;
- heurísticas;
- características estruturais;
- reputação;
- comportamento;
- inteligência de ameaças.

A arquitetura deverá evitar que a adição dessas técnicas obrigue à reconstrução do Scanner.

---

# 9. Risk Engine

O Risk Engine transformará as evidências obtidas pelo Detection Engine em uma classificação de risco.

Exemplo conceitual:

```
Evidências
    │
    ├── Hash conhecido
    ├── Assinatura
    ├── Heurística
    ├── Reputação
    └── Comportamento
            │
            ▼
       Risk Engine
            │
            ▼
     Risk Assessment
```

Na V0.1, o modelo será deliberadamente simples.

No futuro, poderá evoluir para uma pontuação baseada em múltiplos fatores.

---

# 10. Quarantine

O componente de Quarantine será responsável pelo isolamento controlado de ficheiros.

Responsabilidades:

- receber um item identificado;
- mover ou copiar o item para área controlada;
- impedir execução acidental;
- registrar a operação;
- preservar metadados necessários;
- permitir futura restauração controlada;
- permitir futura eliminação definitiva.

A implementação deverá considerar:

- integridade;
- permissões;
- nomes internos;
- armazenamento de metadados;
- recuperação após falha.

A quarentena deverá ser tratada como um componente de segurança, não simplesmente como uma pasta.

---

# 11. Event System

O Event System será responsável pela comunicação interna de eventos relevantes.

Exemplos:

```
ScanStarted
FileAnalyzed
DetectionFound
RiskCalculated
QuarantineStarted
QuarantineCompleted
ScanCompleted
ErrorOccurred
```

A utilização de eventos permitirá futuramente que diferentes componentes consumam informações sem criar dependências excessivas entre eles.

---

# 12. Logging

O Logging deverá registrar informações relevantes para:

- diagnóstico;
- auditoria;
- suporte;
- testes;
- análise de desempenho.

O sistema de logs deverá possuir níveis, por exemplo:

- ERROR;
- WARN;
- INFO;
- DEBUG.

Logs não deverão armazenar dados sensíveis desnecessariamente.

---

# 13. Reporting

O Reporting será responsável por transformar os resultados internos em informações compreensíveis.

Deverá permitir inicialmente:

- resumo da análise;
- quantidade de ficheiros;
- deteções;
- erros;
- tempo de execução;
- resultado final.

A geração de relatórios deverá ser separada da apresentação visual.

Isso permitirá que os mesmos dados sejam utilizados pela CLI, por uma futura UI ou por uma API.

---

# 14. Orchestrator

O Orchestrator será responsável por coordenar uma análise completa.

Fluxo conceitual:

```
Start Scan
    │
    ▼
Discover Files
    │
    ▼
Read File
    │
    ▼
Calculate Hash
    │
    ▼
Check Indicators
    │
    ▼
Detection
    │
    ▼
Risk Assessment
    │
    ├──► Safe
    ├──► Suspicious
    └──► Malicious
             │
             ▼
         Quarantine
    │
    ▼
Generate Events
    │
    ▼
Report
```

O Orchestrator não deverá conter regras específicas de deteção.

---

# 15. Interface de plataforma

Para manter a portabilidade, o Core deverá comunicar com o sistema operativo através de abstrações.

Conceito:

```
                C2L Core
                   │
          Platform Abstraction
                   │
       ┌───────────┼───────────┐
       ▼           ▼           ▼
   Windows       Linux       Android
   Adapter       Adapter      Adapter
```

Exemplos de funcionalidades potencialmente dependentes da plataforma:

- sistema de ficheiros;
- permissões;
- processos;
- monitorização em tempo real;
- serviços;
- notificações;
- mecanismos de isolamento;
- integração com APIs de segurança.

---

# 16. Windows Adapter

A primeira implementação de plataforma será Windows.

Na V0.1, o Adapter deverá fornecer apenas o necessário para:

- acesso aos ficheiros;
- navegação no sistema de ficheiros;
- operações de quarentena;
- informações básicas do sistema;
- execução da aplicação.

Funcionalidades avançadas do Windows serão adicionadas posteriormente.

---

# 17. Linux Adapter

O Linux será implementado depois da estabilização do Core.

O Adapter Linux deverá implementar as mesmas abstrações necessárias ao Core, utilizando os mecanismos próprios do sistema.

O objetivo não será forçar uma implementação idêntica à do Windows, mas preservar uma interface comum.

---

# 18. Android Adapter

Android será considerado uma plataforma futura.

A arquitetura deverá levar em consideração desde já que Android possui restrições diferentes de Windows e Linux, incluindo:

- modelo de permissões;
- sandbox de aplicações;
- acesso limitado ao sistema de ficheiros;
- ciclo de vida de aplicações;
- APIs específicas de segurança.

A implementação Android não deverá ser tratada simplesmente como uma versão móvel do Adapter Windows.

---

# 19. Interface de utilizador

## 19.1 CLI

A CLI será a primeira interface oficial.

Responsabilidades:

- iniciar análises;
- receber parâmetros;
- mostrar resultados;
- apresentar erros;
- consultar versão;
- facilitar automação;
- facilitar testes.

## 19.2 UI futura

A interface gráfica deverá consumir funcionalidades do Core através de uma interface estável.

A UI não deverá conter lógica crítica de deteção.

Conceito:

```
UI
 │
 ▼
Application API
 │
 ▼
C2L Core
```

---

# 20. API interna

Os componentes deverão comunicar através de contratos claros.

Exemplo conceitual:

```
ScanRequest
    ├── Target
    ├── Options
    └── ScanMode

ScanResult
    ├── Status
    ├── FilesScanned
    ├── Detections
    ├── Errors
    └── Duration
```

Os contratos deverão evitar que detalhes internos sejam expostos diretamente à interface.

---

# 21. Estrutura lógica do projeto

A estrutura inicial proposta é:

```
C2L-Antivirus/
│
├── docs/
│
├── core/
│   ├── scanner/
│   ├── hashing/
│   ├── detection/
│   ├── risk/
│   ├── quarantine/
│   ├── indicators/
│   ├── events/
│   ├── logging/
│   └── reporting/
│
├── platform/
│   └── windows/
│
├── application/
│   └── cli/
│
├── tests/
│
├── tools/
│
└── README.md
```

Essa estrutura é **conceitual** neste momento.

Os diretórios de implementação somente deverão ser criados quando a tecnologia e a estrutura de build forem definidas.

---

# 22. Fluxo de uma análise V0.1

O fluxo completo esperado será:

```
Usuário
   │
   ▼
CLI
   │
   ▼
Orchestrator
   │
   ▼
Scanner
   │
   ▼
Hashing
   │
   ▼
Indicator Store
   │
   ▼
Detection Engine
   │
   ▼
Risk Engine
   │
   ├───────────────┐
   │               │
   ▼               ▼
Safe/Suspicious   Malicious
   │               │
   │               ▼
   │           Quarantine
   │               │
   └───────┬───────┘
           ▼
        Events
           │
           ▼
        Logging
           │
           ▼
        Reporting
           │
           ▼
          CLI
```

---

# 23. Fluxo de dependências

As dependências deverão seguir preferencialmente uma direção única:

```
Application
     │
     ▼
Orchestration
     │
     ▼
Core
     │
     ▼
Platform Abstraction
     │
     ▼
Platform Adapter
```

Componentes inferiores não deverão depender diretamente da UI.

Por exemplo:

**Permitido:**

```
CLI → Core
```

**Não recomendado:**

```
Core → CLI
```

Isso preserva a possibilidade de substituir a CLI por uma UI ou API futuramente.

---

# 24. Modelo de concorrência

A V0.1 deverá começar com um modelo simples e previsível.

A arquitetura deverá, entretanto, permitir processamento concorrente de múltiplos ficheiros no futuro.

Possíveis evoluções:

- fila de análise;
- workers;
- paralelização de hashing;
- cache;
- priorização;
- cancelamento de análises.

A concorrência não deverá ser introduzida antes de existirem métricas que justifiquem sua necessidade.

---

# 25. Armazenamento

A arquitetura prevê três categorias principais de dados:

### Definições

Indicadores utilizados pelo Detection Engine.

### Estado

Informações necessárias ao funcionamento da aplicação.

### Eventos

Logs e resultados de execução.

A tecnologia de armazenamento ainda será definida.

A escolha deverá considerar:

- desempenho;
- integridade;
- portabilidade;
- facilidade de atualização;
- recuperação após falhas;
- tamanho esperado da base.

---

# 26. Segurança arquitetural

A arquitetura deverá considerar desde o início:

- validação de entradas;
- tratamento seguro de caminhos;
- prevenção de path traversal;
- permissões mínimas;
- proteção da quarentena;
- integridade dos dados;
- tratamento de erros;
- separação de privilégios;
- proteção contra manipulação dos resultados;
- não execução automática de ficheiros analisados.

Mecanismos mais avançados, como anti-tamper, assinatura de componentes e atualização segura, serão detalhados em documentos posteriores.

---

# 27. Estratégia de evolução

A arquitetura foi desenhada para permitir a evolução:

```
V0.1
Core + Windows + CLI
       │
       ▼
V0.2
Detection avançado
       │
       ▼
V0.3
Linux
       │
       ▼
V0.4
Real-time Protection
       │
       ▼
V0.5
Behavior Engine
       │
       ▼
V0.6
Threat Intelligence
       │
       ▼
V0.7
Desktop UI
       │
       ▼
V0.8
Android
       │
       ▼
V0.9
Cloud
       │
       ▼
V1.0
C2L Security Platform
```

---

# 28. Decisão sobre linguagem

A linguagem de implementação **não será definida exclusivamente neste documento**.

A decisão deverá considerar:

- segurança de memória;
- acesso a APIs de sistema;
- desempenho;
- portabilidade;
- suporte a Windows/Linux/Android;
- maturidade do ecossistema;
- bibliotecas criptográficas;
- tooling;
- facilidade de testes;
- manutenção de longo prazo;
- possibilidade comercial.

As principais candidatas serão avaliadas antes do início da implementação do Core.

---

# 29. Decisões arquiteturais futuras

Antes da V0.1, deverão ser definidos:

1. linguagem do Core;
2. linguagem da CLI;
3. sistema de build;
4. formato da base de indicadores;
5. mecanismo de configuração;
6. formato dos logs;
7. formato dos resultados;
8. estratégia de testes;
9. mecanismo de empacotamento;
10. estratégia de distribuição.

Essas decisões serão registradas antes da implementação correspondente.

---

# 30. Critérios de aceitação da arquitetura

A arquitetura será considerada adequada quando:

- o Core puder ser executado sem depender da UI;
- a CLI puder ser substituída futuramente;
- funcionalidades específicas do Windows estiverem isoladas;
- o Detection Engine puder evoluir sem alterar o Scanner;
- o Risk Engine puder evoluir sem alterar a interface;
- a quarentena puder ser testada independentemente;
- os componentes críticos puderem ser testados isoladamente;
- Linux puder ser adicionado sem reescrever o Core;
- Android puder ser tratado como uma plataforma distinta;
- a arquitetura não exigir uma decisão prematura sobre funcionalidades futuras.

---

# 31. Conclusão

A arquitetura do C2L Antivirus será baseada em um **núcleo multiplataforma cercado por adaptadores específicos de cada sistema operativo**.

A primeira implementação será propositalmente pequena:

**Core + Windows Adapter + CLI.**

O objetivo da arquitetura não é construir imediatamente todas as capacidades de um antivírus comercial, mas criar uma fundação suficientemente sólida para que cada nova capacidade possa ser adicionada de forma controlada.

A principal decisão arquitetural é, portanto:

> **O C2L não será um antivírus Windows ao qual Linux e Android serão adicionados posteriormente. Ele será um motor de segurança multiplataforma que começará sendo executado no Windows.**

Essa decisão orientará as escolhas tecnológicas, a estrutura do código e a evolução futura do projeto.
