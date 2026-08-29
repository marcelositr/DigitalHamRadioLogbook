# UI Architecture v0.11

## Objetivo

A linha v0.11 refatora exclusivamente a camada gráfica do Digital Ham Radio Logbook. A implementação continua em Slint e preserva os contratos Rust existentes, SQLite, migrations, ADIF, configuração, backup, filtros e regras de domínio.

A interface adota uma arquitetura desktop persistente e uma linguagem visual inspirada nos princípios do Material 3, reinterpretada para software desktop operacional. Material é usado como sistema de disciplina para superfícies, hierarquia, estados, espaçamento e semântica de cor; o DHRL não tenta reproduzir a aparência de um aplicativo Android.

A direção é deliberadamente sóbria: superfícies escuras neutras, baixa cromaticidade, poucos níveis de elevação, typography compacta e cor de destaque reservada a ação, seleção e foco. A identidade radioamadora permanece no conteúdo e nos dados do produto, não em ornamentação futurista.

## Princípios

1. **Desktop first.** A janela de referência continua `1050×680`, inclusive em gerenciadores tiled.
2. **Slint permanece a tecnologia gráfica.** Não introduzir Tauri, Electron, Qt, GTK ou camada web.
3. **O shell é persistente.** A navegação e o contexto não são reconstruídos por página.
4. **O workspace muda; a moldura não.** Logbook, editor, Tools e Settings ocupam a mesma área central.
5. **Material como sistema, não como skin mobile.** Usar roles de superfície, state layers, hierarquia e spacing sem inflar controles para densidade de touchscreen.
6. **Cor tem função.** Accent identifica ação, foco ou seleção; success, warning e danger ficam restritos a estados semânticos.
7. **Menos caixas, mais hierarquia.** Espaço, tipografia e alinhamento devem separar conteúdo antes de bordas e containers.
8. **Alta densidade sem poluição.** Callsign, UTC, modo, frequência e rota continuam prioritários.
9. **Uma ação primária por contexto.** Ações secundárias permanecem visíveis, mas visualmente silenciosas.
10. **Teclado é contrato de produto.** `Ctrl+N`, `Ctrl+S`, `Ctrl+Enter`, `Ctrl+F`, `Enter`, `Space` e `Escape` não podem regredir.
11. **Acessibilidade faz parte da arquitetura.** Regiões, botões customizados, foco e status devem manter semântica explícita.
12. **Nenhuma refatoração visual altera persistência ou domínio.** Mudanças de schema, migrations ou formato ADIF são fora de escopo.

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

`main.slint` continua sendo o contrato público compilado por `build.rs`. Os callbacks e properties consumidos pelos handlers Rust devem permanecer estáveis durante a refatoração.

## Shell global

### Top app bar

A faixa superior identifica o produto e comunica discretamente o caráter local/offline. Ela não simula menus que não existem e não duplica a navegação principal.

### Sidebar

A sidebar agrupa a aplicação pelo modelo mental do operador:

- **Operation**: Logbook e New QSO;
- **Data**: Tools;
- **System**: Settings.

O item ativo usa uma superfície tonal e um indicador estreito. Itens inativos permanecem neutros. A sidebar pode ser recolhida sem alterar a página ativa.

### Barra contextual

A barra contextual informa área, página, metadados curtos e estação local. Os rótulos de seção são secundários; o título da página é o elemento tipográfico dominante.

Metadados de filtro, modo atual ou natureza local dos dados podem aparecer à direita, mas não competem com a ação principal do workspace.

### Workspace

É a única região principal mutável. As páginas existentes continuam componentes independentes e recebem os mesmos bindings do `MainWindow`.

### Barra de status

Permanece global, fora do conteúdo rolável. O estado normal é visualmente silencioso; success, warning e error ganham cor somente quando necessário.

## Design system

`ui/design-system.slint` é a fonte central de tokens e componentes básicos.

A direção v0.11 usa:

- superfícies neutras de baixo contraste;
- elevação principalmente tonal;
- outline discreto;
- accent azul/ciano dessaturado somente para foco, seleção e ações;
- estados semanticamente distintos para success, warning e danger;
- grade de espaçamento baseada em 4 px;
- raios moderados de `4/6/8 px`;
- tipografia compacta de desktop;
- pesos médios/semibold no lugar de excesso de bold;
- componentes customizados com hover, focus, active e disabled previsíveis.

`Panel` sem elevação é deliberadamente transparente e sem borda. Isso evita a aparência de "card dentro de card". `raised: true` deve ser reservado a superfícies que realmente precisam de separação.

## Páginas

### Logbook

O Logbook é o principal workspace do produto. A lista assume comportamento visual de data workspace: linhas discretas, pouca ornamentação, busca/filtros previsíveis e `+ New QSO` como ação primária da página.

Callsign, UTC, modo, frequência e banda permanecem no primeiro nível de leitura. Rota, grid e ações ficam secundários. Linhas não usam cards individuais.

### New/Edit QSO

O editor usa título de seção em sentence/title case e labels mais silenciosos. Contact é a superfície principal; Report/Station e Notes podem permanecer visualmente planos. O bloco específico do modo recebe separação suficiente para leitura sem aparência de painel de instrumentação.

Ações de salvar/cancelar continuam fixas e o formulário permanece rolável.

### Tools

Tools é a central administrativa local, organizada em três responsabilidades:

1. interoperabilidade ADIF;
2. diagnóstico read-only;
3. backup SQLite.

ADIF é o fluxo primário e pode usar superfície elevada. Data health e backup permanecem mais planos. Métricas de preview usam superfícies tonais sem bordas desnecessárias.

### Settings

Settings mantém identidade da estação como configuração primária. Serviços externos são secundários e explícitos. Avisos de privacidade e saída para websites devem ser legíveis, mas não dominar a página.

## Compatibilidade

A refatoração v0.11 deve preservar:

- todos os callbacks públicos do `MainWindow`;
- properties ligadas pelos módulos Rust;
- estado dirty do editor;
- confirmação de descarte;
- warning de duplicidade;
- preview ADIF;
- confirmação de saída;
- paginação e filtros;
- lookup externo somente após ação explícita;
- foco inicial de callsign e pesquisa;
- comportamento do clipboard;
- operação offline e local-first.

## Gate visual v0.11

A aprovação visual antiga não é automaticamente herdada pelo novo shell. Antes de concluir a refatoração, executar novamente em `1050×680`:

- shell expandido e recolhido;
- todas as quatro páginas;
- contraste e hierarquia das superfícies Material-inspired;
- hover/focus/selected/disabled nos principais controles;
- QSO genérico, DMR, FT8, D-STAR e YSF/C4FM;
- conteúdo longo;
- banco vazio e resultados vazios;
- filtros abertos e aplicados;
- paginação;
- confirmações e warnings;
- ADIF preview;
- data health e backup;
- tab order;
- Enter/Space/Escape;
- `Ctrl+N`, `Ctrl+S`, `Ctrl+Enter`, `Ctrl+F`;
- encerramento com trabalho pendente.

A aprovação manual deve ser registrada em `docs/VISUAL-QA-v0.11.md` somente depois da inspeção real do build desta branch.
