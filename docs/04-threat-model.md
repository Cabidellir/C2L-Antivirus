# 04 — Threat Model e Benchmark de Segurança

**Projeto:** C2L Antivirus  
**Versão do documento:** 1.0  
**Data:** 08/10/2026  
**Status:** Aprovado  
**Documento:** 04 — Threat Model e Benchmark de Segurança

---

# 1. Objetivo

Este documento define o modelo de ameaças do C2L Antivirus com base em:

- capacidades observadas nas principais plataformas modernas de endpoint security;
- testes independentes de proteção;
- MITRE ATT&CK;
- princípios de segurança reconhecidos;
- necessidade de evolução contínua contra novas ameaças.

O objetivo não é copiar um produto específico.

O objetivo é identificar **quais capacidades uma arquitetura moderna precisa possuir para permanecer defensável à medida que as ameaças evoluem**.

---

# 2. Resultado principal do benchmark

A principal conclusão da análise é:

> **Um antivírus moderno não pode depender de assinaturas ou hashes como mecanismo principal de defesa.**

As plataformas líderes combinam múltiplas camadas:

1. prevenção;
2. reputação e inteligência;
3. análise estática;
4. machine learning;
5. análise comportamental;
6. proteção contra exploração;
7. redução da superfície de ataque;
8. proteção contra ransomware;
9. EDR;
10. resposta automatizada;
11. telemetria;
12. threat intelligence;
13. proteção contra adulteração;
14. atualização contínua;
15. correlação entre endpoint, identidade, rede e cloud.

A arquitetura do C2L deverá seguir essa mesma direção, adaptada à realidade e aos recursos do projeto.

---

# 3. Benchmark de mercado — 2026

Foram analisadas principalmente:

- CrowdStrike Falcon;
- Microsoft Defender for Endpoint;
- SentinelOne Singularity;
- Palo Alto Cortex XDR;
- Bitdefender GravityZone;
- Sophos Intercept X.

Também foram considerados testes independentes da AV-Comparatives e AV-TEST.

## 3.1 Observação importante

Não existe um único ranking capaz de declarar objetivamente que um produto é "o melhor" em todos os cenários.

Os produtos possuem objetivos diferentes:

- proteção doméstica;
- endpoint empresarial;
- EDR;
- XDR;
- SOC;
- cloud;
- identidade;
- ambientes regulados;
- dispositivos móveis.

Por isso, o benchmark do C2L compara **capacidades**, e não simplesmente marcas.

---

# 4. Evidências independentes de proteção

Os testes independentes mostram que os melhores produtos de endpoint já atingem níveis muito altos de proteção, mas não existe proteção perfeita.

No Business Security Test H1 2026 da AV-Comparatives, realizado entre março e junho de 2026, os resultados de proteção real incluíram:

| Produto | Protection Rate | False Alarms |
|---|---:|---:|
| Kaspersky | 99,8% | 3 |
| Bitdefender | 99,8% | 4 |
| Elastic | 99,8% | 12 |
| Avast / Norton | 99,0% | 4 |
| Microsoft | 98,8% | 0 |
| ESET / VIPRE | 98,8% | 2 |
| CrowdStrike | 98,5% | 8 |
| Sophos | 98,3% | 2 |

O teste utilizou 400 casos e permitiu conectividade cloud e atualizações. citeturn1search7

No teste de março-abril de 2026, Bitdefender e Elastic bloquearam 100% dos 200 casos testados; Microsoft atingiu 99,0%, CrowdStrike 98,5% e Sophos 98,0%. citeturn1search6

Esses resultados reforçam duas conclusões:

1. a proteção moderna depende de múltiplas camadas;
2. falsos positivos precisam ser tratados como problema de segurança e usabilidade, não apenas como inconveniência.

---

# 5. Benchmark de capacidades

| Capacidade | CrowdStrike | Microsoft | SentinelOne | Palo Alto | Bitdefender | Sophos |
|---|---|---|---|---|---|---|
| Antivirus / prevenção | Forte | Forte | Forte | Forte | Forte | Forte |
| Behavioral detection | Forte | Forte | Forte | Forte | Forte | Forte |
| EDR | Forte | Forte | Forte | Forte | Forte | Forte |
| Threat Intelligence | Muito forte | Muito forte | Forte | Muito forte | Forte | Forte |
| Attack Surface Reduction | Forte | Muito forte | Forte | Forte | Forte | Forte |
| Anti-ransomware | Forte | Forte | Muito forte | Muito forte | Muito forte | Muito forte |
| Rollback / remediation | Forte | Forte | Muito forte | Forte | Muito forte | Forte |
| Tamper protection | Muito forte | Muito forte | Forte | Forte | Forte | Forte |
| Cloud correlation | Muito forte | Muito forte | Forte | Muito forte | Forte | Muito forte |
| Identity correlation | Muito forte | Muito forte | Forte | Muito forte | Forte | Forte |
| Automated response | Muito forte | Muito forte | Muito forte | Muito forte | Forte | Forte |
| Telemetry / hunting | Muito forte | Muito forte | Muito forte | Muito forte | Forte | Forte |
| Multi-platform | Forte | Muito forte | Forte | Forte | Forte | Forte |

A classificação acima é arquitetural/qualitativa, não uma pontuação de laboratório.

---

# 6. CrowdStrike — referência em plataforma cloud-native

A arquitetura Falcon é especialmente relevante para o C2L porque utiliza um sensor leve, telemetria em tempo real, threat intelligence, comportamento e correlação entre diferentes domínios.

Em 2026, a plataforma também passou a enfatizar segurança de identidade, cloud, dados e agentes de IA, demonstrando a direção de evolução do mercado. citeturn2search2turn2search10

A CrowdStrike também descreve uma arquitetura baseada em indicadores de ataque, inteligência de ameaças, telemetria e resposta automatizada. citeturn2search19

### O que o C2L deve aprender

- sensor leve;
- telemetria rica;
- arquitetura cloud-ready;
- correlação de eventos;
- threat intelligence externa;
- capacidade de atualização sem reinstalar o agente;
- detecção orientada a comportamento;
- evolução para identidade e cloud.

### O que não devemos copiar

O C2L não possui atualmente infraestrutura ou escala comparável.

A arquitetura deve permitir essa evolução, mas não tentar reproduzi-la na V0.1.

---

# 7. Microsoft Defender — referência em integração e defesa em camadas

O Microsoft Defender for Endpoint combina prevenção, EDR, análise comportamental, attack surface reduction, vulnerability management, threat intelligence, resposta e hunting. citeturn0search1turn0search11

A Microsoft também utiliza:

- ASR;
- proteção contra ransomware;
- behavioral blocking;
- feedback-loop blocking;
- EDR em modo de bloqueio;
- proteção contra tampering. citeturn0search7turn0search12

A proteção contra adulteração é particularmente importante: o atacante pode tentar terminar processos, parar serviços, modificar configurações, exclusões, ficheiros ou componentes do próprio antivírus. citeturn0search12

### O que o C2L deve aprender

O C2L não deve pensar somente:

> "Como detecto malware?"

Deve também perguntar:

> "Como impeço o atacante de desativar a capacidade de detecção?"

Essa mudança será fundamental para o Threat Model.

---

# 8. SentinelOne — referência em autonomia e resposta

O SentinelOne enfatiza Behavioral AI, contenção automática, remediação e rollback. A plataforma também correlaciona sinais de endpoint, cloud e identidade. citeturn0search2turn0search17

### O que o C2L deve aprender

A resposta não deve ser um componente separado da detecção.

O fluxo moderno é:

**Detectar → decidir → conter → remediar → recuperar**

e não apenas:

**Detectar → alertar**

---

# 9. Palo Alto Cortex XDR — referência em correlação

O Cortex XDR combina prevenção de endpoint com análise comportamental, dados de diferentes fontes e automação.

A Palo Alto descreve uma arquitetura capaz de detectar diferentes etapas de um ataque, incluindo exploração, execução, comportamento e ransomware. citeturn0search21turn0search22

### O que o C2L deve aprender

O objeto de proteção não deve ser apenas o ficheiro.

Deverá evoluir para:

**ficheiro → processo → árvore de processos → comportamento → cadeia de ataque**

---

# 10. Bitdefender — referência em proteção multicamada e recuperação

O GravityZone utiliza múltiplas camadas de machine learning, heurística e análise comportamental.

O Process Inspector mantém uma trilha de alterações realizadas por processos e pode reverter alterações maliciosas, incluindo alterações em ficheiros e registro. citeturn2search5

A plataforma também possui mecanismos específicos para ransomware que preservam cópias de ficheiros para recuperação. citeturn2search0

### O que o C2L deve aprender

A proteção não termina quando o malware é bloqueado.

Precisamos também responder:

> "O que o malware conseguiu alterar antes de ser bloqueado?"

Isso será importante para o futuro mecanismo de rollback.

---

# 11. Sophos — referência em exploit e ransomware protection

O Sophos Intercept X possui controles específicos para:

- ransomware;
- process hollowing;
- side-loading;
- shellcode;
- AMSI;
- comunicação com command-and-control;
- proteção de processos.

A documentação atual também expõe controles específicos de CryptoGuard e C2 interception. citeturn2search4turn2search17

### O que o C2L deve aprender

A arquitetura precisa detectar não apenas arquivos maliciosos, mas **técnicas de ataque**.

---

# 12. MITRE ATT&CK como referência de ameaças

O MITRE ATT&CK deve ser uma referência permanente para o C2L.

Em 2026, o Enterprise ATT&CK possui centenas de técnicas e sub-técnicas, organizadas em táticas que representam objetivos adversários. citeturn0search4turn0search13

A versão 19 também reforçou a separação entre **Stealth** e **Defense Impairment**, mostrando que impedir ou degradar mecanismos defensivos é uma categoria importante do comportamento adversário. citeturn0search14

Isso significa que o C2L deverá possuir uma matriz:

```
Técnica ATT&CK
      │
      ▼
Indicadores
      │
      ▼
Telemetria
      │
      ▼
Detecção
      │
      ▼
Resposta
```

Essa matriz deverá evoluir continuamente.

---

# 13. O atacante que o C2L precisa considerar

O Threat Model será baseado em um adversário capaz de:

### T1 — Malware tradicional

- executar ficheiros maliciosos;
- modificar ficheiros;
- persistir;
- tentar evitar detecção.

### T2 — Malware polimórfico

- alterar características do ficheiro;
- evitar hashes conhecidos;
- utilizar empacotamento ou ofuscação.

### T3 — Fileless malware

- utilizar ferramentas legítimas;
- executar código sem depender de um ficheiro malicioso tradicional.

### T4 — Living-off-the-Land

Abusar de componentes legítimos do sistema.

Exemplos incluem:

- PowerShell;
- scripts;
- ferramentas administrativas;
- processos legítimos.

A Microsoft destaca explicitamente que ataques modernos utilizam ferramentas legítimas e técnicas fileless que exigem detecção comportamental. citeturn0search7

### T5 — Ransomware

- modificar grande quantidade de ficheiros;
- destruir backups;
- tentar impedir recuperação;
- tentar desativar mecanismos de segurança.

### T6 — Exploitation

- explorar vulnerabilidades;
- obter execução;
- elevar privilégios;
- injetar código.

### T7 — Defense Impairment

O atacante tenta:

- parar o antivírus;
- terminar processos;
- alterar configurações;
- adicionar exclusões;
- apagar logs;
- modificar componentes;
- corromper bases de dados.

### T8 — Supply-chain

O código aparentemente legítimo pode ser comprometido antes de chegar ao utilizador.

### T9 — Ataque remoto

O atacante pode explorar serviços, credenciais ou protocolos para chegar ao endpoint.

### T10 — Ataques assistidos por IA

A evolução do mercado mostra que agentes de IA estão aumentando a velocidade e escala das operações ofensivas e defensivas. A CrowdStrike, por exemplo, já trata agentes de IA como uma nova superfície de segurança. citeturn2search15turn2search16

O C2L deverá manter essa possibilidade no Threat Model futuro.

---

# 14. Ativos que precisam ser protegidos

O C2L deverá proteger:

1. processo do antivírus;
2. configurações;
3. base de indicadores;
4. mecanismo de detecção;
5. quarentena;
6. logs;
7. credenciais;
8. chaves criptográficas;
9. mecanismos de atualização;
10. comunicação com serviços externos;
11. integridade dos binários;
12. dados do utilizador;
13. resultados das análises;
14. infraestrutura cloud futura.

---

# 15. Fronteiras de confiança

O sistema deverá considerar diferentes níveis de confiança:

```
                 INTERNET
                     │
              Untrusted Zone
                     │
             Threat Intelligence
                     │
              Network Boundary
                     │
             C2L Service/API
                     │
             Core Security Zone
                     │
          ┌──────────┴──────────┐
          │                     │
     Detection              Quarantine
          │                     │
          └──────────┬──────────┘
                     │
              User / OS
```

Nenhum dado externo deverá ser considerado confiável simplesmente por ter sido recebido de um serviço conhecido.

---

# 16. Principais ameaças contra o próprio C2L

## TM-001 — Desativação

**Ameaça:** malware tenta parar o C2L.

**Impacto:** perda completa de proteção.

**Proteções futuras:**

- tamper protection;
- componentes protegidos;
- serviços separados;
- watchdog;
- integridade;
- privilégios adequados;
- detecção de alteração.

---

## TM-002 — Manipulação da base de indicadores

**Ameaça:** alteração ou corrupção das definições.

**Impacto:** falsos negativos ou falsos positivos.

**Proteções:**

- integridade;
- assinatura digital;
- versionamento;
- atualização autenticada;
- rollback de definições.

---

## TM-003 — Comprometimento da atualização

**Ameaça:** atacante fornece uma atualização falsa.

**Impacto:** comprometimento total do endpoint.

**Proteções:**

- assinatura digital;
- cadeia de confiança;
- verificação de integridade;
- versionamento;
- proteção contra downgrade;
- canais seguros.

---

## TM-004 — Exploração do Scanner

**Ameaça:** ficheiro especialmente construído tenta explorar uma vulnerabilidade no analisador.

**Impacto:** execução de código com privilégios do C2L.

**Proteções:**

- isolamento;
- parsing defensivo;
- limites de recursos;
- sandboxing quando apropriado;
- fuzzing;
- privilégios mínimos.

Esta ameaça será especialmente importante porque **o antivírus precisa abrir e analisar conteúdo potencialmente hostil por definição**.

---

## TM-005 — Escape da quarentena

**Ameaça:** ficheiro isolado consegue ser executado ou restaurado de forma indevida.

**Impacto:** reinfecção.

**Proteções:**

- isolamento;
- nomes internos;
- permissões;
- metadados;
- integridade;
- restauração explícita.

---

## TM-006 — Manipulação dos logs

**Ameaça:** atacante apaga ou modifica evidências.

**Impacto:** perda de visibilidade e dificuldade de investigação.

**Proteções futuras:**

- logs protegidos;
- integridade;
- armazenamento remoto opcional;
- sequência de eventos;
- timestamps confiáveis.

---

## TM-007 — Falsos positivos

**Ameaça:** software legítimo é classificado como malicioso.

**Impacto:** perda de confiança e possível perda de dados.

**Proteções:**

- múltiplas evidências;
- classificação de confiança;
- allowlist controlada;
- reputação;
- feedback;
- análise contextual.

---

## TM-008 — Falsos negativos

**Ameaça:** malware passa pelas camadas de proteção.

**Impacto:** comprometimento do endpoint.

**Proteções:**

- defesa em profundidade;
- comportamento;
- telemetria;
- threat intelligence;
- atualização rápida;
- EDR;
- hunting.

---

# 17. Modelo de defesa recomendado

O C2L deverá evoluir para uma arquitetura de defesa em camadas:

```
                    ┌──────────────────────┐
                    │ Threat Intelligence  │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │ Cloud / Reputation   │
                    └──────────┬───────────┘
                               │
              ┌────────────────▼────────────────┐
              │      Behavioral Detection       │
              └────────────────┬────────────────┘
                               │
              ┌────────────────▼────────────────┐
              │      Heuristic / ML Layer       │
              └────────────────┬────────────────┘
                               │
              ┌────────────────▼────────────────┐
              │     Signature / Indicator       │
              └────────────────┬────────────────┘
                               │
              ┌────────────────▼────────────────┐
              │       Attack Surface           │
              │          Reduction              │
              └────────────────┬────────────────┘
                               │
              ┌────────────────▼────────────────┐
              │       Runtime / EDR             │
              └────────────────┬────────────────┘
                               │
              ┌────────────────▼────────────────┐
              │      Response / Recovery        │
              └─────────────────────────────────┘
```

Nenhuma camada deverá ser considerada suficiente isoladamente.

---

# 18. Atualização contínua contra novas ameaças

Este é um requisito estratégico do C2L.

O sistema não poderá depender apenas de atualizar o executável.

Deverá existir uma separação entre:

### Código

Mudanças estruturais no produto.

### Detection Content

Informações e regras que podem ser atualizadas rapidamente.

### Threat Intelligence

Informações sobre ameaças emergentes.

### Models

Modelos estatísticos ou ML que possam evoluir independentemente do executável quando tecnicamente viável.

### Policies

Configurações de prevenção e resposta.

Conceito:

```
             C2L Engine
                 │
      ┌──────────┼───────────┐
      │          │           │
   Rules      Models      Policies
      │          │           │
      └──────────┼───────────┘
                 │
          Threat Intelligence
                 │
                 ▼
          Continuous Update
```

---

# 19. Pipeline de atualização futura

O mecanismo futuro deverá seguir aproximadamente:

```
Threat Discovery
       │
       ▼
Analysis / Validation
       │
       ▼
Rule / Indicator Creation
       │
       ▼
Automated Tests
       │
       ▼
Security Validation
       │
       ▼
Signing
       │
       ▼
Distribution
       │
       ▼
Endpoint
       │
       ▼
Monitoring
       │
       └──────────────► Feedback
```

Uma definição nova não deverá chegar ao endpoint sem validação e autenticação.

---

# 20. Telemetria e feedback

A evolução contínua exigirá feedback.

O C2L poderá futuramente coletar, de forma configurável e minimizada:

- hashes;
- eventos de deteção;
- características técnicas;
- resultados de análise;
- versões;
- indicadores de desempenho;
- sinais comportamentais necessários à deteção.

Dados pessoais ou conteúdo dos ficheiros não deverão ser enviados por padrão sem justificativa explícita.

---

# 21. C2L e privacidade

O benchmark de mercado mostra que cloud e telemetria são importantes para proteção moderna.

Porém:

> **Mais telemetria não significa automaticamente mais segurança.**

O C2L deverá procurar o equilíbrio:

**Máxima informação de segurança necessária + mínima exposição de dados.**

Esse princípio deverá ser incorporado na arquitetura cloud futura.

---

# 22. Prioridade de implementação

## Nível 1 — V0.1

- hash;
- indicadores;
- deteção;
- classificação;
- quarentena;
- logs;
- integridade básica;
- CLI;
- testes.

## Nível 2 — V0.2

- assinaturas;
- heurística;
- reputação local;
- melhor classificação de risco;
- conteúdo de deteção atualizável.

## Nível 3 — V0.3/V0.4

- Linux;
- monitorização;
- proteção em tempo real;
- análise de processos;
- comportamento.

## Nível 4 — V0.5/V0.6

- EDR;
- threat intelligence;
- cloud;
- resposta automatizada;
- atualização segura.

## Nível 5 — V0.7+

- UI;
- hunting;
- rollback;
- gestão centralizada;
- identidade;
- cloud workloads;
- Android.

---

# 23. O que não devemos tentar fazer agora

O benchmark também revela o que seria um erro estratégico.

Não devemos tentar construir imediatamente:

- um SOC completo;
- um XDR completo;
- IA generativa para análise de malware;
- driver/kernel complexo;
- sandbox cloud;
- proteção de identidade;
- proteção de cloud workloads;
- infraestrutura global de threat intelligence;
- centenas de regras ATT&CK;
- milhões de indicadores.

Essas capacidades poderão fazer parte da visão de longo prazo, mas seriam desproporcionais para a primeira versão.

---

# 24. O verdadeiro diferencial arquitetural do C2L

O objetivo não deve ser:

> "Criar um antivírus com muitas funcionalidades."

O objetivo deve ser:

> **Criar uma arquitetura de segurança capaz de aprender, atualizar e responder continuamente à evolução das ameaças.**

Isso implica separar:

**Engine ≠ Detection Content ≠ Threat Intelligence ≠ Platform ≠ UI**

Essa separação será um dos princípios fundamentais do C2L.

---

# 25. Referências estratégicas

As referências externas utilizadas neste benchmark incluem:

- MITRE ATT&CK;
- NIST Cybersecurity Framework;
- AV-TEST;
- AV-Comparatives;
- Microsoft Defender for Endpoint;
- CrowdStrike Falcon;
- SentinelOne Singularity;
- Palo Alto Cortex XDR;
- Bitdefender GravityZone;
- Sophos Intercept X.

O benchmark deverá ser revisto periodicamente, pois as capacidades dos produtos e as técnicas dos adversários evoluem continuamente.

---

# 26. Conclusão

O benchmark de 2026 demonstra que a fronteira atual da proteção de endpoint está muito além do antivírus tradicional.

Os melhores produtos combinam:

**Prevenção + comportamento + inteligência + telemetria + EDR + resposta + recuperação + autoproteção + atualização contínua.**

O C2L deverá seguir essa direção.

Entretanto, o projeto não tentará reproduzir imediatamente a escala dos líderes de mercado.

A estratégia será construir uma fundação pequena, segura e extensível, na qual novas capacidades possam ser incorporadas continuamente.

A arquitetura final desejada é:

```
             C2L Security Platform

       ┌───────────────────────────────┐
       │ Threat Intelligence           │
       ├───────────────────────────────┤
       │ Cloud / Reputation             │
       ├───────────────────────────────┤
       │ Detection + Behavioral Engine │
       ├───────────────────────────────┤
       │ EDR / Response                 │
       ├───────────────────────────────┤
       │ Prevention / ASR               │
       ├───────────────────────────────┤
       │ Core Security Engine           │
       ├───────────────────────────────┤
       │ Platform Adapters              │
       └───────────────────────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
      Windows      Linux      Android
```

O princípio fundamental passa a ser:

> **O C2L não precisa conhecer todas as ameaças existentes hoje. Precisa ser projetado para receber novas evidências, novas regras, novos modelos e novos mecanismos de resposta sem precisar ser reconstruído.**

Esse princípio será utilizado como requisito arquitetural para todas as próximas etapas.
