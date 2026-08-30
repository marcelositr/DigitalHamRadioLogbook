# Visual QA v0.11

Status: **correção pós-RC1 aprovada para promoção a RC2; matriz visual formal da release estável permanece registrada abaixo**.

A UI v0.11 foi reconstruída a partir dos contratos funcionais, sem reutilizar a implementação visual anterior como referência. A janela de referência continua `1050×680`.

O style oficial do produto é **Fluent**. O QA estável deve validar os três esquemas disponíveis em Settings: **System**, **Light** e **Dark**.

## Evidência pós-RC1

Em 2026-08-29, após o commit `c3ffd3dd49d2dc18ef3b7cf227e77217b47cc7c4` corrigir clipping e o layout dos filtros avançados do Logbook em `1050×680`, o mantenedor executou novamente a aplicação real localmente e informou o teste como aprovado.

Essa aprovação autoriza a promoção do estado corrigido para `v0.11.0-RC2`. Ela não transforma automaticamente todas as caixas abaixo em evidência histórica individual nem substitui a validação do artefato empacotado da RC2.

## Regra de reprovação imediata

Qualquer ocorrência abaixo reprova a tela até correção:

- [ ] texto cortado sem intenção;
- [ ] label ou texto sobreposto;
- [ ] separador/borda atravessando `LineEdit` ou outro controle;
- [ ] botão com texto truncado;
- [ ] controles sobrepostos;
- [ ] conteúdo necessário inacessível por falta de scroll;
- [ ] foco invisível ou navegação por teclado bloqueada;
- [ ] Light ou Dark tornando uma função essencial ilegível.

## Appearance

Executar o build normal:

```bash
cargo run --locked
```

Em **Settings → Appearance**, validar:

- [ ] `System` é o valor default para configuração nova ou configuração antiga sem a seção `appearance`;
- [ ] `System` acompanha a preferência claro/escuro reportada pelo desktop;
- [ ] `Light` muda a interface imediatamente para claro;
- [ ] `Dark` muda a interface imediatamente para escuro;
- [ ] a troca não exige reiniciar a aplicação;
- [ ] a escolha permanece após fechar e abrir novamente;
- [ ] a seleção persiste no `config.toml` sem alterar banco ou QSOs;
- [ ] falha ao salvar a configuração restaura visualmente a preferência persistida anterior;
- [ ] widgets, menus, sidebar, formulários e status permanecem legíveis nos três estados.

## Shell

- [ ] `MenuBar` nativo abre e fecha corretamente.
- [ ] File, QSO, View e Tools permanecem acessíveis.
- [ ] Sidebar expandida não comprime o workspace de forma destrutiva.
- [ ] Sidebar recolhida mantém os quatro destinos identificáveis.
- [ ] Logbook, New QSO, Tools e Settings navegam corretamente.
- [ ] Estação local aparece sem clipping na sidebar.
- [ ] Callsign longo não invade a navegação.
- [ ] Status global permanece visível.
- [ ] `1050×680` não exige fullscreen.

## Navegação e teclado

- [ ] `Tab` percorre controles em ordem compreensível.
- [ ] Enter/Space acionam ações textuais customizadas quando focadas.
- [ ] `Ctrl+N` abre New QSO e posiciona foco em callsign.
- [ ] `Ctrl+S` salva somente no editor.
- [ ] `Ctrl+Enter` executa Save & New apenas durante criação.
- [ ] `Ctrl+F` posiciona foco na busca do Logbook.
- [ ] `Escape` cancela/fecha somente o fluxo atual.
- [ ] Clipboard não é alterado pelos atalhos.

## Logbook

- [ ] Título, contagem, busca e New QSO formam uma hierarquia simples.
- [ ] New QSO é a ação principal sem dominar a página.
- [ ] Busca não corta placeholder ou texto digitado.
- [ ] Filtros DMR, FT8, D-STAR e YSF/C4FM abrem sem sobreposição.
- [ ] Campos dos filtros permanecem inteiros em `1050×680`.
- [ ] Lista parece um workspace de dados, não uma sequência de cards.
- [ ] UTC, callsign, mode, frequency, band, route, grid e actions permanecem alinhados.
- [ ] Rotas longas usam elide sem empurrar Edit/Delete para fora da tela.
- [ ] Callsign/grid lookup permanece explícito.
- [ ] Banco vazio e busca vazia exibem estado simples e centralizado.
- [ ] Paginação funciona em página vazia, parcial, intermediária e final.
- [ ] Confirmação de exclusão não cobre controles essenciais.

## New/Edit QSO

- [ ] New QSO e Edit QSO são distinguíveis.
- [ ] `Contact` aparece primeiro e sem decoração redundante.
- [ ] Callsign, UTC, mode, frequency, band e grid não sofrem clipping.
- [ ] `Station and report` mantém RST, Name e QTH acessíveis.
- [ ] DMR exibe todos os campos e permite rolagem até Notes.
- [ ] FT8 exibe todos os campos e permite rolagem até Notes.
- [ ] D-STAR exibe todos os campos e permite rolagem até Notes.
- [ ] YSF/C4FM exibe todos os campos e permite rolagem até Notes.
- [ ] Modo genérico não reserva espaço vazio para metadata específica.
- [ ] Nenhuma borda de `GroupBox` cruza um `LineEdit`.
- [ ] Labels não entram dentro dos inputs.
- [ ] Rodapé mantém Save & New, Cancel e Save disponíveis.
- [ ] Possible duplicate preserva Review e Save anyway.
- [ ] Discard unsaved changes preserva Continue editing e Discard changes.

## Tools

- [ ] ADIF file path aceita caminho longo sem expulsar ações da tela.
- [ ] Select import/export continuam acessíveis.
- [ ] Preview import mostra total, new, duplicates e invalid sem cards artificiais.
- [ ] Modes, Bands e UTC range quebram linha quando necessário.
- [ ] Detalhes de registros inválidos permanecem legíveis.
- [ ] Cancel e Import permanecem acessíveis no preview.
- [ ] Data health é claramente read-only.
- [ ] Relatório longo permanece contido/rolável pela página.
- [ ] Backup destination aceita caminho longo.
- [ ] Select destination, Create backup e Verify existing backup permanecem acessíveis.

## Settings

- [ ] Appearance é exibido antes das configurações de estação.
- [ ] ComboBox de aparência mostra exatamente System, Light e Dark.
- [ ] Local station permanece claramente identificável.
- [ ] Callsign vazio, normal e longo não quebram o layout.
- [ ] External lookup links permanece secundário.
- [ ] Templates longos de callsign/grid permanecem editáveis.
- [ ] Restore defaults e Save links ficam visíveis.
- [ ] Informação sobre envio para serviços externos é legível sem dominar a tela.

## Estados globais

- [ ] Exit with pending work permanece completamente visível.
- [ ] Erro ao salvar preferências mantém retry e saída explícita.
- [ ] Status normal, success, warning e error são distinguíveis em Light e Dark.
- [ ] Estação não configurada não bloqueia operação.

## Resultado formal para release estável

Registrar somente depois do teste real correspondente:

- data;
- ambiente/desktop;
- resolução da janela;
- esquema usado: System, Light e Dark;
- comportamento do modo System no desktop testado;
- observações por tela;
- falhas encontradas e commits de correção;
- decisão final de aprovação/reprovação.
