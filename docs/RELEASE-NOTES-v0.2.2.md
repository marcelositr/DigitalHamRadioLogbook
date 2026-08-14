# Digital Ham Radio Logbook v0.2.2

A versão 0.2.2 é um ciclo de hardening sem novas funcionalidades. O foco é preservar dados e tornar falhas de banco, backup, ADIF, configuração e caminhos mais previsíveis.

## Correções de integridade

- troca entre DMR, FT8 e modos genéricos remove transacionalmente metadados incompatíveis;
- backups passam a validar versão e schema da aplicação além de integridade SQLite e foreign keys;
- confirmação ADIF revalida duplicados dentro da transação, inclusive QSOs criados depois do preview;
- campos ADIF conhecidos repetidos são rejeitados em vez de perder valores silenciosamente;
- schemas atuais com índices especializados ausentes não são aceitos como completos.

## Robustez de arquivos e ambiente

- banco inexistente e arquivo zero-byte possuem testes permanentes de inicialização;
- SQLite truncado é recusado sem modificação byte a byte;
- exportações ADIF e configuração usam permissões privadas `0600` no Unix;
- TOML inválido ou truncado é preservado e retorna erro controlado;
- caminhos XDG relativos são ignorados, evitando dados dependentes do diretório de lançamento;
- caminhos absolutos com espaços e Unicode são testados;
- backup e restauração controlada preservam dados genéricos, DMR, FT8 e campos ADIF desconhecidos.

## Validações corrigidas

- frequências explicitamente negativas, incluindo `-0.5`, são rejeitadas;
- períodos FT8 com início posterior ao fim são rejeitados;
- exclusão pela API pública confirma cascata de metadados DMR, rota e FT8.

## Compatibilidade

- schema SQLite permanece na versão 5;
- nenhuma migration nova;
- bancos e configurações das versões anteriores continuam suportados;
- nenhuma mudança de UI, modo digital ou integração externa;
- nenhum dado real é usado pelos novos testes.

## Validação planejada antes da publicação

- Rustfmt, Cargo check, Clippy estrito, testes e build locked;
- migration matrix dos schemas 0–5;
- startup X11 com HOME/XDG isolados;
- regressão manual das funcionalidades existentes;
- build release, tarball, checksum, instalação, atualização e desinstalação isoladas.
