# Digital Ham Radio Logbook

[![CI](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/workflows/ci.yml)

Desktop logbook for amateur radio, built with **Rust**, **Slint** and **SQLite**. The application is local-first and works offline, with dedicated workflows for digital modes and ADIF interoperability.

![Digital Ham Radio Logbook](docs/assets/logbook-v0.11.png)

*Interface v0.11 em desenvolvimento. A captura usa QSOs sintéticos de demonstração.*

## Principais recursos

- registro, pesquisa, edição e exclusão de QSOs;
- suporte a contatos genéricos, DMR, FT8, D-STAR e YSF/System Fusion (`C4FM`);
- filtros e listagem paginada para operação diária;
- importação e exportação ADIF, incluindo preservação de campos desconhecidos;
- backup SQLite, verificação de backup e health check local;
- interface Slint com style Fluent e esquemas **System**, **Light** e **Dark**.

Os dados permanecem no computador do usuário. O projeto não exige conta, serviço em nuvem ou sincronização online para manter o logbook.

## Executar a partir do código

O toolchain Rust usado pelo projeto é fixado em `rust-toolchain.toml`.

```sh
cargo run --locked
```

Para instalação e distribuição no GNU/Linux, consulte [Linux distribution](docs/operations/LINUX-DISTRIBUTION.md).

## Documentação

O [GitHub Wiki](https://github.com/marcelositr/DigitalHamRadioLogbook/wiki) concentra instalação, uso e orientação ao usuário. A documentação de engenharia está organizada em [docs/](docs/README.md), com arquitetura, contratos de dados, qualidade, operações e processo de release.

Consulte também o [changelog](docs/releases/CHANGELOG.md) e a [matriz de suporte](docs/operations/SUPPORT-MATRIX.md).

## Status do projeto

O projeto permanece em desenvolvimento pré-1.0. O checkpoint empacotado de referência é `0.10.0-rc.1`, enquanto a reconstrução da interface v0.11 segue em desenvolvimento e revisão.

## Licença

Distribuído sob a [MIT License](LICENSE).
