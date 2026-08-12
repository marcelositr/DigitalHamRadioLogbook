# Progresso de implementação

Última atualização: 2026-08-12

Este arquivo é o checkpoint persistente do projeto. Ao retomar o trabalho, ler `SPEC.md` e este arquivo antes de modificar código.

## Estado dos marcos

- [x] Marco 0 — Fundação do projeto
  - Cargo, Rust, Slint e rusqlite configurados.
  - Toolchain Rust local em `.tools/` e ignorado pelo Git.
  - Estrutura separada entre domínio, persistência e UI.
  - README e `.gitignore` criados.
- [x] Marco 1 — SQLite e migrations
  - Banco no diretório XDG.
  - Migration inicial versionada e idempotente.
  - Foreign keys habilitadas.
  - Schema comum de QSO criado.
- [x] Marco 2 — CRUD manual mínimo
  - Inserção e listagem persistente.
  - Edição.
  - Exclusão com confirmação.
  - Pesquisa por callsign ou modo.
  - Validações mínimas de callsign, frequência e modo.
- [x] Marco 3 — Campos comuns completos do QSO
  - [x] Data e hora UTC legíveis, editáveis e validadas.
  - [x] Banda derivada da frequência com override possível no domínio.
  - [x] RST enviado e recebido no domínio e repository.
  - [x] Grid locator opcional normalizado e validado.
  - [x] Nome, QTH e observações no domínio e repository.
  - [x] Formulário e tabela atualizados com banda, RST, grid, nome, QTH e observações.
  - [x] Testes de domínio e repository atualizados.
- [x] Marco 4 — DMR
  - [x] Modelo `DmrMetadata`, enums e validadores.
  - [x] Migration 2 para metadados DMR e rota digital.
  - [x] Inserção, leitura e atualização transacionais com QSO.
  - [x] Rollback testado para falhas de inserção e atualização DMR.
  - [x] Filtros combináveis por DMR ID, TG, rede, repetidora, hotspot e timeslot no repository.
  - [x] Formulário condicional, criação, edição e resumo de rota DMR na UI.
  - [x] Filtros DMR na UI com aplicação, validação e limpeza.
- [x] Marco 5 — FT8
  - [x] Modelo `Ft8Metadata` e validadores.
  - [x] Migration 3 versionada com cascade.
  - [x] Inserção, leitura e atualização transacionais com rollback testado.
  - [x] Filtros por callsign, grid, banda, SNR e período no repository.
  - [x] Formulário condicional, edição, resumo e filtros visuais na UI.
- [x] Marco 6 — ADIF
  - [x] Modelo de documento, registros e campos.
  - [x] Parser com header opcional, múltiplos registros, UTF-8 e erros posicionais.
  - [x] Preservação de campos desconhecidos e tipos ADIF.
  - [x] Exportação determinística e testes round-trip.
  - [x] Conversão ADIF ↔ domínio para campos comuns, DMR e FT8.
  - [x] Migration 4 e preservação ordenada de campos desconhecidos por QSO.
  - [x] Importação transacional com validação prévia e rollback integral.
  - [x] Exportação completa do banco com header, metadata e extras.
  - [x] Integração com a UI por caminho de arquivo, sem sobrescrita silenciosa.
- [x] Marco 7 — Configuração e fechamento do MVP
  - [x] Callsign local em TOML/XDG com gravação atômica e UI.
  - [x] Backup consistente por `VACUUM INTO`, sem sobrescrita, integrado à UI.
  - [x] Atalhos essenciais: Enter para buscar/salvar e Escape para cancelar.
  - [x] Logging local simples em stderr, sem telemetria nem dados de QSO.
  - [x] Revisão de mensagens e documentação.
- [x] Marco 8 — Validação final e homologação funcional do MVP
  - [x] Executar aplicação em X11 com diretórios XDG isolados, sem crash.
  - [x] QSO DMR via repetidora.
  - [x] QSO DMR via hotspot.
  - [x] QSO DMR simplex.
  - [x] QSO FT8.
  - [x] QSO genérico digital criado, listado e pesquisado pela UI.
  - [x] Edição de QSO existente validada pela UI.
  - [x] Exclusão com confirmação validada pela UI.
  - [x] Reinício sobre o mesmo banco; migrations 1–4 idempotentes e integridade `ok`.
  - [x] Persistência de QSOs e configuração após reinício.
  - [x] Importação/exportação ADIF pela UI, incluindo recusa de sobrescrita e arquivo inválido.
  - [x] Backup pela UI, incluindo integridade e recusa de sobrescrita.
  - [x] Configuração do callsign local validada pela UI.
  - [x] Fluxos funcionais homologados sem dependência de serviços online.
  - [ ] Bloqueio físico da rede não reproduzido no teste automatizado porque o namespace retornou `Operation not permitted`; a aplicação não possui integrações ou dependências de rede.

## Última validação concluída

- `cargo fmt --check`: passou após a reorganização da UI.
- `cargo check`: passou após a reorganização da UI e retorno automático ao Logbook ao salvar.
- `cargo clippy --all-targets --all-features -- -D warnings`: passou após os ajustes responsivos.
- `cargo test`: 49 testes passaram após os ajustes responsivos.
- `cargo build`: passou após os ajustes responsivos.
- Startup X11 com XDG isolado: aplicação permaneceu ativa até o `timeout` esperado, sem crash.
- Diagnósticos do editor: sem erros ou warnings.

## Correção de usabilidade concluída

- [x] Reorganizar a janela em páginas internas: Logbook, Novo/Editar QSO, Ferramentas e Configuração.
- [x] Manter tabela expansível e formulário rolável.
- [x] Mostrar apenas filtros e campos relevantes.
- [x] Manter status e ações principais sempre visíveis.
- [x] Dividir filtros DMR/FT8 em linhas adequadas à largura disponível.
- [x] Tornar “New QSO” explícito e reservar edição para a ação da tabela.
- [x] Validar graficamente em i3/tamanho 1050×680 antes de retomar o Marco 8.
  - Primeira inspeção por quatro screenshots encontrou menu sem contraste e páginas 2–4 sem altura útil.
  - [x] Menu substituído por navegação própria com cores explícitas, estado ativo e hover.
  - [x] Páginas convertidas de elementos apenas invisíveis para blocos condicionais reais.
  - [x] Fundo da janela definido explicitamente para não depender do tema do sistema.
  - [x] Segunda inspeção confirmou navegação e conteúdo nas quatro páginas.
  - [x] Interface padronizada no Nord Theme dark: Polar Night, Snow Storm, Frost e Aurora.
  - [x] Contraste final do tema Nord e as quatro páginas confirmados pelo usuário no i3.
- [x] Executar novamente fmt, clippy, testes e build após os últimos ajustes.

## Refinamento pós-homologação — Logbook

- [x] Consolidar busca e ações em uma toolbar compacta.
- [x] Deixar os filtros avançados recolhidos por padrão.
- [x] Preservar toda a largura da tabela.
- [x] Exibir resumo discreto de filtro ativo.
- [x] Fechar o painel após aplicar ou limpar filtros com sucesso.
- [x] Manter o painel aberto quando houver erro de validação.
- [x] Separar semanticamente `Clear search` de `Clear filters`.
- [x] Executar fmt, check, clippy, 49 testes e build.
- [x] Primeira inspeção confirmou ganho de área, mas encontrou toolbar comprimida e painel DMR sobrepondo a tabela por alturas rígidas.
- [x] Remover alturas fixas da toolbar, resumo e painel avançado para respeitar as métricas naturais do tema.
- [x] Mostrar a faixa de resumo somente quando houver filtro aplicado.
- [x] Padronizar padding e spacing do painel com os demais formulários.
- [x] Alinhamento final e ausência de sobreposição em DMR/FT8 confirmados visualmente no i3.

## Marco 9 — Polimento visual e UX (CONCLUÍDO)

- [x] Criar design system Nord em `ui/design-system.slint`.
- [x] Centralizar cores, espaçamentos e raios.
- [x] Criar títulos reutilizáveis de página e seção.
- [x] Refinar cabeçalho com identidade do produto, subtítulo e badge da estação.
- [x] Migrar navegação para tokens do tema.
- [x] Refinar cabeçalho da tabela do Logbook.
- [x] Adicionar linhas alternadas, destaque de callsign, badge de modo e detalhes secundários.
- [x] Adicionar estado vazio informativo à listagem.
- [x] Agrupar seções DMR e FT8 em cartões visuais.
- [x] Padronizar títulos e seções do formulário.
- [x] Refinar cartões e descrições de Tools e Settings.
- [x] Remover todas as cores soltas de `ui/main.slint`.
- [x] Adicionar feedback semântico à barra de status: info, sucesso, aviso e erro.
- [x] Centralizar atualização de texto e categoria de status no Rust.
- [x] Migrar todos os fluxos existentes para o feedback semântico sem alterar mensagens ou comportamento.
- [x] Destacar ações principais usando `Button.primary`, preservando foco, Tab e Enter nativos.
- [x] Manter ações secundárias discretas e exclusão protegida por confirmação Aurora.
- [x] Executar fmt, check, clippy, 49 testes, build e startup X11 isolado.
- [x] Quatro páginas, feedback semântico, hierarquia das ações e teclado homologados visualmente no i3.
  - [x] Primeira inspeção encontrou o subtítulo do cabeçalho cortado pelas métricas da fonte do sistema.
  - [x] Cabeçalho ampliado de 58px para 72px, com padding vertical e bloco textual centralizado.
  - [x] Subtítulo completo confirmado visualmente após a correção.

## Marco 10 — Hardening visual e consistência (CONCLUÍDO)

### Etapa 1 — Componentes reutilizáveis

- [x] Extrair `StationBadge`, `ModeBadge`, `EmptyState` e `StatusBar` para o design system.
- [x] Evitar extração de páginas/formulários inteiros para não multiplicar bindings frágeis.

### Etapa 2 — Logbook robusto

- [x] Reservar largura mínima para rota/details longos.
- [x] Exibir `—` quando não há detalhes de rota.
- [x] Estabilizar a coluna de ações.
- [x] Refinar confirmação de exclusão com título, callsign e aviso separados.

### Etapa 3 — Formulários consistentes

- [x] Definir proporções deliberadas para callsign, UTC, modo, frequência, banda e grid.
- [x] Priorizar campos DMR textuais sem superdimensionar Timeslot e Color code.
- [x] Usar espaçamento do design system nos labels principais.
- [x] Remover alturas rígidas dos cartões de Tools e Settings.

### Etapa 4 — Texto e acessibilidade

- [x] Padronizar placeholders e linguagem dos campos.
- [x] Dar contraste semântico à indicação de campos obrigatórios.
- [x] Documentar Enter e Escape no rodapé sem criar atalhos novos.
- [x] Preservar widgets nativos, ordem de Tab e foco.

### Etapa 5 — QA visual e fechamento

- [x] Criar `docs/VISUAL-QA.md` com páginas, estados e casos de conteúdo extremo.
- [x] Executar fmt, check, clippy, 49 testes, build, diagnósticos e startup X11 isolado.
- [x] Marco 10 homologado visualmente no i3 usando o checklist.
- [x] Execução formal do Visual QA registrada em `docs/VISUAL-QA.md` em 2026-08-12.
- [x] Checklist global, Logbook, formulário, Tools, Settings, teclado e casos extremos aprovados.
- [x] Revalidação final: fmt, clippy, 49 testes, build, diagnósticos e startup X11 isolado.

## Marco 11 — Seleção gráfica de arquivos (CONCLUÍDO)

- [x] Adicionar `rfd` com backend XDG Portal no Linux.
- [x] Selecionar graficamente ADIF existente para importação.
- [x] Escolher graficamente destino de exportação ADIF com nome datado sugerido.
- [x] Escolher graficamente destino de backup SQLite com nome datado sugerido.
- [x] Manter campos editáveis e todas as validações/proteções existentes.
- [x] Tratar cancelamento sem erro ou alteração de dados.
- [x] Organizar seletores e ações sem compressão em `1050×680`.

## Marco 12 — Release e distribuição Linux (CONCLUÍDO)

- [x] Adicionar metadados Cargo e licença MIT.
- [x] Criar ícone SVG e desktop entry com application ID estável.
- [x] Criar instalação user-local sem sudo e com publicação atômica.
- [x] Criar desinstalação idempotente que preserva banco/configuração.
- [x] Criar build release locked, tarball e SHA-256.
- [x] Validar dependências dinâmicas sem bibliotecas ausentes.
- [x] Testar tarball extraído, instalação, execução fora do projeto e desinstalação dupla.
- [x] Confirmar preservação de banco/configuração por SHA-256.
- [x] Documentar em `docs/LINUX-DISTRIBUTION.md`.

## Marco 13 — Integridade de dados e recuperação (CONCLUÍDO)

- [x] Recusar schema futuro antes de migrations/escritas.
- [x] Detectar marcadores de migration com objetos obrigatórios ausentes.
- [x] Executar `quick_check` e `foreign_key_check` na abertura.
- [x] Validar backups antes de anunciar sucesso e remover resultados incertos.
- [x] Aplicar permissões privadas `0600` a banco, configuração e backup no Unix.
- [x] Usar temporários únicos e sincronizar diretório após configuração/ADIF atômicos.
- [x] Preservar arquivos não-SQLite inválidos sem substituí-los.
- [x] Documentar restauração segura em `docs/DATA-RECOVERY.md`.
- [x] Ampliar a suíte de 49 para 53 testes.

## Marco 14 — Links externos configuráveis

- [x] Adicionar padrões QRZ para callsign e Levine Central para GridSquare.
- [x] Persistir templates em `config.toml` com compatibilidade retroativa.
- [x] Aceitar somente HTTP/HTTPS e exigir `{callsign}`/`{grid}`.
- [x] Aplicar percent-encoding antes de abrir a URL.
- [x] Adicionar cartão de configuração, aviso de privacidade, restaurar padrões e salvar.
- [x] Tornar callsign e grid clicáveis com hover na tabela.
- [x] Não tornar grid vazio clicável e exibir `—`.
- [x] Abrir somente após clique explícito, sem requests em segundo plano.
- [ ] Homologar visualmente no i3 e confirmar abertura no navegador padrão.
- [x] Corrigir espaço fantasma entre detalhes DMR/FT8 e Notes usando blocos condicionais reais.
- [x] Normalizar modo durante digitação (`DMR`, `dmr`, `Ft8`, espaços etc.).
- [x] Ampliar a suíte para 56 testes.

## Próxima ação exata

Validar no i3: clicar em callsign, clicar em grid, confirmar que grid vazio não abre, salvar templates personalizados, reiniciar e confirmar persistência, restaurar padrões e salvar. Reabrir os itens pendentes de `docs/VISUAL-QA.md`. Alterações permanecem somente na branch `develop` até homologação.
