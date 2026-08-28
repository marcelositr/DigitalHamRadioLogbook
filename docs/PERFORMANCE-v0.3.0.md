# Performance e stress — baseline v0.3.0, atualização v0.8.0

## Objetivo e método

Estas medições caracterizam a implementação; não são SLA. O benchmark usa build `--release`, banco temporário isolado e dataset determinístico com distribuição entre QSO genérico, DMR e FT8, diferentes callsigns, bandas, grids, redes, repetidoras, talkgroups e SNR.

Comando:

```sh
DHRL_STRESS_QSOS=100000 cargo test --release --locked \
  database::repository::stress::benchmarks_deterministic_large_database \
  -- --ignored --exact --nocapture
```

O teste valida também integridade e quantidade representativa do backup e da exportação ADIF. Ele é ignorado na suíte normal.

## Ambiente medido

- CPU: Intel Core i5-2400, 4 núcleos, 3,10 GHz;
- RAM: 7,7 GiB;
- filesystem: ext4 em disco local;
- Rust: `rustc 1.97.1`;
- perfil: release otimizado;
- sistema sob uso normal; tempos aproximados e sujeitos a cache/pressão de memória.

## Resultados

Tempos em milissegundos.

| Operação | 1k | 10k | 100k | 1M stress |
|---|---:|---:|---:|---:|
| gerar dataset | 12,0 | 148,2 | 1.683,8 | 20.842,9 |
| abrir + validar | 2,5 | 17,6 | 162,2 | 1.561,4 |
| primeira página | 0,6 | 1,0 | 7,2 | 65,8 |
| página intermediária | 1,0 | 5,0 | 49,1 | 602,8 |
| página final | 1,5 | 9,0 | 89,8 | 1.136,1 |
| busca callsign | 0,9 | 6,7 | 73,1 | 732,1 |
| busca modo | 1,0 | 3,0 | 20,9 | 198,7 |
| filtro DMR ID | 0,5 | 2,9 | 29,3 | 365,3 |
| filtro DMR TG | 1,0 | 1,9 | 14,2 | 136,5 |
| filtro DMR rede | 1,0 | 2,9 | 20,9 | 220,0 |
| filtro DMR repetidora | 0,7 | 4,2 | 19,2 | 190,6 |
| filtro DMR timeslot | 1,0 | 2,4 | 20,5 | 208,8 |
| filtro FT8 callsign | 0,6 | 4,4 | 56,7 | 577,2 |
| filtro FT8 grid | 0,7 | 5,6 | 32,8 | 189,0 |
| filtro FT8 banda | 1,2 | 3,0 | 23,9 | 233,7 |
| filtro FT8 período | 0,9 | 2,8 | 23,8 | 248,5 |
| filtro FT8 SNR | 1,1 | 4,1 | 34,9 | 406,9 |
| backup | 6,9 | 38,0 | 331,7 | 3.102,6 |
| montar ADIF | 12,7 | 105,4 | 1.066,9 | 638.791,6 |
| serializar ADIF | 1,6 | 16,1 | 139,0 | 2.929,9 |

A medição de 100k antes da correção era aproximadamente 9.259 ms e caiu para 1.067 ms após carregamento em lote.

## Medição de preparação v0.6.0

Tempos release em milissegundos; estes resultados atualizam os volumes mais relevantes após a inclusão de D-STAR e YSF/C4FM, sem substituir o histórico acima.

| Operação | 10k | 100k |
|---|---:|---:|
| primeira página | 1.031 | 7.601 |
| página intermediária | 6.765 | 73.706 |
| página final | 12.481 | 138.205 |
| busca callsign | 6.720 | 74.263 |
| filtro D-STAR | 2.953 | 12.859 |
| YSF room | 1.720 | 9.052 |
| YSF WIRES-X node | 1.784 | 13.297 |
| YSF DG-ID | 1.676 | 9.356 |
| backup | 39.617 | 358.293 |
| montar ADIF | 114.442 | 1200.912 |
| serializar ADIF | 17.727 | 153.598 |

O dataset atual distribui cinco categorias (`DMR`, `FT8`, genérico, `DSTAR` e `C4FM`). Os números continuam sendo caracterização local, não SLA.

## Saúde e exportação filtrada v0.8.0

Medição release local em 100.000 QSOs, executada em 2026-08-27 no gerador determinístico existente:

- health check read-only: `520.017 ms`;
- exportação filtrada pequena: `51.072 ms`;
- exportação filtrada ampla: `322.552 ms`;
- exportação completa para domínio ADIF: `1164.705 ms`;
- backup validado: `319.754 ms`.

O health check permanece utilizável como ação explícita de Tools e não é executado continuamente. A exportação filtrada pequena evita materializar extras fora da seleção; a seleção ampla permanece significativamente abaixo da exportação completa. Nenhum índice ou migration foi adicionado.

## Regressão de performance v0.9.0

Medição release local executada em 2026-08-28 durante o feature freeze, no mesmo gerador determinístico. Tempos em milissegundos:

| Operação | 10k | 100k |
|---|---:|---:|
| gerar dataset | 157.585 | 1993.582 |
| abrir + validar | 18.352 | 192.596 |
| health check read-only | 51.861 | 540.741 |
| primeira página | 1.424 | 7.977 |
| página intermediária | 6.861 | 75.016 |
| página final | 12.728 | 144.773 |
| busca callsign | 6.458 | 75.576 |
| backup | 38.177 | 331.083 |
| exportação filtrada pequena | 4.884 | 54.129 |
| exportação filtrada ampla | 29.685 | 349.397 |
| montar ADIF completo | 118.913 | 1206.397 |
| serializar ADIF | 18.546 | 164.172 |

Os resultados permanecem compatíveis com as medições históricas de v0.6.0 e v0.8.0. Não foi identificada regressão que justificasse otimização ou índice novo. O stress de 1 milhão não foi repetido neste checkpoint: a execução histórica já caracteriza paginação profunda acima de um segundo e exportação completa de aproximadamente 10,6 minutos com forte pressão de memória neste host de 7,7 GiB.

## Verificação de duplicidade manual v0.7.0

Medição em build release, banco determinístico de 100 mil QSOs e 200 iterações por caso. Tempos médios em milissegundos:

| Caso | Média |
|---|---:|
| hit | 0.029352 |
| miss | 0.028080 |
| self em edição | 0.028480 |
| collision em edição | 0.029927 |

`EXPLAIN QUERY PLAN` usa `idx_qsos_datetime_start`. A medição não justificou migration ou índice adicional; o schema permanece 7.

## SQLite e planos

Índices existentes cobrem ordenação temporal, modo, identificadores DMR, talkgroup, timeslot, SNR e colunas textuais normalizadas. `EXPLAIN QUERY PLAN` confirmou:

- primeira página percorre índice temporal existente;
- busca substring percorre o índice de ordenação, pois `%valor%` não possui prefixo indexável;
- filtro de talkgroup usa `idx_dmr_metadata_talkgroup` e lookup da PK do QSO;
- filtro de SNR usa `idx_ft8_metadata_snr_received` e lookup da PK do QSO;
- filtros especializados usam B-tree temporária para a ordenação temporal após selecionar metadata;
- YSF usa índices somente em `tx_dg_id` e `rx_dg_id`; `EXPLAIN QUERY PLAN` não justificou índices para room ou WIRES-X node porque os filtros são substring (`%valor%`).

Os planos são impressos pelo benchmark para reprodução. Não foi adicionada migration de índice sem benefício demonstrado.

A paginação `OFFSET` degrada linearmente em páginas profundas, mas permaneceu abaixo de ~100 ms em 100 mil QSOs no ambiente medido. Keyset pagination não foi adotada sem necessidade real e exigiria mudança de contrato de navegação.

## Gargalo corrigido

A exportação consultava DMR, FT8 e extras separadamente para cada QSO. A implementação agora carrega QSO+metadados em uma consulta e extras em uma consulta ordenada. Em 100 mil registros, a montagem caiu cerca de 8,7 vezes sem alterar o documento exportado.

## Limites conhecidos

- 100 mil QSOs permaneceram confortáveis para operações normais.
- 1 milhão é cenário extremo: paginação profunda ultrapassou 1 segundo.
- A exportação de 1 milhão constrói documento e texto completos em memória; neste equipamento com 7,7 GiB e swap já pressionada, levou cerca de 10,6 minutos para montar ~382 MB. Uma exportação streaming exigiria mudança de API e fica documentada como possível melhoria futura, não como regressão de uso comum.
