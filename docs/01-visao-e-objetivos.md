# C2L Antivirus — Documento 01: Visão e Objetivos

**Nome do projeto:** C2L Antivirus  
**Marca:** C2L Security  
**Conceito:** Custom to Line  
**Versão do documento:** 1.0  
**Documento:** 01 — Visão e Objetivos  
**Data:** Outubro de 2026  
**Status:** Em definição

## 1. Visão do projeto

O C2L Antivirus é um projeto de desenvolvimento de uma plataforma de segurança multiplataforma destinada à detecção, análise e prevenção de software malicioso.

O projeto será desenvolvido inicialmente para **Windows e Linux**, mantendo desde a sua arquitetura inicial a possibilidade de expansão para **Android**.

O objetivo de longo prazo é criar uma plataforma tecnicamente capaz de evoluir de um projeto de investigação e desenvolvimento para um possível **produto comercial de segurança**, sem comprometer a qualidade da engenharia, a segurança ou a transparência técnica.

O projeto será construído de forma incremental, documentando suas decisões arquiteturais, limitações, resultados de testes e evolução.

## 2. Origem do nome

### C2L

O nome **C2L** possui uma dupla referência.

A primeira é pessoal:

**C + 2L → C2L**

representando as letras presentes no sobrenome **Cabidelli**, particularmente a combinação entre o C inicial e o LL.

A segunda referência é conceitual:

**C2L → Custom to Line**

A expressão representa a ideia de desenvolver soluções de segurança adaptáveis, construídas de acordo com necessidades reais e evoluindo continuamente.

### Identidade

**C2L Antivirus**

> Custom to Line. Security by Design.

A marca **C2L Security** poderá futuramente representar uma organização ou linha de produtos de segurança, enquanto **C2L Antivirus** será inicialmente o principal produto/projeto.

## 3. Objetivo principal

Desenvolver um mecanismo de segurança capaz de:

- analisar arquivos;
- identificar ameaças conhecidas;
- detectar comportamentos suspeitos;
- realizar análise heurística;
- monitorizar atividades do sistema;
- bloquear ou isolar ameaças;
- manter arquivos suspeitos em quarentena;
- registrar eventos de segurança;
- atualizar sua inteligência de ameaças;
- funcionar em diferentes sistemas operacionais.

O projeto deverá priorizar uma arquitetura modular, segura e extensível.

## 4. Visão de longo prazo

O objetivo final não é simplesmente criar um scanner de arquivos.

O C2L deverá evoluir para uma **plataforma de proteção de endpoints**, composta por diferentes mecanismos de análise e proteção.

A visão arquitetural de longo prazo é:

    C2L SECURITY
         |
    THREAT INTELLIGENCE
         |
    +-----------------------+
    |                       |
    CLOUD SERVICES      LOCAL ENGINE
    |                       |
    +-----------+-----------+
                |
         C2L CORE ENGINE
                |
       +--------+--------+
       |        |        |
    WINDOWS   LINUX   ANDROID
       |        |        |
       +--------+--------+
                |
       USER APPLICATIONS

Essa arquitetura deverá permitir que diferentes componentes evoluam independentemente.

## 5. Plataformas

### 5.1 Windows

Será a primeira plataforma prioritária.

O Windows permitirá desenvolver inicialmente funcionalidades como:

- análise de arquivos;
- análise de processos;
- monitorização de alterações;
- análise de executáveis;
- quarentena;
- proteção em tempo real;
- integração com mecanismos de segurança do sistema.

### 5.2 Linux

O suporte Linux será desenvolvido como segunda plataforma.

O projeto deverá considerar as diferenças existentes entre distribuições Linux e evitar depender excessivamente de características específicas de uma única distribuição.

Entre os componentes previstos estão:

- análise de arquivos;
- processos;
- permissões;
- serviços;
- monitorização do sistema;
- análise de executáveis ELF;
- mecanismos de proteção em tempo real.

### 5.3 Android

Android não será necessariamente implementado na primeira versão.

Entretanto, sua existência futura deverá ser considerada desde a arquitetura inicial.

O objetivo será posteriormente permitir:

- análise de aplicações APK;
- análise de permissões;
- avaliação de aplicações;
- reputação de aplicações;
- detecção de comportamentos suspeitos;
- integração com serviços de inteligência de ameaças.

As limitações do modelo de segurança do Android deverão ser consideradas desde o planejamento.

## 6. Arquitetura multiplataforma

O projeto deverá separar o código específico de cada sistema operacional do núcleo de análise.

### Core

Responsável por funcionalidades independentes do sistema operacional:

- hashing;
- análise;
- regras;
- assinaturas;
- heurística;
- classificação;
- avaliação de risco;
- eventos;
- quarentena;
- threat intelligence.

### Platform Layer

Responsável pela comunicação com o sistema operacional:

    platform/
    ├── windows/
    ├── linux/
    └── android/

Cada plataforma deverá implementar somente os mecanismos necessários para interagir com o respectivo sistema.

## 7. Princípios do projeto

O desenvolvimento deverá seguir alguns princípios fundamentais.

### Segurança por design

Segurança não será adicionada posteriormente.

Ela deverá fazer parte da arquitetura desde o início.

### Modularidade

Componentes deverão possuir responsabilidades bem definidas e baixo acoplamento.

### Observabilidade

O sistema deverá registrar informações suficientes para permitir compreender o que aconteceu durante uma detecção.

### Testabilidade

Cada componente importante deverá possuir testes automatizados.

### Transparência

Limitações, falsos positivos, falsos negativos e decisões técnicas deverão ser documentados.

### Evolução incremental

Nenhuma funcionalidade deverá ser criada apenas por aumentar a complexidade do projeto.

Cada versão deverá possuir objetivos claros.

### Portabilidade

O núcleo deverá evitar dependências desnecessárias de uma plataforma específica.

## 8. Modelo inicial de detecção

O C2L não deverá depender exclusivamente de assinaturas.

A arquitetura deverá permitir combinar diferentes fontes de evidência.

    ARQUIVO
       |
    +--+---------+
    |            |
    HASH      METADATA
    |            |
    +-----+------+
          |
    SIGNATURE ENGINE
          |
    HEURISTIC ENGINE
          |
    BEHAVIOR ENGINE
          |
    REPUTATION ENGINE
          |
      RISK ENGINE
          |
    +-----+-----+
    |     |     |
   LOW  MEDIUM HIGH
    |     |     |
  Allow Review Quarantine

O sistema deverá evitar decisões baseadas em uma única característica quando isso puder gerar falsos positivos.

## 9. Inteligência de ameaças

O projeto deverá possuir uma camada independente para inteligência de ameaças.

Inicialmente poderá conter:

- hashes;
- assinaturas;
- indicadores de comprometimento;
- regras;
- famílias de malware;
- níveis de severidade;
- reputação.

Em versões futuras poderá existir uma infraestrutura centralizada:

    C2L Client
         |
      C2L API
         |
    +----+----------------+
    |         |            |
    Threat   Reputation   Detection
    Database              Rules
         |
    Machine Learning

Isso permitirá atualizar a inteligência sem necessariamente distribuir uma nova versão completa do software.

## 10. Quarentena

Arquivos considerados perigosos não deverão ser automaticamente apagados sempre que isso puder ser evitado.

O sistema deverá possuir um mecanismo de quarentena capaz de:

- remover o arquivo do local original;
- impedir sua execução;
- armazenar metadados;
- registrar o motivo da detecção;
- permitir restauração controlada;
- permitir eliminação definitiva.

Cada evento deverá possuir informações suficientes para auditoria.

## 11. Proteção em tempo real

Uma das metas de evolução do projeto será implementar proteção contínua.

O sistema deverá ser capaz de detectar eventos relevantes, analisá-los e tomar uma decisão.

    Arquivo criado
         |
    Monitor do sistema
         |
      C2L Engine
         |
      +--+------+
      |         |
    Seguro   Suspeito
      |         |
    Permitir  Quarentena
                 |
               Alertar

A implementação desse mecanismo será específica para cada plataforma.

## 12. Laboratório de testes

O desenvolvimento deverá utilizar ambientes isolados para testes de segurança.

A arquitetura inicial deverá prever:

    HOST
     |
     +-- Windows VM
     |      +-- C2L
     |
     +-- Linux VM
     |      +-- C2L
     |
     +-- Android Emulator
            +-- C2L

Arquivos potencialmente maliciosos não deverão ser executados no computador pessoal utilizado para desenvolvimento.

Os testes deverão priorizar inicialmente:

- arquivos benignos;
- amostras de teste;
- padrões simulados;
- datasets controlados;
- indicadores conhecidos;
- ambientes isolados.

## 13. Escopo inicial

A primeira versão do C2L deverá concentrar-se em:

### V0.1

- Windows;
- scanner manual;
- cálculo de SHA-256;
- banco de indicadores;
- motor básico de detecção;
- sistema de risco;
- quarentena;
- logs;
- CLI;
- testes automatizados;
- documentação.

A interface gráfica completa não será obrigatória na primeira versão.

## 14. Roadmap inicial

### V0.1 — Foundation

Criar o núcleo do projeto.

### V0.2 — Detection

Adicionar assinaturas e heurística.

### V0.3 — Linux

Adicionar suporte Linux.

### V0.4 — Real-Time Protection

Adicionar monitorização em tempo real.

### V0.5 — Behavioral Detection

Adicionar análise comportamental.

### V0.6 — Threat Intelligence

Criar infraestrutura de atualização de inteligência.

### V0.7 — Desktop Application

Criar interface gráfica completa.

### V0.8 — Android

Iniciar implementação para Android.

### V0.9 — Cloud

Adicionar serviços centralizados.

### V1.0 — C2L Security Platform

Consolidar o produto e avaliar sua viabilidade comercial.

## 15. Possibilidade comercial

O projeto deverá permanecer tecnicamente aberto à transformação em produto comercial.

Entretanto, até que essa decisão seja tomada, o desenvolvimento deverá priorizar:

- qualidade técnica;
- aprendizagem;
- documentação;
- segurança;
- testes;
- arquitetura;
- demonstração de resultados.

Uma eventual comercialização deverá considerar posteriormente:

- infraestrutura;
- atualização de ameaças;
- suporte;
- privacidade;
- conformidade legal;
- licenciamento;
- proteção contra abuso;
- assinatura digital;
- distribuição;
- telemetria;
- infraestrutura de cloud;
- modelo de negócio.

## 16. Critério de sucesso

O sucesso da primeira fase não será medido pela quantidade de funcionalidades.

O primeiro objetivo será provar que o projeto possui uma arquitetura sólida e que consegue executar corretamente o ciclo:

    DETECTAR
       |
    ANALISAR
       |
    CLASSIFICAR
       |
    DECIDIR
       |
    ISOLAR
       |
    REGISTRAR

Posteriormente, o sucesso será medido pela capacidade de aumentar a detecção mantendo uma taxa aceitável de falsos positivos.

## 17. O que o C2L não pretende ser inicialmente

O C2L não pretende, na primeira versão:

- substituir soluções comerciais existentes;
- competir diretamente com Microsoft Defender;
- oferecer proteção perfeita;
- analisar todos os tipos de malware;
- possuir capacidades completas de EDR;
- executar código potencialmente perigoso fora de ambientes isolados;
- prometer segurança absoluta.

O projeto deverá declarar suas limitações explicitamente.

## 18. Visão final

A visão do C2L é evoluir de um mecanismo de análise de arquivos para uma plataforma de segurança capaz de compreender não apenas **o que um arquivo é**, mas também **o que ele faz**.

A evolução conceitual será:

    V0.1
    "Este arquivo é conhecido?"
          |
    V0.2
    "Este arquivo parece suspeito?"
          |
    V0.4
    "O que este arquivo está fazendo?"
          |
    V0.6
    "O que sabemos sobre esta ameaça?"
          |
    V0.8
    "Como esta ameaça se comporta em diferentes plataformas?"
          |
    V1.0+
    "Como podemos proteger o endpoint de forma contínua?"

O objetivo final é construir uma plataforma na qual **detecção, contexto, comportamento e inteligência de ameaças trabalhem em conjunto**.

**C2L Antivirus**

*Custom to Line. Security by Design.*
