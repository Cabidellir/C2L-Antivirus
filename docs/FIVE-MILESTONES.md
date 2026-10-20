# Cinco marcos — desktop 0.5.0

Implementação e validação local: 19 de outubro de 2026. Esta é uma entrega
experimental de engenharia, não uma certificação de antivírus de produção.
Não foram desativadas proteções, criadas exclusões, alteradas permissões de
volumes, executadas amostras de malware ou publicados dados pessoais.

## 1. Deteção demonstrável e qualidade

O C2L identificou **Clamav.Test.File-6**, amostra inofensiva oficial do fornecedor,
usando ClamAV 1.5.4 e as bases oficiais. O teste exige uma deteção, zero erros,
resumo completo e a mesma deteção ao reabrir SQLite; uma falha não passa como
deteção. Tempo observado: cerca de 9,2 s, incluindo carregamento das bases.

`scripts/prepare-clamav-test.ps1` obtém a fixture do repositório Cisco-Talos e
verifica o SHA-256 decodificado antes de escrever. Nunca executa a amostra.
O script para imediatamente se o conteúdo do fornecedor mudar.

Corpus sintético benigno: 100 textos Unicode, 10 JSON e 5 binários de 1 MiB;
115 ficheiros, zero alertas/erros, 11,83 s observados. É uma regressão funcional,
não um corpus representativo nem uma taxa de falsos positivos comercial.
Uma base com cabeçalho plausível e conteúdo inválido foi rejeitada pelo motor;
o relatório conservou falha explícita. Cancelamento durante execução: cerca de
656 ms observados numa execução, não uma garantia de latência em todo o hardware.

**Pendente:** EICAR permaneceu inconclusivo por erro de leitura na validação
anterior; a deteção positiva acima não deve ser descrita como EICAR aprovado.
Faltam corpus benigno representativo, falsos negativos, testes de formatos e
campanha independente de eficácia. Não foram criadas heurísticas próprias sem
evidência; continuam a ser usadas as capacidades do motor ClamAV.

## 2. Bases com publicação transacional

A atualização obtém um lock do sistema operativo, copia a base atual para uma
nova geração, executa FreshClam e confirma que o motor consegue carregar apenas
bases oficiais. Só depois muda o apontador ativo numa transação SQLite durável.
Scans em curso continuam a usar a geração anterior. Falhas conservam a referência
anterior; não há atualização em cima dos ficheiros que um scan está a ler.

A versão diária não pode descer relativamente ao estado local. Datas inválidas
ou mais de um dia no futuro são recusadas; a GUI mostra idade e avisa após sete
dias. Uma atualização que continue com mais de sete dias não é publicada.
Assinaturas/formato/validação criptográfica são responsabilidade do ClamAV e
FreshClam; o C2L não implementa criptografia de distribuição alternativa.

Validação: atualização real publicada (base 28158), geração anterior conservada,
rejeição de downgrade e de cabeçalhos/datas inválidos em testes. Uma falha real
de sincronização em disco durante desenvolvimento preservou a base anterior;
o problema foi corrigido e a atualização repetida com sucesso.

**Limites:** atualização manual; máximo de dez diretórios de geração, incluindo
tentativas falhadas, antes de solicitar manutenção. Sem limpeza automática,
atualização automática do executável ClamAV ou política própria de revogação.
Estado local não é resistente a adulteração pelo próprio utilizador/admin.

## 3. Quarentena e recuperação com consentimento

Fluxo: selecionar ficheiro → mostrar caminho/tamanho/hash → confirmar → abrir
com acesso exclusivo → revalidar conteúdo → cifrar com DPAPI do utilizador →
verificar → gravar cópia e intenção numa transação → reler/verificar a cópia
persistida → remover o original pelo mesmo handle → registar conclusão.

O cofre tem DACL protegida para proprietário, SYSTEM e administradores. O
conteúdo é cifrado e autenticado por DPAPI; não é guardado como executável.
Mantém caminho, hash, razão/consentimento, data, atributos, data de modificação
e descritor de segurança de origem. Auditoria local SQLite regista cópia,
remoção, pedidos de recuperação, reanálise e resultado. Não é uma auditoria
resistente ao administrador nem uma âncora externa de confiança.

Antes de recuperar pela GUI, a cópia é reavaliada em memória com a base atual.
Falhas/cobertura incompleta impedem a confirmação; deteções continuam visíveis
na mensagem de reavaliação. Depois há confirmação específica do destino. O
destino é criado exclusivamente, nunca sobrescrito, sincronizado e verificado;
a cópia cifrada permanece. A confirmação expira após cinco minutos.

Testes: alteração após confirmação; hard links e streams adicionais; cópia
adulterada; destino existente; quota SQLite esgotada; confirmação expirada;
falhas injetadas e saída efetiva de subprocesso em ambas as fases de remoção.
O original ficou preservado quando a gravação falhou; a cópia foi recuperável
depois da saída abrupta. Também há teste de reanálise/restauração com motor real.

**Limites:** manual, por utilizador, 32 MiB por ficheiro e cerca de 1 GiB por
banco (journal precisa de espaço adicional). Não aceita UNC, caminhos de
dispositivo, junctions/reparse points, hard links, EFS ou streams adicionais.
Recuperação herda permissões/datas do destino; os metadados originais ficam
registados, mas não são reaplicados automaticamente. DPAPI exige as credenciais
do mesmo perfil/máquina; não é um backup portátil. Sem limpeza automática.
Estado “cópia guardada” após crash significa remoção não confirmada, não sucesso.
Testes de saída de processo não simulam falha elétrica real ou hardware de disco.

Não existe serviço privilegiado nem remediação de ficheiros protegidos do
sistema. A GUI não recebe elevação e não transforma deteções automaticamente
em remoções. O adaptador Windows concentra o pequeno conjunto de chamadas
nativas; núcleo portátil/Android não recebe dependências Windows.

## 4. Observação pós-alteração

Página Monitorização: seleção explícita de uma pasta; análise inicial e
reanálises após eventos. `notify` usa o backend Windows ReadDirectoryChangesW.
Fila de 1024 eventos, agrupamento de alterações, espera normal de dois segundos
e espera máxima de dez segundos para lotes contínuos; intervalo mínimo de
trinta segundos entre inícios de scan. Apenas uma análise principal de cada vez.
Pasta contendo dados C2L é recusada para evitar ciclos com os próprios relatórios.

Overflow/erros pedem análise integral, incrementam um contador visível de lacunas
e não são tratados como ausência de alterações. Ao reabrir, observação está
desligada; ao iniciar novamente há novo scan integral. Remoção da pasta para a
observação com mensagem. Teste nativo confirma eventos de escrita; teste injeta
overflow e verifica necessidade de reconciliação.

Processos ClamAV/FreshClam pertencem a Job Objects com término ao fechar o
último handle e teto de memória por processo de 2 GiB. Scans têm limite global
de duas horas e 30 s por conteúdo no motor, além dos limites de tamanho. A GUI
mostra duração e pico de memória comprometida comunicado pelo Job Object.
Isso não é isolamento de segurança completo do parser nem limite de CPU/I/O.

**Pendente para M5 do roadmap original:** serviço SCM, IPC autorizado, USN,
persistência de cursores e cobertura entre reinícios. Esta observação depende
da GUI aberta, não impede execução e não é anunciada como proteção em tempo real.
Reanálise recupera o estado atual, não ficheiros já desaparecidos durante lacunas.

## 5. Pacote e validação de distribuição

`scripts/package-desktop.ps1` produz um ZIP experimental com executável, licença,
instruções, instalador por utilizador e hashes de conteúdo. Motor/bases são
instalados separadamente, preservando o pacote/licenças oficiais do ClamAV.
`scripts/install-desktop.ps1` verifica conteúdo, usa diretório por versão e
recusa sobrescrever uma instalação existente; relatórios/cofre permanecem fora
da pasta da aplicação. Não instala serviço, não cria arranque automático e não
pede escrita irrestrita em discos. Versões anteriores permanecem disponíveis.

`-RequireSigned` recusa o executável sem Authenticode válido. Não existe nesta
máquina um certificado de assinatura disponível, e não foi criado um certificado
autossinado para simular confiança. **Distribuição de produção bloqueada:**
faltam assinatura confiável de todos os componentes/pacote, atualização
autenticada, testes em máquina limpa/VM e revisão independente. Hashes locais
detetam corrupção; não autenticam o editor. `productionReady` permanece falso.

## Como testar esta entrega

1. Abrir `c2l-desktop.exe` 0.5.0. Em Analisar, usar Antimalware e atualizar bases.
2. Analisar uma pasta conhecida e reabrir o relatório. Conferir base, erros e duração.
3. Em Quarentena, testar primeiro uma cópia descartável de um documento inofensivo.
   Confirmar isolamento, atualizar lista, recuperar para outro nome e comparar conteúdo.
4. Em Monitorização, escolher uma pasta de teste fora de AppData, escrever um
   ficheiro e acompanhar o resultado; parar a observação quando terminar.
5. Para outra instalação experimental, extrair o ZIP e executar
   `scripts/install-desktop.ps1`; preparar motor com `scripts/setup-clamav.ps1`.

Para testes de engenharia, consultar `USER-VALIDATION.md` e os testes opcionais
identificados por `real_`. Não executar malware real nem desativar outro antivírus.

## Referências técnicas

- [ClamAV/FreshClam](https://docs.clamav.net/manual/Usage/SignatureManagement.html)
- [Fixture inofensiva oficial](https://github.com/Cisco-Talos/clamav/tree/main/unit_tests/input)
- [DPAPI](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata)
- [Remoção pelo handle](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle)
- [Eventos de diretório](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw)
- [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)
