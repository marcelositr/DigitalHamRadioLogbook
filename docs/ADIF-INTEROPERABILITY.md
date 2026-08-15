# Interoperabilidade ADIF

O projeto suporta os campos necessários ao aplicativo e preserva campos desconhecidos quando possível. Isto não representa suporte integral a todo o universo ADIF.

## Compatibilidade

| Recurso | Import | Export |
|---|---:|---:|
| QSO comum | sim | sim |
| Header | sim | header canônico |
| DMR suportado | sim | sim |
| FT8 suportado | sim | sim |
| D-STAR suportado | `MODE=DSTAR` histórico ou `DIGITALVOICE` + `DSTAR` | `DIGITALVOICE` + `DSTAR` canônico |
| Campos desconhecidos | preserva | preserva |
| Tipos de campos desconhecidos | preserva | preserva |
| Unicode UTF-8 | sim | sim |
| LF e CRLF | sim | LF canônico |
| BOM UTF-8 inicial | sim | não emitido |

## Campos comuns

Campos obrigatórios para converter um registro em QSO:

- `CALL`;
- `QSO_DATE` (`YYYYMMDD`);
- `TIME_ON` (`HHMM` ou `HHMMSS`);
- `FREQ` em MHz com até seis casas;
- `MODE`.

Opcionais modelados:

- `BAND`;
- `RST_SENT`;
- `RST_RCVD`;
- `GRIDSQUARE`;
- `NAME`;
- `QTH`;
- `COMMENT`, com `NOTES` como alias de importação.

Callsign, modo e grid são normalizados para uppercase. Valores comuns passam pelas validações do domínio. Frequência é convertida sem ponto flutuante e preserva resolução de 1 Hz.

## D-STAR

O modo de domínio é `DSTAR`. Na importação são aceitas duas formas:

- canônica: `MODE=DIGITALVOICE` e `SUBMODE=DSTAR`;
- histórica: `MODE=DSTAR`.

A exportação sempre usa a forma canônica `DIGITALVOICE` + `DSTAR`. O subconjunto modelado contém reflector, module, MYCALL, URCALL, RPT1, RPT2 e notes. `STATION_CALLSIGN` é o campo ADIF canônico usado para MYCALL; `APP_DHRL_DSTAR_MYCALL` permanece aceito como alias de importação. Se ambos aparecerem com valores diferentes, o registro é recusado em vez de perder informação silenciosamente. Os demais valores D-STAR usam as extensões descritas em `docs/ADIF-EXTENSIONS.md`.

Esse mapeamento não representa suporte total a D-STAR, a todos os equipamentos ou a todos os campos/dialetos ADIF relacionados.

## Parser

- comprimentos são bytes UTF-8, não quantidade de caracteres;
- tags e tipos são case-insensitive e normalizados para uppercase;
- preâmbulo textual antes da primeira tag é tolerado;
- entrada vazia/whitespace é documento sem registros;
- entrada não vazia sem tag é erro;
- nomes com caracteres estruturais/whitespace e tipos malformados são recusados;
- registros estruturais quebrados retornam erro com byte offset;
- arquivo sintaticamente válido pode conter registro semanticamente inválido; o preview reporta esse registro sem gravá-lo.

## Header exportado

O exporter usa:

- `ADIF_VER=3.1.4`;
- `PROGRAMID=Digital Ham Radio Logbook`;
- `PROGRAMVERSION` igual à versão compilada.

O header original de um arquivo importado não é persistido por QSO; uma exportação cria header canônico da aplicação.

## Duplicidade

A identidade exata de importação é:

```text
callsign + início UTC + frequência Hz + modo
```

Duplicados já existentes ou repetidos no mesmo arquivo são ignorados e contabilizados; não há merge ou sobrescrita. Inserção manual continua permitindo identidades iguais.

## Corpus

Fixtures sintéticas e sem dados pessoais estão em `tests/fixtures/adif/`:

- 16 válidas;
- 8 inválidas;
- QSO comum, DMR, FT8, múltiplos modos, Unicode, unknown/APP fields, tipos, caixa mista, whitespace, CRLF e estruturas truncadas.

Elas representam padrões interoperáveis gerais, não alegam homologação formal de um software externo específico.

## Fuzzing

Requisitos de desenvolvimento:

```sh
cargo install cargo-fuzz --locked
rustup toolchain install nightly --profile minimal
```

Execução usando as fixtures como seeds:

```sh
cargo +nightly fuzz run adif_parser tests/fixtures/adif/valid tests/fixtures/adif/invalid -- -max_total_time=60
```

O target aceita bytes arbitrários, envia somente UTF-8 válido ao parser e não acessa banco ou dados reais.

## Limitações

- o documento é carregado integralmente em memória;
- tipos explícitos de campos conhecidos são canonicalizados, não preservados;
- ordem relativa entre unknown fields é preservada, mas sua posição entre campos conhecidos não;
- `TIME_OFF` e `QSO_DATE_OFF` ainda são preservados como campos desconhecidos, não integrados às colunas de domínio; `SUBMODE=DSTAR` é reconhecido no mapeamento D-STAR;
- ADIF é intercâmbio; backup SQLite continua sendo o mecanismo de recuperação integral recomendado.

Extensões privadas estão especificadas em `docs/ADIF-EXTENSIONS.md`.
