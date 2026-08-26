# Four-mode architecture review

## Escopo

A revisão considera os quatro modos com metadata específica: DMR, FT8, D-STAR e YSF/System Fusion (`C4FM`). O objetivo é verificar se o quarto modo torna necessária uma arquitetura de traits ou plugins.

## Padrões comuns

Todos os modos seguem o mesmo fluxo geral:

1. validação no domínio;
2. metadata 1:1 ligada ao QSO;
3. escrita transacional e remoção de metadata incompatível;
4. materialização junto ao QSO;
5. conversão ADIF e reconciliação de extras;
6. campos, resumo e filtros condicionais na UI.

A fatoração útil foi consolidada em `ModeMetadata` (`Generic`, `Dmr`, `Ft8`, `Dstar`, `Ysf`). O enum torna a integridade `mode ↔ metadata` explícita e permite que repository, ADIF e aplicação transportem um agregado sem combinações arbitrárias.

## Partes deliberadamente específicas

Os detalhes não são uniformes o suficiente para um plugin simples:

- **SQL e tabelas:** cada modo tem colunas, checks, joins e índices próprios;
- **consultas:** DMR, FT8, D-STAR e YSF têm critérios e planos diferentes;
- **ADIF:** campos canônicos, extensões privadas, aliases e conflitos variam por modo;
- **UI:** quantidade, tipo, validação e disposição dos campos e filtros são específicos;
- **rotas:** `digital_routes` pertence somente a DMR; D-STAR e YSF mantêm seus dados nas próprias tabelas.

Essas diferenças continuariam existindo atrás de traits e exigiriam dispatch, downcast ou contratos extensos. Isso deslocaria a complexidade sem remover SQL ou UI específicos.

## Pontos lineares

Adicionar um modo ainda exige tocar em uma lista previsível de locais: domínio/enum, migration, repository CRUD, queries/materialização, ADIF, handlers/modelos de UI, páginas Slint, testes e documentação. Alguns `match` crescem uma variante por modo.

Esse crescimento é linear e explícito. Com quatro modos, os pontos são pequenos, localizáveis e cobertos por tipos/testes; não há carregamento dinâmico, dependências opcionais por modo nem necessidade de extensões de terceiros.

## Decisão

Manter a arquitetura explícita. Não introduzir traits ou plugins neste momento.

A aceitabilidade depende de três guardrails:

- `ModeMetadata` continua sendo a representação consolidada e valida a correspondência com o modo;
- operações de persistência permanecem transacionais e removem metadata incompatível;
- novos índices e abstrações só são adicionados com evidência de consultas/duplicação reais.

Reavaliar apenas se o número de modos tornar os `match` difíceis de manter, se surgir requisito real de plugins externos ou se uma interface comum puder remover implementação — não apenas esconder diferenças.
