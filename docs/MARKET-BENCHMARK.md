# Benchmark de mercado e decisão de implementação

Pesquisa inicial: 16 de outubro de 2026; decisão e integração concluídas em 19 de
outubro de 2026. Benchmark documental, não um teste comparativo local com malware.

| Referência | Evidência consultada | Aplicação ao C2L |
|---|---|---|
| Bitdefender Total Security 27.0 | AV-TEST agosto/2026: proteção 6, desempenho 6, usabilidade 6 | Referência de produto completo; resultado não transferível para o C2L |
| ESET Security Ultimate 19.1/19.2 | AV-TEST: 6/6/6 | Referência de equilíbrio entre deteção, desempenho e falsos positivos |
| Microsoft Defender 4.18 | AV-TEST: 6/5,5/6; ferramentas de análise documentadas no Windows | Manter a proteção existente; uma interface que apenas invoca Defender não seria motor próprio |
| ClamAV | Motor, análise de ficheiros/arquivos e bases CVD oficiais; FreshClam atualiza as bases | Integrar agora para deteção manual; não confundir com uma suite Windows de proteção em tempo real |
| C2L 0.3.1 | Hashes/CSV não autenticado, 772 108 ficheiros lidos, histórico incremental | Boa infraestrutura de leitura; sem evidência de eficácia antivírus |

Fonte das notas: [AV-TEST, agosto de 2026](https://www.av-test.org/en/antivirus/home-windows/).
São notas de três categorias até 6 pontos, não percentagens de deteção. Não foi
feita comparação de preços nem avaliação empírica da eficácia do C2L contra essas suites.

## Decisão

Integrar ClamAV 1.5.4 como processo separado, com caminhos absolutos e argumentos
fixos, sem shell nem opções de eliminação. Usar `--official-db-only=yes` e as bases
oficiais atualizadas por FreshClam. CVD é um formato assinado; a validação de base
fica no motor/atualizador existente, sem inventar um formato criptográfico novo.
O modo SHA-256 continua disponível como diagnóstico, mas deixa de ser a opção
principal. Bases CSV não são usadas no modo antimalware.

O motor foi descarregado da publicação Cisco-Talos oficial, com SHA-256 do pacote
fixado no instalador. O executável dessa distribuição não tem Authenticode;
verificação do hash do pacote não é assinatura de código do C2L. O pacote mantém
os seus avisos GPLv2; não foi incorporado código libclamav no binário MIT do C2L.
A distribuição pública deve preservar licenças e cumprir as obrigações aplicáveis.

Fontes primárias:
- [ClamAV: análise, limites e ausência de suite endpoint](https://docs.clamav.net/manual/Usage/Scanning.html)
- [ClamAV: atualização oficial](https://docs.clamav.net/manual/Usage/SignatureManagement.html)
- [ClamAV: contentores CVD assinados](https://docs.clamav.net/manual/Signatures.html)
- [ClamAV: licença](https://docs.clamav.net/Introduction.html)
- [Defender: interface de linha de comandos](https://learn.microsoft.com/en-us/defender-endpoint/command-line-arguments-microsoft-defender-antivirus)

## Critérios das entregas, por ordem

1. **0.4: deteção manual operacional.** Motor instalado, atualização verificável,
   análise de ficheiro/pasta, relatório persistido, identificação do motor/base,
   cancelamento e falhas explícitas. Testar benignos, limites e EICAR; registar
   interferência do antivírus residente sem o desativar. EICAR não mede eficácia.
2. **Resposta segura.** Quarentena transacional com consentimento por ficheiro,
   metadados e hash, ACL restrita, prevenção de execução e restauração sem
   sobrescrita; testes de crash e falha de disco antes de usar dados reais.
3. **Qualidade e distribuição.** Corpus benigno representativo, falsos positivos,
   amostras de teste controladas, latência/memória, instalador e assinatura do
   aplicativo, deteção de bases antigas e estratégia de atualização do motor.
4. **Proteção contínua Windows.** Serviço mínimo separado, IPC autorizado,
   observação de eventos e cobertura mensurável. Só avaliar minifilter se houver
   necessidade demonstrada de bloqueio pré-acesso. Não anunciar tempo real antes disso.

Acelerar significa reutilizar deteção e atualização maduras e suspender melhorias
cosméticas não essenciais. Não significa apresentar o scanner manual como uma
proteção completa ou atribuir-lhe as notas de outro fornecedor.
