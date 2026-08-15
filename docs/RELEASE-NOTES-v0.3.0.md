# Digital Ham Radio Logbook v0.3.0

A versão 0.3.0 fortalece a estrutura interna, testes de escala e distribuição sem adicionar funcionalidades de produto ou alterar o fluxo operacional do usuário.

## Robustez estrutural

- repository SQLite organizado por agregado QSO, consultas, ADIF e backup;
- API pública e transações QSO/DMR/FT8 preservadas;
- migrations publicadas permanecem inalteradas e schema SQLite continua na versão 5;
- documentação curta indica onde evoluir persistência, consultas, ADIF e backup.

## Escala e performance

- gerador determinístico e benchmark manual para 1 mil, 10 mil, 100 mil e 1 milhão de QSOs;
- medições cobrem abertura, paginação, busca, filtros DMR/FT8, backup e ADIF;
- exportação ADIF deixou de executar consultas por QSO e ficou aproximadamente 8,7 vezes mais rápida no teste de 100 mil registros;
- 100 mil QSOs permaneceram confortáveis nas operações normais do ambiente medido;
- limites do stress de 1 milhão estão documentados sem promessa de SLA.

## SQLite e migrations

- matriz permanente dos schemas 0–5 preserva dados representativos e verifica idempotência;
- `quick_check` e `foreign_key_check` continuam obrigatórios;
- planos SQLite confirmam uso dos índices de ordenação, talkgroup e SNR;
- nenhum índice ou migration foi adicionado sem benefício medido.

## Distribuição Linux

- tarball normalizado para geração determinística com GNU tar/gzip;
- permissões, ordem, ownership e timestamps do arquivo são normalizados;
- tarball e checksum são concluídos em temporários antes da publicação;
- smoke test testa conteúdo, checksum, instalação, reinstalação, desinstalação e preservação XDG;
- CI passa a testar o contrato do pacote Linux;
- upgrade real de `v0.2.2` para `v0.3.0` preservou QSO genérico, DMR, FT8 e configuração.

## Compatibilidade

- nenhuma funcionalidade nova;
- nenhuma mudança de UI;
- schema SQLite permanece na versão 5;
- nenhuma dependência de runtime nova;
- bancos e configurações `v0.2.x` continuam compatíveis.

## Artefatos Linux planejados

- `digital-ham-radio-logbook-0.3.0-linux-x86_64.tar.gz`
- `digital-ham-radio-logbook-0.3.0-linux-x86_64.tar.gz.sha256`

Consulte `CHANGELOG.md`, `docs/PERFORMANCE-v0.3.0.md`, `docs/LINUX-DISTRIBUTION.md` e `docs/DATA-RECOVERY.md`.
