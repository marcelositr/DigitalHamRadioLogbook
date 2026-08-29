# UI Architecture v0.11

## Objetivo

A linha v0.11 reconstrói exclusivamente a camada gráfica do Digital Ham Radio Logbook. O backend Rust, domínio, SQLite, migrations, ADIF, backup, filtros, atalhos e contratos funcionais permanecem preservados. A configuração local recebe somente uma preferência visual retrocompatível para o esquema de cores.

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

`Cargo.toml` usa a faixa `1.9`, e o `Cargo.lock` atual resolve Slint/Slint Build 1.17.1. A reconstrução usa recursos disponíveis nessa geração, incluindo `MenuBar`, `Palette`, `StyleMetrics` e alteração runtime de `Palette.color-scheme`, sem alterar dependências ou lockfile.

## Estrutura

```text
ui/
├── app.slint
├── appearance.slint
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

`ui/app.slint` é o entrypoint compilado por `build.rs`. Ele não contém layout: apenas reexporta `MainWindow` e o global `Appearance` para a API Rust gerada. `main.slint` continua concentrando o contrato da janela, e suas properties/callbacks consumidas pelos handlers Rust permanecem estáveis.

## Fundação visual

`ui/design-system.slint` deixa de ser um tema proprietário. Ele contém somente pequenos componentes de semântica/layout que não existem diretamente em `std-widgets.slint`, como:

- `FormField`: label + `LineEdit` com foco e acessibilidade;
- `TextAction`: ação textual acessível para callsign/grid;
- `EmptyState`: estado vazio simples;
- `StatusBar`: faixa de status global.

Esses componentes usam `Palette` e `StyleMetrics`. Não definem paleta própria, raios próprios, níveis de superfície ou uma coleção paralela de botões/cards.

`ui/appearance.slint` concentra somente o esquema de cores runtime e traduz a preferência persistida para `Palette.color-scheme`.

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

Settings usa três grupos principais:

1. Appearance;
2. Local station;
3. External lookup links.

Appearance não escolhe um design diferente para o produto. O style é **Fluent** e permanece fixo. O usuário escolhe somente o esquema de cores:

- `System`: padrão; usa `ColorScheme.unknown` e acompanha a preferência claro/escuro reportada pelo desktop;
- `Light`: força `ColorScheme.light`;
- `Dark`: força `ColorScheme.dark`.

A mudança é aplicada imediatamente por `Palette.color-scheme` e persistida no `config.toml`. Configurações antigas sem a seção `appearance` continuam válidas e recebem `system` como default.

A identidade local permanece prioritária. Serviços externos continuam opcionais e explícitos.

## Style do produto

A comparação inicial entre Fluent, Material, Cupertino e Cosmic foi concluída durante o ciclo de reconstrução. **Fluent foi escolhido como style oficial do Digital Ham Radio Logbook.**

`build.rs` fixa `fluent` por `CompilerConfiguration::with_style`, portanto o produto não depende de `SLINT_STYLE` para definir sua identidade e não oferece troca runtime entre famílias de widgets.

Claro/escuro continua independente do style. O mesmo build Fluent deve ser validado nos três estados de aparência disponíveis em Settings: System, Light e Dark.

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
- schema SQLite, migrations e formatos persistidos.

A preferência `appearance.color_scheme` pertence apenas ao arquivo local de configuração e não altera o schema SQLite.

## Gate de aceitação

O gate técnico continua sendo CI completa. Depois dele é obrigatória uma nova inspeção manual em `1050×680` usando Fluent.

O QA deve cobrir System, Light e Dark, incluindo troca runtime e persistência após reiniciar a aplicação.

Falha visual imediata:

- texto cortado sem intenção;
- label sobreposta;
- borda ou separador atravessando input;
- botão truncado;
- controles sobrepostos;
- conteúdo inacessível por falta de scroll;
- mudança de página quebrando estado existente;
- Light ou Dark tornando uma tela ou estado essencial ilegível.

A aprovação visual deve ser registrada em [`../quality/VISUAL-QA-v0.11.md`](../quality/VISUAL-QA-v0.11.md) somente após executar o build real desta reconstrução.
