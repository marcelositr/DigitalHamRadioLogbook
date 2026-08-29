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
- backup e restauração controlada preservam dados genéricos, DMR, FT8 e campos ADIF desconhecidos;
- backup recusa diretório inexistente, preserva destinos existentes e usa permissões privadas `0600` no Unix;
- falhas operacionais de ADIF, backup e configuração registram a causa sem expor conteúdo dos QSOs.

## Validações corrigidas

- frequências explicitamente negativas, incluindo `-0.5`, são rejeitadas;
- períodos FT8 com início posterior ao fim são rejeitados;
- exclusão pela API pública confirma cascata de metadados DMR, rota e FT8;
- gravação ou exclusão já confirmada não é apresentada como falha quando somente a atualização visual falha;
- busca e filtros restauram o estado anterior quando a consulta não pode ser atualizada.

## Compatibilidade

- schema SQLite permanece na versão 5;
- nenhuma migration nova;
- bancos e configurações das versões anteriores continuam suportados;
- nenhuma mudança de UI, modo digital ou integração externa;
- nenhum dado real é usado pelos novos testes.

## Artefatos Linux

- `digital-ham-radio-logbook-0.2.2-linux-x86_64.tar.gz`
- `digital-ham-radio-logbook-0.2.2-linux-x86_64.tar.gz.sha256`

Valide o checksum antes de extrair. Consulte `docs/LINUX-DISTRIBUTION.md` para instalação/atualização e `docs/DATA-RECOVERY.md` para backup e restauração.

## Validação da publicação

- Rustfmt, Cargo check, Clippy estrito, testes e build locked;
- 99 testes aprovados;
- migration matrix dos schemas 0–5;
- startup com HOME/XDG isolados;
- regressão manual das funcionalidades existentes aprovada;
- build release, tarball, checksum, instalação, atualização e desinstalação isoladas.

## Comparação completa

https://github.com/marcelositr/DigitalHamRadioLogbook/compare/v0.2.1...v0.2.2
