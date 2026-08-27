# Progresso de implementação

Última atualização: 2026-08-26

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

## Marco 14 — Links externos configuráveis (CONCLUÍDO)

- [x] Adicionar padrões QRZ para callsign e Levine Central para GridSquare.
- [x] Persistir templates em `config.toml` com compatibilidade retroativa.
- [x] Aceitar somente HTTP/HTTPS e exigir `{callsign}`/`{grid}`.
- [x] Aplicar percent-encoding antes de abrir a URL.
- [x] Adicionar cartão de configuração, aviso de privacidade, restaurar padrões e salvar.
- [x] Tornar callsign e grid clicáveis com hover na tabela.
- [x] Não tornar grid vazio clicável e exibir `—`.
- [x] Abrir somente após clique explícito, sem requests em segundo plano.
- [x] Homologado visualmente no i3; callsign e grid abrem corretamente no navegador padrão.
- [x] Corrigir espaço fantasma entre detalhes DMR/FT8 e Notes usando blocos condicionais reais; homologado pelo usuário no i3.
- [x] Normalizar modo durante digitação (`DMR`, `dmr`, `Ft8`, espaços etc.); M17 confirmado como modo genérico.
- [x] Ampliar a suíte para 56 testes.

## Marco 15 — Integridade da importação ADIF

- [x] Definir duplicidade exata por callsign, data/hora UTC inicial, frequência e modo normalizados.
- [x] Validar o documento inteiro antes de iniciar a transação.
- [x] Ignorar duplicados existentes e repetições dentro do mesmo documento.
- [x] Preservar o primeiro registro sem mesclar ou sobrescrever metadados.
- [x] Manter duplicados manuais legítimos permitidos.
- [x] Informar separadamente QSOs importados e duplicados ignorados na UI e no log.
- [x] Cobrir reimportação, QSO manual existente e diferenças em cada campo da identidade.
- [x] Validar fmt, clippy, 60 testes, build e startup X11 isolado.
- [x] Homologar visualmente a reimportação e a exportação pela UI no i3.

## Marco 16 — Preview seguro da importação ADIF

- [x] Analisar o arquivo sem alterar o banco.
- [x] Exibir total, novos QSOs, duplicados, inválidos e distribuição por modo.
- [x] Manter um plano imutável em memória, sem reler o arquivo na confirmação.
- [x] Importar somente registros válidos e novos após confirmação explícita.
- [x] Cancelar pelo botão ou `Esc` sem escrita no banco.
- [x] Desabilitar a confirmação quando não houver QSO novo.
- [x] Cobrir preview sem escrita, cancelamento e confirmação por testes.
- [x] Validar fmt, clippy, 62 testes, build e startup X11 isolado.
- [x] Homologar visualmente o preview no i3.

## Marco 17A — Proteção contra perda de edição

- [x] Comparar o formulário atual com um snapshot completo do estado inicial.
- [x] Não exibir aviso quando o formulário não foi alterado.
- [x] Proteger troca de aba, `Esc`, Cancel e abertura de um novo QSO.
- [x] Oferecer Continue editing e Discard changes em confirmação inline responsiva.
- [x] Cobrir campos comuns, DMR e FT8 no snapshot.
- [x] Manter salvamento e erros de validação sem interferência.
- [x] Validar fmt, clippy, 63 testes, build e startup X11 isolado.
- [x] Homologar visualmente os fluxos de descarte no i3.

## Marco 17B — Preferências operacionais

- [x] Adicionar seção TOML retrocompatível para preferências operacionais.
- [x] Lembrar separadamente as últimas pastas de importação, exportação e backup.
- [x] Ignorar pastas removidas ou inacessíveis e usar o fallback do seletor.
- [x] Restaurar última aba, tipo de filtro e painel expandido com valores sanitizados.
- [x] Não persistir pesquisa, valores de filtros, filtro aplicado ou rascunhos de QSO.
- [x] Preservar gravação atômica e permissões privadas da configuração.
- [x] Validar fmt, clippy, 64 testes, build e startup X11 isolado.
- [x] Homologar persistência operacional no i3.

## Marco 18 — Fechamento seguro da aplicação

- [x] Interceptar fechamento da janela pela API nativa do Slint.
- [x] Detectar edição de QSO e preview ADIF pendentes por estado compartilhado.
- [x] Oferecer Continue working e Discard and exit em confirmação global responsiva.
- [x] Salvar preferências operacionais antes de toda saída normal.
- [x] Manter a janela aberta quando a persistência falhar.
- [x] Oferecer Try again e Exit without saving preferences após falha.
- [x] Não tocar no banco ao descartar trabalho não confirmado.
- [x] Validar fmt, clippy, 65 testes, build e startup X11 isolado.
- [x] Homologar fechamento seguro no i3.

## Marco 19 — Diagnóstico operacional acionável

- [x] Centralizar mensagens acionáveis sem ocultar o detalhe técnico.
- [x] Orientar escolha de novo nome quando o destino já existe.
- [x] Orientar seleção de caminho existente para arquivo ou pasta ausente.
- [x] Orientar uso de local gravável para permissão negada ou filesystem somente leitura.
- [x] Aplicar a configuração, links externos, ADIF, backup e fechamento seguro.
- [x] Preservar mensagens de validação já claras sem ruído adicional.
- [x] Cobrir erros tipados e mensagens internas por testes.
- [x] Validar fmt, clippy, 67 testes, build e startup X11 isolado.
- [x] Homologar mensagens acionáveis no i3.

## Marco 20 — Refatoração estrutural sem mudança de comportamento

- [x] Reduzir `src/main.rs` de 1.636 para 81 linhas como composition root.
- [x] Separar handlers Rust em módulos coesos sob `src/app/`.
- [x] Mover os testes do binário para os módulos proprietários sem reduzir cobertura.
- [x] Manter `MainWindow` como contrato público e proprietário do estado Slint.
- [x] Extrair Logbook, editor, Tools e Settings para `ui/pages/`.
- [x] Extrair `QsoRow` para `ui/models/` sem alterar a API gerada.
- [x] Preservar FocusScope, callbacks, bindings, ordem de efeitos e layout `1050×680`.
- [x] Manter o design system Nord inalterado.
- [x] Validar fmt, clippy, 67 testes, build e startup X11 isolado.
- [x] Homologar visualmente e funcionalmente as quatro páginas no i3.

## Marco 21 — Paginação e consultas eficientes

- [x] Adicionar migration 5 com índices para ordenação e filtros frequentes.
- [x] Paginar busca geral, DMR e FT8 diretamente no SQLite em páginas de 100 QSOs.
- [x] Calcular total separadamente e manter ordenação estável por data e ID.
- [x] Carregar QSO, DMR e FT8 por joins em uma consulta, eliminando N+1 na lista.
- [x] Manter em memória o contexto de busca/filtro ao navegar Previous/Next.
- [x] Reposicionar para uma página válida após exclusão e atualizar após salvar/importar.
- [x] Exibir faixa, total, página atual e quantidade de páginas no Logbook.
- [x] Preservar APIs antigas para exportação e compatibilidade interna.
- [x] Validar com 10.000 QSOs isolados: schema 5, integridade ok e zero violações de FK.
- [x] Medir primeira página (~0,42 ms), página 50 (~2,22 ms) e filtros (<2,3 ms) no ambiente local.
- [x] Validar fmt, clippy, 72 testes, build e startup X11 com 10.000 QSOs.
- [x] Homologar paginação, busca e filtros no i3 com a base isolada de 10.000 QSOs.

## Marco 22 — Relatório detalhado da importação ADIF (CONCLUÍDO)

- [x] Manter total, novos QSOs, duplicados, inválidos e distribuição por modo.
- [x] Adicionar distribuição por banda para todos os registros ADIF válidos.
- [x] Exibir o intervalo entre a primeira e a última data/hora UTC válida.
- [x] Expor a regra exata de duplicidade no próprio preview.
- [x] Listar número e motivo dos registros inválidos, limitando a apresentação a 20 detalhes e informando omissões.
- [x] Manter registros inválidos fora do plano de escrita e a confirmação sem releitura do arquivo.
- [x] Preservar cancelamento e preview sem qualquer escrita no banco.
- [x] Validar `cargo fmt --check`, clippy estrito, 72 testes e build locked.
- [x] Executar startup X11 com HOME/XDG isolados; aplicação permaneceu ativa até o `timeout` esperado.
- [x] Homologar visualmente no i3 em `1050×680`, incluindo preview com registros inválidos.

## Marco 23 — CI no GitHub e matriz de migrations (CONCLUÍDO)

- [x] Criar workflow para pushes e pull requests em `main` e `develop`, além de execução manual.
- [x] Restringir permissões do workflow a leitura de conteúdo e cancelar execuções obsoletas da mesma referência.
- [x] Fixar Ubuntu 24.04, instalar dependências Linux explícitas e usar toolchain Rust estável.
- [x] Executar fmt, clippy estrito, 73 testes e build com `Cargo.lock` no job de qualidade.
- [x] Criar matriz paralela para bancos novos e schemas v1–v5.
- [x] Preservar fixtures representativos de QSO, DMR, rota digital, FT8 e campos ADIF conforme disponíveis em cada versão.
- [x] Validar chegada ao schema atual, segunda execução idempotente, `quick_check` e ausência de violações de chave estrangeira.
- [x] Executar localmente a matriz completa e cada uma das seis entradas isoladas.
- [x] Executar startup X11 com HOME/XDG isolados; aplicação permaneceu ativa até o `timeout` esperado.
- [x] Documentar comandos locked, cobertura da CI e adicionar badge da branch `develop` ao README.
- [x] Publicar em `develop` e confirmar os sete jobs verdes no GitHub Actions ([execução #1](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/runs/31740488824)).

## Marco 24 — Acessibilidade e navegação por teclado (CONCLUÍDO)

- [x] Auditar widgets nativos, `TouchArea`, ordem de foco, atalhos e suporte de acessibilidade do Slint 1.17.
- [x] Criar ação visual reutilizável com mouse, Tab, Enter, Espaço e foco Nord de alto contraste.
- [x] Corrigir a regressão em que o `FocusScope` exigia dois cliques, preservando clique único e ativação por teclado.
- [x] Tornar as quatro opções da navegação superior acessíveis por teclado.
- [x] Tornar links de callsign e GridSquare acessíveis por teclado sem criar foco para grids vazios.
- [x] Expor papel, rótulo, descrição, estado habilitado e ação padrão às tecnologias assistivas.
- [x] Marcar banner, navegação, conteúdo principal, busca, formulário e status live region.
- [x] Adicionar rótulos explícitos aos campos essenciais de Logbook, editor, Tools e Settings.
- [x] Preservar Search/Notes com Enter e cancelamento global com Escape.
- [x] Documentar teclado no README e adicionar regressão específica ao checklist visual.
- [x] Validar fmt, clippy estrito, 73 testes, build e startup X11 isolado.
- [x] Homologar clique único, teclado, foco e layout no i3 em `1050×680`.

## Marco 25 — Publicação da versão v0.2.0 (CONCLUÍDO)

- [x] Confirmar `v0.1.0` como última tag/release pública e revisar o histórico até `develop`.
- [x] Atualizar a versão do pacote para `0.2.0` em `Cargo.toml` e `Cargo.lock`.
- [x] Criar release notes verificáveis em `docs/RELEASE-NOTES-v0.2.0.md`.
- [x] Validar fmt, clippy estrito, 73 testes, build locked e startup X11 isolado.
- [x] Gerar tarball Linux release e checksum para `0.2.0`.
- [x] Verificar checksum, conteúdo mínimo, permissões e bibliotecas compartilhadas.
- [x] Testar instalação, atualização, execução e desinstalação dupla em HOME/XDG isolados.
- [x] Confirmar preservação do banco e da configuração por SHA-256.
- [x] Publicar o commit de preparação em `develop` e confirmar sete jobs verdes ([execução](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/runs/31744072246)).
- [x] Atualizar `actions/checkout` para v5, eliminando o aviso de Node.js 20, e confirmar sete jobs verdes novamente ([execução](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/runs/31744311303)).
- [x] Integrar `develop` em `main` por fast-forward e confirmar sete jobs verdes ([CI main](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/runs/31744589146)).
- [x] Criar e publicar a tag anotada `v0.2.0` no commit `e918765`.
- [x] Publicar a GitHub Release não-draft, não-prerelease e marcada como Latest, com tarball e checksum.
- [x] Baixar os assets publicados e confirmar SHA-256 e igualdade byte a byte com os artefatos testados.

## Marco 26 — Redesign visual piloto do Logbook (CONCLUÍDO)

### Etapa 1 — Design system técnico (CONCLUÍDA)

- [x] Ler integralmente `SPEC.md`, `PROGRESS.md`, `README.md`, design system, Logbook, shell e modelo `QsoRow`.
- [x] Definir linguagem própria de instrumento técnico desktop, sem tratar Nord como design.
- [x] Centralizar três níveis de profundidade, estrutura, texto, accent e estados operacionais.
- [x] Definir escalas compactas de tipografia, espaçamento e raios técnicos.
- [x] Preservar as APIs dos componentes consumidos pelas páginas existentes.
- [x] Refinar foco, navegação, estação local, modos, vazio e status sem alterar comportamento.
- [x] Adicionar somente primitivas necessárias ao piloto: `Panel`, `Divider`, `Tag`, `FilterChip`, `DataLabel` e `TechnicalValue`.
- [x] Não alterar Rust, banco, modelo, callbacks, páginas ou funcionalidades.
- [x] Validar fmt, check, clippy estrito, 73 testes e build locked.

### Etapa 2 — Shell e navegação (CONCLUÍDA)

- [x] Unificar cabeçalho e navegação em uma barra operacional compacta de 66px.
- [x] Manter as quatro páginas e todos os callbacks da navegação.
- [x] Preservar `New QSO` como ação primária e a estação local como contexto global.
- [x] Evitar sidebar para não comprimir páginas fora do escopo piloto.
- [x] Validar fmt, check, clippy estrito, 73 testes e build locked.

### Etapa 3 — Toolbar, pesquisa e filtros (CONCLUÍDA)

- [x] Criar cabeçalho contextual com total local e contexto `LOCAL / UTC`.
- [x] Integrar pesquisa própria ao design, preservando busca por Enter.
- [x] Reorganizar Search, Clear e Filters com hierarquia de ação clara.
- [x] Manter filtros DMR/FT8 recolhíveis e indicar filtros ativos com chip.
- [x] Preservar critérios, validações e callbacks existentes.
- [x] Validar fmt, check, clippy estrito, 73 testes e build locked.

### Etapa 4 — Lista operacional de QSOs (CONCLUÍDA)

- [x] Substituir visualmente a tabela de oito colunas por linhas operacionais compactas de dois níveis.
- [x] Priorizar timestamp, callsign, modo, frequência e banda.
- [x] Agrupar rota, detalhes técnicos, grid e ações no nível secundário.
- [x] Manter callsign/grid clicáveis e acessíveis por teclado.
- [x] Preservar integralmente edição, exclusão e callbacks externos.
- [x] Validar fmt, check, clippy estrito, 73 testes e build locked.

### Etapa 5 — Estados e interações (CONCLUÍDA)

- [x] Refinar estado vazio e ausência de resultados na nova linguagem visual.
- [x] Refinar paginação com intervalo, total e página atual legíveis.
- [x] Tornar Previous/Next consistentes com as ações customizadas e acessíveis.
- [x] Converter confirmação de exclusão em bloco condicional sem espaço reservado.
- [x] Aplicar hierarquia destrutiva discreta com Cancel e Confirm delete claros.
- [x] Validar fmt, check, clippy estrito, 73 testes e build locked.

### Etapa 6 — QA e homologação (CONCLUÍDA)

- [x] Atualizar o checklist visual específico do piloto.
- [x] Executar validação técnica final completa: fmt, check, clippy estrito, 73 testes e build locked.
- [x] Executar startup X11 com HOME/XDG isolados; aplicação permaneceu ativa até o timeout esperado, sem crash.
- [x] Corrigir o recorte da faixa secundária dos QSOs identificado no primeiro print: linha ajustada à altura natural das duas faixas e callsign compactado; fmt, check, clippy, 73 testes e build passaram novamente.
- [x] Homologar visualmente o Logbook no i3 em `1050×680`.
- [x] Confirmar compatibilidade visual mínima das páginas não redesenhadas.

## Próxima ação exata

Abrir o aplicativo para homologação visual das quatro páginas em `1050×680` usando os checklists dos Marcos 26 e 27 em `docs/VISUAL-QA.md`.

## Marco 27 — Propagação da identidade visual (CONCLUÍDO)

### Etapa 1 — Auditoria das páginas (CONCLUÍDA)

- [x] Revisar integralmente Editor, Tools e Settings contra o novo design system.
- [x] Confirmar que a propagação pode permanecer restrita aos três arquivos Slint.
- [x] Mapear todos os bindings, callbacks, estados condicionais e requisitos de teclado.

### Etapa 2 — New/Edit QSO (CONCLUÍDA)

- [x] Criar cabeçalho operacional com contexto New/Edit, modo e UTC.
- [x] Reorganizar campos comuns em painéis técnicos compactos.
- [x] Diferenciar seções condicionais DMR e FT8 sem alterar campos ou regras.
- [x] Refinar Notes, confirmação de descarte e rodapé fixo.
- [x] Preservar Enter, Escape, Tab, bindings e callbacks.

### Etapa 3 — Tools (CONCLUÍDA)

- [x] Tornar ADIF o fluxo operacional principal e backup uma operação secundária clara.
- [x] Reorganizar caminhos, seletores e ações sem alterar os diálogos existentes.
- [x] Tornar preview ADIF escaneável para totais, modos, bandas, UTC, duplicados e inválidos.
- [x] Preservar preview, cancelamento, confirmação, exportação e backup.

### Etapa 4 — Settings (CONCLUÍDA)

- [x] Destacar a estação local como identidade operacional.
- [x] Manter links externos como configuração secundária.
- [x] Refinar aviso de privacidade/offline sem alterar templates ou validações.
- [x] Preservar salvar estação, restaurar padrões e salvar links.

### Etapa 5 — Integração e validação técnica (CONCLUÍDA)

- [x] Manter a mesma linguagem de superfícies, tipografia, labels e ações nas quatro páginas.
- [x] Remover domínio visual de botões std sem remover LineEdit dos formulários.
- [x] Confirmar todos os callbacks visuais conectados.
- [x] Validar fmt, check, clippy estrito, 73 testes e build locked.
- [x] Confirmar diagnósticos sem erros ou warnings.

### Etapa 6 — QA visual e fechamento (CONCLUÍDA)

- [x] Registrar checklist visual específico do Marco 27.
- [x] Executar startup X11 com HOME/XDG isolados; aplicação permaneceu ativa até o timeout esperado, sem crash.
- [x] Revisar prints das quatro páginas e identificar expansão vertical inconsistente das ações e sobreposição em Local Station.
- [x] Padronizar ações em 33px, compactas em 27px e links em 20px, impedindo expansão automática.
- [x] Padronizar margens e padding dos painéis em 11px e corrigir a linha de edição do indicativo local.
- [x] Medir o segundo print de Settings e fixar `Local Station` em 210px para conter campo, borda e padding inferior integralmente.
- [x] Auditar preventivamente todas as alturas rígidas e expansões restantes em Logbook, Editor, Tools e Settings; nenhuma outra sobreposição equivalente foi encontrada.
- [x] Revalidar fmt, check, clippy estrito, 73 testes, build e startup X11 após a harmonização dimensional.
- [x] Homologar as quatro páginas no i3 em `1050×680`.
- [x] Confirmar campos genéricos, DMR e FT8, incluindo rolagem completa.
- [x] Confirmar Tools e Settings com caminhos e templates longos.

## Marco 28 — Publicação da versão v0.2.1 (CONCLUÍDO)

- [x] Confirmar `develop` como branch de trabalho e `main` como branch principal do repositório.
- [x] Confirmar autenticação do GitHub CLI e acesso SSH ao remote.
- [x] Atualizar a versão do pacote para `0.2.1` em `Cargo.toml` e `Cargo.lock`.
- [x] Atualizar o README para documentar a nova identidade visual técnica.
- [x] Criar `docs/RELEASE-NOTES-v0.2.1.md`.
- [x] Registrar homologação final em `docs/VISUAL-QA.md`.
- [x] Executar validação release completa e startup X11 isolado.
- [x] Gerar e verificar tarball Linux e SHA-256.
- [x] Confirmar conteúdo mínimo, permissões e ausência de bibliotecas compartilhadas não resolvidas.
- [x] Testar instalação, atualização, execução e desinstalação dupla em HOME/XDG isolados.
- [x] Confirmar preservação de banco e configuração por SHA-256.
- [x] Commitar e publicar a preparação em `develop` no commit `f0884c3`.
- [x] Confirmar sete jobs verdes em `develop` ([execução](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/runs/31845011500)).
- [x] Integrar `develop` em `main` por fast-forward no commit `26b6669` e publicar.
- [x] Confirmar sete jobs verdes em `main` ([execução](https://github.com/marcelositr/DigitalHamRadioLogbook/actions/runs/31845281406)).
- [x] Criar e publicar a tag anotada `v0.2.1` no commit `26b6669`.
- [x] Criar GitHub Release final, não-draft e não-prerelease, com tarball e checksum.
- [x] Baixar os assets publicados e confirmar SHA-256 e igualdade byte a byte.
- [x] Confirmar release pública em https://github.com/marcelositr/DigitalHamRadioLogbook/releases/tag/v0.2.1.

## Próxima ação exata

Versão `v0.2.1` publicada e verificada em https://github.com/marcelositr/DigitalHamRadioLogbook/releases/tag/v0.2.1. `main` e a tag permanecem no commit `26b6669`; continuar novos trabalhos exclusivamente em `develop`.

## Marco 29 — Hardening e confiabilidade v0.2.2 (CONCLUÍDO)

### Fase 1 — Auditoria e inventário (CONCLUÍDA)

- [x] Ler integralmente `SPEC.md`, `PROGRESS.md`, `README.md`, CI e documentação de recuperação.
- [x] Confirmar branch `develop`, versão fonte `0.2.1`, schema SQLite 5 e baseline de 73 testes.
- [x] Auditar banco, migrations, transações, backup, configuração, XDG, ADIF, domínio, CRUD, filtros, encerramento, logging e panics.
- [x] Classificar lacunas por severidade e registrar o inventário em `docs/HARDENING-v0.2.2.md`.
- [x] Definir `0.2.2` como versão alvo em `Cargo.toml` e `Cargo.lock`.

### Fase 2 — Integridade de modo e transações (CONCLUÍDA)

- [x] Reproduzir por testes a permanência indevida de metadados ao trocar DMR, FT8 e modo genérico.
- [x] Confirmar os três testes vermelhos antes da correção.
- [x] Remover todos os metadados incompatíveis dentro da mesma transação de update.
- [x] Confirmar filtros do modo anterior vazios após a troca.
- [x] Confirmar que os testes existentes de rollback DMR/FT8 continuam passando.
- [x] Executar fmt, check, clippy estrito, 76 testes e build locked.
- [x] Criar commit lógico `bd49d62` (`Harden mode transition metadata`).

### Fase 3 — Abertura do banco, schema, migrations e corrupção (CONCLUÍDA)

- [x] Testar criação e reabertura de banco inexistente em arquivo temporário.
- [x] Testar inicialização segura de banco existente com zero bytes.
- [x] Testar SQLite real truncado e confirmar preservação byte a byte após recusa.
- [x] Confirmar recusa de arquivo não-SQLite e schema futuro pelos testes existentes.
- [x] Reproduzir aceitação indevida de índices DMR/FT8 ausentes em schema v5.
- [x] Exigir todos os índices publicados pelas migrations 1–5 na validação final.
- [x] Revalidar migration matrix local dos schemas 0–5.
- [x] Executar fmt, check, clippy estrito, 80 testes e build locked.
- [x] Criar commit lógico `c89077d` (`Test database opening and schema integrity`).

### Fase 4 — Backup e restauração controlada (CONCLUÍDA)

- [x] Validar snapshots contra integridade, foreign keys, versão, tabelas e índices da aplicação.
- [x] Rejeitar e remover backup incerto com schema incompleto ou futuro.
- [x] Executar restauração controlada em diretório temporário.
- [x] Confirmar preservação de QSO genérico, DMR, FT8 e campo ADIF desconhecido.
- [x] Criar commit `ed7ffa1` (`Harden backup and ADIF transactions`).

### Fase 5 — Parser, importação e exportação ADIF (CONCLUÍDA)

- [x] Revalidar duplicados dentro da transação de confirmação do preview.
- [x] Rejeitar campos conhecidos repetidos sem bloquear repetição de campos desconhecidos.
- [x] Cobrir vazio, espaços, header-only, truncamentos, comprimentos e boundaries UTF-8.
- [x] Cobrir destino existente, parent ausente, cleanup e permissões privadas `0600`.
- [x] Criar commit `9f7c5e1` (`Harden ADIF parsing and file writes`).

### Fase 6 — Configuração, XDG e encerramento (CONCLUÍDA)

- [x] Preservar TOML inválido/truncado e retornar erro controlado.
- [x] Confirmar escrita atômica, Unicode/espaços e permissões `0600`.
- [x] Ignorar XDG relativo e exigir fallback HOME absoluto.
- [x] Cobrir XDG ausente e arquivo no lugar do diretório de dados.
- [x] Manter fluxo de encerramento existente e documentar risco pós-rename sem refatoração ampla.
- [x] Criar commit `18ca4ae` (`Harden configuration paths and input validation`).

### Fase 7 — CRUD, pesquisa, filtros, logging, panics e limites (CONCLUÍDA)

- [x] Rejeitar frequência explicitamente negativa, inclusive `-0.5`.
- [x] Rejeitar intervalo UTC FT8 invertido.
- [x] Testar cascata DMR/rota e FT8 pela API pública de exclusão.
- [x] Auditar `panic!`, `unwrap()` e `expect()`; nenhum uso recuperável sobre entrada externa permaneceu no runtime auditado.
- [x] Não introduzir limites arbitrários de tamanho sem política de produto.
- [x] Registrar riscos residuais de refresh pós-mutation, corrida de rename e recursos no documento de hardening.

### Fase 8 — Regressão funcional, gates finais e publicação da v0.2.2 (CONCLUÍDA)

- [x] Executar fmt, check, clippy estrito, 99 testes e build locked.
- [x] Atualizar README, recuperação, hardening e release notes; criar commit `2242347`.
- [x] Executar migration matrix isolada para schemas 0–5.
- [x] Executar startup X11 isolado com versão `0.2.2`.
- [x] Gerar tarball Linux e SHA-256 sem publicar.
- [x] Confirmar conteúdo, dependências compartilhadas, instalação, atualização e startup instalado.
- [x] Confirmar desinstalação dupla e preservação de banco/configuração por hash.
- [x] Confirmar ausência de bugs Critical/High conhecidos de integridade.
- [x] Publicar commits em `develop` e confirmar CI remoto.
- [x] Executar regressão funcional manual pelo mantenedor.
- [x] Integrar em `main`, publicar tag e GitHub Release após autorização explícita.
- [x] Confirmar CI completo de `main`, incluindo schemas 0–5.
- [x] Publicar `v0.2.2` em https://github.com/marcelositr/DigitalHamRadioLogbook/releases/tag/v0.2.2.

## Marco 30 — Robustez estrutural v0.3.0 (CONCLUÍDO)

### Fase 1 — Auditoria e baseline (CONCLUÍDA)

- [x] Ler especificação, progresso, README, hardening, CI, migrations, repository e chamadas da aplicação.
- [x] Confirmar baseline `v0.2.2`, schema 5 e 99 testes.
- [x] Mapear responsabilidades e preservar API pública/transações.
- [x] Identificar N+1 na exportação ADIF e ausência de CI para packaging.

### Fase 2 — Organização do repository (CONCLUÍDA)

- [x] Separar CRUD/agregado, queries, ADIF e backup sem traits ou camadas artificiais.
- [x] Manter SQL, API pública e chamadas de `src/app`.
- [x] Revalidar rollback, CRUD, filtros, paginação, ADIF e backup.
- [x] Documentar fronteiras em `docs/ARCHITECTURE.md`.

### Fase 3 — Volume e performance (CONCLUÍDA)

- [x] Criar gerador determinístico ignorado por padrão.
- [x] Medir 1k, 10k, 100k e 1M em release.
- [x] Medir abertura, páginas, buscas, filtros DMR/FT8, backup e ADIF.
- [x] Remover N+1 da exportação ADIF; reduzir 100k de ~9,26 s para ~1,07 s.
- [x] Manter OFFSET: ~90 ms na página final de 100k não justifica mudança de contrato.
- [x] Documentar ambiente, resultados e limite de 1M em `docs/PERFORMANCE-v0.3.0.md`.

### Fase 4 — Migrations e SQLite (CONCLUÍDA)

- [x] Preservar migrations publicadas 1–5 sem alteração.
- [x] Manter matriz determinística de schemas 0–5 com dados, idempotência, quick check e foreign keys.
- [x] Registrar planos representativos e decisão de não adicionar índices sem benefício medido.
- [x] Reexecutar matriz completa dos schemas 0–5 após mudanças finais.

### Fase 5 — Distribuição e release engineering (CONCLUÍDA)

- [x] Tornar tarball normalizado e publicação tarball/checksum resistente a falhas.
- [x] Criar smoke test POSIX de conteúdo, checksum, instalação, reinstalação e remoção.
- [x] Preservar banco/configuração em XDG isolado.
- [x] Adicionar job pequeno de packaging à CI.
- [x] Normalizar permissões e comprovar tarballs idênticos sob `umask 002` e `077`.
- [x] Executar pacote real, instalação `v0.2.2`, atualização para `v0.3.0` e desinstalação isoladas.
- [x] Confirmar schema 5, integridade e preservação por hash de QSO/configuração.

### Fase 6 — Gates finais e publicação (CONCLUÍDA)

- [x] Finalizar changelog, arquitetura, performance e release notes.
- [x] Executar fmt, check, clippy estrito, 99 testes ativos, build e migration matrix.
- [x] Executar stress determinístico 1k, 10k, 100k e 1M.
- [x] Executar validação automatizada de distribuição e upgrade real.
- [x] Executar regressão manual pelo mantenedor sem novas funcionalidades.
- [x] Preparar artefato final normalizado (`7298e558d9b901cf551ff91fc7964cfdb16bb6f119a38c78440e5c335c4dfd9d`).
- [x] Aguardar e receber aprovação da regressão manual.
- [x] Publicar `develop` e confirmar os oito jobs de CI, incluindo packaging e schemas 0–5.
- [x] Parar antes de `main`, tag e GitHub Release.
- [x] Integrar em `main`, publicar tag/release `v0.3.0` e confirmar oito jobs de CI após autorização.

## Marco 31 — Interoperabilidade ADIF v0.4.0 (CONCLUÍDO)

### Fase 1 — Auditoria e inventário (CONCLUÍDA)

- [x] Ler documentação, parser, exporter, converter, repository e testes ADIF.
- [x] Inventariar campos comuns, DMR, FT8, aliases, APP fields e unknown fields.
- [x] Identificar perda silenciosa em aliases e RX/TX DMR.
- [x] Confirmar baseline de 25 testes ADIF filtrados.

### Fase 2 — Corpus e parser (CONCLUÍDA)

- [x] Criar 16 fixtures válidas e 8 inválidas, sintéticas e sem dados pessoais.
- [x] Cobrir header, múltiplos registros, DMR, FT8, Unicode, tipos, unknowns, caixa, whitespace e CRLF.
- [x] Tornar BOM inicial explícito e rejeitar arquivo não vazio sem tags.
- [x] Validar nomes/tipos estruturais e melhorar diagnóstico de EOH fora de ordem.

### Fase 3 — Preservação e round-trip (CONCLUÍDA)

- [x] Rejeitar conflitos de aliases em vez de descartar valores.
- [x] Preservar DMR RX/TX com campos privados estáveis.
- [x] Adicionar `PROGRAMVERSION` ao header exportado.
- [x] Cobrir round-trip completo por dois bancos SQLite para genérico, DMR, FT8, unknown/APP fields e Unicode.

### Fase 4 — Fuzzing (CONCLUÍDA)

- [x] Criar target mínimo `bytes → UTF-8 válido → parser` com `cargo-fuzz`.
- [x] Usar as 24 fixtures como seeds.
- [x] Executar 60 segundos, 3.618.168 entradas e zero crashes conhecidos.

### Fase 5 — Documentação e gates (CONCLUÍDA)

- [x] Documentar interoperabilidade e extensões privadas.
- [x] Atualizar versão fonte para `0.4.0` e changelog.
- [x] Executar fmt, check, clippy estrito, 113 testes ativos e build locked.
- [x] Confirmar exportação de 100k em ~1,03 s, sem regressão relevante.
- [x] Executar regressão manual final de DMR, FT8, unknown/APP fields, Unicode, CRLF e arquivo quebrado.
- [x] Publicar `develop` e confirmar oito jobs de CI.
- [x] Preparar artefato final normalizado (`8bd5d126deabab13e333ae16e37a5e60da5bd39a8ccefbee78bbde2f3232f5b1`).
- [x] Parar antes de `main`, tag e GitHub Release.
- [x] Concluir posteriormente a publicação de `v0.4.0` como release final.

## Marco 32 — Suporte D-STAR e v0.5.0 (CONCLUÍDO E PUBLICADO)

- [x] Confirmar baseline inicial de 113 testes ativos + 1 ignored.
- [x] Adicionar modelo `DStarMetadata`: reflector, module, MYCALL, URCALL, RPT1, RPT2 e notes.
- [x] Adicionar schema 6 com tabela `dstar_metadata`, cascade e índices para reflector, module e RPT1.
- [x] Cobrir migrations determinísticas dos schemas 0–6.
- [x] Adicionar CRUD, materialização e filtros por reflector, module e RPT1 no repository/query.
- [x] Adicionar editor, listagem e filtros D-STAR específicos na UI.
- [x] Exportar ADIF canônico `MODE=DIGITALVOICE` + `SUBMODE=DSTAR` e importar também o histórico `MODE=DSTAR`.
- [x] Definir `APP_DHRL_DSTAR_*`, usando `STATION_CALLSIGN` para MYCALL e aceitando `APP_DHRL_DSTAR_MYCALL` como alias de importação.
- [x] Manter `digital_routes` específico de DMR e não introduzir traits/plugins.
- [x] Fatorar somente a limpeza de metadata incompatível já existente para incluir D-STAR.
- [x] Confirmar suíte final atual: 133 testes ativos + 1 ignored.
- [x] Registrar stress release de 10k: first 1.039 ms; final 10.522 ms; DSTAR 2.599 ms; backup 35.099 ms; export-domain 111.243 ms; serialize 16.221 ms.
- [x] Registrar stress release de 100k: first 7.530 ms; final 113.225 ms; DSTAR 9.444 ms; backup 363.867 ms; export 1132.022 ms; serialize 148.753 ms.
- [x] Atualizar versão fonte e documentação para `0.5.0`.
- [x] Publicar a tag/release `v0.5.0`; `main` no commit `ef262bd`.

## Marco 33 — YSF/C4FM e v0.6.0 (CONCLUÍDO E PUBLICADO)

- [x] Confirmar baseline inicial de 133 testes ativos + 1 ignored.
- [x] Adicionar `YsfMetadata` e consolidar `ModeMetadata` com variantes Generic, DMR, FT8, D-STAR e YSF.
- [x] Representar o modo internamente como `C4FM`; aceitar `YSF` e `SYSTEM FUSION` como aliases de UI.
- [x] Modelar room, WIRES-X node, repeater, network, access type, TX/RX DG-ID e notes.
- [x] Adicionar schema 7 com `ysf_metadata`, cascade e índices apenas para TX/RX DG-ID após `EXPLAIN QUERY PLAN`.
- [x] Manter room e WIRES-X node como buscas substring sem índice e `digital_routes` específico de DMR.
- [x] Adicionar CRUD, materialização, filtros e UI específicos de YSF/C4FM.
- [x] Exportar ADIF canônico `MODE=DIGITALVOICE` + `SUBMODE=C4FM` e importar também o histórico `MODE=C4FM`.
- [x] Definir os campos `APP_DHRL_YSF_*` e preservar/reconciliar extras ADIF sem duplicar campos que se tornam conhecidos.
- [x] Exigir integridade entre modo do QSO e variante de `ModeMetadata` nas operações agregadas.
- [x] Revisar a arquitetura de quatro modos e manter implementação explícita, sem traits/plugins.
- [x] Confirmar suíte final atual: 157 testes ativos + 1 ignored.
- [x] Registrar stress release de 10k: first 1.031 ms; middle 6.765 ms; final 12.481 ms; search 6.720 ms; DSTAR 2.953 ms; YSF room 1.720 ms; node 1.784 ms; DG-ID 1.676 ms; backup 39.617 ms; export-domain 114.442 ms; serialize 17.727 ms.
- [x] Registrar stress release de 100k: first 7.601 ms; middle 73.706 ms; final 138.205 ms; search 74.263 ms; DSTAR 12.859 ms; YSF room 9.052 ms; node 13.297 ms; DG-ID 9.356 ms; backup 358.293 ms; export-domain 1200.912 ms; serialize 153.598 ms.
- [x] Atualizar versão fonte e documentação para `0.6.0`.
- [x] Publicar a tag/release `v0.6.0`; `main` e a tag no commit `034996f`.

## Marco 34 — Save & New e duplicidade manual v0.7.0 (PUBLICADO)

- [x] Adicionar **Save & New** somente ao fluxo de novo QSO.
- [x] Executar validar → commit → refresh → limpeza integral → novo UTC fixo, sem criar um segundo QSO.
- [x] Limpar todos os campos comuns, metadados de Generic/DMR/FT8/D-STAR/YSF e metadata interna do editor.
- [x] Adicionar guard contra double-submit e atualizar o snapshot somente após commit bem-sucedido.
- [x] Adicionar aviso de duplicidade manual por callsign + UTC inicial + frequência em Hz + modo normalizados.
- [x] Oferecer **Review** e **Save anyway**; nunca fazer merge ou bloquear duplicados intencionais.
- [x] Excluir o próprio ID da consulta durante edição.
- [x] Manter schema 7, sem migration, índice novo ou constraint `UNIQUE`.
- [x] Cobrir `Ctrl+N`, `Ctrl+S`, `Ctrl+Enter`, `Ctrl+F`, `Enter` em Notes e `Escape` exclusivo; preservar clipboard.
- [x] Direcionar foco para callsign em novo QSO e para search com `Ctrl+F`.
- [x] Confirmar suíte final atual: 165 testes ativos + 1 ignored.
- [x] Medir duplicate lookup release em 100k, 200 iterações: hit 0.029352 ms; miss 0.028080 ms; self 0.028480 ms; collision 0.029927 ms.
- [x] Confirmar plano por `idx_qsos_datetime_start` e decisão de não adicionar índice.
- [x] Atualizar versão fonte e documentação para `0.7.0` sem publicar.
- [x] Publicar `v0.7.0`; `main` e tag no commit `a56a7d9`.

## Marco 35 — Saúde do acervo e manutenção operacional v0.8.0 (EM DESENVOLVIMENTO)

- [x] Auditar health, backup, restore, migrations, Tools, ADIF, filesystem, CI e baseline real da v0.7.0.
- [x] Adicionar health check read-only com quick check, foreign keys, schema, migrations, objetos, contagens e invariantes de metadata.
- [x] Distinguir schema atual, antigo migrável, futuro incompatível, inválido/corrompido e ilegível.
- [x] Preservar privacidade do relatório: sem callsigns, nomes, QTH, notes ou paths completos.
- [x] Publicar backup somente após temporário, validação read-only, permissões, sync e publicação sem overwrite.
- [x] Adicionar verificação operacional read-only de backups existentes.
- [x] Adicionar exportação ADIF de todos os resultados do filtro atual, sem paginação e sem N+1.
- [x] Cobrir 17/100, 350 resultados, metadata DMR/FT8/D-STAR/YSF e unknown fields.
- [x] Manter restore assistido/documentado; não substituir banco com conexão ativa.
- [x] Manter schema 7 e dependências atuais.
- [x] Confirmar suíte atual: 175 testes ativos + 1 stress ignored.
- [ ] Executar stress de health/export em 100k e QA visual/manual de Tools em 1050×680.
- [ ] Executar gates finais, packaging, startup e migration matrix.

## Próxima ação exata

Executar stress 100k, gates finais e checklist manual de Tools/recuperação. Não integrar em `main`, criar tag ou publicar sem autorização explícita.
