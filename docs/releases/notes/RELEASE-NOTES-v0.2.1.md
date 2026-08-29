# Digital Ham Radio Logbook v0.2.1

A versão 0.2.1 apresenta uma nova identidade visual para toda a aplicação, mantendo intactos o domínio Rust, o banco SQLite, o schema, os fluxos ADIF e todas as regras funcionais da versão 0.2.0.

## Destaques

- design system técnico próprio, inspirado em estações de rádio e instrumentação de telecomunicações;
- shell global mais compacto, com navegação clara, estação local visível e ação **New QSO** destacada;
- Logbook convertido de tabela tradicional para lista operacional de duas linhas;
- callsign, horário, modo e frequência com maior prioridade visual;
- rota digital, GridSquare e ações organizados como contexto secundário;
- pesquisa, filtros ativos, paginação, estados vazios e confirmação de exclusão redesenhados;
- Editor reorganizado em seções técnicas compactas para campos comuns, DMR e FT8;
- Tools com fluxos de ADIF e backup mais claros e preview de importação mais escaneável;
- Settings com estação local em destaque e links externos tratados como configuração secundária;
- ações customizadas com dimensões consistentes, clique único, foco visível, `Tab`, `Enter` e `Space`;
- layout revisado e homologado no i3 em `1050×680`.

## Compatibilidade e comportamento

- nenhuma migration nova;
- schema SQLite permanece na versão 5;
- nenhuma mudança nas regras de domínio DMR ou FT8;
- nenhuma mudança em importação/exportação ADIF, backup, filtros ou persistência;
- bancos e configurações da versão 0.2.0 permanecem compatíveis;
- atualização e desinstalação continuam preservando `logbook.sqlite3` e `config.toml`;
- recomenda-se criar um backup pela aba **Tools** antes de atualizar qualquer instalação existente.

## Acessibilidade e ergonomia

- navegação e ações continuam acessíveis por teclado;
- foco possui contraste explícito em superfícies escuras;
- ações normais, compactas e links usam dimensões centralizadas no design system;
- conteúdo permanece rolável onde necessário;
- nenhuma página exige fullscreen para operar no tamanho padrão de `1050×680`.

## Artefatos Linux

- `digital-ham-radio-logbook-0.2.1-linux-x86_64.tar.gz`
- `digital-ham-radio-logbook-0.2.1-linux-x86_64.tar.gz.sha256`

Valide o checksum antes de extrair. Consulte `docs/LINUX-DISTRIBUTION.md` para instalação/atualização e `docs/DATA-RECOVERY.md` para backup e restauração.

## Validação da publicação

- Rustfmt;
- Cargo check locked;
- Clippy com warnings tratados como erro;
- 73 testes;
- build debug e release com `Cargo.lock`;
- matriz de migrations v0–v5 no GitHub Actions;
- startup X11 com HOME/XDG isolados;
- homologação visual das quatro páginas no i3 em `1050×680`;
- checksum e inspeção do pacote Linux;
- instalação, atualização, execução e desinstalação em ambiente isolado;
- preservação do banco e da configuração.

## Comparação completa

https://github.com/marcelositr/DigitalHamRadioLogbook/compare/v0.2.0...v0.2.1
