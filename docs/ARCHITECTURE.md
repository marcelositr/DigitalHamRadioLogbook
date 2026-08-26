# Arquitetura

## Visão geral

O aplicativo permanece desktop, local e síncrono. Slint apresenta a interface, os módulos de domínio validam dados e `QsoRepository` encapsula SQLite/rusqlite. Não há servidor, ORM, pool de conexões ou runtime assíncrono.

## Persistência

A API externa continua centralizada em `QsoRepository`; consumidores não conhecem a organização dos arquivos internos.

```text
src/database/
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
- Migrations continuam isoladas e são executadas somente durante abertura do repository.

## Garantias

- foreign keys habilitadas em toda abertura;
- migrations transacionais e schemas futuros recusados;
- `PRAGMA quick_check` e `foreign_key_check` na abertura e validação de backup;
- QSO + DMR/FT8/D-STAR/YSF e importação ADIF são transacionais;
- ordenação da listagem: `datetime_start_utc DESC, id DESC`;
- paginação atual usa `LIMIT/OFFSET`, preservada por não haver degradação relevante em 100 mil QSOs;
- SQLite permanece a fonte de verdade.

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
- backup e integridade operacional: `repository/backup.rs`;
- evolução do schema: nova migration em `migrations.rs`, sem editar migrations publicadas.

Os índices YSF são limitados a TX/RX DG-ID. `EXPLAIN QUERY PLAN` não demonstrou benefício para room e WIRES-X node, consultados por substring, portanto essas colunas permanecem sem índice.
