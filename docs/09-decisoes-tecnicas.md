# 09 — Decisões Técnicas e Fundação da Implementação

**Projeto:** C2L Antivirus  
**Versão do documento:** 1.0  
**Data:** 08/10/2026  
**Status:** Aprovado

## 1. Objetivo

Este documento transforma a arquitetura conceitual do C2L em decisões técnicas concretas para iniciar a implementação da V0.1.

A regra deste documento é simples:

> **Escolher tecnologias que reduzam risco técnico hoje sem fechar portas para a evolução futura.**

As decisões aqui registradas podem ser revistas, mas uma alteração deverá ser documentada juntamente com sua justificativa e impacto.

---

# 2. Decisão principal: linguagem do Core

## 2.1 Decisão

A linguagem principal do **C2L Core será Rust**.

A decisão é baseada principalmente em:

- segurança de memória;
- ausência de garbage collector;
- desempenho adequado para processamento de ficheiros;
- controle explícito de recursos;
- bom suporte a concorrência segura;
- compilação para Windows e Linux;
- possibilidade de compilação para Android;
- ecossistema adequado para CLI, criptografia, filesystem e sistemas;
- forte alinhamento com software de infraestrutura e segurança.

Rust não elimina vulnerabilidades. Ele reduz uma classe importante de erros, mas ainda exige desenvolvimento seguro, testes, revisão, fuzzing e controle de dependências.

## 2.2 Alternativas avaliadas

### C/C++

**Vantagens**
- enorme ecossistema de sistemas;
- excelente integração com APIs nativas;
- desempenho;
- grande quantidade de bibliotecas existentes.

**Desvantagens**
- maior exposição a erros de memória;
- maior superfície para vulnerabilidades de corrupção de memória;
- exige disciplina adicional para atingir o nível de segurança desejado.

### C#

**Vantagens**
- excelente experiência de desenvolvimento;
- bom ecossistema Windows;
- produtividade elevada;
- boa integração com ferramentas Microsoft.

**Desvantagens**
- não é a melhor escolha para um core de baixo nível multiplataforma de longo prazo;
- dependência maior do runtime;
- arquitetura Android/Linux exige decisões adicionais;
- menor alinhamento com a estratégia de núcleo de segurança de sistemas.

### Rust

**Vantagens**
- segurança de memória;
- desempenho;
- concorrência segura;
- bom suporte a sistemas;
- multiplataforma;
- bom encaixe com o objetivo do C2L.

**Desvantagens**
- curva de aprendizagem;
- ecossistema menor que C/C++;
- algumas integrações nativas exigem FFI;
- exige disciplina para definir APIs e ownership corretamente.

### Decisão

**Rust vence para o Core.**

A escolha não significa que todo o projeto futuro terá de ser Rust. Interfaces ou componentes específicos poderão utilizar outras tecnologias quando existir justificativa arquitetural.

---

# 3. Linguagem da CLI

A primeira CLI também será implementada em **Rust**.

Motivos:

- evita uma camada tecnológica desnecessária na V0.1;
- acesso direto às APIs do Core;
- distribuição simplificada;
- mesma linguagem e tooling;
- facilidade de reutilização dos tipos e contratos.

A futura GUI poderá utilizar outra tecnologia sem alterar o Core.

---

# 4. Estrutura do projeto

A estrutura inicial será organizada como um workspace Rust:

```
C2L-Antivirus/
├── docs/
├── crates/
│   ├── c2l-core/
│   ├── c2l-scanner/
│   ├── c2l-hashing/
│   ├── c2l-indicators/
│   ├── c2l-detection/
│   ├── c2l-risk/
│   ├── c2l-quarantine/
│   ├── c2l-events/
│   ├── c2l-reporting/
│   ├── c2l-platform/
│   ├── c2l-platform-windows/
│   └── c2l-cli/
├── tests/
├── tools/
├── Cargo.toml
├── Cargo.lock
├── README.md
└── .gitignore
```

A divisão em crates deverá ser pragmática.

Não será criada uma crate apenas para aumentar a quantidade de módulos. Se dois componentes tiverem forte coesão e não existir benefício real de isolamento, poderão permanecer juntos.

---

# 5. Regra de dependências

A direção preferencial será:

```
c2l-cli
   ↓
c2l-core
   ↓
domain / services
   ↓
platform abstraction
   ↓
platform implementation
```

As dependências deverão apontar para baixo, evitando ciclos.

O Core não poderá depender da CLI.

A lógica de detecção não poderá depender de apresentação.

---

# 6. Modelos de domínio

Os tipos fundamentais serão definidos no início e usados pelos componentes.

Exemplos:

```
ScanRequest
ScanOptions
ScanResult

FileMetadata
FileHash

Indicator
IndicatorMatch

Evidence
Detection
RiskAssessment

QuarantineItem

C2LEvent
ScanError
```

Esses tipos deverão representar conceitos de domínio e não detalhes da CLI.

---

# 7. Hashing

## 7.1 Algoritmo inicial

O algoritmo principal da V0.1 será **SHA-256**.

Motivos:

- amplamente suportado;
- adequado para identificação de integridade;
- simples de validar;
- disponibilidade de implementações maduras.

MD5 e SHA-1 não serão utilizados como identificadores criptográficos de segurança.

Hashes adicionais poderão ser suportados futuramente por compatibilidade ou integração, mas isso não deve complicar a V0.1.

## 7.2 Regra

O hash será tratado como **indicador**, não como prova universal de que um ficheiro é seguro.

Um hash desconhecido significa apenas que não existe correspondência conhecida no conjunto de indicadores consultado.

---

# 8. Indicator Store

## 8.1 Tecnologia inicial

A V0.1 utilizará **SQLite** para o armazenamento local de indicadores.

Motivos:

- banco embutido;
- sem servidor externo;
- transações;
- integridade;
- consultas eficientes;
- suporte multiplataforma;
- facilidade de backup e atualização;
- maturidade.

## 8.2 Estrutura conceitual

```
indicators
├── id
├── type
├── value
├── classification
├── source
├── version
├── created_at
└── updated_at
```

Índice principal:

```
(type, value)
```

## 8.3 Segurança

O banco não será tratado como fonte confiável simplesmente por estar localmente instalado.

Futuramente deverá possuir:

- integridade;
- versão;
- assinatura do conteúdo distribuído;
- validação;
- proteção contra downgrade;
- atualização atômica.

A V0.1 deverá manter o formato preparado para essas evoluções.

---

# 9. Configuração

A configuração inicial será separada do código e do detection content.

Estrutura conceitual:

```
config/
├── application
├── scanning
├── quarantine
└── logging
```

O formato exato poderá ser TOML ou outro formato adequado ao ecossistema Rust.

Regras:

- configuração inválida não deve ser silenciosamente ignorada;
- valores críticos deverão possuir defaults seguros;
- caminhos deverão ser validados;
- configurações desconhecidas deverão gerar erro ou aviso explícito;
- segredos não deverão ser armazenados em texto puro.

A decisão definitiva do formato será feita durante a implementação da configuração.

---

# 10. Logging

A V0.1 utilizará logging estruturado.

Cada evento relevante deverá possuir, quando aplicável:

- timestamp;
- nível;
- componente;
- evento;
- resultado;
- identificador da operação;
- erro;
- contexto mínimo necessário.

Exemplo conceitual:

```json
{
  "event": "file_analyzed",
  "scan_id": "...",
  "path": "...",
  "sha256": "...",
  "result": "clean"
}
```

O exemplo é conceitual. O formato final será definido na implementação.

## Regra de privacidade

Não registrar conteúdo de ficheiros, credenciais ou dados pessoais desnecessários.

---

# 11. Event System

O Event System utilizará tipos estruturados internos.

Exemplos:

```
ScanStarted
FileDiscovered
FileHashed
IndicatorMatched
DetectionFound
RiskCalculated
QuarantineStarted
QuarantineCompleted
ScanCompleted
ErrorOccurred
```

Eventos deverão ser suficientemente genéricos para serem utilizados futuramente por:

- CLI;
- UI;
- telemetria;
- auditoria;
- testes;
- EDR.

---

# 12. Quarentena

A quarentena será implementada como componente próprio.

Estrutura conceitual:

```
quarantine/
├── objects/
└── metadata/
```

O nome original do ficheiro não será utilizado como nome físico do objeto em quarentena quando isso puder facilitar colisões ou manipulação.

Cada item terá um identificador interno.

Metadados deverão preservar, quando necessário:

- ID;
- hash;
- caminho original;
- timestamp;
- motivo;
- classificação;
- versão do C2L;
- estado.

A implementação deverá impedir execução acidental.

Restauração deverá exigir validação e registrar a operação.

---

# 13. Scanner

O Scanner deverá receber uma abstração de filesystem e produzir objetos para análise.

Responsabilidades:

- arquivo individual;
- diretório;
- recursão;
- filtros;
- erros de acesso;
- cancelamento futuro;
- contagem.

O Scanner não deverá:

- decidir malware;
- executar o ficheiro;
- modificar o ficheiro analisado;
- alterar configurações de segurança.

---

# 14. Detection Engine V0.1

A primeira implementação será deliberadamente simples:

```
File
 ↓
SHA-256
 ↓
Indicator Store
 ↓
Match?
 ├── No → No Detection
 └── Yes → Detection
```

O Detection Engine deverá retornar uma evidência estruturada, e não simplesmente `true/false`.

Exemplo:

```
Detection {
    indicator_id
    indicator_type
    classification
    source
    evidence
}
```

Isso prepara o motor para receber múltiplas evidências no futuro.

---

# 15. Risk Engine V0.1

O Risk Engine será simples e determinístico.

Estados iniciais:

```
Clean
Unknown
Detected
Error
```

Regra fundamental:

> **Error nunca será convertido automaticamente em Clean.**

Na V0.1, uma correspondência confiável com indicador malicioso produzirá risco compatível com detecção.

Um ficheiro sem correspondência será **Unknown/No Known Detection**, e não "garantidamente seguro".

Essa distinção será mantida nas APIs e nos relatórios.

---

# 16. Orchestrator

O Orchestrator coordenará:

```
Request
 ↓
Scanner
 ↓
Hashing
 ↓
Indicators
 ↓
Detection
 ↓
Risk
 ↓
Action
 ↓
Event
 ↓
Report
```

Ele não conterá regras específicas de malware.

Isso permitirá trocar o Detection Engine sem reescrever o fluxo de scan.

---

# 17. CLI inicial

Comandos conceituais:

```
c2l scan <path>
c2l version
c2l status
c2l quarantine list
c2l quarantine restore <id>
c2l quarantine delete <id>
```

Nem todos precisam estar presentes no primeiro commit.

A prioridade da V0.1 será:

```
c2l scan <path>
```

e uma forma simples de consultar a versão.

---

# 18. Tratamento de erros

Erros serão classificados e propagados explicitamente.

Exemplos:

- arquivo inexistente;
- acesso negado;
- arquivo removido durante scan;
- erro de leitura;
- banco indisponível;
- indicador inválido;
- quarentena interrompida;
- falta de espaço;
- configuração inválida.

O sistema deverá distinguir:

```
Detection
≠
Operational Error
```

Um arquivo que não pôde ser analisado não deverá ser contado como limpo.

---

# 19. Concorrência

A V0.1 começará com execução predominantemente sequencial ou com concorrência limitada e controlada.

Motivo:

- facilitar diagnóstico;
- reduzir race conditions;
- simplificar testes;
- estabelecer baseline de performance.

Depois de medir o desempenho, poderão ser adicionados workers e processamento paralelo.

---

# 20. Dependências de terceiros

A política será:

> **Quanto menor a superfície de dependências, melhor.**

Antes de adicionar uma dependência, deverão ser considerados:

- manutenção;
- licença;
- histórico de vulnerabilidades;
- atividade do projeto;
- quantidade de dependências transitivas;
- necessidade real;
- suporte multiplataforma;
- possibilidade de atualização segura.

Dependências críticas deverão ser monitoradas no CI.

---

# 21. Build e gestão de dependências

O projeto utilizará:

- Cargo;
- Cargo workspace;
- Cargo.lock versionado;
- builds separados por plataforma;
- ferramentas de lint e formatação;
- testes automatizados.

O CI deverá verificar, no mínimo:

```
cargo fmt
cargo check
cargo test
cargo clippy
```

As verificações de segurança de dependências serão adicionadas ao pipeline inicial de CI.

---

# 22. CI inicial

Fluxo:

```
Commit
 ↓
Format
 ↓
Check
 ↓
Clippy
 ↓
Unit Tests
 ↓
Integration Tests
 ↓
Security / Dependency Checks
 ↓
Build
```

Falha em uma etapa crítica impedirá a produção do artefato de release.

---

# 23. Windows V0.1

O primeiro adapter será Windows.

Ele deverá fornecer somente o necessário para:

- filesystem;
- paths;
- permissões necessárias;
- quarentena;
- informações básicas da aplicação.

Não serão adicionados ainda:

- driver;
- kernel component;
- proteção em tempo real;
- ETW avançado;
- AMSI;
- integração profunda com Windows Defender;
- mecanismos complexos de anti-tamper.

Essas capacidades pertencem a fases posteriores.

---

# 24. Estratégia de segurança da implementação

Desde o primeiro código:

- entradas não confiáveis serão tratadas como hostis;
- caminhos serão normalizados e validados;
- symlinks/reparse points serão tratados conscientemente;
- limites de tamanho existirão;
- operações de filesystem serão verificadas;
- erros serão explícitos;
- componentes privilegiados serão minimizados;
- nenhum ficheiro analisado será executado;
- testes negativos serão obrigatórios.

---

# 25. Primeira estrutura física

Após esta decisão, a implementação inicial poderá começar com:

```
C2L-Antivirus/
├── crates/
│   ├── c2l-core/
│   ├── c2l-scanner/
│   ├── c2l-hashing/
│   ├── c2l-indicators/
│   ├── c2l-detection/
│   ├── c2l-risk/
│   ├── c2l-quarantine/
│   ├── c2l-events/
│   ├── c2l-reporting/
│   ├── c2l-platform/
│   ├── c2l-platform-windows/
│   └── c2l-cli/
├── tests/
├── tools/
└── docs/
```

A estrutura poderá ser simplificada se a implementação demonstrar que determinados crates são desnecessários.

---

# 26. Ordem de implementação

A ordem recomendada da V0.1 é:

### Etapa 1 — Workspace

- Cargo workspace;
- crates;
- CI;
- lint;
- testes.

### Etapa 2 — Domínio

- tipos;
- erros;
- contratos;
- eventos;
- resultados.

### Etapa 3 — Hashing

- SHA-256;
- streaming;
- testes conhecidos.

### Etapa 4 — Indicator Store

- SQLite;
- schema;
- consultas;
- testes.

### Etapa 5 — Detection

- match de hash;
- evidência;
- classificação.

### Etapa 6 — Scanner

- arquivo;
- diretório;
- recursão;
- erros.

### Etapa 7 — Risk Engine

- estados;
- decisão;
- testes.

### Etapa 8 — Quarantine

- isolamento;
- metadata;
- restauração;
- testes de falha.

### Etapa 9 — Orchestrator

- fluxo completo;
- eventos;
- resultados.

### Etapa 10 — CLI

- `scan`;
- output;
- códigos de saída;
- versão.

### Etapa 11 — Testes E2E

- scan benigno;
- detecção controlada;
- quarentena;
- erros;
- regressões.

---

# 27. Primeiro milestone técnico

O primeiro milestone não será "ter uma interface bonita".

Será:

> **Executar um scan de um ficheiro no Windows, calcular SHA-256, consultar um indicador local, produzir uma decisão estruturada e registrar o resultado.**

Depois:

> **Executar o mesmo fluxo sobre um diretório recursivo e isolar de forma segura um ficheiro de teste previamente classificado como ameaça.**

Esse será o primeiro resultado técnico significativo do C2L.

---

# 28. Decisões adiadas

As seguintes decisões não serão tomadas agora:

- GUI;
- framework de UI;
- cloud;
- backend;
- threat intelligence comercial;
- ML;
- driver/kernel;
- arquitetura EDR completa;
- Android UI;
- sistema de licenciamento;
- contas de utilizador;
- telemetria externa obrigatória.

Adiar essas decisões é deliberado.

---

# 29. Critérios para revisar uma decisão

Uma decisão deste documento poderá ser alterada quando:

1. surgir evidência técnica nova;
2. uma dependência crítica apresentar risco;
3. a portabilidade exigir mudança;
4. performance demonstrar necessidade;
5. segurança exigir alternativa;
6. testes mostrarem que a arquitetura não está funcionando;
7. uma tecnologia deixar de ser mantida.

Toda mudança deverá registrar:

- decisão anterior;
- nova decisão;
- motivo;
- impacto;
- plano de migração;
- versão afetada.

---

# 30. Resultado das decisões

A fundação técnica definida para o C2L é:

| Área | Decisão |
|---|---|
| Core | Rust |
| CLI | Rust |
| Build | Cargo Workspace |
| Hash principal | SHA-256 |
| Indicator Store | SQLite |
| Logs | Estruturados |
| Eventos | Tipados |
| Plataforma inicial | Windows |
| UI | CLI inicialmente |
| Detecção V0.1 | Hash + Indicator Store |
| Risk V0.1 | Determinístico |
| Concorrência | Inicialmente limitada |
| Atualizações assinadas | V0.2 |
| Heurística | V0.3 |
| Behavior | V0.4 |
| Response/Recovery | V0.5 |
| Cloud/TI | Posterior |
| Android | Posterior |

---

# 31. Conclusão

Com estas decisões, o C2L deixa de estar apenas na fase de desenho conceitual.

A arquitetura agora possui uma base tecnológica concreta:

**Rust + Core modular + Windows Adapter + SQLite + SHA-256 + CLI + testes automatizados.**

A V0.1 continuará deliberadamente pequena.

O objetivo não é construir um antivírus completo no primeiro ciclo, mas criar o primeiro núcleo funcional sobre o qual todas as próximas camadas poderão ser construídas.

> **A primeira versão do C2L deverá provar a arquitetura antes de tentar provar a ambição do produto.**
