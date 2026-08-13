# Digital Ham Radio Logbook v0.2.0

A versão 0.2.0 fortalece a operação diária do logbook com importação ADIF mais segura, melhor desempenho em bases grandes, proteção contra perda de trabalho, diagnóstico acionável, CI automatizada e navegação acessível por teclado.

## Destaques

- links externos configuráveis para consulta de callsign e GridSquare;
- detecção de duplicidade ADIF por callsign, início UTC, frequência e modo normalizados;
- preview ADIF sem escrita, com confirmação explícita e plano imutável em memória;
- relatório de importação com modos, bandas, período UTC, regra de duplicidade e detalhes de registros inválidos;
- proteção contra descarte acidental de edições e fechamento seguro com trabalho pendente;
- persistência das últimas pastas e do estado operacional de navegação, sem armazenar rascunhos ou filtros sensíveis;
- mensagens de erro operacionais com orientação para caminhos ausentes, permissões e destinos existentes;
- paginação SQLite de 100 QSOs, consultas com joins e índices para bases grandes;
- interface e handlers reorganizados em módulos menores, sem mudança do modelo de dados;
- navegação superior e links da tabela acessíveis por `Tab`, `Enter` e `Space`;
- semântica para tecnologias assistivas e status anunciado como live region;
- CI no GitHub com fmt, Clippy estrito, 73 testes, build e matriz de migrations dos schemas v0–v5.

## Integridade e compatibilidade

- schema SQLite atual: versão 5;
- bancos das versões anteriores são migrados automaticamente e de forma transacional;
- a matriz automatizada verifica preservação de dados, idempotência, `quick_check` e chaves estrangeiras;
- banco e configuração continuam locais e offline;
- atualização e desinstalação preservam `logbook.sqlite3` e `config.toml`;
- recomenda-se criar um backup pela aba **Tools** antes de atualizar qualquer instalação existente.

## Artefatos Linux

- `digital-ham-radio-logbook-0.2.0-linux-x86_64.tar.gz`
- `digital-ham-radio-logbook-0.2.0-linux-x86_64.tar.gz.sha256`

Valide o checksum antes de extrair. Consulte `docs/LINUX-DISTRIBUTION.md` para instalação/atualização e `docs/DATA-RECOVERY.md` para backup e restauração.

## Validação planejada para publicação

- Rustfmt;
- Clippy com warnings tratados como erro;
- 73 testes;
- build release com `Cargo.lock`;
- matriz de migrations v0–v5 no GitHub Actions;
- startup X11 com HOME/XDG isolados;
- checksum do pacote;
- inspeção do conteúdo mínimo do tarball;
- instalação e atualização user-local em ambiente isolado;
- execução do binário instalado;
- desinstalação idempotente;
- preservação do banco e da configuração por SHA-256.

## Comparação completa

https://github.com/marcelositr/DigitalHamRadioLogbook/compare/v0.1.0...v0.2.0
