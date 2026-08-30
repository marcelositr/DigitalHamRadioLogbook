# Changelog

Este projeto segue [Semantic Versioning](https://semver.org/). As release notes de cada versão permanecem em [`notes/`](notes/).

> **Nota histórica:** entradas anteriores a `0.11.0-rc.1` registram checkpoints de desenvolvimento e a evolução técnica do projeto. Tags e GitHub Releases antigas usadas durante o desenvolvimento foram removidas na limpeza de governança de 2026-08-29. A primeira publicação preservada como distribuição pública corrente é `v0.11.0-RC1`, marcada como **Pre-release**.

## [0.11.0-rc.1] - 2026-08-29

**Pre-release:** candidato público para avaliação; não representa release estável nem declaração de prontidão.

Release notes: [`notes/RELEASE-NOTES-v0.11.0-RC1.md`](notes/RELEASE-NOTES-v0.11.0-RC1.md).

### Changed

- reconstruída a camada gráfica v0.11 a partir dos contratos funcionais, sem reutilizar a interface anterior como referência visual;
- adotada uma arquitetura Slint-native inspirada na Widgets Gallery, com `MenuBar`, `GroupBox`, widgets padrão, `Palette`, `StyleMetrics` e dimensionamento natural pelo conteúdo;
- substituídos menu superior simulado, barra contextual e tema proprietário por menu nativo, sidebar simples, workspace central e status global;
- Logbook reconstruído como workspace de dados; editor de QSO, Tools e Settings foram reescritos com layouts e widgets nativos sem alterar regras de domínio ou persistência SQLite;
- **Fluent** definido como style oficial do produto em `build.rs`;
- Settings passa a oferecer **System / Light / Dark** em Appearance, aplicados imediatamente via `Palette.color-scheme` e persistidos de forma retrocompatível no `config.toml`, com `System` como padrão;
- adicionados `../architecture/UI-ARCHITECTURE-v0.11.md` e `../quality/VISUAL-QA-v0.11.md` para documentar a arquitetura e o novo gate de regressão manual;
- documentação reorganizada por responsabilidade em `docs/architecture/`, `docs/data/`, `docs/operations/`, `docs/quality/`, `docs/releases/` e `docs/project/`, com `docs/README.md` como índice técnico;
- testes estruturais passaram a proteger a arquitetura Slint-native, o style Fluent e o contrato de aparência System/Light/Dark.

### Compatibility

- Slint permanece como toolkit gráfico; nenhuma migração para Tauri, Electron, Qt ou GTK foi realizada;
- schema SQLite, migrations, ADIF, repository e dependências de runtime permanecem inalterados;
- configurações antigas sem a nova seção `appearance` continuam válidas e usam `system` por padrão;
- a homologação visual das versões anteriores não é herdada pela reconstrução v0.11; nova aprovação manual em `1050×680`, cobrindo System, Light e Dark, permanece necessária antes de concluir o ciclo.

## [0.10.0-rc.1] - 2026-08-28 · checkpoint histórico

### Changed

- criado registro factual de maturidade pré-1.0, sem declarar o projeto pronto para `1.0.0`;
- documentadas as categorias primary, tested, best effort e not tested para ambientes e contratos suportados;
- consolidado checklist reproduzível de release, upgrade, ADIF, recuperação, artefato exato e autorização;
- adicionados pacotes `.deb` e AppImage, com metadata AppStream e checksums, derivados do mesmo binário validado do tarball;
- corrigido o checklist visual que ainda apresentava a regressão já aprovada como pendente.

### Compatibility

- feature freeze permanece ativo: nenhuma funcionalidade, migration, índice ou dependência foi adicionada;
- schema permanece na versão 7 e downgrade automático continua não suportado;
- o checkpoint deriva de `0.9.0-rc.1` validado no fluxo de desenvolvimento usado naquele ciclo; `v0.8.0` e `v0.9.0` não tiveram distribuição pública preservada.

### Maturity

- não existe declaração de prontidão para `1.0.0`;
- uso cotidiano prolongado, múltiplos ciclos estáveis e cobertura adicional de ambientes permanecem evidências ainda necessárias.

## [0.9.0-rc.1] - 2026-08-28 · checkpoint histórico

### Fixed

- restaurada a reprodução locked/offline do target de fuzz ADIF após a evolução da versão do pacote;
- corpus mutável do libFuzzer separado das fixtures ADIF versionadas, evitando que execuções de regressão poluam o corpus permanente;
- documentação do corpus corrigida para refletir as 22 fixtures válidas existentes.

### Validation

- upgrades reais sequenciais de v0.4.0 até a baseline v0.8.0 e upgrades diretos preservaram schema, cinco modos, extras ADIF, configuração e integridade;
- fuzzing ADIF executou 3.622.542 entradas em 60 segundos sem crash;
- regressões de virada UTC, leap day e datas/horas inválidas adicionadas;
- suíte locked passou três vezes consecutivas com 176 testes ativos, além da matriz de migrations 0–7, packaging e disaster drill automatizado;
- baselines release de 10 mil e 100 mil QSOs não mostraram regressão relevante.

### Compatibility

- feature freeze: nenhuma funcionalidade, migration, índice ou dependência de runtime foi adicionada;
- schema permanece na versão 7;
- o release candidate teve QA manual e artefato exato validados no fluxo de desenvolvimento daquele ciclo, sem permanecer como publicação pública corrente.

## [0.8.0] - checkpoint histórico, não publicado

### Added

- **Check data health** read-only para integridade SQLite, foreign keys, schema, migrations, contagens e invariantes de metadata por modo;
- verificação read-only de backups existentes, distinguindo schema atual, antigo migrável, futuro incompatível e arquivo inválido/corrompido;
- **Export current results**, incluindo todos os QSOs da pesquisa/filtro atual e não somente a página visível.

### Changed

- backups são criados em temporário, validados read-only, sincronizados e publicados sem sobrescrever o destino;
- Tools separa diagnóstico, ADIF e backup com relatório sem conteúdo pessoal de QSO;
- recuperação permanece assistida/documentada: não existe troca destrutiva do banco enquanto a aplicação está aberta.

### Compatibility

- schema permanece na versão 7, sem migration ou índice novo;
- SQLite backup continua sendo o backup nativo; ADIF permanece o formato de interoperabilidade;
- nenhuma dependência de runtime foi adicionada.

## [0.7.0] - checkpoint histórico

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

Este marco teve tag/release de desenvolvimento à época; essas referências públicas antigas foram removidas durante a limpeza de governança e o conteúdo permanece apenas como histórico técnico.

## [0.6.0] - checkpoint histórico

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

Este marco teve tag/release de desenvolvimento à época; essas referências públicas antigas foram removidas durante a limpeza de governança e o conteúdo permanece apenas como histórico técnico.

## [0.5.0] - checkpoint histórico

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

Este marco teve tag/release de desenvolvimento à época; essas referências públicas antigas foram removidas durante a limpeza de governança e o conteúdo permanece apenas como histórico técnico.

## [0.4.0] - 2026-08-15 · checkpoint histórico

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

Este marco teve tag/release de desenvolvimento à época; essas referências públicas antigas foram removidas durante a limpeza de governança e o conteúdo permanece apenas como histórico técnico.

## [0.3.0] - 2026-08-15 · checkpoint histórico

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

## [0.2.2] - 2026-08-15 · checkpoint histórico

- hardening de banco, migrations, backup, configuração, XDG, ADIF e transações;
- detalhes completos em [`notes/RELEASE-NOTES-v0.2.2.md`](notes/RELEASE-NOTES-v0.2.2.md).

## [0.2.1] - 2026-08-14 · checkpoint histórico

- redesign visual completo e homologação em i3/`1050×680`;
- detalhes completos em [`notes/RELEASE-NOTES-v0.2.1.md`](notes/RELEASE-NOTES-v0.2.1.md).

## [0.2.0] - checkpoint histórico

- links externos configuráveis, testes de escala, paginação e refinamentos de distribuição;
- detalhes completos em [`notes/RELEASE-NOTES-v0.2.0.md`](notes/RELEASE-NOTES-v0.2.0.md).

## [0.1.0] - checkpoint histórico
