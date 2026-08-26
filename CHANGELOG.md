# Changelog

Este projeto segue [Semantic Versioning](https://semver.org/). As release notes de cada versão permanecem em `docs/RELEASE-NOTES-v*.md`.

## [Unreleased]

## [0.7.0] - Não publicada

### Added

- ação **Save & New** exclusiva da criação de QSO: valida e grava o formulário, atualiza a listagem, limpa todos os campos e metadados e inicia o próximo formulário com um novo UTC fixo;
- aviso de possível duplicidade manual pela identidade callsign + UTC inicial + frequência em Hz + modo, com as ações **Review** e **Save anyway**;
- atalhos `Ctrl+N`, `Ctrl+S`, `Ctrl+Enter` e `Ctrl+F`, preservando o conteúdo do clipboard.

### Changed

- foco inicial direcionado ao callsign ao abrir um novo QSO e à pesquisa ao usar `Ctrl+F`;
- `Enter` em Notes continua salvando e `Escape` permanece exclusivamente reservado ao cancelamento/fechamento do fluxo atual;
- salvamentos usam proteção contra double-submit e atualizam o snapshot do formulário somente após commit bem-sucedido;
- edição exclui o próprio QSO da detecção de duplicidade.

### Compatibility

- duplicidade manual é somente um aviso: não mescla, não bloqueia e não adiciona restrição `UNIQUE`;
- schema permanece na versão 7, sem migration ou índice novo;
- medições release em 100 mil QSOs confirmaram o plano por `idx_qsos_datetime_start`; nenhum índice adicional foi adotado.

## [0.6.0] - Publicada

### Added

- suporte específico a YSF/System Fusion, representado internamente pelo modo `C4FM`, em domínio, schema 7, repository, filtros, ADIF e UI;
- metadados YSF para room, WIRES-X node, repeater, network, access type, TX/RX DG-ID e notes;
- filtros YSF por room, WIRES-X node e DG-ID;
- enum consolidado `ModeMetadata` para metadados DMR, FT8, D-STAR, YSF e modo genérico.

### Changed

- UI aceita os aliases `YSF` e `SYSTEM FUSION`, normalizando-os para `C4FM`;
- ADIF YSF é exportado como `MODE=DIGITALVOICE` + `SUBMODE=C4FM` e também importa o histórico `MODE=C4FM`;
- persistência e importação exigem integridade entre modo e variante de metadata e reconciliam campos ADIF extras ao trocar de modo.

### Compatibility

- `digital_routes` continua específico de DMR;
- schema 7 adiciona `ysf_metadata`; somente TX/RX DG-ID receberam índices após inspeção com `EXPLAIN QUERY PLAN`;
- não foram introduzidos traits nem plugins de modo.

A versão 0.6.0 foi publicada como tag/release; `main` e a tag estavam no commit `034996f`.

## [0.5.0] - Publicada

### Added

- suporte específico e limitado a D-STAR no domínio, SQLite, repository, queries, ADIF e UI;
- modelo D-STAR com reflector, module, MYCALL, URCALL, RPT1, RPT2 e observações;
- migration para schema 6 e filtros por reflector, module e RPT1;
- extensões ADIF privadas `APP_DHRL_DSTAR_*`, com `STATION_CALLSIGN` como representação canônica de MYCALL.

### Changed

- exportação D-STAR usa `MODE=DIGITALVOICE` + `SUBMODE=DSTAR`; importação continua aceitando o histórico `MODE=DSTAR`;
- limpeza transacional de metadata incompatível foi fatorada a partir do comportamento já existente para acomodar D-STAR, sem introduzir traits ou plugins.

### Compatibility

- `digital_routes` permanece específico de DMR;
- suporte D-STAR cobre somente o subconjunto documentado, sem promessa de interoperabilidade total.

A versão 0.5.0 foi publicada como tag/release; `main` estava no commit `ef262bd`.

## [0.4.0] - 2026-08-15

### Added

- corpus ADIF permanente com fixtures válidas e inválidas;
- round-trip completo por dois bancos SQLite para QSO comum, DMR, FT8, unknown fields e Unicode;
- target `cargo-fuzz` isolado para o parser;
- documentação de interoperabilidade e extensões `APP_DHRL_*`.

### Changed

- parser trata BOM/CRLF conscientemente e valida nomes/tipos estruturais;
- header exportado inclui `PROGRAMVERSION` derivado da versão compilada.

### Fixed

- conflitos entre aliases ADIF agora são recusados em vez de descartar um valor;
- frequências RX/TX DMR passam a sobreviver round-trip ADIF.

A versão 0.4.0 foi concluída e publicada como release final.

## [0.3.0] - 2026-08-15

### Added

- benchmark pesado, determinístico e ignorado por padrão para bancos de 1 mil a 1 milhão de QSOs;
- smoke test POSIX do pacote Linux, instalação, reinstalação e desinstalação em XDG isolado;
- job de CI para o contrato do pacote Linux.

### Changed

- repository SQLite organizado internamente por CRUD, consultas, ADIF e backup, sem alterar sua API pública;
- exportação ADIF carrega metadados e campos adicionais em lote, eliminando consultas por QSO;
- geração do pacote Linux normaliza metadados do tarball e publica tarball/checksum por arquivos temporários.

### Fixed

- custo N+1 da exportação ADIF, reduzindo significativamente o tempo observado em bases grandes;
- possibilidade de checksum antigo permanecer ao lado de um novo tarball após falha intermediária de empacotamento.

### Security

- nenhuma dependência de runtime adicionada;
- pacote continua gerado com `Cargo.lock` e checksum SHA-256.

## [0.2.2] - 2026-08-15

- hardening de banco, migrations, backup, configuração, XDG, ADIF e transações;
- detalhes completos em `docs/RELEASE-NOTES-v0.2.2.md`.

## [0.2.1] - 2026-08-14

- redesign visual completo e homologação em i3/`1050×680`;
- detalhes completos em `docs/RELEASE-NOTES-v0.2.1.md`.

## [0.2.0]

- links externos configuráveis, testes de escala, paginação e refinamentos de distribuição;
- detalhes completos em `docs/RELEASE-NOTES-v0.2.0.md`.

## [0.1.0]

- primeira release funcional;
- detalhes completos em `docs/RELEASE-NOTES-v0.1.0.md`.
