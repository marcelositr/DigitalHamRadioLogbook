# Extensões ADIF do Digital Ham Radio Logbook

Campos `APP_DHRL_*` publicados são contratos de compatibilidade. O importer continuará aceitando estes nomes; o exporter usa somente os nomes canônicos abaixo. Valores são emitidos sem tipo explícito, salvo campos desconhecidos originalmente tipados.

## DMR

| Campo | Significado | Exemplo |
|---|---|---|
| `APP_DHRL_REMOTE_DMR_ID` | DMR ID remoto | `7241234` |
| `APP_DHRL_LOCAL_DMR_ID` | DMR ID local | `7245678` |
| `APP_DHRL_TALKGROUP` | Talkgroup | `724` |
| `APP_DHRL_TIMESLOT` | Timeslot 1 ou 2 | `1` |
| `APP_DHRL_COLOR_CODE` | Color code 0–15 | `1` |
| `APP_DHRL_NETWORK` | Rede informada | `Local` |
| `APP_DHRL_CALL_TYPE` | `group` ou `private` | `group` |
| `APP_DHRL_ACCESS_TYPE` | `simplex`, `repeater` ou `hotspot` | `repeater` |
| `APP_DHRL_REPEATER` | Callsign da repetidora | `PY2RPT` |
| `APP_DHRL_HOTSPOT` | Identificação do hotspot | `MMDVM` |
| `APP_DHRL_DMR_NOTES` | Observação específica DMR | `TG regional` |
| `APP_DHRL_RX_FREQUENCY_HZ` | Frequência RX exata em Hz | `438500000` |
| `APP_DHRL_TX_FREQUENCY_HZ` | Frequência TX exata em Hz | `431500000` |

Aliases históricos/externos aceitos apenas na importação:

- `MY_SIG_INFO` como fallback de talkgroup;
- `SIG` como fallback de rede.

Se alias e campo canônico aparecerem simultaneamente, o registro é recusado para não descartar um valor silenciosamente.

## D-STAR

| Campo | Significado | Exemplo |
|---|---|---|
| `APP_DHRL_DSTAR_REFLECTOR` | Reflector informado | `REF001 C` |
| `APP_DHRL_DSTAR_MODULE` | Module informado | `C` |
| `APP_DHRL_DSTAR_URCALL` | URCALL | `CQCQCQ` |
| `APP_DHRL_DSTAR_RPT1` | RPT1 | `PY2RPT B` |
| `APP_DHRL_DSTAR_RPT2` | RPT2 | `PY2RPT G` |
| `APP_DHRL_DSTAR_NOTES` | Observação específica D-STAR | `Via reflector` |

`STATION_CALLSIGN` é o campo canônico exportado para MYCALL. `APP_DHRL_DSTAR_MYCALL` é aceito somente como alias de importação para compatibilidade. Se os dois campos aparecerem com valores diferentes, o registro é recusado por ambiguidade. A exportação D-STAR usa `MODE=DIGITALVOICE` e `SUBMODE=DSTAR`; o importer também aceita o histórico `MODE=DSTAR`.

## YSF / System Fusion (`C4FM`)

| Campo | Significado | Exemplo |
|---|---|---|
| `APP_DHRL_YSF_ROOM` | Room YSF/WIRES-X | `BRAZIL` |
| `APP_DHRL_YSF_WIRES_X_NODE` | Identificação do nó WIRES-X | `PY2YSF-ND01` |
| `APP_DHRL_YSF_REPEATER` | Repetidora utilizada | `PY2RPT` |
| `APP_DHRL_YSF_NETWORK` | Rede informada | `WIRES-X` |
| `APP_DHRL_YSF_ACCESS_TYPE` | `simplex`, `repeater` ou `hotspot` | `repeater` |
| `APP_DHRL_YSF_TX_DG_ID` | DG-ID de transmissão, `00`–`99` | `01` |
| `APP_DHRL_YSF_RX_DG_ID` | DG-ID de recepção, `00`–`99` | `99` |
| `APP_DHRL_YSF_NOTES` | Observação específica YSF | `Room regional` |

A exportação usa `MODE=DIGITALVOICE` e `SUBMODE=C4FM`; a importação também aceita o histórico `MODE=C4FM`. DG-IDs são exportados com dois dígitos. Estes oito nomes são os campos privados canônicos exatos; não há aliases privados adicionais documentados.

## FT8

| Campo | Significado | Exemplo |
|---|---|---|
| `APP_DHRL_SNR_SENT` | SNR enviado, dB | `-10` |
| `APP_DHRL_AUDIO_FREQUENCY` | Frequência de áudio, Hz | `1500` |
| `APP_DHRL_SOURCE_SOFTWARE` | Software de origem | `WSJT-X` |
| `APP_DHRL_PROTOCOL` | Protocolo informado | `FT8` |
| `APP_DHRL_FINAL_MESSAGE` | Mensagem final | `RR73` |

Campos padrão usados quando disponíveis:

- `SNR` para SNR recebido;
- `TX_PWR` para potência.

`APP_DHRL_SNR_RECEIVED` continua aceito como alias de importação. Se aparecer junto com `SNR`, o registro é recusado por ambiguidade.

## Campos desconhecidos

Campos não reconhecidos são preservados por QSO com:

- nome normalizado para uppercase;
- valor exato;
- tipo ADIF quando presente;
- duplicatas;
- ordem relativa entre os campos desconhecidos.

Na exportação, campos conhecidos são emitidos primeiro em ordem canônica e os desconhecidos são anexados em sua ordem relativa original. A posição original entre campos conhecidos e desconhecidos não é preservada.
