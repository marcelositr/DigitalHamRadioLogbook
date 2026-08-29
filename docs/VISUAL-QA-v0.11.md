# Visual QA v0.11

Status: **pendente de aprovação manual**.

A refatoração v0.11 altera o shell e a hierarquia visual, portanto a homologação visual das versões anteriores não é herdada automaticamente. A janela de referência permanece `1050×680`.

## Shell

- [ ] Menu superior permanece totalmente visível em `1050×680`.
- [ ] Sidebar expandida não comprime o workspace a ponto de cortar controles.
- [ ] Sidebar recolhida mantém Logbook, New QSO, Tools e Settings identificáveis e acionáveis.
- [ ] Trocar de página pela sidebar mantém o mesmo comportamento de dirty-state existente.
- [ ] Barra contextual apresenta seção, página e estação local sem clipping.
- [ ] Status global permanece visível em todas as páginas.
- [ ] Nenhuma página exige fullscreen.

## Navegação e teclado

- [ ] `Tab` segue ordem visual coerente no menu, sidebar e workspace.
- [ ] `Enter` e `Space` acionam comandos customizados focados.
- [ ] `Ctrl+N` abre New QSO e posiciona foco em callsign.
- [ ] `Ctrl+S` salva somente no editor.
- [ ] `Ctrl+Enter` executa Save & New somente durante criação.
- [ ] `Ctrl+F` posiciona foco na pesquisa do Logbook.
- [ ] `Escape` continua reservado a cancelar/fechar o fluxo atual.
- [ ] Clipboard não é alterado pelos atalhos.

## Logbook

- [ ] Busca, Clear e Filters permanecem alinhados e utilizáveis.
- [ ] Filtros DMR, FT8, D-STAR e YSF/C4FM abrem sem sobreposição.
- [ ] Indicador de filtro aplicado continua visível sem dominar a tela.
- [ ] Lista de QSO permanece elemento visual dominante.
- [ ] Callsign, UTC, modo, frequência e banda são escaneáveis rapidamente.
- [ ] Route/grid/actions permanecem no nível visual secundário.
- [ ] Conteúdo longo elide sem deslocar ações.
- [ ] Banco vazio e busca sem resultados exibem empty state adequado.
- [ ] Paginação funciona em página vazia, parcial, intermediária e final.
- [ ] Lookup de callsign/grid continua exigindo ação explícita.
- [ ] Confirmação de exclusão continua acessível por mouse e teclado.

## New/Edit QSO

- [ ] Novo e edição continuam distinguíveis pelo contexto da janela.
- [ ] Campos comuns permanecem acessíveis em `1050×680`.
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

- [ ] Área ADIF comporta caminhos longos sem retirar ações da tela.
- [ ] Preview mostra total, novos, duplicados, inválidos, modos, bandas e UTC range.
- [ ] Cancel e Import permanecem utilizáveis na parte inferior do preview.
- [ ] Data health permanece read-only e relatório fica contido.
- [ ] Backup destination, Verify backup e Create backup permanecem acessíveis.

## Settings

- [ ] Identidade da estação continua sendo a configuração principal.
- [ ] Callsign vazio, normal e longo ficam contidos.
- [ ] Templates de callsign/grid suportam valores longos sem clipping destrutivo.
- [ ] Restore defaults e Save links permanecem acessíveis.
- [ ] Aviso de privacidade continua secundário e legível.

## Estados globais

- [ ] Exit confirmation permanece totalmente visível.
- [ ] Falha ao salvar preferências mantém retry e saída explícita sem salvar.
- [ ] Mensagens INFO/DONE/NOTICE/ERROR possuem contraste adequado.
- [ ] Estação não configurada mantém warning perceptível sem bloquear operação.

## Resultado

Registrar aqui data, ambiente, tamanho da janela e observações somente depois da validação manual real.
