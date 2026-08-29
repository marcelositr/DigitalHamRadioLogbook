# Digital Ham Radio Logbook

[![CI](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/workflows/ci.yml)

Aplicativo desktop local e offline para registrar contatos de radioamador realizados em modos digitais.

## Estado atual

O checkpoint funcional de referência continua sendo `0.10.0-rc.1`, uma consolidação pré-1.0. A branch de interface v0.11 abre um ciclo separado de reconstrução gráfica: nenhuma funcionalidade de domínio, schema, migration, ADIF ou persistência SQLite é adicionada por esse trabalho.

A v0.11 reconstrói a interface a partir dos contratos funcionais usando uma arquitetura **Slint-native**, inspirada na organização da Slint Widgets Gallery. A UI usa widgets e layouts padrão, `MenuBar` real, sidebar simples, workspace e status global; `Palette` e `StyleMetrics` substituem o antigo tema visual proprietário. **Fluent** é o style oficial do produto, com **System / Light / Dark** configuráveis em runtime. A especificação está em `docs/UI-ARCHITECTURE-v0.11.md` e o novo gate manual em `docs/VISUAL-QA-v0.11.md`.

O MVP funcional inclui:

- interface Slint desktop-first baseada em widgets nativos, com alta legibilidade e operação confortável em `1050×680`;
- style Fluent fixo, com esquema de cores System, Light ou Dark persistido na configuração local;
- banco SQLite local com migrations versionadas;
- listagem, pesquisa, criação, edição e exclusão confirmada de QSOs;
- fluxo **Save & New** para registrar QSOs consecutivos sem duplicar o contato recém-gravado;
- campos comuns e metadados específicos de DMR, FT8, D-STAR e YSF/System Fusion (`C4FM`);
- filtros gerais, DMR, FT8, D-STAR e YSF;
- importação ADIF transacional, exportação completa e exportação de todos os resultados do filtro atual;
- preservação de campos ADIF desconhecidos;
- configuração local da estação;
- links externos configuráveis para consulta de callsign e GridSquare;
- backup consistente publicado somente após validação e verificação read-only de backups existentes;
- health check local para integridade, schema, migrations e metadata por modo;
- persistência entre execuções e operação sem serviços online.

## Requisitos

- toolchain Rust estável (`rustc` e `cargo`);
- dependências nativas exigidas pelo backend gráfico selecionado pelo Slint.

O SQLite é compilado através do recurso `bundled` do `rusqlite`.

## Compilação

```sh
cargo build
```

## Distribuição Linux

Para gerar o pacote release user-local (`tar.gz`):

```sh
packaging/linux/make-release.sh
```

A pre-release também fornece `.deb` e AppImage produzidos do mesmo binário validado. O `.deb` integra o aplicativo ao sistema Debian/Ubuntu; o AppImage pode ser executado diretamente. Consulte `docs/LINUX-DISTRIBUTION.md` para comandos, dependências e limites de portabilidade.

Depois de validar o arquivo `.sha256`, extraia o `tar.gz` e execute:

```sh
./install.sh --dry-run
./install.sh
```

A instalação não exige `sudo`; atualização e desinstalação preservam banco e configuração. Consulte `docs/LINUX-DISTRIBUTION.md` para instalação, atualização, remoção e compatibilidade.

## Execução

```sh
cargo run
```

O aplicativo é compilado com o style **Fluent**. Em **Settings → Appearance**, o esquema de cores pode ser alterado sem reiniciar:

- **System**: padrão; acompanha a preferência claro/escuro reportada pelo desktop;
- **Light**: força o modo claro;
- **Dark**: força o modo escuro.

A escolha é salva no `config.toml`. Configurações antigas sem essa preferência continuam válidas e usam **System**.

## Testes e qualidade

```sh
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked
```

O workflow `.github/workflows/ci.yml` executa esses controles em `main`, `develop`, pull requests e manualmente. Jobs adicionais testam em paralelo a migração dos schemas 0–7 e o contrato do pacote Linux, incluindo instalação e desinstalação em XDG isolado.

O caminho factual até uma possível versão `1.0.0` está em `docs/PRE-1.0-READINESS.md`; ele não define prazo nem declara prontidão. Ambientes efetivamente testados e limitações estão em `docs/SUPPORT-MATRIX.md`, e o processo reproduzível de release está em `docs/RELEASE-CHECKLIST.md`.

Testes pesados são ignorados por padrão. Consulte `docs/PERFORMANCE-v0.3.0.md` para gerar datasets determinísticos de 1 mil a 1 milhão de QSOs e reproduzir as medições. A organização interna da persistência está resumida em `docs/ARCHITECTURE.md`.

## Localização dos dados

No GNU/Linux, o projeto segue a XDG Base Directory Specification. Valores XDG relativos são ignorados; os fallbacks exigem uma `HOME` absoluta para impedir gravação dependente do diretório de execução.

- banco com `XDG_DATA_HOME`: `$XDG_DATA_HOME/digital-ham-log/logbook.sqlite3`;
- banco sem `XDG_DATA_HOME`: `~/.local/share/digital-ham-log/logbook.sqlite3`;
- configuração com `XDG_CONFIG_HOME`: `$XDG_CONFIG_HOME/digital-ham-log/config.toml`;
- configuração sem `XDG_CONFIG_HOME`: `~/.config/digital-ham-log/config.toml`.

## Seleção de arquivos, ADIF e backup

Em **Tools**, os botões gráficos permitem selecionar um ADIF existente para importação e escolher destinos para exportação e backup. Os campos de caminho continuam editáveis para uso avançado. Cancelar o diálogo não altera dados nem gera erro. O aplicativo lembra separadamente as últimas pastas usadas para importação, exportação e backup; se uma pasta deixar de existir, o seletor usa seu fallback normal.

Antes de gravar, a importação apresenta um preview com o total de registros, novos QSOs, duplicados, inválidos e distribuição dos registros válidos por modo. O arquivo é analisado uma única vez e a confirmação grava exatamente o plano exibido; cancelar ou pressionar `Esc` descarta o plano sem alterar o banco. Registros inválidos são informados e não são importados.

A importação ignora duplicados exatos, inclusive registros repetidos no mesmo arquivo. A identidade é composta por callsign normalizado, data/hora UTC inicial, frequência e modo normalizado. O QSO já existente é preservado sem mesclar ou sobrescrever metadados; a interface informa quantos registros foram importados e quantos duplicados foram ignorados. A criação e edição manual continuam permitindo QSOs com a mesma identidade quando isso for intencional.

A interface permite criar um snapshot consistente para um caminho terminado em `.sqlite3`. O destino não pode existir, evitando sobrescrita acidental. O snapshot é criado em um temporário no mesmo diretório, validado em modo read-only, sincronizado e somente então publicado. **Verify backup** verifica um SQLite existente sem restaurar, migrar ou modificar o arquivo.

**Export all QSOs** mantém a exportação completa. **Export current results** exporta todos os registros correspondentes à pesquisa ou filtro atual, atravessando todas as páginas; resultado vazio é informado sem criar arquivo. **Check data health** executa verificações read-only de SQLite, foreign keys, schema, migrations e consistência de metadata por modo.

Também é possível fechar o aplicativo e copiar `logbook.sqlite3` manualmente. O banco contém os QSOs e metadados armazenados pelo aplicativo. Para restauração, corrupção, permissões e schema incompatível, siga `docs/DATA-RECOVERY.md`.

## Arquitetura resumida

- `src/domain/`: entidades e validações independentes da interface e do banco;
- `src/database/`: migrations e acesso ao SQLite;
- `src/app/`: handlers e serviços de apresentação separados por fluxo — editor, lista, filtros, ADIF, backup, configuração, arquivos e fechamento;
- `src/main.rs`: composition root enxuto, responsável apenas por criar dependências, inicializar a janela e conectar os módulos;
- `ui/main.slint`: contrato público, `MenuBar` nativo e composição do shell global;
- `ui/appearance.slint`: ligação runtime entre a preferência System/Light/Dark e `Palette.color-scheme`;
- `ui/components/app-shell.slint`: sidebar recolhível baseada em layouts e estados do Slint;
- `ui/pages/`: páginas independentes de Logbook, editor, Tools e Settings;
- `ui/models/`: tipos Slint compartilhados entre páginas;
- `ui/design-system.slint`: primitivas semânticas mínimas que usam `Palette` e `StyleMetrics`, sem definir um tema paralelo.

A interface não executa SQL e a camada de banco não depende de Slint.

## Interface Slint-native

A v0.11 não tenta reproduzir manualmente um design system externo. A camada visual usa o **Fluent** do próprio Slint, fixado em `build.rs`, e permite apenas variar o esquema de cores em runtime.

Os princípios são:

- widgets de `std-widgets.slint` antes de componentes customizados;
- `GroupBox`, `Button`, `LineEdit`, `ListView`, `ScrollView` e layouts nativos como blocos principais;
- `Palette` e `StyleMetrics` para extensões necessárias;
- sizing natural, `preferred-*`, `min-*` e stretch antes de larguras/alturas absolutas;
- medidas fixas somente quando justificadas por estrutura previsível, como a sidebar e colunas tabulares;
- nenhuma borda decorativa deve atravessar inputs, nenhum botão essencial pode ser truncado e nenhum conteúdo necessário pode ficar inacessível.

A UI anterior foi homologada no i3 em `1050×680`; essa aprovação não é herdada. A reconstrução v0.11 exige nova validação usando `docs/VISUAL-QA-v0.11.md` nos esquemas **System**, **Light** e **Dark**.

## Navegação da interface

A moldura v0.11 usa duas regiões persistentes:

- `MenuBar` nativo do Slint para comandos globais e secundários;
- sidebar recolhível simples para as quatro áreas principais.

As áreas funcionais são:

- **Logbook**: pesquisa, filtros, lista paginada e ações de edição/exclusão;
- **New QSO**: formulário rolável para criar um contato;
- **Tools**: health check, importação/exportação ADIF, criação e verificação de backup;
- **Settings**: aparência, configuração do callsign da estação local e links externos.

O callsign local permanece visível na sidebar quando expandida. Não existem categorias decorativas `Operation`, `Data` ou `System`, nem barra contextual duplicando o título da página.

O Logbook consulta até 100 QSOs por página diretamente no SQLite e mostra a faixa atual, o total e os controles **Previous/Next**. Busca e filtros DMR/FT8/D-STAR/YSF preservam seus critérios ao navegar entre páginas. Metadados DMR, FT8, D-STAR e YSF são carregados junto dos QSOs, evitando consultas adicionais por linha.

Para D-STAR, o editor modela `reflector`, `module`, `mycall`, `urcall`, `rpt1`, `rpt2` e `notes`; a listagem pode ser filtrada por reflector, module e RPT1. A interoperabilidade ADIF usa a forma canônica `MODE=DIGITALVOICE` + `SUBMODE=DSTAR` na exportação e também aceita o histórico `MODE=DSTAR` na importação.

Para YSF/System Fusion, o modo interno é `C4FM`; a UI também aceita os aliases `YSF` e `SYSTEM FUSION`. O editor modela `room`, `wires_x_node`, `repeater`, `network`, `access_type`, TX/RX DG-ID e `notes`; os filtros cobrem room, WIRES-X node e DG-ID. ADIF é exportado como `MODE=DIGITALVOICE` + `SUBMODE=C4FM` e também importa o histórico `MODE=C4FM`.

Esses são os subconjuntos suportados pelo aplicativo, não uma promessa de suporte integral aos protocolos, equipamentos ou dialetos ADIF. `digital_routes` continua específico de DMR.

Ao editar um registro pela lista, o mesmo formulário é aberto preenchido. As ações de salvar e cancelar permanecem disponíveis no rodapé enquanto os campos podem ser rolados. Ao abrir um novo QSO, o foco vai para callsign.

No fluxo de criação, **Save & New** valida e grava o QSO, atualiza a listagem, limpa todos os campos comuns, metadados de modo e metadata do editor e prepara um formulário novo com outro UTC fixo. A ação não cria um segundo QSO e não aparece na edição. Uma proteção contra double-submit impede duas gravações concorrentes, e o snapshot de estado limpo é atualizado somente depois do commit bem-sucedido.

Antes de criar ou editar manualmente, o aplicativo procura uma possível duplicidade pela identidade exata callsign normalizado + data/hora UTC inicial + frequência em Hz + modo normalizado. Na edição, o próprio registro é excluído da consulta. O aviso oferece **Review** ou **Save anyway**: nunca mescla registros, nunca bloqueia a gravação e não depende de constraint `UNIQUE`.

A interface pode ser percorrida por `Tab`. Navegação e links de callsign/GridSquare aceitam `Enter` ou `Space`, campos e botões seguem a ordem visual, `Enter` executa a pesquisa ou salva a partir de Notes, e `Escape` cancela o fluxo atual. Regiões principais, campos essenciais, ações personalizadas e mensagens de status também expõem semântica para tecnologias assistivas.

Ao criar ou editar um QSO, sair do formulário pela navegação, pelo botão **Cancel** ou por `Esc` exige confirmação quando houver alterações não salvas. **Continue editing** preserva o formulário atual e **Discard changes** limpa o editor e conclui a navegação solicitada. Formulários sem mudanças não exibem aviso, e rascunhos não são persistidos em disco.

No encerramento normal, o aplicativo lembra a última página, o tipo de filtro selecionado e se o painel de filtros estava expandido. A aparência é salva imediatamente quando alterada em Settings. Pesquisa, valores preenchidos nos filtros, indicador de filtro aplicado e conteúdo parcial de QSO não são persistidos.

Fechar a janela pelo gerenciador de janelas é interceptado quando há edição de QSO ou preview ADIF pendente. **Continue working** mantém a janela e o estado atual; **Discard and exit** descarta somente o trabalho não confirmado, salva as preferências operacionais e encerra. Se a configuração não puder ser salva, a aplicação permanece aberta e oferece nova tentativa ou saída explícita sem salvar preferências. Término forçado do processo (`SIGKILL`) e queda de energia não podem ser interceptados.

## Consultas externas

Na lista do Logbook, clicar em um callsign ou GridSquare abre o navegador padrão usando os templates configurados em **Settings**.

Padrões:

- callsign: `https://www.qrz.com/db/{callsign}`;
- grid: `https://www.levinecentral.com/ham/grid_square.php?Grid={grid}`.

Somente templates `http://` ou `https://` são aceitos, e os placeholders correspondentes são obrigatórios. Nenhuma consulta é feita automaticamente: o callsign ou grid é enviado ao site externo apenas após clique explícito do usuário.

## Atalhos de teclado

- `Ctrl+N` abre um novo QSO e posiciona o foco em callsign.
- `Ctrl+S` salva o QSO atual.
- `Ctrl+Enter` executa **Save & New** somente durante a criação.
- `Ctrl+F` retorna ao Logbook e posiciona o foco na pesquisa.
- `Enter` no campo de busca executa a busca.
- `Enter` no campo de observações salva o QSO atual.
- `Escape` é exclusivo para cancelar ou fechar o fluxo atual, incluindo confirmações pendentes.

Esses atalhos foram cobertos por testes, inclusive a preservação do clipboard.

## Logging

O aplicativo escreve logs operacionais simples em `stderr`, incluindo startup, encerramento, configuração, importação/exportação ADIF e backup. Não há telemetria, analytics ou envio automático pela rede. Conteúdo de QSOs e callsigns não é escrito nesses logs operacionais.

Erros operacionais comuns exibem orientação prática junto do detalhe técnico: escolher outro nome quando o destino existe, selecionar arquivo/pasta existente quando o caminho desapareceu e usar um local gravável quando faltar permissão. Validações de formulário que já são claras permanecem diretas, sem texto extra.

## Integridade e recuperação

A abertura do banco recusa schemas futuros, valida os objetos esperados e executa verificações SQLite de integridade e foreign keys. O health check e a verificação de backup abrem arquivos em modo read-only, não executam migrations e não alteram dados. Backups são publicados somente depois da validação, e configuração/exportação ADIF usam publicação atômica; arquivos privados usam `0600` no Unix.

O procedimento seguro de restauração está em `docs/DATA-RECOVERY.md`. Nunca substitua o banco enquanto a aplicação estiver aberta. Downgrade automático não é suportado: um banco aberto por uma versão com schema mais novo deve ser usado com uma aplicação compatível ou substituído por um backup anterior compatível enquanto o aplicativo estiver fechado.

O contrato de campos, normalizações, corpus e limitações ADIF está em `docs/ADIF-INTEROPERABILITY.md`. As extensões privadas compatíveis estão em `docs/ADIF-EXTENSIONS.md`. Para implementar outro modo, consulte `docs/ADDING-A-DIGITAL-MODE.md`; a decisão de manter a arquitetura explícita de quatro modos está em `docs/FOUR-MODE-ARCHITECTURE-REVIEW.md`.

## Licença e autoria

Desenvolvido por [Marcelo Trindade](https://github.com/marcelositr) e distribuído sob a licença MIT. Consulte `LICENSE`.

## Limitações conhecidas

- a configuração atual da estação local contém apenas o callsign; o `MYCALL` D-STAR pode ser informado por QSO;
- o backup exige que o diretório de destino já exista;
- o backup recusa sobrescrever arquivo existente;
- não há integração automática com WSJT-X, rádios ou serviços online;
- a exportação ADIF recusa sobrescrever um arquivo existente;
- os seletores gráficos no Linux dependem de um XDG Desktop Portal funcional; os campos de caminho permanecem disponíveis como alternativa.