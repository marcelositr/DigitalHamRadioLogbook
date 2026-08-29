# Visual QA v0.11

Status: **pendente de aprovação manual após o enterprise polish**.

A refatoração v0.11 altera shell, superfícies, hierarquia tipográfica e estados visuais. A homologação das versões anteriores não é herdada automaticamente. A janela de referência permanece `1050×680`.

## Direção visual

- [ ] A interface parece software desktop operacional, não dashboard, HUD ou aplicativo mobile ampliado.
- [ ] Accent aparece principalmente em ação, foco e seleção, sem decoração gratuita.
- [ ] Superfícies são diferenciadas prioritariamente por tonalidade e espaçamento, não por bordas fortes.
- [ ] Não há excesso de cards ou containers aninhados.
- [ ] Títulos usam sentence/title case; caixa alta fica restrita a siglas e dados técnicos quando útil.
- [ ] Pesos tipográficos mantêm hierarquia sem excesso de bold.
- [ ] Radius e spacing permanecem consistentes entre páginas.
- [ ] Hover, focus, active e disabled são distinguíveis sem glow ou animação chamativa.

## Shell

- [ ] Top app bar permanece totalmente visível em `1050×680`.
- [ ] Indicador `Local database · Offline` é legível e secundário.
- [ ] Sidebar expandida não comprime o workspace a ponto de cortar controles.
- [ ] Sidebar recolhida mantém Logbook, New QSO, Tools e Settings identificáveis e acionáveis.
- [ ] Item ativo da sidebar é claro sem dominar visualmente a navegação.
- [ ] Trocar de página pela sidebar mantém o mesmo comportamento de dirty-state existente.
- [ ] Barra contextual apresenta seção, página e estação local sem clipping.
- [ ] Status global permanece visível e silencioso em estado normal.
- [ ] Nenhuma página exige fullscreen.

## Navegação e teclado

- [ ] `Tab` segue ordem visual coerente no shell e workspace.
- [ ] `Enter` e `Space` acionam comandos customizados focados.
- [ ] `Ctrl+N` abre New QSO e posiciona foco em callsign.
- [ ] `Ctrl+S` salva somente no editor.
- [ ] `Ctrl+Enter` executa Save & New somente durante criação.
- [ ] `Ctrl+F` posiciona foco na pesquisa do Logbook.
- [ ] `Escape` continua reservado a cancelar/fechar o fluxo atual.
- [ ] Clipboard não é alterado pelos atalhos.

## Logbook

- [ ] `+ New QSO` é percebido como ação primária da página sem competir com a pesquisa.
- [ ] Busca, Clear e Filters permanecem alinhados e utilizáveis.
- [ ] Filtros DMR, FT8, D-STAR e YSF/C4FM abrem sem sobreposição.
- [ ] Indicador de filtro aplicado continua visível sem dominar a tela.
- [ ] Lista de QSO permanece elemento visual dominante.
- [ ] Linhas parecem uma lista de dados, não uma sequência de cards independentes.
- [ ] Callsign, UTC, modo, frequência e banda são escaneáveis rapidamente.
- [ ] Route/grid/actions permanecem no nível visual secundário.
- [ ] Conteúdo longo elide sem deslocar ações.
- [ ] Banco vazio e busca sem resultados exibem empty state sóbrio.
- [ ] Paginação funciona em página vazia, parcial, intermediária e final.
- [ ] Lookup de callsign/grid continua exigindo ação explícita.
- [ ] Confirmação de exclusão continua acessível por mouse e teclado.

## New/Edit QSO

- [ ] Novo e edição continuam distinguíveis pelo contexto da janela.
- [ ] Campos comuns permanecem acessíveis em `1050×680`.
- [ ] Contact é o primeiro agrupamento visual e não há excesso de caixas.
- [ ] Painel DMR mostra todos os campos específicos.
- [ ] Painel FT8 mostra todos os campos específicos.
- [ ] Painel D-STAR mostra todos os campos específicos.
- [ ] Painel YSF/C4FM mostra todos os campos específicos.
- [ ] Modo genérico não deixa espaço reservado para painel específico.
- [ ] Rolagem alcança Notes e todos os campos condicionais.
- [ ] Footer fixo mantém Save & New, Cancel e Save visíveis.
- [ ] Warning de duplicidade preserva Review e Save anyway.
- [ ] Confirmação de descarte preserva o formulário quando cancelada.

## Tools

- [ ] ADIF é percebido como fluxo primário sem transformar a página em dashboard.
- [ ] Área ADIF comporta caminhos longos sem retirar ações da tela.
- [ ] Preview mostra total, novos, duplicados, inválidos, modos, bandas e UTC range.
- [ ] Métricas do preview usam superfícies tonais discretas e não parecem cards promocionais.
- [ ] Cancel e Import permanecem utilizáveis na parte inferior do preview.
- [ ] Data health permanece read-only e relatório fica contido.
- [ ] Backup destination, Verify backup e Create backup permanecem acessíveis.

## Settings

- [ ] Identidade da estação continua sendo a configuração principal.
- [ ] Callsign vazio, normal e longo ficam contidos.
- [ ] External lookup links é claramente secundário em relação à estação local.
- [ ] Templates de callsign/grid suportam valores longos sem clipping destrutivo.
- [ ] Restore defaults e Save links permanecem acessíveis.
- [ ] Aviso de privacidade continua secundário e legível.

## Estados globais

- [ ] Exit confirmation permanece totalmente visível.
- [ ] Falha ao salvar preferências mantém retry e saída explícita sem salvar.
- [ ] Status normal, success, warning e error possuem contraste adequado.
- [ ] Estação não configurada mantém warning perceptível sem bloquear operação.
- [ ] Nenhum warning/error usa cor sem texto ou contexto suficiente.

## Resultado

Registrar aqui data, ambiente, tamanho da janela e observações somente depois da validação manual real desta versão visual.
