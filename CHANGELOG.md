# Changelog

Este projeto segue [Semantic Versioning](https://semver.org/). As release notes de cada versão permanecem em `docs/RELEASE-NOTES-v*.md`.

## [Unreleased]

## [0.4.0] - Em desenvolvimento

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
