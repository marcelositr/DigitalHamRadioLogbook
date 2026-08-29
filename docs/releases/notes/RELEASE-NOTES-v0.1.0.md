# Digital Ham Radio Logbook v0.1.0

Primeira versão pública do aplicativo desktop local/offline para registro de QSOs digitais.

## Destaques

- CRUD completo de QSOs comuns;
- metadados especializados para DMR e FT8;
- filtros gerais, DMR e FT8;
- importação e exportação ADIF transacionais;
- preservação de campos ADIF desconhecidos;
- seleção gráfica de arquivos via XDG Desktop Portal;
- backup SQLite consistente e validado;
- verificação de integridade e compatibilidade do schema na abertura;
- configuração local da estação;
- interface Nord responsiva e homologada no i3 em `1050×680`;
- instalação Linux user-local sem `sudo`;
- desinstalação que preserva banco e configuração.

## Artefatos Linux

- `digital-ham-radio-logbook-0.1.0-linux-x86_64.tar.gz`
- `digital-ham-radio-logbook-0.1.0-linux-x86_64.tar.gz.sha256`

Consulte `docs/LINUX-DISTRIBUTION.md` para instalação e `docs/DATA-RECOVERY.md` para integridade, backup e restauração.

## Validação

- Rustfmt;
- Clippy com warnings tratados como erro;
- 53 testes;
- build release locked;
- startup X11;
- instalação, execução e desinstalação em HOME/XDG isolados;
- preservação de dados verificada por SHA-256.
