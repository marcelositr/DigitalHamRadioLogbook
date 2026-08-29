# UI Architecture v0.11

## Objetivo

A linha v0.11 reconstrói exclusivamente a camada gráfica do Digital Ham Radio Logbook. O backend Rust, domínio, SQLite, migrations, ADIF, configuração, backup, filtros, atalhos e contratos públicos do `MainWindow` permanecem preservados.

A referência de engenharia visual passa a ser a **Slint Widgets Gallery**: layouts naturais, widgets de `std-widgets.slint`, dimensionamento pelo conteúdo e adaptação ao style selecionado. A UI anterior não é referência visual para esta reconstrução.

## Princípio central

A interface deve trabalhar com o toolkit, não contra ele.

Isso significa:

1. usar widgets nativos antes de criar componentes customizados;
2. preferir `HorizontalBox`, `VerticalBox`, `HorizontalLayout`, `VerticalLayout`, `GroupBox`, `Button`, `LineEdit`, `ListView`, `ScrollView` e `MenuBar`;
3. preferir `preferred-*`, `min-*`, stretch e métricas do style antes de tamanhos absolutos;
4. considerar clipping, sobreposição, borda atravessando input ou botão truncado como bug;
5. usar `Palette` e `StyleMetrics` para qualquer extensão necessária;
6. não manter um tema paralelo que redesenhe os widgets padrão;
7. preservar densidade de desktop, sem transformar o produto em uma interface mobile ampliada.

## Versão do Slint

`Cargo.toml` usa a faixa `1.9`, e o `Cargo.lock` atual resolve Slint/Slint Build 1.17.1. A reconstrução usa recursos disponíveis nessa geração, incluindo `MenuBar`, `Palette` e `StyleMetrics`, sem alterar dependências ou lockfile.

## Estrutura

```text
ui/
├── design-system.slint
├── main.slint
├── components/
│   └── app-shell.slint
├── models/
│   └── qso-types.slint
└── pages/
    ├── logbook-page.slint
    ├── qso-editor-page.slint
    ├── tools-page.slint
    └── settings-page.slint
```

`main.slint` continua sendo o contrato compilado por `build.rs`. Properties e callbacks usados pelos handlers Rust devem permanecer estáveis.

## Fundação visual

`ui/design-system.slint` deixa de ser um tema proprietário. Ele contém somente pequenos componentes de semântica/layout que não existem diretamente em `std-widgets.slint`, como:

- `FormField`: label + `LineEdit` com foco e acessibilidade;
- `TextAction`: ação textual acessível para callsign/grid;
- `EmptyState`: estado vazio simples;
- `StatusBar`: faixa de status global.

Esses componentes usam `Palette` e `StyleMetrics`. Não definem paleta própria, raios próprios, níveis de superfície ou uma coleção paralela de botões/cards.

## Shell

### MenuBar

O aplicativo usa `MenuBar`, `Menu`, `MenuItem` e `MenuSeparator` reais do Slint. Não existe mais uma barra customizada simulando menus.

Os menus oferecem acesso secundário a Logbook, New QSO, ações do editor, Tools, Settings e sidebar.

### Sidebar

A sidebar segue o padrão estrutural da Gallery:

- Logbook;
- New QSO;
- Tools;
- Settings.

Não há categorias visuais `Operation`, `Data` ou `System`. O item ativo é indicado pelo próprio estado da navegação, e a sidebar pode ser recolhida.

### Workspace

Somente o conteúdo principal muda. A sidebar e a barra de status permanecem estáveis.

### Status

A barra inferior permanece global e pequena. Cor ou tratamento semântico deve ser usado somente quando necessário; o estado normal deve permanecer discreto.

## Páginas

### Logbook

O Logbook é um workspace de dados, não uma pilha de cards.

A estrutura é:

- título e contagem;
- ação primária `New QSO`;
- busca;
- filtros avançados opcionais em `GroupBox`;
- cabeçalho de colunas;
- `ListView` com linhas separadas discretamente;
- paginação;
- confirmação de exclusão.

Larguras fixas são permitidas apenas onde são úteis para alinhamento tabular previsível, como UTC, callsign, modo, frequência e grid. A coluna de rota permanece elástica.

### New/Edit QSO

O editor é um formulário rolável composto por `GroupBox` nativos:

- Contact;
- Station and report;
- DMR, FT8, D-STAR ou YSF/C4FM quando aplicável;
- Notes.

Campos usam `FormField` e deixam o `LineEdit` nativo definir sua geometria. Não existem linhas decorativas atravessando campos nem containers desenhados manualmente em torno dos inputs.

Save & New, Cancel e Save permanecem disponíveis no rodapé do editor.

### Tools

Tools é dividido em três `GroupBox` naturais:

1. ADIF import and export;
2. Data health;
3. Database backup.

O preview ADIF usa texto e layouts simples, sem cards de métricas promocionais.

### Settings

Settings usa dois grupos principais:

1. Local station;
2. External lookup links.

A identidade local permanece prioritária. Serviços externos continuam opcionais e explícitos.

## Styles

A mesma UI deve poder ser avaliada sem reescrita nos quatro styles principais do Slint:

```bash
SLINT_STYLE=fluent-dark cargo run --locked
SLINT_STYLE=material-dark cargo run --locked
SLINT_STYLE=cupertino-dark cargo run --locked
SLINT_STYLE=cosmic-dark cargo run --locked
```

O style final não deve ser escolhido por preferência teórica. A decisão deve ocorrer após executar o mesmo build e comparar legibilidade, densidade, foco, menus, forms e lista em `1050×680`.

## Compatibilidade obrigatória

A reconstrução preserva:

- callbacks e properties do `MainWindow` consumidos pelo Rust;
- dirty state do editor;
- confirmação de descarte;
- warning de duplicidade;
- preview ADIF;
- confirmação de saída;
- paginação e filtros;
- lookup externo somente após ação explícita;
- foco inicial de callsign e busca;
- `Ctrl+N`, `Ctrl+S`, `Ctrl+Enter`, `Ctrl+F` e `Escape`;
- operação offline/local-first;
- schema, migrations e formatos persistidos.

## Gate de aceitação

O gate técnico continua sendo CI completa. Depois dele é obrigatória uma nova inspeção manual em `1050×680`.

Falha visual imediata:

- texto cortado sem intenção;
- label sobreposta;
- borda ou separador atravessando input;
- botão truncado;
- controles sobrepostos;
- conteúdo inacessível por falta de scroll;
- mudança de página quebrando estado existente;
- style alternativo tornando uma tela inutilizável.

A aprovação visual deve ser registrada em `docs/VISUAL-QA-v0.11.md` somente após executar o build real desta reconstrução.
