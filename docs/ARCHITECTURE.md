# Arquitetura

## Visão geral

O aplicativo permanece desktop, local e síncrono. Slint apresenta a interface, os módulos de domínio validam dados e `QsoRepository` encapsula SQLite/rusqlite. Não há servidor, ORM, pool de conexões ou runtime assíncrono.

## Persistência

A API externa continua centralizada em `QsoRepository`; consumidores não conhecem a organização dos arquivos internos.

```text
src/database/
├── migrations.rs
└── repository/
    ├── mod.rs      conexão, agregado QSO, CRUD e transações DMR/FT8
    ├── queries.rs  listagem, paginação, pesquisa, filtros e materialização
    ├── adif.rs     preview, importação, exportação e campos extras
    ├── backup.rs   snapshot, integridade e permissões
    └── stress.rs   benchmark pesado de teste, ignorado por padrão
```

### Fronteiras preservadas

- QSO comum e metadados DMR/FT8 permanecem juntos porque inserções e mudanças de modo exigem atomicidade.
- Queries permanecem SQLite explícito e retornam os mesmos tipos públicos.
- ADIF foi separado por possuir conversão, política de duplicidade e caminho de exportação próprios.
- Backup foi separado por combinar snapshot SQLite, filesystem, validação e durabilidade.
- Migrations continuam isoladas e são executadas somente durante abertura do repository.

## Garantias

- foreign keys habilitadas em toda abertura;
- migrations transacionais e schemas futuros recusados;
- `PRAGMA quick_check` e `foreign_key_check` na abertura e validação de backup;
- QSO + DMR/FT8 e importação ADIF são transacionais;
- ordenação da listagem: `datetime_start_utc DESC, id DESC`;
- paginação atual usa `LIMIT/OFFSET`, preservada por não haver degradação relevante em 100 mil QSOs;
- SQLite permanece a fonte de verdade.

## Onde alterar

- novo comportamento comum de persistência: `repository/mod.rs`;
- listagem, pesquisa ou filtro: `repository/queries.rs`;
- importação/exportação ADIF: `repository/adif.rs`;
- backup e integridade operacional: `repository/backup.rs`;
- evolução do schema: nova migration em `migrations.rs`, sem editar migrations publicadas.
