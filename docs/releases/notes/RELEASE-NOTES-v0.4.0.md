# Digital Ham Radio Logbook v0.4.0

A versão 0.4.0 fortalece interoperabilidade ADIF, preservação de metadados e resistência do parser sem adicionar novos modos ou integrações.

## Interoperabilidade

- corpus permanente com 16 fixtures válidas e 8 inválidas;
- cobertura de header, múltiplos registros, DMR, FT8, Unicode, unknown fields, APP fields, tipos, caixa mista, whitespace, BOM e CRLF;
- entrada não vazia sem tags agora retorna erro em vez de parecer documento vazio;
- nomes e tipos estruturalmente malformados são recusados, mantendo extensões seguras.

## Preservação

- conflitos entre aliases conhecidos são recusados para impedir perda silenciosa;
- frequências RX/TX DMR sobrevivem round-trip usando extensões privadas estáveis;
- round-trip completo passa por parse, dois bancos SQLite e duas exportações;
- QSO comum, DMR completo, FT8 completo, unknown/APP fields tipados e Unicode são verificados semanticamente.

## Exportação

- header canônico mantém `ADIF_VER=3.1.4`;
- `PROGRAMID` permanece estável;
- `PROGRAMVERSION` reflete a versão compilada;
- ordem dos registros e campos permanece determinística;
- performance de 100 mil QSOs permaneceu em aproximadamente 1,03 segundo para montar o documento no ambiente de referência.

## Fuzzing

- target isolado `bytes → UTF-8 válido → parser` com `cargo-fuzz`;
- 24 fixtures usadas como seeds;
- sessão de 60 segundos executou 3.618.168 entradas sem crash conhecido.

## Compatibilidade

- schema SQLite permanece na versão 5;
- nenhuma migration nova;
- nenhuma dependência de runtime nova;
- bancos e configurações anteriores permanecem compatíveis;
- APP fields publicados anteriormente continuam aceitos.

Consulte `docs/ADIF-INTEROPERABILITY.md` e `docs/ADIF-EXTENSIONS.md` para o contrato detalhado. ADIF continua sendo formato de intercâmbio; backup SQLite permanece recomendado para recuperação integral.
