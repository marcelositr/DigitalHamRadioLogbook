# Arquitetura

## Visão geral

O aplicativo permanece desktop, local e síncrono. Slint apresenta a interface, os módulos de domínio validam dados e `QsoRepository` encapsula SQLite/rusqlite. Não há servidor, ORM, pool de conexões ou runtime assíncrono.

## Persistência

A API externa continua centralizada em `QsoRepository`; consumidores não conhecem a organização dos arquivos internos.

```text
src/database/
├── health.rs       inspeção read-only, relatório e invariantes de metadata
├── migrations.rs
└── repository/
    ├── mod.rs      conexão, agregado QSO, CRUD e transações DMR/FT8/D-STAR/YSF
    ├── queries.rs  listagem, paginação, pesquisa, filtros e materialização
    ├── adif.rs     preview, importação, exportação e campos extras
    ├── backup.rs   snapshot, integridade e permissões
    └── stress.rs   benchmark pesado de teste, ignorado por padrão
```

### Fronteiras preservadas

- QSO comum e metadados DMR/FT8/D-STAR/YSF permanecem juntos porque inserções e mudanças de modo exigem atomicidade.
- Queries permanecem SQLite explícito e retornam os mesmos tipos públicos.
- ADIF foi separado por possuir conversão, política de duplicidade e caminho de exportação próprios.
- Backup foi separado por combinar snapshot SQLite, filesystem, validação e durabilidade.
- Health check usa conexão read-only/query-only separada, não executa migrations e distingue schema atual, antigo migrável, futuro incompatível e arquivo inválido.
- Migrations continuam isoladas e são executadas somente durante abertura do repository.

## Garantias

- foreign keys habilitadas em toda abertura;
- migrations transacionais e schemas futuros recusados;
- `PRAGMA quick_check` e `foreign_key_check` na abertura e validação de backup;
- QSO + DMR/FT8/D-STAR/YSF e importação ADIF são transacionais;
- ordenação da listagem: `datetime_start_utc DESC, id DESC`;
- paginação atual usa `LIMIT/OFFSET`, preservada por não haver degradação relevante em 100 mil QSOs;
- SQLite permanece a fonte de verdade;
- schema atual permanece 7; os fluxos de `v0.8.0` não exigem migration ou índice novo.
- `QsoSelection` transporta a busca/filtro atual para exportação ADIF sem paginação; metadata é carregada no SELECT principal e extras em uma segunda query restrita.

## Arquitetura de quatro modos

DMR, FT8, D-STAR e YSF/C4FM seguem caminhos explícitos em cada camada. `ModeMetadata` consolida os agregados como `Generic`, `Dmr`, `Ft8`, `Dstar` ou `Ysf` e verifica a integridade entre modo e variante: `DMR`, `FT8`, `DSTAR` e `C4FM` exigem suas variantes; os demais modos exigem `Generic`.

D-STAR usa `dstar_metadata` no schema 6. YSF/System Fusion usa `ysf_metadata` no schema 7, com room, WIRES-X node, repeater, network, access type, TX/RX DG-ID e notes. O nome interno é `C4FM`; `YSF` e `SYSTEM FUSION` são aliases da UI.

A tabela `digital_routes` permanece específica de DMR. SQL, tabelas, consultas e fluxos de UI continuam próprios de cada modo. Não foram criados traits nem plugins: os pontos de extensão crescem linearmente, mas permanecem pequenos, factuais e aceitáveis para quatro modos. A fatoração transversal cobre o enum agregado, a validação modo↔metadata, a limpeza transacional de metadata incompatível e a reconciliação de extras ADIF.

Na reconciliação ADIF, campos privados reconhecidos pelo modo atual são materializados como metadata e removidos da coleção de extras; campos realmente desconhecidos permanecem preservados. Isso impede duplicação ou sobrevivência de campos específicos obsoletos após mudança de modo.

Consulte `docs/WHAT-ADDING-DSTAR-REQUIRED.md`, `docs/ADDING-A-DIGITAL-MODE.md` e `docs/FOUR-MODE-ARCHITECTURE-REVIEW.md`.

## Onde alterar

- novo comportamento comum de persistência: `repository/mod.rs`;
- listagem, pesquisa ou filtro: `repository/queries.rs`;
- importação/exportação ADIF: `repository/adif.rs`;
- backup e durabilidade: `repository/backup.rs`;
- diagnóstico read-only e invariantes do acervo: `database/health.rs`;
- evolução do schema: nova migration em `migrations.rs`, sem editar migrations publicadas.

Os índices YSF são limitados a TX/RX DG-ID. `EXPLAIN QUERY PLAN` não demonstrou benefício para room e WIRES-X node, consultados por substring, portanto essas colunas permanecem sem índice.

## Salvamento manual e aviso de duplicidade

O editor envia um snapshot imutável do formulário para validação e persistência. Um guard de submissão impede double-submit; o snapshot de referência para dirty state só é substituído depois do commit bem-sucedido. Em **Save & New**, exclusivo da criação, a sequência é validar → commit → refresh da listagem → limpeza integral dos campos e metadados → captura de um novo UTC fixo. O QSO recém-gravado não é reenviado.

A consulta de possível duplicidade usa callsign normalizado, UTC inicial, frequência inteira em Hz e modo normalizado. Updates excluem o próprio ID. O resultado é deliberadamente apenas um aviso com **Review** e **Save anyway**: não há merge, bloqueio, migration, índice novo ou constraint `UNIQUE`, preservando duplicados manuais intencionais.

## Arquitetura da interface v0.11

A v0.11 preserva Slint e o contrato público consumido pelo Rust, mas reconstrói a superfície gráfica a partir de princípios Slint-native. A implementação anterior não é usada como referência visual.

```text
ui/
├── appearance.slint
├── design-system.slint
├── main.slint
├── components/
│   └── app-shell.slint
├── models/
└── pages/
    ├── logbook-page.slint
    ├── qso-editor-page.slint
    ├── tools-page.slint
    └── settings-page.slint
```

`ui/main.slint` continua sendo o contrato público compilado por `build.rs`. Ele mantém os callbacks e properties usados por `src/app/*`; a mudança de interface permanece separada do domínio e do SQLite.

O shell é composto por:

- `MenuBar`, `Menu`, `MenuItem` e `MenuSeparator` nativos para comandos globais e secundários;
- sidebar recolhível simples com Logbook, New QSO, Tools e Settings;
- workspace central ocupado por uma das quatro páginas;
- barra de status global fora do conteúdo rolável.

Não existe barra contextual, menu superior simulado nem categorização visual `Operation`, `Data` ou `System`.

`ui/design-system.slint` deixa de representar um tema proprietário. Ele contém somente primitivas semânticas pequenas que faltam em `std-widgets.slint`, como `FormField`, `TextAction`, `EmptyState` e `StatusBar`. Essas primitivas usam `Palette` e `StyleMetrics`.

`ui/appearance.slint` concentra a preferência de esquema de cores e escreve em `Palette.color-scheme`. O style do produto é fixado como **Fluent** em `build.rs`; não existe troca runtime entre Fluent, Material, Cupertino ou Cosmic.

As páginas priorizam widgets padrão e layouts naturais:

- Logbook é um workspace de dados em linhas/colunas, sem cards individuais por QSO;
- editor usa `GroupBox` para Contact, Station and report, Notes e metadata condicional de modo;
- Tools usa grupos para ADIF, Data health e Database backup;
- Settings usa grupos para Appearance, Local station e External lookup links.

Dimensionamento deve preferir conteúdo, `preferred-*`, `min-*` e stretch. Medidas fixas são reservadas a casos estruturais previsíveis, como largura da sidebar e alinhamento de colunas da listagem.

A aparência oferece `System`, `Light` e `Dark`. `System` é o default e usa `ColorScheme.unknown`, permitindo que o Fluent acompanhe o esquema reportado pelo desktop; Light e Dark forçam seus respectivos valores. A preferência é armazenada em `config.toml` na seção `appearance`, de forma retrocompatível, sem migration ou mudança no schema SQLite.

Clipping, sobreposição, separador atravessando input, botão truncado ou conteúdo essencial inacessível são falhas de QA em qualquer um dos três esquemas.

Detalhes, restrições e gate da reconstrução ficam em `docs/UI-ARCHITECTURE-v0.11.md` e `docs/VISUAL-QA-v0.11.md`. A homologação visual anterior não é herdada; o novo layout exige regressão manual em `1050×680`, nos modos System, Light e Dark, antes de ser considerado aprovado.
