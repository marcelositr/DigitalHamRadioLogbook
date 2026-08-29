# UI Architecture v0.11

## Objetivo

A linha v0.11 refatora exclusivamente a camada gráfica do Digital Ham Radio Logbook. A implementação continua em Slint e preserva os contratos Rust existentes, SQLite, migrations, ADIF, configuração, backup, filtros e regras de domínio.

A nova interface adota uma arquitetura de aplicativo desktop persistente: menu superior, navegação lateral, barra contextual, workspace central e barra de status. A organização é inspirada em padrões de software operacional, mas a identidade visual continua própria do DHRL: instrumentação de rádio compacta, dados técnicos densos, ciano moderado, superfícies escuras e estados semânticos discretos.

## Princípios

1. **Desktop first.** A janela de referência continua `1050×680`, inclusive em gerenciadores tiled.
2. **Slint permanece a tecnologia gráfica.** Não introduzir Tauri, Electron, Qt, GTK ou camada web.
3. **O shell é persistente.** A navegação e o contexto não são reconstruídos por página.
4. **O workspace muda; a moldura não.** Logbook, editor, Tools e Settings ocupam a mesma área central.
5. **Alta densidade sem poluição.** Callsign, UTC, modo, frequência e rota continuam prioritários.
6. **Ações frequentes ficam visíveis; ações raras ou destrutivas devem ser progressivamente secundárias.**
7. **Teclado é contrato de produto.** `Ctrl+N`, `Ctrl+S`, `Ctrl+Enter`, `Ctrl+F`, `Enter`, `Space` e `Escape` não podem regredir.
8. **Acessibilidade faz parte da arquitetura.** Regiões, botões customizados, foco e status devem manter semântica explícita.
9. **Nenhuma refatoração visual altera persistência ou domínio.** Mudanças de schema, migrations ou formato ADIF são fora de escopo.

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

### Menu superior

Faixa compacta de comandos globais. Ela oferece acesso secundário a Logbook, navegação, Tools e Settings sem competir com o workspace.

### Sidebar

A sidebar agrupa a aplicação pelo modelo mental do operador:

- **Operation**: Logbook e New QSO;
- **Data**: Tools;
- **System**: Settings.

Pode ser recolhida para uma faixa compacta sem alterar a página ativa.

### Barra contextual

A barra contextual informa a área atual e mantém a identidade da estação visível. Exemplos:

- `OPERATION / Logbook`;
- `CONTACT / New QSO`;
- `DATA / Tools`;
- `SYSTEM / Settings`.

Metadados curtos, como estado de filtro, modo atual ou natureza local dos dados, podem aparecer à direita sem substituir informações da página.

### Workspace

É a única região principal mutável. As páginas existentes continuam componentes independentes e recebem os mesmos bindings do `MainWindow`.

### Barra de status

Permanece global, fora do conteúdo rolável. Mensagens `STATUS`, `DONE`, `NOTICE` e `ERROR` continuam discretas e semanticamente identificáveis.

## Design system

`ui/design-system.slint` continua como fonte central de tokens e componentes básicos.

A identidade deve preservar:

- níveis controlados de profundidade;
- accent ciano moderado;
- cores semânticas de baixo brilho;
- tipografia compacta;
- raios pequenos;
- bordas discretas;
- foco de alto contraste;
- valores técnicos visualmente estáveis.

O objetivo não é reproduzir a paleta de outro projeto. O que é compartilhado é a arquitetura de interação, não a aparência temática.

## Páginas

### Logbook

A lista continua sendo o elemento visual dominante. Cada QSO deve permanecer escaneável sem assumir aparência de planilha tradicional. Pesquisa, filtros, paginação e ações devem ocupar regiões previsíveis.

### New/Edit QSO

O editor deve se comportar como um workspace de registro técnico. Dados comuns, relatório/estação e metadata específica do modo permanecem agrupados. Ações de salvar/cancelar continuam fixas e o formulário permanece rolável.

### Tools

Tools é a central administrativa local, organizada em três responsabilidades visuais:

1. interoperabilidade ADIF;
2. diagnóstico read-only;
3. backup SQLite.

### Settings

Settings mantém identidade da estação como configuração primária e serviços externos como configuração secundária e explícita.

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

A aprovação manual deve ser registrada em `docs/VISUAL-QA.md` somente depois da inspeção real do build da branch v0.11.
