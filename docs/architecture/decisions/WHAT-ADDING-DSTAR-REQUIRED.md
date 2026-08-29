# What adding D-STAR required

A inclusão de D-STAR na v0.5.0 foi uma extensão explícita das camadas existentes, não a criação de um sistema genérico de modos.

- **Domínio:** `DStarMetadata` para reflector, module, MYCALL, URCALL, RPT1, RPT2 e notes.
- **SQLite:** schema 6 com `dstar_metadata`, cascade por QSO e índices para reflector, module e RPT1.
- **Repository e queries:** CRUD transacional, materialização junto ao QSO e filtros específicos.
- **ADIF:** exportação canônica `MODE=DIGITALVOICE` + `SUBMODE=DSTAR`, importação adicional do histórico `MODE=DSTAR`, extensões `APP_DHRL_DSTAR_*` e `STATION_CALLSIGN` para MYCALL.
- **UI:** campos condicionais, resumo e filtros D-STAR próprios.

Não foram introduzidos traits ou plugins. `digital_routes` permaneceu específico de DMR. A fatoração compartilhada limitou-se à limpeza de metadata incompatível que já existia nas mudanças de modo.

O suporte cobre esse modelo de dados e contrato de intercâmbio; não implica suporte integral ao protocolo D-STAR, a todos os rádios ou a todos os dialetos ADIF.
