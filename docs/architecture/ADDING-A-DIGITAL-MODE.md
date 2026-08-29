# Adding a digital mode

Este guia registra o caminho concreto usado pelos quatro modos específicos atuais. Ele é um checklist de manutenção, não uma API de plugins.

## 1. Domínio e normalização

- Crie o tipo de metadata, input, enums e validações em `src/domain/`.
- Adicione uma variante a `ModeMetadata` e atualize `is_compatible_with`/`expected_mode`.
- Escolha um único nome interno persistido. Para YSF/System Fusion, o nome é `C4FM`; `YSF` e `SYSTEM FUSION` são apenas aliases da UI.
- Defina limites e normalizações no domínio, não somente no formulário.

A regra central é `mode ↔ metadata`: DMR exige `ModeMetadata::Dmr`, FT8 exige `Ft8`, DSTAR exige `Dstar`, C4FM exige `Ysf` e outros modos exigem `Generic`.

## 2. SQLite

- Adicione uma migration nova; nunca edite migrations publicadas.
- Use tabela de metadata 1:1 com `qso_id` como PK/FK e `ON DELETE CASCADE`.
- Inclua checks que reproduzam invariantes do domínio.
- Adicione índices somente após consultas e `EXPLAIN QUERY PLAN` demonstrarem benefício.
- Atualize validação de objetos e matriz de migrations.

No schema 7, YSF usa `ysf_metadata`. Somente TX/RX DG-ID têm índices; room e WIRES-X node usam substring e permanecem sem índice. `digital_routes` continua sendo uma estrutura exclusiva de DMR.

## 3. Repository e consultas

- Adicione insert/update transacionais, leitura agregada e exclusão de metadata incompatível.
- Faça as APIs agregadas validarem a correspondência entre modo e variante de `ModeMetadata`.
- Materialize metadata na listagem sem N+1.
- Implemente filtros e SQL específicos quando os campos e planos diferirem.

## 4. ADIF

- Defina as formas histórica aceita e canônica exportada.
- Liste campos privados exatos em [`../data/ADIF-EXTENSIONS.md`](../data/ADIF-EXTENSIONS.md).
- Marque os campos conhecidos conforme o modo para que não permaneçam também em extras.
- Preserve campos realmente desconhecidos, inclusive tipo, duplicatas e ordem relativa.
- Reconcile extras quando um campo passa a ser conhecido ou o modo muda, evitando metadata privada duplicada/obsoleta.
- Cubra importação, exportação e round-trip por dois bancos.

Para YSF/C4FM, a forma canônica é `MODE=DIGITALVOICE` + `SUBMODE=C4FM`; `MODE=C4FM` é aceito como histórico.

## 5. UI

- Adicione campos condicionais, preenchimento de edição, limpeza, dirty-state, resumo de rota e filtros.
- Converta aliases para o modo interno antes de salvar.
- Preserve teclado, acessibilidade e layout alvo.

## 6. Validação e documentação

- Teste validações de domínio, rollback, transições entre todos os modos, migrations, filtros, ADIF e UI handlers.
- Execute formatação, check e suíte pertinente.
- Atualize README, changelog, progresso, arquitetura, interoperabilidade e performance quando aplicável.

Traits ou plugins só devem ser considerados se eliminarem duplicação real sem ocultar SQL, tabelas, contratos ADIF ou UI específicos. Quatro modos não justificaram essa abstração; consulte [`decisions/FOUR-MODE-ARCHITECTURE-REVIEW.md`](decisions/FOUR-MODE-ARCHITECTURE-REVIEW.md).
