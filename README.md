# Digital Ham Radio Logbook

Aplicativo desktop local e offline para registrar contatos de radioamador realizados em modos digitais.

## Estado atual

O MVP funcional inclui:

- interface Slint organizada em páginas internas e adequada à janela de `1050×680`;
- banco SQLite local com migrations versionadas;
- listagem, pesquisa, criação, edição e exclusão confirmada de QSOs;
- campos comuns e metadados específicos de DMR e FT8;
- filtros gerais, DMR e FT8;
- importação e exportação ADIF transacionais;
- preservação de campos ADIF desconhecidos;
- configuração local da estação;
- links externos configuráveis para consulta de callsign e GridSquare;
- backup consistente do banco;
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

Para gerar um pacote release user-local:

```sh
packaging/linux/make-release.sh
```

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

## Testes e qualidade

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Localização dos dados

No GNU/Linux, o projeto segue a XDG Base Directory Specification.

- banco com `XDG_DATA_HOME`: `$XDG_DATA_HOME/digital-ham-log/logbook.sqlite3`;
- banco sem `XDG_DATA_HOME`: `~/.local/share/digital-ham-log/logbook.sqlite3`;
- configuração com `XDG_CONFIG_HOME`: `$XDG_CONFIG_HOME/digital-ham-log/config.toml`;
- configuração sem `XDG_CONFIG_HOME`: `~/.config/digital-ham-log/config.toml`.

## Seleção de arquivos, ADIF e backup

Em **Tools**, os botões gráficos permitem selecionar um ADIF existente para importação e escolher destinos para exportação e backup. Os campos de caminho continuam editáveis para uso avançado. Cancelar o diálogo não altera dados nem gera erro. O aplicativo lembra separadamente as últimas pastas usadas para importação, exportação e backup; se uma pasta deixar de existir, o seletor usa seu fallback normal.

Antes de gravar, a importação apresenta um preview com o total de registros, novos QSOs, duplicados, inválidos e distribuição dos registros válidos por modo. O arquivo é analisado uma única vez e a confirmação grava exatamente o plano exibido; cancelar ou pressionar `Esc` descarta o plano sem alterar o banco. Registros inválidos são informados e não são importados.

A importação ignora duplicados exatos, inclusive registros repetidos no mesmo arquivo. A identidade é composta por callsign normalizado, data/hora UTC inicial, frequência e modo normalizado. O QSO já existente é preservado sem mesclar ou sobrescrever metadados; a interface informa quantos registros foram importados e quantos duplicados foram ignorados. A criação e edição manual continuam permitindo QSOs com a mesma identidade quando isso for intencional.

A interface permite criar um snapshot consistente para um caminho terminado em `.sqlite3`. O destino não pode existir, evitando sobrescrita acidental. O snapshot é validado antes de ser anunciado como concluído.

Também é possível fechar o aplicativo e copiar `logbook.sqlite3` manualmente. O banco contém os QSOs e metadados armazenados pelo aplicativo. Para restauração, corrupção, permissões e schema incompatível, siga `docs/DATA-RECOVERY.md`.

## Arquitetura resumida

- `src/domain/`: entidades e validações independentes da interface e do banco;
- `src/database/`: migrations e acesso ao SQLite;
- `src/app/`: handlers e serviços de apresentação separados por fluxo — editor, lista, filtros, ADIF, backup, configuração, arquivos e fechamento;
- `src/main.rs`: composition root enxuto, responsável apenas por criar dependências, inicializar a janela e conectar os módulos;
- `ui/main.slint`: contrato público e shell global da janela;
- `ui/pages/`: páginas independentes de Logbook, editor, Tools e Settings;
- `ui/models/`: tipos Slint compartilhados entre páginas;
- `ui/design-system.slint`: tema Nord e componentes visuais reutilizáveis.

A interface não executa SQL e a camada de banco não depende de Slint.

## Tema visual

A interface utiliza o [Nord Theme](https://www.nordtheme.com/docs/colors-and-palettes) em modo escuro:

- **Polar Night** para fundos, painéis, cartões e controles;
- **Snow Storm** para texto principal e secundário;
- **Frost** para navegação, seleção, hover e ações em destaque;
- **Aurora** para estados destrutivos, avisos e confirmações.

As superfícies principais têm cores explícitas para preservar contraste em diferentes temas de desktop e gerenciadores de janela. Os tokens de cor, espaçamento e raio ficam centralizados em `ui/design-system.slint`, junto aos componentes tipográficos reutilizáveis.

O acabamento visual inclui cabeçalho com identificação da estação, navegação consistente, tabela com linhas alternadas e destaque semântico, estado vazio informativo, formulários agrupados por seção e cartões padronizados em Tools e Settings. Componentes estáveis como badge da estação, badge de modo, estado vazio e barra de status também ficam centralizados no design system.

Após mudanças de interface, use o checklist persistente em `docs/VISUAL-QA.md` para validar a janela padrão de `1050×680`, conteúdo longo, estados vazios, filtros, mensagens e navegação por teclado.

## Navegação da interface

O menu superior separa as tarefas para manter todos os controles acessíveis mesmo em gerenciadores de janela tiled, como i3:

- **Logbook**: pesquisa, filtros, tabela paginada e ações de edição/exclusão;
- **New QSO**: formulário rolável para criar um contato;
- **Tools**: importação/exportação ADIF e backup;
- **Settings**: configuração do callsign da estação local.

O Logbook consulta até 100 QSOs por página diretamente no SQLite e mostra a faixa atual, o total e os controles **Previous/Next**. Busca e filtros DMR/FT8 preservam seus critérios ao navegar entre páginas. Metadados DMR e FT8 são carregados junto dos QSOs, evitando consultas adicionais por linha.

Ao editar um registro pela tabela, o mesmo formulário é aberto preenchido. As ações de salvar e cancelar permanecem fixas no rodapé enquanto os campos podem ser rolados.

Ao criar ou editar um QSO, sair do formulário por uma aba, pelo botão **Cancel** ou por `Esc` exige confirmação quando houver alterações não salvas. **Continue editing** preserva o formulário atual e **Discard changes** limpa o editor e conclui a navegação solicitada. Formulários sem mudanças não exibem aviso, e rascunhos não são persistidos em disco.

No encerramento normal, o aplicativo lembra a última aba, o tipo de filtro selecionado e se o painel de filtros estava expandido. Pesquisa, valores preenchidos nos filtros, indicador de filtro aplicado e conteúdo parcial de QSO não são persistidos.

Fechar a janela pelo gerenciador de janelas é interceptado quando há edição de QSO ou preview ADIF pendente. **Continue working** mantém a janela e o estado atual; **Discard and exit** descarta somente o trabalho não confirmado, salva as preferências operacionais e encerra. Se a configuração não puder ser salva, a aplicação permanece aberta e oferece nova tentativa ou saída explícita sem salvar preferências. Término forçado do processo (`SIGKILL`) e queda de energia não podem ser interceptados.

## Consultas externas

Na tabela do Logbook, clicar em um callsign ou GridSquare abre o navegador padrão usando os templates configurados em **Settings**.

Padrões:

- callsign: `https://www.qrz.com/db/{callsign}`;
- grid: `https://www.levinecentral.com/ham/grid_square.php?Grid={grid}`.

Somente templates `http://` ou `https://` são aceitos, e os placeholders correspondentes são obrigatórios. Nenhuma consulta é feita automaticamente: o callsign ou grid é enviado ao site externo apenas após clique explícito do usuário.

## Atalhos de teclado

- `Enter` no campo de busca executa a busca.
- `Enter` no campo de observações salva o QSO atual.
- `Escape` cancela o formulário e retorna ao Logbook, além de fechar uma confirmação de exclusão pendente.

## Logging

O aplicativo escreve logs operacionais simples em `stderr`, incluindo startup, encerramento, configuração, importação/exportação ADIF e backup. Não há telemetria, analytics ou envio automático pela rede. Conteúdo de QSOs e callsigns não é escrito nesses logs operacionais.

Erros operacionais comuns exibem orientação prática junto do detalhe técnico: escolher outro nome quando o destino existe, selecionar arquivo/pasta existente quando o caminho desapareceu e usar um local gravável quando faltar permissão. Validações de formulário que já são claras permanecem diretas, sem texto extra.

## Integridade e recuperação

A abertura do banco recusa schemas futuros, valida os objetos esperados e executa verificações SQLite de integridade e foreign keys. Backups passam pelas mesmas verificações. A configuração e a exportação ADIF são publicadas por temporário + rename, reduzindo o risco de arquivos parciais.

O procedimento seguro de restauração está em `docs/DATA-RECOVERY.md`. Nunca substitua o banco enquanto a aplicação estiver aberta.

## Licença e autoria

Desenvolvido por [Marcelo Trindade](https://github.com/marcelositr) e distribuído sob a licença MIT. Consulte `LICENSE`.

## Limitações conhecidas

- a configuração atual da estação local contém apenas o callsign;
- o backup exige que o diretório de destino já exista;
- o backup recusa sobrescrever arquivo existente;

- não há integração automática com WSJT-X, rádios ou serviços online;
- a exportação ADIF recusa sobrescrever um arquivo existente;
- os seletores gráficos no Linux dependem de um XDG Desktop Portal funcional; os campos de caminho permanecem disponíveis como alternativa.
