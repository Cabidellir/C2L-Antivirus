# 07 — Estratégia de Testes

**Projeto:** C2L Antivirus  
**Versão do documento:** 1.0  
**Data:** 08/10/2026  
**Status:** Aprovado

## 1. Objetivo

Este documento define como o C2L será validado durante o desenvolvimento.

O objetivo não é apenas verificar se o programa "funciona". O C2L deverá demonstrar, por evidências reproduzíveis, que:

- detecta corretamente o que foi projetado para detectar;
- preserva arquivos legítimos;
- trata erros de forma segura;
- protege a quarentena;
- não confia em dados inválidos;
- suporta entradas hostis;
- mantém desempenho aceitável;
- não introduz regressões;
- consegue evoluir seu conteúdo de detecção com segurança.

A regra central será:

> **Nenhuma capacidade de segurança será considerada pronta apenas porque funciona em um exemplo.**

---

## 2. Princípios

A estratégia seguirá estes princípios:

1. testes automatizados sempre que possível;
2. testes reproduzíveis;
3. testes negativos são tão importantes quanto testes positivos;
4. segurança deve ser testada como requisito funcional;
5. regressões devem permanecer reproduzíveis;
6. resultados devem possuir métricas;
7. arquivos potencialmente perigosos serão analisados apenas em laboratório isolado;
8. testes destrutivos nunca serão realizados no computador pessoal de desenvolvimento;
9. um teste que falha deve produzir informação suficiente para diagnóstico;
10. alterações no motor de detecção devem ser acompanhadas por testes de regressão.

---

## 3. Pirâmide de testes

O C2L utilizará diferentes níveis de testes.

~~~text
                 E2E / Laboratory
                       ▲
                  Integration
                       ▲
                 Component Tests
                       ▲
                  Unit Tests
                       ▲
                Static Analysis
~~~

### Base

Testes rápidos, numerosos e executados a cada alteração.

### Meio

Testes de componentes e integração.

### Topo

Testes completos em máquinas virtuais e laboratório controlado.

Quanto mais alto o teste, maior o custo de execução.

---

## 4. Categorias

A estratégia contempla:

- testes unitários;
- testes de componentes;
- testes de integração;
- testes end-to-end;
- testes de regressão;
- testes de segurança;
- fuzzing;
- testes de performance;
- testes de compatibilidade;
- testes de atualização;
- testes de quarentena;
- testes de recuperação;
- testes de confiabilidade;
- testes de instalação/desinstalação.

---

## 5. Testes unitários

Cada componente crítico deverá possuir testes unitários.

Exemplos:

### Hashing

- arquivo vazio;
- arquivo pequeno;
- arquivo grande;
- conteúdo alterado;
- erro de leitura.

### Scanner

- arquivo existente;
- arquivo inexistente;
- diretório vazio;
- diretório recursivo;
- caminho inválido;
- permissão negada.

### Indicator Store

- indicador válido;
- indicador inválido;
- duplicidade;
- corrupção;
- versão incompatível.

### Risk Engine

- evidência única;
- múltiplas evidências;
- conflito de evidências;
- ausência de evidência;
- classificação desconhecida.

### Quarantine

- entrada válida;
- arquivo inexistente;
- colisão de identificador;
- restauração autorizada;
- restauração inválida.

---

## 6. Testes negativos

O C2L deverá testar explicitamente situações que não deveriam funcionar.

Exemplos:

- caminho inexistente;
- caminho fora da área permitida;
- arquivo sem permissão;
- Indicator Store corrompido;
- Detection Content inválido;
- atualização sem assinatura;
- pacote incompatível;
- versão inferior à permitida;
- resposta externa inválida;
- arquivo excessivamente grande;
- timeout;
- falta de espaço;
- interrupção durante uma operação.

Resultado esperado:

> **A falha deverá ocorrer de maneira previsível e segura.**

---

## 7. Testes de componentes

Componentes serão testados isoladamente de suas dependências externas sempre que possível.

Exemplo:

~~~text
Scanner
  ↓
Mock Indicator Store
  ↓
Detection Engine
  ↓
Expected Result
~~~

Isso permite identificar rapidamente se o problema está:

- na leitura;
- no hashing;
- no indicador;
- na classificação;
- na decisão.

---

## 8. Testes de integração

Deverão validar a interação entre componentes reais.

Exemplo V0.1:

~~~text
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
Detection
 ↓
Risk
 ↓
Quarantine
 ↓
Report
~~~

Casos de integração deverão validar tanto sucesso quanto falhas intermediárias.

---

## 9. Testes end-to-end

Testes E2E deverão representar cenários completos.

Exemplo:

~~~text
Arquivo de teste
      ↓
C2L scan
      ↓
Identificação
      ↓
Classificação
      ↓
Ação
      ↓
Log
      ↓
Relatório
~~~

O teste deverá verificar o resultado final e também os estados intermediários críticos.

---

## 10. Corpus de testes

O projeto deverá possuir um corpus versionado de arquivos de teste.

Categorias:

### Benignos

- documentos;
- imagens;
- arquivos compactados;
- executáveis legítimos;
- scripts benignos;
- arquivos vazios;
- arquivos grandes.

### Malformados

- cabeçalhos inválidos;
- estruturas truncadas;
- tamanhos inconsistentes;
- compressões inválidas;
- formatos inesperados.

### Detecção

Para V0.1, o corpus deverá utilizar principalmente:

- indicadores sintéticos;
- arquivos de teste controlados;
- hashes conhecidos de arquivos de laboratório;
- EICAR quando apropriado para validar integração de detecção antivírus.

A criação ou distribuição de malware real não fará parte do repositório.

---

## 11. Laboratório de segurança

Testes de maior risco deverão ocorrer em ambiente isolado.

Modelo:

~~~text
Development Machine
        │
        ▼
   Build Artifact
        │
        ▼
 Isolated Test VM
        │
   ┌────┴────┐
   ▼         ▼
Windows    Linux
 VM          VM
~~~

Características:

- máquinas virtuais descartáveis;
- snapshots;
- rede controlada;
- dados de teste separados;
- sem credenciais pessoais;
- sem arquivos pessoais;
- possibilidade de restauração completa.

O laboratório deverá ser separado do ambiente cotidiano de desenvolvimento.

---

## 12. Regra para malware real

O projeto não deverá utilizar malware real nas primeiras fases.

Quando pesquisa de amostras reais se tornar necessária, ela deverá ocorrer somente:

- em laboratório dedicado;
- em máquinas isoladas;
- com snapshots;
- sem acesso desnecessário à rede;
- sem credenciais pessoais;
- com procedimentos documentados.

O objetivo será estudar comportamento defensivamente, nunca produzir ou aperfeiçoar malware.

---

## 13. Testes de segurança

Os testes deverão verificar se o próprio C2L pode ser comprometido.

Áreas:

- permissões;
- configuração;
- atualização;
- quarentena;
- logs;
- parsing;
- comunicação;
- Indicator Store;
- Detection Content;
- privilégios.

Exemplos de perguntas:

- Um utilizador comum consegue modificar regras?
- Uma atualização não assinada é aceita?
- Um arquivo pode escapar da quarentena?
- Um caminho malformado consegue atingir outro diretório?
- Uma entrada gigantesca derruba o scanner?
- Uma falha de parsing resulta em classificação "Clean"?
- Uma operação privilegiada pode ser solicitada com parâmetros arbitrários?

---

## 14. Fuzzing

Fuzzing será especialmente importante porque o C2L processará dados não confiáveis.

Alvos futuros:

- parsers;
- formatos de arquivo;
- caminhos;
- Indicator Store;
- Detection Content;
- protocolos;
- configurações;
- APIs internas.

Modelo:

~~~text
Input Generator
      ↓
Parser / Component
      ↓
Crash?
Timeout?
Memory?
Unexpected State?
      ↓
Reproduce
      ↓
Regression Test
~~~

Todo crash reproduzível deverá resultar em um caso de regressão.

---

## 15. Testes de recursos

O C2L deverá testar limites de:

- CPU;
- memória;
- disco;
- quantidade de arquivos;
- tamanho de arquivo;
- profundidade de diretório;
- concorrência;
- tempo de execução.

O objetivo é garantir que uma entrada adversarial não consiga consumir recursos indefinidamente.

---

## 16. Testes de performance

As principais métricas serão:

### Latência

Tempo para analisar um arquivo.

### Throughput

Quantidade de dados ou arquivos analisados por unidade de tempo.

### CPU

Uso médio e pico.

### Memória

Uso médio e pico.

### I/O

Impacto sobre disco.

### Escalabilidade

Comportamento com:

- 1 arquivo;
- 100 arquivos;
- 1.000 arquivos;
- diretórios grandes;
- arquivos grandes.

Performance nunca deverá ser otimizada sacrificando uma propriedade de segurança sem decisão explícita.

---

## 17. Baseline de performance

Antes de otimizar, será estabelecida uma linha de base.

~~~text
Baseline
   ↓
Alteração
   ↓
Benchmark
   ↓
Comparação
   ↓
Aceitar / Rejeitar
~~~

Uma alteração deverá ser considerada regressão quando ultrapassar limites previamente definidos.

Os limites serão refinados depois que existir uma implementação real.

---

## 18. Testes de detecção

Quando novas camadas forem implementadas, serão avaliadas separadamente.

### Indicadores

- match correto;
- ausência de match;
- indicador inválido.

### Assinaturas

- padrão correto;
- padrão semelhante;
- falso match.

### Heurística

- evidência forte;
- evidência fraca;
- combinação de evidências.

### ML

- classificação;
- confiança;
- dados fora da distribuição;
- comportamento diante de entrada desconhecida.

### Comportamento

- evento individual;
- sequência de eventos;
- comportamento legítimo semelhante;
- correlação temporal.

---

## 19. Métricas de detecção

O projeto não deverá utilizar somente "detectou/não detectou".

Métricas futuras:

### True Positive

Ameaça corretamente detectada.

### False Positive

Arquivo legítimo classificado incorretamente.

### True Negative

Arquivo legítimo corretamente permitido.

### False Negative

Ameaça não detectada.

A partir delas:

**Precision**

~~~text
TP / (TP + FP)
~~~

**Recall**

~~~text
TP / (TP + FN)
~~~

**F1**

~~~text
2 × Precision × Recall / (Precision + Recall)
~~~

Essas métricas serão utilizadas de acordo com a camada de detecção e não deverão ser interpretadas isoladamente.

---

## 20. Métricas específicas de segurança

À medida que o projeto evoluir, serão acompanhadas:

- taxa de detecção;
- taxa de falso positivo;
- tempo até detecção;
- tempo até contenção;
- tempo até recuperação;
- cobertura ATT&CK;
- tempo para distribuir uma nova regra;
- tempo para rollback;
- taxa de falha de atualização;
- disponibilidade do serviço;
- número de regressões;
- vulnerabilidades encontradas no pipeline.

---

## 21. Testes de quarentena

A quarentena deverá possuir uma suíte própria.

Testes:

- mover objeto;
- preservar hash;
- preservar metadata;
- impedir execução;
- impedir acesso indevido;
- restaurar;
- cancelar restauração;
- objeto inexistente;
- corrupção;
- colisão;
- falta de espaço;
- interrupção no meio da operação.

Uma operação incompleta deverá deixar o estado consistente ou recuperável.

---

## 22. Testes de atualização

Quando o sistema de atualização existir, serão testados:

### Sucesso

~~~text
Download
 ↓
Verify
 ↓
Install
 ↓
Activate
~~~

### Falhas

- pacote corrompido;
- assinatura inválida;
- certificado inválido;
- versão incompatível;
- downgrade;
- interrupção de rede;
- falta de espaço;
- interrupção durante instalação;
- servidor indisponível.

O resultado esperado será sempre um estado consistente.

---

## 23. Testes de recuperação

Toda operação crítica que possa falhar deverá possuir cenário de recuperação.

Exemplo:

~~~text
State A
  ↓
Update
  ↓
Failure
  ↓
Rollback
  ↓
State A
~~~

O rollback deverá ser testado, não apenas implementado.

---

## 24. Testes de regressão

Cada vulnerabilidade, bug de segurança ou falso positivo importante deverá gerar um teste permanente.

~~~text
Bug
 ↓
Fix
 ↓
Regression Test
 ↓
CI
 ↓
Future Protection
~~~

O conjunto de regressão crescerá continuamente.

---

## 25. Testes de compatibilidade

O C2L deverá testar versões suportadas das plataformas.

### Windows

Inicialmente será a plataforma principal.

### Linux

Testes iniciais poderão validar componentes portáveis antes da produção.

### Android

A validação ocorrerá quando o adapter correspondente existir.

A lógica de negócio do Core deverá ser testada independentemente do sistema operacional sempre que possível.

---

## 26. CI — Integração contínua

Cada alteração relevante deverá executar automaticamente, conforme a maturidade do projeto:

~~~text
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
Security Checks
 ↓
Artifact
~~~

Testes mais pesados poderão ocorrer em etapas posteriores.

---

## 27. Quality Gates

Uma versão não deverá ser considerada pronta apenas porque compila.

Gates futuros:

- build sem erros;
- testes unitários aprovados;
- testes de integração aprovados;
- regressões aprovadas;
- análise estática aprovada;
- dependências verificadas;
- testes de segurança aprovados;
- benchmarks dentro dos limites;
- documentação atualizada.

Uma falha crítica deverá bloquear o release.

---

## 28. Reprodutibilidade

Um teste deverá informar:

- versão do C2L;
- versão do Detection Content;
- sistema operacional;
- configuração;
- dataset/corpus;
- parâmetros;
- resultado;
- timestamp;
- identificação do artefato.

Isso permitirá comparar resultados entre versões.

---

## 29. Testes de falsos positivos

Falsos positivos são particularmente críticos em segurança.

O corpus benigno deverá conter:

- software comum;
- scripts;
- ferramentas administrativas;
- arquivos compactados;
- instaladores;
- documentos;
- workloads reais de utilização legítima.

Sempre que possível, os testes deverão representar o uso real do sistema.

Uma detecção que bloqueia grande quantidade de software legítimo não será considerada sucesso.

---

## 30. Testes de falso negativo

Também serão mantidos casos conhecidos que o sistema deverá detectar.

Quando uma ameaça ou indicador deixar de ser detectado:

1. registrar o caso;
2. identificar a camada responsável;
3. corrigir;
4. adicionar regressão;
5. medir novamente;
6. avaliar impacto em falsos positivos.

---

## 31. Testes de correlação

Quando o C2L possuir comportamento e EDR, não será suficiente testar eventos isolados.

Será necessário testar sequências.

Exemplo abstrato:

~~~text
Evento A
   +
Evento B
   +
Evento C
   ↓
Correlação
   ↓
Risk Score
   ↓
Decision
~~~

Também deverá existir o caso:

~~~text
A + B + C
   ↓
Comportamento legítimo
   ↓
Não bloquear
~~~

Isso será essencial para controlar falsos positivos.

---

## 32. Testes de explicabilidade

Uma decisão de alto risco deverá permitir reconstruir por que ocorreu.

Exemplo:

~~~text
Decision: QUARANTINE

Evidence:
- Indicator match
- Suspicious metadata
- Behavioral evidence

Risk:
HIGH

Policy:
P-001

Action:
QUARANTINE
~~~

Isso será importante para diagnóstico, suporte e evolução do motor.

---

## 33. Testes de privacidade

Quando telemetria existir, serão testados:

- quais dados são coletados;
- quais dados são enviados;
- se o envio pode ser desabilitado;
- se dados pessoais são minimizados;
- se falhas de comunicação alteram a proteção local;
- se dados sensíveis aparecem em logs.

---

## 34. Testes de falha externa

Serviços externos deverão ser tratados como dependências opcionais quando possível.

Testes:

- internet indisponível;
- DNS indisponível;
- servidor indisponível;
- resposta lenta;
- resposta inválida;
- certificado inválido;
- serviço parcialmente disponível.

O C2L deverá manter comportamento seguro mesmo sem conectividade, dentro das capacidades locais.

---

## 35. Testes de instalação e remoção

Deverão ser testados:

- instalação limpa;
- atualização;
- downgrade autorizado;
- interrupção da instalação;
- permissões;
- reinicialização;
- desinstalação;
- remoção de componentes;
- preservação/remoção correta de dados;
- recuperação após falha.

---

## 36. Matriz de cobertura

O projeto deverá manter uma matriz relacionando requisitos, implementação e testes.

Exemplo:

| Requisito | Componente | Teste | Status |
|---|---|---|---|
| Scan recursivo | Scanner | INT-SCAN-001 | Planejado |
| Hash | Hashing | UNIT-HASH-001 | Planejado |
| Match | Detection | UNIT-DET-001 | Planejado |
| Quarentena | Quarantine | INT-QUAR-001 | Planejado |
| Integridade | Security | SEC-INT-001 | Planejado |
| Update | Updater | SEC-UPD-001 | Futuro |

O objetivo é evitar requisitos sem validação.

---

## 37. V0.1 — plano mínimo de testes

Antes do primeiro release funcional, deverão existir pelo menos:

### Scanner

- arquivo;
- diretório;
- recursividade;
- caminho inválido;
- permissão negada.

### Hash

- arquivos conhecidos;
- alteração de conteúdo;
- erro de leitura.

### Indicator Store

- match;
- no-match;
- indicador inválido;
- corrupção.

### Detection

- positivo conhecido;
- negativo conhecido;
- resultado desconhecido.

### Quarantine

- isolamento;
- metadata;
- restauração;
- falha.

### Segurança

- path traversal;
- limites de tamanho;
- interrupção;
- permissões.

### CLI

- parâmetros válidos;
- parâmetros inválidos;
- códigos de saída;
- relatório.

---

## 38. Critérios de release da V0.1

A V0.1 deverá cumprir:

- todos os testes obrigatórios aprovados;
- zero falhas críticas conhecidas;
- nenhuma vulnerabilidade crítica aberta conhecida;
- nenhum crash conhecido em fluxo normal;
- resultados de detecção reproduzíveis;
- quarentena validada;
- logs funcionais;
- documentação correspondente;
- artefato identificável por versão;
- possibilidade de reproduzir o build/teste dentro das capacidades do projeto.

---

## 39. Evolução da estratégia

### V0.1

Testes unitários, integração, regressão básica, segurança estrutural e performance básica.

### V0.2

Atualizações, conteúdo assinado, fuzzing inicial e laboratório mais completo.

### V0.3

Heurísticas, análise mais complexa, fuzzing ampliado e testes de isolamento.

### V0.4

Comportamento, correlação e resposta.

### V0.5+

EDR, threat intelligence, modelos, testes de ataque controlado e recuperação avançada.

---

## 40. Decisões registradas

1. Testes de segurança fazem parte do produto, não apenas do desenvolvimento.
2. O C2L utilizará testes automatizados desde a primeira versão.
3. Todo bug de segurança relevante deverá gerar regressão quando possível.
4. Fuzzing será obrigatório para componentes que processem entradas complexas e não confiáveis.
5. Malware real não fará parte do desenvolvimento inicial.
6. Testes de maior risco ocorrerão em laboratório isolado.
7. Falsos positivos e falsos negativos serão métricas de primeira classe.
8. Performance será medida antes de otimizações.
9. Atualizações e rollback deverão ser testados como operações de segurança.
10. Falhas externas não deverão transformar automaticamente o sistema em estado permissivo.
11. Requisitos deverão possuir rastreabilidade até testes.
12. Um release deverá ser bloqueado por falhas críticas conhecidas.

## 41. Próximo documento

O próximo documento será:

**08 — Roadmap**

Ele consolidará:

- versões;
- marcos;
- dependências;
- prioridades;
- V0.1;
- evolução do Detection Engine;
- Windows → Linux → Android;
- laboratório;
- Threat Intelligence;
- EDR;
- recuperação;
- critérios de passagem entre versões.
