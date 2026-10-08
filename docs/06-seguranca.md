# 06 — Segurança

**Projeto:** C2L Antivirus  
**Versão do documento:** 1.0  
**Data:** 08/10/2026  
**Status:** Aprovado

## 1. Objetivo

Este documento define os princípios e controles destinados a proteger o próprio C2L Antivirus contra comprometimento, manipulação, abuso e indisponibilidade.

Um antivírus possui uma característica especial:

> **Ele precisa analisar conteúdo potencialmente hostil enquanto simultaneamente protege a própria capacidade de realizar essa análise.**

A segurança do C2L será uma propriedade de toda a arquitetura.

## 2. Objetivos de segurança

O C2L deverá proteger principalmente:

- execução do próprio C2L;
- configurações;
- Indicator Store;
- Detection Content;
- modelos futuros;
- quarentena;
- logs e eventos;
- mecanismo de atualização;
- credenciais e chaves;
- comunicação externa;
- resultados das análises;
- integridade das decisões de segurança.

Objetivos:

1. impedir ou dificultar desativação não autorizada;
2. impedir alteração silenciosa de componentes;
3. impedir instalação de conteúdo de detecção não confiável;
4. proteger itens em quarentena;
5. preservar evidências e auditoria;
6. limitar o impacto de uma falha em um componente;
7. recuperar o sistema para um estado conhecido;
8. detectar tentativas de comprometer o próprio C2L.

## 3. Modelo de confiança

Nem todo componente possuirá o mesmo nível de confiança.

~~~text
                    Trust Boundary
                         │
        ┌────────────────┴────────────────┐
        │                                 │
   Trusted Core                     Untrusted Input
        │                                 │
        ▼                                 ▼
  Detection Engine                Arquivos / Eventos
        │
        ▼
  Decision / Policy
~~~

Todo conteúdo externo deverá ser considerado não confiável até ser validado.

Isso inclui arquivos analisados, nomes, caminhos, argumentos, conteúdo, bases importadas, respostas externas, atualizações, modelos e configurações.

## 4. Privilégio mínimo

Cada componente deverá operar com o menor privilégio necessário.

~~~text
Necessidade de privilégio
        ↓
Conceder somente o necessário
        ↓
Executar operação
        ↓
Retornar ao menor privilégio possível
~~~

O C2L não deverá executar toda a aplicação com privilégios elevados apenas porque algumas operações exigem acesso administrativo.

Quando a plataforma permitir, deverá existir separação entre componente de análise, serviço privilegiado, interface e tarefas administrativas.

## 5. Separação de privilégios

~~~text
             CLI / UI
                 │
                 ▼
        User-level Controller
                 │
          ┌──────┴──────┐
          │             │
          ▼             ▼
      Core Worker   Privileged Service
          │             │
          ▼             ▼
       Analysis     OS Operations
~~~

O componente privilegiado deverá possuir uma interface pequena e controlada.

Deverá aceitar operações específicas, como RequestQuarantine(file_id), e não comandos arbitrários fornecidos pela interface.

## 6. Secure Bootstrapping

O processo inicial deverá verificar o ambiente antes de confiar em componentes críticos.

~~~text
Start
 ↓
Load trusted configuration
 ↓
Verify component integrity
 ↓
Verify detection content
 ↓
Verify policy
 ↓
Initialize protected services
 ↓
Ready
~~~

Se uma verificação crítica falhar, o sistema deverá impedir o carregamento do componente comprometido, registrar o evento e utilizar um estado seguro conhecido quando possível.

## 7. Integridade dos binários

Os componentes executáveis deverão possuir mecanismos de integridade e autenticidade.

Futuramente, o produto deverá considerar:

- assinatura digital;
- cadeia de confiança;
- verificação de integridade;
- identificação de versão;
- proteção contra downgrade;
- validação durante instalação;
- validação durante atualização.

O C2L não deverá confiar apenas em um hash local armazenado no mesmo diretório protegido. A autenticidade deverá depender de uma raiz de confiança apropriada.

## 8. Proteção contra tampering

Tampering é a tentativa de modificar, desativar ou degradar o funcionamento do C2L.

Possíveis alvos:

- processos;
- serviços;
- binários;
- bibliotecas;
- configurações;
- políticas;
- Indicator Store;
- Detection Content;
- quarentena;
- logs;
- mecanismo de atualização.

O C2L deverá detectar e, quando tecnicamente possível, impedir alterações não autorizadas.

~~~text
Protect
   +
Detect
   +
Recover
~~~

A arquitetura não deverá depender de um único mecanismo anti-tamper.

## 9. Proteção da configuração

Configurações críticas incluem:

- exclusões;
- diretórios protegidos;
- políticas de quarentena;
- endpoints de atualização;
- autoridades confiáveis;
- políticas de resposta;
- nível de proteção.

Alterações críticas deverão exigir autorização adequada, ser validadas, registradas e protegidas contra valores inesperados.

Uma configuração inválida não deverá resultar em comportamento permissivo por acidente.

## 10. Indicator Store

O Indicator Store é um componente de segurança e não apenas uma base de dados.

Riscos:

- alteração de indicadores;
- remoção de indicadores;
- corrupção;
- substituição por base falsa;
- downgrade;
- injeção de dados inválidos.

Controles futuros:

- formato validado;
- integridade;
- versionamento;
- assinatura;
- controle de permissões;
- transações ou mecanismo equivalente;
- backup;
- rollback;
- validação antes de ativação.

Uma atualização parcial não deverá deixar a base em estado inconsistente.

## 11. Detection Content

O conteúdo de detecção deverá possuir ciclo de vida próprio.

~~~text
Create
  ↓
Validate
  ↓
Test
  ↓
Sign
  ↓
Distribute
  ↓
Verify
  ↓
Activate
  ↓
Monitor
  ↓
Rollback if necessary
~~~

Cada pacote deverá possuir, conforme a evolução:

- identificador;
- versão;
- origem;
- timestamp;
- compatibilidade;
- assinatura;
- dependências;
- validade;
- hash/integridade.

O endpoint deverá rejeitar conteúdo cuja assinatura ou compatibilidade não possa ser validada.

## 12. Segurança de atualizações

O mecanismo de atualização será uma das áreas de maior criticidade.

~~~text
Servidor / Distribuição
        ↓
Download
        ↓
Integridade
        ↓
Autenticidade
        ↓
Compatibilidade
        ↓
Instalação atômica
        ↓
Validação
        ↓
Ativação
~~~

Requisitos:

- transporte protegido;
- autenticação da origem;
- assinatura do conteúdo;
- proteção contra replay/downgrade;
- validação de versão;
- instalação segura;
- rollback;
- registro da operação.

Nunca deverá existir lógica equivalente a download → executar sem validação intermediária.

## 13. Rotação e proteção de chaves

O sistema deverá evitar chaves privadas incorporadas diretamente ao código-fonte.

A arquitetura deverá prever:

- separação de funções;
- armazenamento seguro;
- rotação;
- revogação;
- identificação de versão da chave;
- recuperação em caso de comprometimento.

No futuro, a infraestrutura de distribuição poderá utilizar diferentes chaves para diferentes funções.

## 14. Quarentena

A quarentena deverá funcionar como uma área de contenção.

Ela deverá:

- impedir execução acidental;
- restringir acesso;
- preservar metadados;
- identificar unicamente o objeto;
- registrar origem;
- registrar data;
- registrar motivo da quarentena;
- preservar hash;
- suportar restauração controlada.

Os nomes originais não deverão ser utilizados de forma que facilitem execução ou acesso indevido.

~~~text
Original:
C:\Users\User\Downloads\arquivo.exe

Quarantine:
<ID interno> + metadata
~~~

A restauração deverá exigir uma operação explícita e passar por validação.

## 15. Segurança contra path traversal

Qualquer componente que manipule caminhos deverá considerar entradas maliciosas.

Exemplos de risco:

- ../;
- caminhos absolutos inesperados;
- links simbólicos;
- junctions/reparse points;
- nomes especiais;
- caminhos muito longos;
- dispositivos especiais;
- ambiguidades de normalização.

O C2L deverá normalizar e validar caminhos antes de executar operações sensíveis.

Nunca deverá assumir que um caminho fornecido pelo utilizador é seguro.

## 16. Análise de conteúdo hostil

O Scanner e parsers estarão expostos a arquivos potencialmente malformados.

Riscos:

- buffer overflow em bibliotecas nativas;
- consumo excessivo de memória;
- arquivos recursivos;
- compressão maliciosa;
- parsing extremamente lento;
- formatos corrompidos;
- exploração de bibliotecas de terceiros.

Controles:

- limites de tamanho;
- timeouts;
- limites de profundidade;
- limites de memória;
- parsers atualizados;
- validação de entradas;
- isolamento quando necessário;
- fuzzing;
- tratamento robusto de erros.

> **O C2L deve assumir que o arquivo que está analisando pode estar tentando explorar o próprio C2L.**

## 17. Isolamento do Scanner

Conforme a capacidade do produto evoluir, componentes de análise de maior risco deverão poder ser isolados.

~~~text
             C2L Core
                │
                ▼
        Analysis Controller
                │
          Trust Boundary
                │
                ▼
        Isolated Analyzer
                │
                ▼
          Untrusted File
~~~

Isso será particularmente importante para parsers complexos, análise de documentos, scripts, descompressão, executáveis e componentes de ML.

## 18. Proteção contra DoS

Um arquivo malicioso pode tentar consumir recursos sem explorar uma vulnerabilidade.

Exemplos:

- arquivo gigantesco;
- árvore de diretórios enorme;
- compressão com expansão extrema;
- milhares de arquivos;
- loops de processamento;
- estruturas profundamente aninhadas.

O C2L deverá possuir limites configuráveis para:

- tamanho;
- tempo;
- memória;
- profundidade;
- quantidade de objetos;
- concorrência.

Quando um limite for atingido, o resultado deverá ser registrado como análise limitada, e não simplesmente como seguro.

## 19. Logs e auditoria

Logs são parte do mecanismo de segurança.

Deverão registrar, conforme a criticidade:

- inicialização;
- falhas de integridade;
- alterações de configuração;
- atualização de conteúdo;
- atualização do produto;
- alterações de política;
- deteções;
- quarentenas;
- restaurações;
- tentativas de tampering;
- erros críticos.

Para eventos críticos, a arquitetura poderá futuramente considerar encadeamento criptográfico, assinatura, armazenamento protegido e exportação segura.

## 20. Privacidade

O C2L deverá seguir o princípio de minimização de dados.

Antes de enviar informação externamente:

1. O dado é realmente necessário?
2. É possível enviar somente um indicador?
3. É possível anonimizar?
4. O utilizador pode controlar o envio?
5. O dado possui informação pessoal?
6. Existe uma alternativa local?

Por padrão, o produto deverá evitar enviar arquivos completos para serviços externos sem finalidade explícita e política apropriada.

## 21. Comunicação externa

Quando serviços externos forem introduzidos, a comunicação deverá considerar:

- TLS;
- validação de certificados;
- autenticação do servidor;
- autenticação do cliente quando necessário;
- timeout;
- retry controlado;
- proteção contra replay;
- validação de respostas;
- limites de tamanho;
- tratamento de indisponibilidade.

Uma resposta externa não deverá ser considerada confiável apenas porque veio de uma conexão TLS. O conteúdo recebido também deverá ser validado.

## 22. Segurança da Threat Intelligence

Threat Intelligence externa será tratada como dados não confiáveis até validação.

Mesmo uma fonte legítima pode sofrer comprometimento, publicar conteúdo incorreto ou entregar dados incompatíveis.

~~~text
Fonte
 ↓
Autenticidade
 ↓
Formato
 ↓
Schema
 ↓
Assinatura
 ↓
Compatibilidade
 ↓
Conteúdo
 ↓
Ativação
~~~

## 23. Defesa contra downgrade

Atacantes podem tentar instalar uma versão antiga e vulnerável do C2L ou de seu conteúdo.

A arquitetura deverá manter:

- versão atual;
- versão mínima permitida;
- versão de conteúdo;
- identificação de geração;
- política contra rollback não autorizado.

Rollback legítimo deverá existir como mecanismo de recuperação, mas não deverá permitir arbitrariamente a instalação de versões inseguras.

## 24. Fail-safe

Quando um componente crítico falhar, o comportamento deverá ser definido explicitamente.

Exemplos:

- Falha do Indicator Store: não assumir automaticamente que todos os arquivos são seguros.
- Falha na validação de atualização: não ativar o pacote.
- Falha de quarentena: não informar que a quarentena foi concluída.
- Falha de integridade do Core: não continuar como se nada tivesse ocorrido.

O produto deverá distinguir:

~~~text
Clean
Unknown
Error
Detected
~~~

> **Erro não é sinônimo de seguro.**

## 25. Recuperação

Segurança não termina no bloqueio.

O C2L deverá evoluir para suportar recuperação de:

- configuração;
- Detection Content;
- Indicator Store;
- componentes;
- quarentena;
- estado operacional.

~~~text
Detect
 ↓
Contain
 ↓
Recover
 ↓
Verify
 ↓
Resume
~~~

Backups e snapshots internos deverão ser protegidos contra a mesma ameaça que compromete o estado principal.

## 26. Anti-tamper

A proteção contra tampering deverá evoluir por níveis.

### V0.1

- permissões corretas;
- integridade básica;
- proteção de arquivos críticos;
- logs;
- validação de conteúdo.

### Futuro

- monitoramento de processos;
- proteção de serviços;
- detecção de alterações;
- proteção de configurações;
- resposta a tentativas de desativação;
- recuperação automática;
- integração com mecanismos nativos da plataforma.

O anti-tamper deverá ser projetado para não criar um mecanismo de bloqueio tão agressivo que impeça administração legítima do equipamento.

## 27. Modelo de ameaça específico do próprio C2L

| ID | Ameaça | Impacto | Controles |
|---|---|---|---|
| SEC-001 | Desativação do C2L | Alto | privilégios, anti-tamper, monitoramento |
| SEC-002 | Alteração de regras | Crítico | assinatura, integridade, permissões |
| SEC-003 | Update comprometido | Crítico | assinatura, TLS, rollback |
| SEC-004 | Corrupção do Indicator Store | Alto | integridade, versionamento, backup |
| SEC-005 | Escape da quarentena | Crítico | isolamento, permissões, validação |
| SEC-006 | Manipulação de logs | Alto | proteção, auditoria, integridade |
| SEC-007 | Exploração do parser | Crítico | limites, fuzzing, isolamento |
| SEC-008 | DoS no scanner | Médio/Alto | quotas, timeouts, limites |
| SEC-009 | Downgrade | Alto | versionamento e política |
| SEC-010 | Comprometimento de chave | Crítico | armazenamento seguro, rotação, revogação |

## 28. Secure Development Lifecycle

A segurança deverá fazer parte de todo o ciclo de desenvolvimento.

~~~text
Design
 ↓
Threat Modeling
 ↓
Implementation
 ↓
Static Analysis
 ↓
Unit Tests
 ↓
Security Tests
 ↓
Fuzzing
 ↓
Integration Tests
 ↓
Release Validation
 ↓
Signed Build
 ↓
Distribution
 ↓
Monitoring
~~~

Cada vulnerabilidade encontrada deverá resultar, quando apropriado, em correção, teste de regressão, análise de causa, atualização do threat model e documentação.

## 29. Dependências de terceiros

Bibliotecas externas representam parte da superfície de ataque.

O projeto deverá futuramente manter:

- inventário de dependências;
- versões controladas;
- análise de vulnerabilidades;
- atualização periódica;
- verificação de origem;
- revisão de dependências críticas.

Quando possível, deverá ser evitada dependência desnecessária de bibliotecas com privilégios elevados.

## 30. Supply Chain

A cadeia de software deverá ser tratada como parte do modelo de ameaça.

~~~text
Source Code
 ↓
Dependencies
 ↓
Build Environment
 ↓
Artifacts
 ↓
Signing
 ↓
Distribution
 ↓
Endpoint
~~~

Futuramente, o projeto deverá considerar:

- builds reproduzíveis quando viável;
- ambientes de build protegidos;
- controle de acesso;
- assinatura de artefatos;
- provenance;
- SBOM;
- revisão de alterações críticas.

## 31. Gestão de vulnerabilidades

O projeto deverá possuir processo para:

- receber relatos;
- classificar vulnerabilidades;
- reproduzir;
- corrigir;
- testar;
- publicar correção;
- registrar impacto.

O arquivo SECURITY.md do repositório será utilizado como ponto de partida.

Vulnerabilidades críticas deverão receber prioridade máxima, especialmente as que permitam execução de código, elevação de privilégio, bypass da proteção, comprometimento de atualização, fuga da quarentena ou alteração de Detection Content.

## 32. Segurança por padrão

As configurações iniciais deverão priorizar proteção.

Exemplos:

- proteção ativa habilitada quando aplicável;
- atualização de conteúdo validada;
- quarentena protegida;
- logs habilitados;
- comportamento seguro diante de erro;
- telemetria externa mínima;
- exclusões vazias ou explicitamente configuradas.

Configurações que reduzam significativamente a proteção deverão exigir uma ação consciente do administrador.

## 33. Administração e recuperação

Um produto de segurança precisa permitir administração legítima.

Portanto, anti-tamper não deverá significar "impossível de alterar".

Deverá existir uma distinção entre:

~~~text
Alteração autorizada
        ≠
Alteração não autorizada
~~~

A arquitetura deverá futuramente permitir:

- modo administrativo;
- recuperação;
- diagnóstico;
- manutenção;
- desinstalação segura;
- restauração de configuração.

Operações sensíveis deverão ser autenticadas e auditadas.

## 34. V0.1 — controles obrigatórios

Antes de considerar a V0.1 funcional, deverão existir pelo menos:

- validação rigorosa de caminhos;
- tratamento de erros;
- permissões adequadas;
- proteção da quarentena;
- integridade do Indicator Store;
- validação dos dados de indicadores;
- logs de operações críticas;
- limites básicos de análise;
- não execução dos arquivos analisados;
- testes para entradas malformadas;
- proteção contra corrupção do estado;
- identificação clara de erros versus arquivos limpos.

Não será necessário implementar todo o anti-tamper avançado na V0.1.

## 35. Evolução dos controles

### V0.1

Segurança estrutural e integridade básica.

### V0.2

Conteúdo assinado, atualização segura e proteção de configuração.

### V0.3

Análise isolada e fuzzing mais abrangente.

### V0.4

Monitoramento comportamental e defesa contra tampering.

### V0.5+

Proteção avançada de endpoint, resposta automática, recuperação, gestão de chaves em infraestrutura e capacidades de EDR.

## 36. Critérios de aceitação

A arquitetura de segurança será considerada adequada quando:

- componentes críticos puderem ser protegidos contra alteração não autorizada;
- conteúdo de detecção puder ser autenticado;
- atualizações puderem ser verificadas antes da ativação;
- a quarentena não depender de uma simples pasta;
- erros críticos não forem classificados como arquivos seguros;
- entradas hostis forem tratadas de forma defensiva;
- limites de recursos existirem;
- operações críticas forem auditáveis;
- privilégios forem mínimos;
- o produto puder ser recuperado para um estado conhecido;
- vulnerabilidades do próprio scanner forem tratadas como prioridade de segurança.

## 37. Decisões arquiteturais registradas

1. O C2L será tratado como alvo de ataque.
2. O scanner será tratado como uma superfície de ataque de alta importância.
3. Privilégio mínimo será obrigatório.
4. Conteúdo de detecção será autenticado antes de ativação.
5. Atualizações deverão possuir integridade, autenticidade e proteção contra downgrade.
6. Quarentena será considerada um componente de segurança.
7. Erros não serão tratados como evidência de segurança.
8. Arquivos analisados nunca deverão ser executados automaticamente.
9. Serviços externos serão considerados não confiáveis além da camada de transporte e deverão ter suas respostas validadas.
10. Recuperação fará parte do modelo de segurança.
11. O anti-tamper será evolutivo e não dependerá de um único mecanismo.
12. A cadeia de desenvolvimento e distribuição será parte do threat model.

## 38. Próximo documento

O próximo documento será:

**07 — Testes**

Ele deverá transformar os princípios anteriores em uma estratégia prática de validação, incluindo:

- testes unitários;
- integração;
- regressão;
- segurança;
- fuzzing;
- performance;
- corpus de arquivos;
- testes de quarentena;
- testes de atualização;
- métricas de detecção;
- laboratório Windows/Linux;
- CI/CD;
- critérios objetivos de release.
