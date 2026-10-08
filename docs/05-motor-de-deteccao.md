# 05 — Motor de Detecção

**Projeto:** C2L Antivirus  
**Versão do documento:** 1.0  
**Data:** 08/10/2026  
**Status:** Aprovado

## 1. Objetivo

O Motor de Detecção é o componente responsável por transformar evidências técnicas em uma avaliação de segurança sobre arquivos, processos, eventos e, futuramente, cadeias de ataque.

O C2L não deverá depender de uma única técnica de detecção. O motor será construído como uma arquitetura em camadas, na qual diferentes fontes de evidência podem ser combinadas para aumentar a capacidade de detecção e reduzir falsos positivos.

O princípio central é:

> **Detectar não significa apenas encontrar um arquivo conhecido. Significa avaliar evidências, contexto e comportamento para determinar risco e decidir uma ação proporcional.**

A arquitetura deverá permitir adicionar novas técnicas de detecção sem reescrever o núcleo do sistema.

---

## 2. Princípios

O Motor de Detecção seguirá os seguintes princípios:

1. **Defesa em profundidade** — nenhuma técnica será tratada como suficiente isoladamente.
2. **Separação entre código e conteúdo de detecção** — indicadores, regras, políticas e modelos deverão poder ser atualizados independentemente do executável principal.
3. **Evidência explicável** — uma decisão deverá registrar quais evidências contribuíram para o resultado.
4. **Detecção incremental** — o motor deverá poder parar cedo quando existir evidência suficientemente forte ou continuar a análise quando o risco for incerto.
5. **Fail-safe** — falhas de componentes auxiliares não deverão silenciosamente transformar uma detecção em permissão.
6. **Baixo privilégio** — a análise deverá utilizar o menor nível de privilégio necessário.
7. **Resistência a entradas hostis** — qualquer arquivo analisado deve ser considerado potencialmente malformado ou malicioso.
8. **Atualização contínua** — o conteúdo de detecção deverá evoluir sem depender de uma nova versão completa do produto.
9. **Privacidade** — informações enviadas para serviços externos deverão ser minimizadas e controladas por política.
10. **Testabilidade** — cada camada deverá possuir testes unitários, de integração e, quando aplicável, corpus de avaliação.

---

## 3. O que é uma evidência

Uma evidência é um fato observado pelo C2L que pode contribuir para uma decisão de segurança.

Exemplos:

- hash SHA-256 conhecido como malicioso;
- correspondência com uma assinatura;
- arquivo executável com características suspeitas;
- processo que executa uma sequência incomum de ações;
- conexão com infraestrutura associada a ameaça;
- tentativa de modificar mecanismos de proteção;
- criação de persistência;
- combinação de eventos compatível com uma técnica do MITRE ATT&CK.

Uma evidência não precisa, isoladamente, determinar a decisão final.

O motor deverá preservar a distinção entre:

**Evidência → Avaliação → Decisão → Ação**

---

## 4. Camadas de detecção

A arquitetura conceitual será:

```text
                 Objeto / Evento
                       │
                       ▼
             ┌───────────────────┐
             │ Aquisição e       │
             │ normalização      │
             └─────────┬─────────┘
                       │
                       ▼
             ┌───────────────────┐
             │ Indicadores /     │
             │ Hashes            │
             └─────────┬─────────┘
                       │
                       ▼
             ┌───────────────────┐
             │ Análise estática  │
             │ / assinaturas     │
             └─────────┬─────────┘
                       │
                       ▼
             ┌───────────────────┐
             │ Reputação /       │
             │ Threat Intel      │
             └─────────┬─────────┘
                       │
                       ▼
             ┌───────────────────┐
             │ Heurística / ML   │
             └─────────┬─────────┘
                       │
                       ▼
             ┌───────────────────┐
             │ Comportamento /   │
             │ Runtime           │
             └─────────┬─────────┘
                       │
                       ▼
             ┌───────────────────┐
             │ Correlação de     │
             │ eventos           │
             └─────────┬─────────┘
                       │
                       ▼
             ┌───────────────────┐
             │ Risk Engine       │
             └─────────┬─────────┘
                       │
                       ▼
       Allow / Monitor / Quarantine / Block
       / Isolate / Remediate / Recover
```

Nem todas as camadas estarão disponíveis na V0.1. A arquitetura deverá, porém, reservar contratos claros para sua futura implementação.

---

## 5. Hashes e indicadores

### 5.1 Objetivo

Hashes são uma forma rápida de reconhecer objetos previamente identificados.

A primeira versão utilizará principalmente:

- SHA-256;
- futuramente outros identificadores quando houver justificativa técnica.

O hash deverá ser calculado durante a aquisição do objeto e poderá ser usado para consulta no Indicator Store.

### 5.2 Limitação

Hash não pode ser tratado como principal mecanismo de detecção de um antivírus moderno.

Uma alteração mínima no arquivo normalmente produz um novo hash. Portanto:

```text
Hash conhecido
    ↓
Excelente para reconhecer o conhecido
    ↓
Fraco para reconhecer variantes desconhecidas
```

O hash deverá ser uma camada rápida de alta confiança, não a estratégia completa de detecção.

### 5.3 Indicadores

O Indicator Store deverá suportar, progressivamente:

- hashes;
- identificadores de arquivos;
- domínios;
- endereços IP;
- URLs;
- certificados;
- nomes ou características de processos;
- indicadores associados a técnicas;
- outros tipos de IOC/IOA definidos futuramente.

Cada indicador deverá possuir metadados mínimos:

- tipo;
- valor;
- origem;
- versão;
- data de publicação;
- confiança;
- classificação;
- validade, quando aplicável;
- assinatura/integridade do conteúdo.

---

## 6. Assinaturas

Assinaturas representam padrões conhecidos que podem identificar famílias ou características de ameaça.

Podem incluir, conforme a evolução do projeto:

- padrões binários;
- características estruturais;
- regras sobre metadados;
- regras sobre conteúdo;
- combinações de características;
- regras orientadas a eventos.

As assinaturas deverão possuir:

- identificador único;
- versão;
- descrição;
- severidade;
- confiança;
- origem;
- estado ativo/inativo;
- testes associados;
- versão mínima do motor, quando necessário.

### Regra importante

As regras de detecção não devem exigir alteração do código-fonte sempre que surgir uma nova ameaça.

O objetivo é aproximar:

```text
Nova ameaça
   ↓
Nova regra/indicador
   ↓
Validação
   ↓
Assinatura da atualização
   ↓
Distribuição
   ↓
Endpoint atualizado
```

em vez de:

```text
Nova ameaça
   ↓
Alteração do código
   ↓
Novo build
   ↓
Nova versão do produto
   ↓
Distribuição
```

---

## 7. Análise estática

A análise estática examina um objeto sem executá-lo.

Dependendo do tipo de arquivo e da plataforma, poderá considerar:

- tipo real do arquivo;
- tamanho;
- entropia;
- estrutura;
- cabeçalhos;
- seções;
- imports/dependências;
- certificados;
- metadados;
- macros e scripts;
- características de empacotamento;
- indicadores de ofuscação;
- relações entre componentes.

A análise estática deve produzir **features/evidências**, e não necessariamente uma decisão isolada.

Exemplo conceitual:

```text
Arquivo PE
 ├─ certificado ausente
 ├─ seção incomum
 ├─ entropia elevada
 ├─ comportamento esperado desconhecido
 └─ indicador externo associado a ameaça
                    │
                    ▼
              conjunto de evidências
```

---

## 8. Heurística

A heurística procura características suspeitas sem depender de uma correspondência exata com uma ameaça conhecida.

Exemplos de categorias:

- combinação incomum de características;
- tentativa de executar ações de alto risco;
- manipulação de mecanismos de proteção;
- criação de persistência;
- comportamento compatível com ransomware;
- execução de scripts ou comandos em contexto suspeito.

A heurística deve ser calibrada para evitar que uma única característica genérica produza bloqueios excessivos.

### Confiança e severidade

O motor deverá distinguir pelo menos:

- **confidence** — quão confiável é a evidência;
- **severity** — quão grave é o comportamento, caso seja malicioso.

Esses valores não são equivalentes.

---

## 9. Machine Learning

Machine Learning poderá ser incorporado posteriormente para classificação de arquivos, eventos e comportamentos.

O ML não deverá ser tratado como uma caixa-preta que substitui todas as outras camadas.

A arquitetura deverá permitir:

```text
Features
   ↓
Modelo
   ↓
Score / classificação
   ↓
Evidência explicável
   ↓
Risk Engine
```

Cada modelo deverá possuir:

- identificador;
- versão;
- tipo;
- plataforma;
- conjunto de features esperado;
- versão do dataset, quando aplicável;
- métricas conhecidas;
- data de treinamento/validação;
- assinatura/integridade.

Modelos deverão ser tratados como **conteúdo atualizável e verificável**, não como arquivos arbitrários aceitos pelo endpoint.

---

## 10. Reputação e Threat Intelligence

A reputação adiciona contexto externo ou histórico ao objeto analisado.

Exemplos:

- arquivo conhecido;
- domínio associado a malware;
- infraestrutura relacionada a uma campanha;
- certificado associado a atividade maliciosa;
- indicador recentemente observado.

Threat Intelligence deverá ir além de uma lista de bloqueio. O objetivo futuro é relacionar:

```text
Indicador
   ↓
Ameaça / campanha
   ↓
Ator ou infraestrutura
   ↓
Técnicas observadas
   ↓
Contexto
   ↓
Risco
```

A integração externa deverá ser opcional e controlada por política, permitindo funcionamento offline com capacidade reduzida.

O C2L deverá evitar o envio desnecessário de arquivos completos para serviços externos.

---

## 11. Detecção comportamental

A detecção comportamental observa o que acontece em vez de analisar somente o objeto inicial.

Exemplos de sinais futuros:

- criação de processos;
- relações pai/filho;
- alterações de arquivos;
- alterações de registro/configuração;
- criação de persistência;
- alterações de serviços;
- acesso anômalo a processos;
- tentativas de desativar mecanismos de segurança;
- atividade de rede;
- execução de scripts;
- sequências associadas a técnicas do MITRE ATT&CK.

O comportamento deverá ser analisado temporalmente.

Uma ação isolada pode ser legítima:

```text
Processo A → cria arquivo
```

Uma cadeia pode ser significativamente mais suspeita:

```text
Processo A
   ↓
execução de script
   ↓
download
   ↓
criação de processo
   ↓
persistência
   ↓
alteração de mecanismo de segurança
```

Por isso, a arquitetura deverá suportar correlação de eventos.

---

## 12. Correlação e Attack Chain

A correlação é uma das principais evoluções do C2L.

Em vez de analisar cada evento independentemente, o motor deverá poder relacionar:

```text
Evento 1 ─┐
Evento 2 ─┼──► contexto ─► técnica ─► cadeia ─► risco
Evento 3 ─┤
Evento 4 ─┘
```

A matriz futura deverá seguir o modelo:

**Técnica → Indicadores → Telemetria → Detecção → Resposta**

Essa estrutura permitirá relacionar o motor ao MITRE ATT&CK sem transformar o framework em uma simples lista de assinaturas.

---

## 13. Risk Engine

O Risk Engine será responsável por transformar múltiplas evidências em uma avaliação consolidada.

Modelo conceitual:

```text
Evidence[]
   ↓
Normalização
   ↓
Confiança
   ↓
Severidade
   ↓
Contexto
   ↓
Correlação
   ↓
Risk Score
   ↓
Decision
```

Um possível resultado lógico:

| Risk | Interpretação | Ação potencial |
|---|---|---|
| 0–19 | Muito baixo | Allow |
| 20–39 | Baixo | Allow / Monitor |
| 40–59 | Moderado | Monitor / análise adicional |
| 60–79 | Alto | Quarantine / Block |
| 80–100 | Crítico | Block / Quarantine / Isolate |

Essas faixas são **iniciais e não constituem valores definitivos**. A calibração deverá ser baseada em testes e telemetria.

### Importante

O score não deverá ser uma simples soma arbitrária.

A arquitetura deverá permitir:

- pesos por tipo de evidência;
- confiança;
- severidade;
- contexto;
- idade da informação;
- reputação;
- correlação;
- políticas da máquina;
- exceções controladas.

Evidências independentes e de alta confiança poderão elevar o risco de maneira significativa.

---

## 14. Decision Engine

O Risk Engine produz avaliação; o Decision Engine determina o que fazer de acordo com política.

Exemplo:

```text
Risk = 87
Confidence = High
Evidence = Known malicious hash
Policy = Default
        ↓
Decision = Quarantine
```

Outro exemplo:

```text
Risk = 48
Confidence = Medium
Evidence = Heurística + contexto desconhecido
        ↓
Decision = Monitor / Deep Analysis
```

A decisão deverá ser separada da detecção para permitir que políticas diferentes utilizem as mesmas evidências.

---

## 15. Ações

As ações futuras do C2L poderão incluir:

- **Allow** — permitir;
- **Monitor** — permitir com observação;
- **Deep Analysis** — análise adicional;
- **Quarantine** — isolar o objeto;
- **Block** — impedir execução/ação;
- **Terminate** — encerrar atividade maliciosa;
- **Isolate** — isolar endpoint ou recurso;
- **Remediate** — desfazer alterações;
- **Recover** — restaurar estado seguro.

A V0.1 utilizará apenas o subconjunto necessário para:

- permitir;
- detectar;
- classificar;
- colocar em quarentena;
- registrar.

---

## 16. Explicabilidade

Cada decisão deverá poder produzir um registro semelhante a:

```text
Decision: QUARANTINE
Risk: 91
Confidence: HIGH

Evidence:
- Known malicious SHA-256
- Indicator source: Local DB
- Indicator confidence: High
- Detection rule: C2L-HASH-0001
- Policy: Default
```

No futuro, uma decisão comportamental deverá indicar a cadeia relevante:

```text
Decision: BLOCK
Risk: 88

Evidence:
- Suspicious script execution
- Child process anomaly
- Persistence attempt
- Security control modification attempt

Technique:
- Defense Impairment
- Persistence
```

Isso será importante para auditoria, suporte, investigação e melhoria das regras.

---

## 17. Pipeline de detecção

O pipeline conceitual será:

```text
1. Acquire
      ↓
2. Normalize
      ↓
3. Identify
      ↓
4. Hash / Indicators
      ↓
5. Static Analysis
      ↓
6. Reputation / Threat Intel
      ↓
7. Heuristic / ML
      ↓
8. Runtime Behavior
      ↓
9. Correlation
      ↓
10. Risk Engine
      ↓
11. Decision Engine
      ↓
12. Action
      ↓
13. Event / Audit
```

O pipeline deverá suportar execução parcial.

Por exemplo, um hash confirmado como malicioso poderá permitir uma decisão imediata sem executar todas as análises posteriores.

Por outro lado, uma evidência ambígua deverá permitir aprofundamento.

---

## 18. Early Exit e análise progressiva

Para equilibrar segurança e desempenho, o motor deverá utilizar análise progressiva.

### Exemplo de alta confiança

```text
SHA-256
   ↓
Known malicious
   ↓
High confidence
   ↓
Quarantine
```

### Exemplo de baixa confiança

```text
Arquivo desconhecido
   ↓
Análise estática
   ↓
Heurística
   ↓
Reputação
   ↓
Classificação
```

### Exemplo futuro

```text
Evento suspeito
   ↓
Monitoramento
   ↓
Mais evidências
   ↓
Correlação
   ↓
Block / Remediate
```

---

## 19. Separação entre Engine, Detection Content e Threat Intelligence

O C2L deverá manter três conceitos distintos:

### Engine

Código executável que implementa a lógica de análise.

### Detection Content

Conteúdo que define o que procurar:

- regras;
- assinaturas;
- indicadores;
- políticas;
- modelos.

### Threat Intelligence

Contexto sobre ameaças:

- campanhas;
- infraestrutura;
- técnicas;
- atores;
- relações entre indicadores.

Essa separação permite que o produto evolua continuamente sem transformar toda atualização de conhecimento em atualização do executável.

---

## 20. Segurança do conteúdo de detecção

Conteúdo de detecção será considerado parte da superfície de segurança do C2L.

Atualizações deverão futuramente possuir:

- integridade;
- autenticidade;
- versão;
- compatibilidade;
- validação;
- assinatura digital;
- proteção contra downgrade;
- mecanismo de rollback seguro.

O endpoint não deverá aceitar silenciosamente conteúdo não confiável.

Uma atualização comprometida pode transformar um sistema de segurança em um vetor de ataque.

---

## 21. Offline e Cloud-Enhanced

O C2L deverá suportar dois modos conceituais.

### Offline

Utiliza:

- engine local;
- regras locais;
- indicadores locais;
- modelos locais;
- cache de reputação.

Vantagens:

- privacidade;
- funcionamento sem Internet;
- menor dependência externa.

Limitações:

- inteligência potencialmente desatualizada;
- menor contexto global.

### Cloud-Enhanced

Pode adicionar:

- reputação;
- threat intelligence;
- análise de contexto;
- correlação global;
- atualização rápida;
- modelos mais avançados.

A arquitetura deverá garantir que a ausência do serviço externo não provoque falhas silenciosas de segurança.

---

## 22. Falsos positivos e falsos negativos

O motor deverá tratar os dois tipos de erro como métricas de primeira classe.

### False Positive

Objeto legítimo classificado como ameaça.

Impactos:

- interrupção de trabalho;
- perda de confiança;
- quarentena indevida;
- custo operacional.

### False Negative

Ameaça que não é detectada.

Impactos:

- comprometimento do sistema;
- perda de dados;
- persistência;
- movimento lateral;
- dano operacional.

O objetivo não será simplesmente maximizar a detecção.

O objetivo será encontrar um equilíbrio mensurável entre:

```text
Detection
Precision
Recall
False Positives
False Negatives
Performance
Latency
```

---

## 23. Métricas

O Motor de Detecção deverá ser avaliado por métricas técnicas.

### Segurança

- taxa de detecção;
- false positive rate;
- false negative rate;
- cobertura de técnicas ATT&CK;
- cobertura por categoria de ameaça;
- tempo para incorporar novo indicador.

### Performance

- latência por arquivo;
- arquivos processados por segundo;
- CPU;
- memória;
- I/O;
- impacto no sistema.

### Operação

- tempo de atualização;
- sucesso/falha das atualizações;
- capacidade de rollback;
- disponibilidade dos componentes;
- qualidade dos logs.

Nenhuma métrica isolada será suficiente para definir a qualidade do produto.

---

## 24. V0.1

A V0.1 implementará somente o núcleo mínimo necessário.

### Incluído

- aquisição de arquivo;
- metadados básicos;
- SHA-256;
- Indicator Store local;
- consulta de indicadores;
- classificação de correspondência;
- Risk Engine básico;
- decisão de quarentena;
- logs;
- relatório;
- testes automatizados.

### Preparado, mas não implementado

- assinaturas avançadas;
- análise estática profunda;
- heurística;
- ML;
- reputação;
- Threat Intelligence;
- comportamento;
- correlação;
- resposta avançada;
- atualização remota de conteúdo.

---

## 25. Contrato conceitual do motor

A interface interna deverá evoluir em torno de conceitos semelhantes a:

```text
ScanTarget
DetectionEvidence
DetectionResult
RiskAssessment
Decision
Action
```

Exemplo conceitual:

```text
ScanTarget
    ↓
Detector[]
    ↓
DetectionEvidence[]
    ↓
RiskEngine
    ↓
RiskAssessment
    ↓
DecisionEngine
    ↓
Decision
    ↓
Action
```

O contrato deve permitir adicionar novos detectores sem alterar o fluxo principal.

---

## 26. Extensibilidade

Os detectores deverão seguir um modelo de componentes independentes.

Exemplo futuro:

```text
HashDetector
SignatureDetector
StaticDetector
ReputationDetector
HeuristicDetector
MLDetector
BehaviorDetector
CorrelationDetector
```

O Orchestrator não deverá precisar conhecer detalhes internos de cada detector.

Isso permitirá:

- ativar/desativar camadas;
- testar cada detector isoladamente;
- atualizar componentes;
- comparar versões;
- executar análises em paralelo quando seguro;
- introduzir novas tecnologias sem reescrever o núcleo.

---

## 27. Testabilidade

Cada camada deverá ser testável separadamente.

### Unitários

- cálculo de hash;
- consulta de indicador;
- parser;
- cálculo de score;
- regras de decisão.

### Integração

- scanner + indicator store;
- scanner + quarantine;
- detector + risk engine;
- atualização + validação.

### Segurança

- arquivos malformados;
- arquivos excessivamente grandes;
- entradas truncadas;
- formatos inesperados;
- corrupção de banco;
- conteúdo de atualização inválido.

### Regressão

Toda correção de detecção deverá gerar um caso de teste permanente.

O objetivo é evitar que uma melhoria para uma ameaça introduza regressões em outras categorias.

---

## 28. Laboratório de avaliação

O desenvolvimento deverá utilizar um ambiente controlado.

A estratégia inicial deverá priorizar:

- arquivos benignos;
- casos sintéticos;
- datasets autorizados;
- EICAR quando aplicável;
- amostras de malware somente em ambiente isolado e quando legalmente apropriado;
- máquinas virtuais;
- snapshots;
- rede controlada;
- ausência de dados pessoais.

O repositório não deverá armazenar malware real.

Os testes de segurança devem avaliar o C2L sem transformar o próprio projeto em distribuição de conteúdo malicioso.

---

## 29. Relação com MITRE ATT&CK

A evolução do motor deverá manter uma matriz:

| Técnica | Indicadores | Telemetria | Detecção | Resposta |
|---|---|---|---|---|
| Persistence | processos/serviços/artefatos | eventos de sistema | regras comportamentais | block/remediate |
| Defense Impairment | alterações em mecanismos de segurança | eventos de configuração | behavioral detector | block/isolate |
| Credential Access | acesso anômalo a processos/credenciais | process telemetry | behavior/correlation | block |
| Command and Control | rede/domínios/infraestrutura | network telemetry | reputation/correlation | block |
| Impact | alterações massivas de arquivos | file telemetry | behavioral detector | block/recover |

Essa matriz deverá evoluir juntamente com o produto.

---

## 30. Roadmap técnico do motor

### V0.1

- SHA-256;
- Indicator Store;
- matching;
- Risk Engine básico;
- quarantine;
- logs;
- testes.

### V0.2

- regras de assinatura;
- conteúdo versionado;
- metadados avançados;
- pipeline de atualização local;
- melhor explicabilidade.

### V0.3

- análise estática;
- heurísticas;
- reputação;
- primeira integração de Threat Intelligence.

### V0.4

- monitoramento de processos;
- eventos comportamentais;
- correlação temporal;
- primeiras técnicas ATT&CK.

### V0.5+

- ML;
- EDR;
- resposta automática;
- remediation;
- recovery;
- cloud-enhanced intelligence;
- Linux;
- expansão para Android;
- políticas avançadas.

As versões são indicativas e poderão ser alteradas conforme os resultados dos testes.

---

## 31. Decisões arquiteturais registradas

1. O C2L não será baseado exclusivamente em assinaturas.
2. Hashes serão uma camada rápida de alta confiança, não o motor completo.
3. Detection Content será separado do executável.
4. Threat Intelligence será separada do código de detecção.
5. Evidências serão separadas de decisões.
6. Risk Engine será separado do Decision Engine.
7. O comportamento futuro será analisado como sequência/cadeia, e não apenas como eventos isolados.
8. O motor deverá ser preparado para operação offline.
9. Serviços externos serão complementares, não uma dependência estrutural obrigatória.
10. Atualizações de segurança deverão ser autenticadas e verificadas.
11. O scanner será tratado como superfície de ataque e deverá receber controles específicos.
12. Toda evolução importante deverá ser acompanhada de testes de regressão.

---

## 32. Critério de sucesso

O Motor de Detecção será considerado arquiteturalmente bem-sucedido quando permitir:

- adicionar uma nova regra sem modificar o núcleo do produto;
- adicionar um novo tipo de detector sem reescrever o Orchestrator;
- combinar múltiplas evidências;
- explicar por que uma decisão foi tomada;
- funcionar sem Internet nas capacidades locais;
- receber conteúdo atualizado com integridade e autenticidade;
- medir false positives e false negatives;
- evoluir de análise de arquivos para análise comportamental;
- mapear detecções para técnicas de ataque;
- suportar Windows inicialmente sem comprometer a futura portabilidade para Linux e Android.

---

## 33. Próximo documento

O próximo documento será:

**06 — Segurança**

Ele deverá detalhar a segurança do próprio C2L, incluindo:

- modelo de privilégios;
- proteção contra tampering;
- integridade dos binários;
- segurança do Indicator Store;
- segurança de atualizações;
- proteção da quarentena;
- logs e auditoria;
- comunicação externa;
- gestão de chaves;
- isolamento;
- secure bootstrapping;
- defesa contra exploração do próprio scanner.

