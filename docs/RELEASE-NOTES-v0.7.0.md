# Digital Ham Radio Logbook v0.7.0

Status: preparação local; não publicada.

## Destaques

### Save & New

Durante a criação de um QSO, **Save & New** executa uma sequência única e protegida: valida o formulário, faz commit de um QSO, atualiza a listagem, limpa todos os campos comuns e metadados de modo e prepara o próximo formulário com um novo UTC fixo. A ação não cria um segundo QSO e não fica disponível durante edição.

Um guard impede double-submit. O snapshot usado pelo dirty state é atualizado somente depois do commit bem-sucedido, evitando que falhas de validação ou persistência façam o formulário parecer salvo.

### Atalhos e foco

Os atalhos cobertos por testes são:

- `Ctrl+N`: novo QSO, com foco em callsign;
- `Ctrl+S`: salvar;
- `Ctrl+Enter`: **Save & New**, somente em novo QSO;
- `Ctrl+F`: Logbook, com foco na pesquisa;
- `Enter` em Notes: salvar;
- `Escape`: exclusivamente cancelar ou fechar o fluxo atual.

O tratamento preserva o clipboard e não intercepta operações normais de copiar/colar.

### Possíveis duplicidades manuais

Antes do commit, a aplicação verifica a identidade formada por callsign normalizado, UTC inicial, frequência em Hz e modo normalizado. Em edição, o próprio ID é excluído da consulta.

Quando encontra correspondência, a interface oferece **Review** e **Save anyway**. O comportamento é deliberadamente não destrutivo: não faz merge, não bloqueia duplicados intencionais e não adiciona constraint `UNIQUE`.

## Banco e compatibilidade

- schema SQLite permanece na versão 7;
- nenhuma migration foi adicionada;
- nenhum índice foi adicionado;
- nenhuma restrição `UNIQUE` foi adicionada.

Em build release com 100 mil QSOs e 200 iterações por caso, a verificação registrou médias de `0.029352 ms` para hit, `0.028080 ms` para miss, `0.028480 ms` para self em edição e `0.029927 ms` para collision em edição. O plano usa `idx_qsos_datetime_start`; os resultados não justificam outro índice.

## Qualidade

A suíte final atual contém 165 testes ativos e 1 ignored. O checklist visual de `docs/VISUAL-QA.md` para Generic, DMR, FT8, D-STAR, YSF/C4FM, **Save & New**, warning, dirty state, atalhos e `1050×680` está pendente e não aprovado.

## Histórico

A versão anterior `v0.6.0` foi publicada; `main` e a tag apontavam para o commit `034996f`.

Esta preparação não inclui publicação, integração em `main`, criação de tag ou release remota.
