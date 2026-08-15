# Arquitetura

## Visão geral

O aplicativo permanece desktop, local e síncrono. Slint apresenta a interface, os módulos de domínio validam dados e `QsoRepository` encapsula SQLite/rusqlite. Não há servidor, ORM, pool de conexões ou runtime assíncrono.

## Persistência

A API externa continua centralizada em `QsoRepository`; consumidores não conhecem a organização dos arquivos internos.

```text
src/database/
├── migrations.rs
└── repository/
    ├── mod.rs      conexão, agregado QSO, CRUD e transações DMR/FT8/D-STAR
    ├── queries.rs  listagem, paginação, pesquisa, filtros e materialização
    ├── adif.rs     preview, importação, exportação e campos extras
    ├── backup.rs   snapshot, integridade e permissões
    └── stress.rs   benchmark pesado de teste, ignorado por padrão
```

### Fronteiras preservadas

- QSO comum e metadados DMR/FT8/D-STAR permanecem juntos porque inserções e mudanças de modo exigem atomicidade.
- Queries permanecem SQLite explícito e retornam os mesmos tipos públicos.
- ADIF foi separado por possuir conversão, política de duplicidade e caminho de exportação próprios.
- Backup foi separado por combinar snapshot SQLite, filesystem, validação e durabilidade.
- Migrations continuam isoladas e são executadas somente durante abertura do repository.

## Garantias

- foreign keys habilitadas em toda abertura;
- migrations transacionais e schemas futuros recusados;
- `PRAGMA quick_check` e `foreign_key_check` na abertura e validação de backup;
- QSO + DMR/FT8/D-STAR e importação ADIF são transacionais;
- ordenação da listagem: `datetime_start_utc DESC, id DESC`;
- paginação atual usa `LIMIT/OFFSET`, preservada por não haver degradação relevante em 100 mil QSOs;
- SQLite permanece a fonte de verdade.

## Recorte arquitetural de D-STAR

D-STAR foi adicionado por caminhos específicos em cada camada: `DStarMetadata` no domínio, tabela `dstar_metadata` no schema 6, operações no repository, joins e filtros em queries, conversão ADIF e campos/fluxos próprios na UI. Não foi criada uma arquitetura de plugins nem traits de modo; a implementação segue a estrutura explícita já usada pelo projeto.

A tabela `digital_routes` permaneceu específica de DMR. D-STAR não reutiliza essa tabela: reflector, module, MYCALL, URCALL, RPT1, RPT2 e notes ficam em `dstar_metadata`. A única fatoração transversal foi a limpeza transacional de metadata incompatível, comportamento que já existia para mudanças entre modos e foi organizado para incluir a nova tabela.

Consulte `docs/WHAT-ADDING-DSTAR-REQUIRED.md` para o resumo curto da mudança.

## Onde alterar

- novo comportamento comum de persistência: `repository/mod.rs`;
- listagem, pesquisa ou filtro: `repository/queries.rs`;
- importação/exportação ADIF: `repository/adif.rs`;
- backup e integridade operacional: `repository/backup.rs`;
- evolução do schema: nova migration em `migrations.rs`, sem editar migrations publicadas.
