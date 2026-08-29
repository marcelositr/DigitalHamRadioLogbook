# Documentação

A documentação do Digital Ham Radio Logbook é organizada por responsabilidade. O `README.md` da raiz continua sendo a porta de entrada do produto; esta página é o índice técnico e operacional do repositório.

## Por onde começar

| Objetivo | Documento |
|---|---|
| Entender a arquitetura atual | [`architecture/ARCHITECTURE.md`](architecture/ARCHITECTURE.md) |
| Entender a reconstrução da UI v0.11 | [`architecture/UI-ARCHITECTURE-v0.11.md`](architecture/UI-ARCHITECTURE-v0.11.md) |
| Adicionar suporte específico a outro modo digital | [`architecture/ADDING-A-DIGITAL-MODE.md`](architecture/ADDING-A-DIGITAL-MODE.md) |
| Consultar o contrato ADIF | [`data/ADIF-INTEROPERABILITY.md`](data/ADIF-INTEROPERABILITY.md) |
| Consultar extensões `APP_DHRL_*` | [`data/ADIF-EXTENSIONS.md`](data/ADIF-EXTENSIONS.md) |
| Fazer backup, diagnóstico ou recuperação | [`data/DATA-RECOVERY.md`](data/DATA-RECOVERY.md) |
| Instalar, empacotar ou distribuir no Linux | [`operations/LINUX-DISTRIBUTION.md`](operations/LINUX-DISTRIBUTION.md) |
| Entender CI, segurança e release automation | [`operations/CI-CD.md`](operations/CI-CD.md) |
| Ver ambientes testados e limites de suporte | [`operations/SUPPORT-MATRIX.md`](operations/SUPPORT-MATRIX.md) |
| Executar o QA visual atual da v0.11 | [`quality/VISUAL-QA-v0.11.md`](quality/VISUAL-QA-v0.11.md) |
| Consultar performance e stress | [`quality/PERFORMANCE-v0.3.0.md`](quality/PERFORMANCE-v0.3.0.md) |
| Preparar uma release reproduzível | [`releases/RELEASE-CHECKLIST.md`](releases/RELEASE-CHECKLIST.md) |
| Consultar o histórico de mudanças | [`releases/CHANGELOG.md`](releases/CHANGELOG.md) |
| Consultar notas de versões | [`releases/notes/`](releases/notes/) |
| Consultar o contrato fundador de escopo | [`project/SPEC.md`](project/SPEC.md) |
| Consultar o histórico de implementação | [`project/PROGRESS.md`](project/PROGRESS.md) |

## Estrutura

### `architecture/`

Documentação viva sobre a organização interna do software e decisões de design que orientam manutenção e evolução.

- `ARCHITECTURE.md`: arquitetura atual do sistema;
- `UI-ARCHITECTURE-v0.11.md`: arquitetura da interface Slint-native;
- `ADDING-A-DIGITAL-MODE.md`: checklist técnico para ampliar os modos suportados;
- `decisions/`: registros de decisões e estudos arquiteturais específicos.

### `data/`

Contratos de dados, interoperabilidade e preservação.

- interoperabilidade ADIF;
- extensões privadas ADIF publicadas;
- integridade, backup e recuperação SQLite.

### `operations/`

Documentação para executar, distribuir e automatizar o ciclo de engenharia do aplicativo.

- `LINUX-DISTRIBUTION.md`: distribuição GNU/Linux;
- `SUPPORT-MATRIX.md`: ambientes testados e limites de suporte;
- `CI-CD.md`: arquitetura de CI, migrations, documentação, segurança, Dependabot e release candidate.

### `quality/`

Checklists, regressões, hardening e evidências de engenharia.

`VISUAL-QA-v0.11.md` é o gate visual corrente da reconstrução v0.11. Os demais documentos preservam baselines e evidências de ciclos anteriores quando indicado pelo nome/versão.

### `releases/`

Histórico e disciplina de publicação.

- `CHANGELOG.md`: histórico consolidado;
- `RELEASE-CHECKLIST.md`: processo reproduzível de release;
- `PRE-1.0-READINESS.md`: registro factual de maturidade;
- `notes/`: notas específicas de versões.

### `project/`

Documentos de governança e memória do projeto.

- `SPEC.md`: especificação/contrato fundador e guardrails de escopo;
- `PROGRESS.md`: diário histórico de marcos e checkpoints de implementação.

## Documentos vivos e históricos

A documentação tem dois papéis distintos:

- **documentos vivos** descrevem o comportamento, arquitetura ou processo vigente e devem acompanhar o código atual;
- **documentos históricos** registram decisões, baselines, regressões e releases no contexto em que foram produzidos.

Release notes, checkpoints de hardening, regressões antigas e o histórico de progresso podem mencionar estados, nomes de interface ou caminhos que eram válidos naquele ciclo. Eles não substituem `architecture/`, `data/`, `operations/` e os gates atuais como referência do comportamento presente.

## Convenções

- novos documentos devem entrar na categoria correspondente, não diretamente na raiz de `docs/`;
- documentação de versão deve carregar a versão no nome quando for um snapshot histórico;
- mudanças de arquitetura vigente devem atualizar `architecture/`;
- mudanças de contratos de dados devem atualizar `data/`;
- mudanças de distribuição/suporte/automação devem atualizar `operations/`;
- evidências de testes e regressões devem ficar em `quality/`;
- notas e processos de publicação devem ficar em `releases/`;
- decisões de projeto que precisam ser preservadas, mas não são documentação operacional corrente, devem ficar em `project/` ou `architecture/decisions/`.
