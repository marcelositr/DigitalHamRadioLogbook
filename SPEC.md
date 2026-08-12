# ESPECIFICAÇÃO E PROMPT DE IMPLEMENTAÇÃO

## Digital Ham Radio Logbook

Você está atuando como agente de engenharia de software dentro de um ambiente GNU/Linux.

Sua missão é projetar e implementar um aplicativo desktop local para radioamadores, especializado no registro de contatos realizados por modos digitais.

O projeto deve priorizar simplicidade, robustez, portabilidade, privacidade, interoperabilidade e manutenção futura.

Este documento é simultaneamente:

1. especificação funcional;
2. especificação técnica;
3. definição arquitetural;
4. contrato de escopo;
5. conjunto de restrições;
6. guia de implementação;
7. critérios de aceite;
8. mecanismo de controle para impedir expansão indevida de escopo.

Leia todo o documento antes de modificar qualquer arquivo.

---

# 1. OBJETIVO DO PROJETO

Criar um logbook desktop dedicado principalmente a modos digitais utilizados no radioamadorismo.

O software deverá registrar não apenas o QSO tradicional entre duas estações, mas também o contexto técnico e a infraestrutura digital utilizada naquele contato.

Um contato digital pode envolver:

* estação local;
* estação remota;
* frequência;
* modo;
* submodo;
* repetidora;
* hotspot;
* talkgroup;
* reflector;
* room;
* timeslot;
* color code;
* rede;
* gateway;
* grid locator;
* rota lógica;
* protocolo;
* data e horário.

O programa deve tratar esses elementos como informações relevantes do contato.

Exemplo conceitual:

PU2AAA
→ DMR ID 7240001
→ TG 724
→ TS1
→ Repetidora PY2XYZ
→ BrandMeister
→ PU2BBB

Não é necessário reproduzir exatamente essa representação visual.

O objetivo é preservar essa informação estrutural no banco de dados.

---

# 2. FILOSOFIA DO SOFTWARE

O aplicativo deverá seguir estes princípios:

* local-first;
* offline-first;
* GNU/Linux first;
* nenhuma conta obrigatória;
* nenhuma nuvem obrigatória;
* nenhum servidor obrigatório;
* nenhum daemon obrigatório;
* banco local;
* arquivos fáceis de copiar e fazer backup;
* comportamento previsível;
* interface simples;
* baixa utilização de recursos;
* arquitetura compreensível;
* interoperabilidade através de formatos conhecidos;
* dependências externas mínimas.

O programa não deverá tentar ser uma plataforma social.

Também não deverá tentar substituir todos os softwares existentes no ecossistema de radioamadorismo.

A proposta é deliberadamente especializada.

---

# 3. STACK PRINCIPAL

Utilizar preferencialmente:

* Rust
* Slint
* SQLite
* rusqlite
* Serde

Dependências adicionais devem ser introduzidas somente quando houver necessidade concreta.

---

# 4. JUSTIFICATIVA DA STACK

## Rust

Vantagens:

* segurança de memória;
* excelente desempenho;
* bom suporte a aplicações desktop;
* bom suporte a serialização;
* bom suporte a networking;
* bom suporte a comunicação serial;
* binários independentes;
* tipagem forte;
* facilidade para modelar protocolos e estruturas diferentes.

Desvantagens:

* curva de aprendizado;
* tempos de compilação;
* maior complexidade inicial comparado a Python;
* desenvolvimento de GUI pode exigir mais trabalho.

Mesmo assim, Rust deverá ser considerado a linguagem principal do projeto.

---

## Slint

Vantagens:

* toolkit relativamente leve;
* boa integração com Rust;
* adequado para aplicações desktop;
* separação razoável entre interface e lógica;
* baixo consumo comparado a aplicações Electron.

Desvantagens:

* ecossistema menor que Qt ou GTK;
* menos componentes prontos;
* documentação e exemplos podem ser mais limitados.

O projeto deverá aceitar essas limitações em troca de uma aplicação enxuta.

Não trocar Slint por Electron, Tauri, Qt, GTK ou framework web sem autorização explícita.

---

## SQLite

Vantagens:

* banco embutido;
* extremamente estável;
* sem servidor;
* fácil backup;
* fácil distribuição;
* SQL padrão;
* excelente para aplicações desktop.

Desvantagens:

* não destinado a grandes sistemas distribuídos;
* concorrência limitada para escrita simultânea.

Essas limitações não são relevantes para o caso de uso atual.

SQLite deverá ser o banco principal.

---

## rusqlite

Preferir rusqlite inicialmente.

Motivos:

* API direta;
* pouca complexidade;
* não exige runtime async;
* adequado para aplicação desktop local.

Não introduzir ORM inicialmente.

Não utilizar Diesel, SeaORM ou framework semelhante apenas por conveniência arquitetural.

ORM poderá ser reconsiderado futuramente somente se existir problema concreto.

---

## Serde

Utilizar para:

* configuração;
* JSON;
* importações;
* exportações;
* integração futura com APIs.

---

# 5. ASYNC E TOKIO

Não incluir Tokio automaticamente.

Adicionar async runtime somente quando existir uma necessidade concreta.

Exemplos que podem justificar:

* múltiplas conexões simultâneas;
* comunicação contínua com rádio;
* APIs externas;
* sockets;
* serviços concorrentes.

Se o MVP funcionar corretamente de forma síncrona, permanecer síncrono.

Regra:

NÃO adicionar Tokio simplesmente porque é comum no ecossistema Rust.

---

# 6. MODOS DIGITAIS INICIALMENTE CONSIDERADOS

A arquitetura deverá permitir os seguintes modos:

* DMR
* D-STAR
* YSF / C4FM
* M17
* NXDN
* P25
* FT8
* FT4
* JS8Call

Outros modos poderão ser adicionados futuramente.

IMPORTANTE:

não é obrigatório implementar suporte completo a todos no primeiro MVP.

O projeto deverá permitir expansão futura sem obrigar que todos sejam implementados imediatamente.

---

# 7. PRIMEIRO MVP

O primeiro MVP deverá priorizar:

1. DMR;
2. FT8;
3. registro manual genérico.

Isso oferece dois modelos muito diferentes de comunicação digital e ajuda a validar a arquitetura.

DMR representa modos digitais baseados em infraestrutura de voz.

FT8 representa modos digitais orientados a comunicação de dados e sinal fraco.

---

# 8. MODELO DE DADOS PRINCIPAL

O modelo não deverá tentar colocar todos os modos digitais em uma única tabela gigantesca com dezenas de campos opcionais.

Evitar estruturas como:

qsos(
callsign,
frequency,
dmr_id,
talkgroup,
timeslot,
color_code,
reflector,
room,
module,
snr,
grid,
...
)

Isso produz acoplamento ruim.

Preferir separação entre dados comuns e dados específicos de cada modo.

---

# 9. ENTIDADE QSO

Cada contato deverá ter uma entidade principal QSO.

Campos mínimos sugeridos:

* id;
* callsign;
* datetime_start;
* datetime_end opcional;
* frequency;
* band;
* mode;
* submode opcional;
* rst_sent;
* rst_received;
* grid_locator opcional;
* name opcional;
* qth opcional;
* notes;
* created_at;
* updated_at.

Data e hora deverão ser armazenadas internamente em UTC.

A interface poderá apresentar horário local.

---

# 10. METADADOS ESPECÍFICOS DE MODO

Criar estruturas específicas.

Exemplo conceitual:

QSO
|
+-- DmrMetadata
|
+-- Ft8Metadata
|
+-- DStarMetadata
|
+-- YsfMetadata

Não precisa utilizar herança.

Rust enums, structs ou composição devem ser avaliados.

---

# 11. DMR

Campos relevantes:

* DMR ID da estação remota;
* DMR ID local;
* talkgroup;
* timeslot;
* color code;
* rede;
* repetidora;
* callsign da repetidora;
* hotspot;
* tipo de chamada;
* group call;
* private call;
* simplex;
* duplex;
* frequência RX;
* frequência TX;
* observações.

Rede poderá conter exemplos como:

* BrandMeister;
* TGIF;
* DMR+;
* local;
* desconhecida.

Não hardcodar uma lista fechada.

---

# 12. D-STAR

Campos possíveis:

* reflector;
* módulo;
* gateway;
* repetidora;
* RPT1;
* RPT2;
* MYCALL;
* URCALL.

Não implementar obrigatoriamente no MVP.

Apenas garantir que a arquitetura futura suporte dados dessa natureza.

---

# 13. YSF / C4FM

Campos possíveis:

* room;
* Wires-X node;
* gateway;
* repetidora;
* DG-ID.

Novamente:

não implementar antes de terminar o núcleo do MVP.

---

# 14. M17

Preparar possibilidade de armazenar:

* reflector;
* module;
* CAN;
* gateway;
* repeater;
* destination.

---

# 15. FT8 E FT4

Metadados possíveis:

* grid;
* SNR enviado;
* SNR recebido;
* potência;
* frequência;
* frequência de áudio opcional;
* software de origem;
* protocolo;
* mensagem final;
* estação trabalhada;
* distância calculada opcional.

Não tentar reimplementar WSJT-X.

O aplicativo é um logger.

---

# 16. ROTA DIGITAL

Criar conceito separado de rota ou infraestrutura do contato.

Nome interno sugerido:

DigitalRoute

ou:

NetworkPath

A finalidade é registrar COMO o QSO ocorreu.

Exemplo:

Local Station
→ Repeater
→ Network
→ Talkgroup
→ Remote Station

Outro exemplo:

Local Station
→ Hotspot
→ BrandMeister
→ TG 724
→ Remote Station

Essa rota poderá inicialmente ser apenas estruturada em campos.

Não é necessário criar visualização gráfica no MVP.

---

# 17. PRINCÍPIO FUNDAMENTAL

Separar:

WHO

de:

HOW

WHO:

quem foi contactado.

HOW:

como a comunicação digital chegou até ele.

Isso é um dos diferenciais centrais do projeto.

---

# 18. ADIF

Suporte a ADIF é obrigatório.

O software deverá eventualmente permitir:

* importar ADIF;
* exportar ADIF;
* preservar campos conhecidos;
* preservar, quando possível, campos desconhecidos.

O parser deve ser isolado da interface.

Criar módulo específico.

Exemplo:

src/adif/

Nunca misturar parser ADIF com código GUI.

---

# 19. EXTENSÕES ADIF

Como alguns dados específicos de modos digitais podem não possuir correspondência perfeita no ADIF padrão, avaliar:

1. campos ADIF existentes;
2. campos APP_* específicos do aplicativo.

Nunca sobrescrever informação silenciosamente.

Se determinada informação não puder ser exportada diretamente, ela deverá:

* ser preservada internamente;
* ou ser representada através de campo privado documentado.

---

# 20. ARQUITETURA DO PROJETO

Estrutura sugerida:

src/
main.rs

```
app/
    mod.rs

domain/
    mod.rs
    qso.rs
    mode.rs
    route.rs

modes/
    mod.rs
    dmr.rs
    ft8.rs
    dstar.rs
    ysf.rs
    m17.rs

database/
    mod.rs
    schema.rs
    migrations.rs
    repository.rs

adif/
    mod.rs
    parser.rs
    exporter.rs

radio/
    mod.rs
    backend.rs

config/
    mod.rs

ui/
    mod.rs
```

ui/
main.slint

tests/

Não seguir essa estrutura cegamente caso Rust indique solução melhor.

Porém qualquer mudança arquitetural relevante deve ser justificada.

---

# 21. CAMADAS

Separar claramente:

UI

Domain

Persistence

Integration

A interface não deverá executar SQL diretamente.

A camada de banco não deverá depender de Slint.

O parser ADIF não deverá depender da interface.

Os módulos de modo digital não deverão depender diretamente do banco.

---

# 22. RADIO BACKEND

Preparar arquitetura para futura integração com rádios.

Interface conceitual:

RadioBackend

Possíveis implementações futuras:

HamlibBackend

SerialBackend

TcpBackend

ManualBackend

Nenhuma integração direta com hardware é necessária no primeiro MVP.

O objetivo inicialmente é apenas garantir que a arquitetura não impeça isso.

---

# 23. HAMLIB

Hamlib deverá ser considerada a integração preferencial futura.

Vantagens:

* grande suporte a equipamentos;
* abstração de fabricantes;
* projeto consolidado no radioamadorismo.

Desvantagens:

* dependência externa;
* diferenças entre equipamentos;
* funcionalidades variáveis por modelo.

Não tornar Hamlib dependência obrigatória do aplicativo inicialmente.

Ela deverá ser opcional.

---

# 24. CONFIGURAÇÕES

Preferir configuração local.

Exemplo:

~/.config/digital-ham-log/config.toml

Dados:

~/.local/share/digital-ham-log/

Banco:

~/.local/share/digital-ham-log/logbook.sqlite3

Seguir XDG Base Directory Specification no Linux.

Não inventar diretórios dentro da HOME fora do padrão XDG sem motivo.

---

# 25. BACKUP

Backup deve ser simples.

Um usuário deve conseguir copiar:

logbook.sqlite3

e possuir praticamente todo o seu log.

Caso existam anexos no futuro:

data/
logbook.sqlite3
attachments/

Nenhum mecanismo proprietário de backup deve ser obrigatório.

---

# 26. MIGRATIONS

O banco deverá possuir migrations.

Cada alteração estrutural futura deverá ser versionada.

Não modificar schema silenciosamente.

Criar versão de schema.

---

# 27. INTERFACE

A interface deve ser desktop tradicional.

Prioridades:

* teclado;
* rapidez;
* navegação simples;
* tabela clara;
* busca rápida.

Não criar interface mobile.

Não criar UI parecida com rede social.

Não criar dashboard exagerado.

---

# 28. TELAS INICIAIS

MVP:

## Tela principal

Tabela de QSOs.

Colunas sugeridas:

* data;
* hora;
* callsign;
* modo;
* frequência;
* banda;
* rota/resumo;
* grid.

## Novo QSO

Formulário.

Campos comuns primeiro.

Campos específicos aparecem conforme modo escolhido.

## Detalhes do QSO

Mostrar:

* informações gerais;
* dados específicos do modo;
* infraestrutura;
* observações.

## Busca

Busca por:

* callsign;
* DMR ID;
* TG;
* modo;
* período;
* banda.

---

# 29. EXPERIÊNCIA DE TECLADO

O aplicativo deverá ser confortável sem mouse.

Considerar atalhos para:

* novo QSO;
* salvar;
* cancelar;
* buscar;
* editar;
* excluir;
* navegar pela tabela.

Não definir dezenas de atalhos prematuramente.

---

# 30. EXCLUSÃO

Nenhum registro deve ser apagado acidentalmente.

Excluir QSO deverá exigir confirmação.

Não implementar lixeira complexa inicialmente.

---

# 31. EDIÇÃO

Toda edição deve atualizar:

updated_at

Opcionalmente preparar arquitetura futura para histórico de alterações.

Não implementar auditoria completa no MVP.

---

# 32. BUSCA

SQLite deverá realizar a busca.

Não criar Elasticsearch.

Não criar mecanismo externo de indexação.

Não criar servidor de busca.

SQLite é suficiente.

---

# 33. ESTATÍSTICAS FUTURAS

Possibilidades:

* QSOs por modo;
* QSOs por TG;
* QSOs por banda;
* contatos por país;
* contatos por estado;
* contatos por grid;
* repetidoras utilizadas;
* hotspots utilizados;
* redes utilizadas;
* contatos por mês;
* contatos por ano.

Não implementar dashboard completo antes do CRUD básico estar sólido.

---

# 34. GEOLOCALIZAÇÃO

Grid Locator poderá ser armazenado.

Conversão Maidenhead → latitude/longitude poderá ser adicionada futuramente.

Não depender de mapas online no MVP.

---

# 35. CALLSIGN DATABASE

Futuras integrações podem incluir:

* QRZ;
* HamQTH;
* RadioID;
* bancos públicos.

Essas integrações NÃO fazem parte do primeiro MVP.

Qualquer API deverá ser opcional.

Nenhuma funcionalidade básica poderá depender delas.

---

# 36. PRIVACIDADE

O programa deverá funcionar completamente sem enviar dados para a Internet.

Nenhuma telemetria.

Nenhum analytics.

Nenhum crash report automático.

Nenhum tracking.

Qualquer integração futura deverá ser explicitamente habilitada pelo usuário.

---

# 37. SEGURANÇA

Não armazenar credenciais em texto puro quando integrações externas forem adicionadas.

Preferir:

* keyring do sistema;
* secret service;
* variável de ambiente;
* arquivo protegido quando estritamente necessário.

Isso não precisa ser implementado no primeiro MVP.

---

# 38. LICENÇA

Preparar projeto para licença open source.

Sugestão inicial:

GPL-3.0-or-later

ou:

AGPL-3.0-or-later

ou:

MPL-2.0.

NÃO escolher automaticamente.

Deixar licença pendente caso não tenha sido explicitamente definida pelo mantenedor.

---

# 39. PORTABILIDADE

Prioridade:

1. GNU/Linux.

Posteriormente:

2. Windows;
3. macOS.

Não sacrificar arquitetura Linux para obter portabilidade prematura.

Ao mesmo tempo, evitar chamadas Linux específicas na camada de domínio.

---

# 40. EMPACOTAMENTO

Futuramente considerar:

* binário;
* pacote .deb;
* RPM;
* Flatpak;
* AppImage.

Não implementar todos no MVP.

Primeiro garantir:

cargo build

e:

cargo run

---

# 41. TESTES

Criar testes principalmente para:

* parser ADIF;
* exporter ADIF;
* validação de callsign;
* conversão de dados;
* database repository;
* migrations;
* serialização;
* regras de domínio.

Não criar testes inúteis apenas para aumentar percentual de cobertura.

Testar comportamento relevante.

---

# 42. LOGGING

Utilizar logging interno simples.

Possíveis crates:

log

ou:

tracing

Escolher somente uma.

Não criar infraestrutura complexa de observabilidade.

---

# 43. ERROS

Não utilizar panic! para erros recuperáveis.

Utilizar Result.

Erros apresentados ao usuário devem ser compreensíveis.

Erros técnicos podem ser registrados no log.

---

# 44. VALIDADORES

Validações devem existir para:

* callsign vazio;
* frequência inválida;
* data inválida;
* DMR ID inválido;
* timeslot inválido;
* color code inválido;
* grid locator inválido quando preenchido.

Não exagerar em validações que impeçam registros legítimos.

Radioamadorismo possui muitos casos especiais.

Quando houver dúvida, preferir permitir registro manual.

---

# 45. DMR ID

DMR ID deverá ser armazenado numericamente ou em representação validada.

Não consultar banco externo automaticamente.

---

# 46. TALKGROUP

TG deverá aceitar números válidos sem depender de lista fixa.

Não limitar aos TGs atualmente conhecidos.

---

# 47. REDES DIGITAIS

Não hardcodar arquitetura em torno de BrandMeister.

BrandMeister é uma rede importante, mas não única.

Representar rede através de entidade ou campo genérico.

---

# 48. HOTSPOT E REPETIDORA

Não tratar hotspot e repetidora como a mesma entidade.

Ambos são pontos de acesso RF, mas possuem características diferentes.

A estrutura deverá permitir diferenciação.

---

# 49. MODO MANUAL

Sempre existir modo de registro manual.

Se uma integração falhar, o usuário deverá continuar podendo registrar o QSO.

Integrações são conveniência.

Nunca requisito.

---

# 50. FUNCIONAMENTO SEM INTERNET

Teste obrigatório:

desconectar a máquina da Internet.

O programa deverá:

* iniciar;
* listar QSOs;
* adicionar QSO;
* editar QSO;
* pesquisar;
* importar ADIF;
* exportar ADIF.

---

# 51. O QUE NÃO FAZER

Este bloco é obrigatório.

NÃO criar:

* servidor web;
* backend HTTP;
* API REST;
* GraphQL;
* Kubernetes;
* Docker como requisito;
* microsserviços;
* Redis;
* PostgreSQL;
* MongoDB;
* Elasticsearch;
* Kafka;
* RabbitMQ;
* blockchain;
* autenticação;
* usuário/senha local;
* sistema social;
* chat;
* feed;
* ranking;
* gamificação;
* NFTs;
* cloud sync;
* SaaS;
* aplicação Electron;
* frontend React;
* frontend Vue;
* frontend Angular.

A menos que o mantenedor solicite explicitamente no futuro.

---

# 52. NÃO INVENTAR FUNCIONALIDADES

Se durante o desenvolvimento surgir uma ideia que não esteja especificada:

NÃO implementar automaticamente.

Registrar como:

Future Idea

ou:

Possible Enhancement

e continuar a tarefa atual.

---

# 53. REGRA DE CABRESTO DE ESCOPO

Antes de implementar qualquer funcionalidade, verificar:

1. Ela está diretamente relacionada à tarefa atual?
2. É necessária para concluir o requisito?
3. Existe alternativa mais simples?
4. Ela introduz dependência nova?
5. Ela aumenta significativamente a arquitetura?

Se a resposta indicar expansão de escopo, NÃO implementar.

---

# 54. REGRA PARA NOVAS DEPENDÊNCIAS

Antes de adicionar crate:

* justificar necessidade;
* verificar manutenção;
* verificar licença;
* verificar se std já resolve;
* verificar se dependência existente resolve.

Evitar adicionar dependência para funções triviais.

---

# 55. REGRA PARA REFACTORING

Não realizar refatorações gigantescas não relacionadas à tarefa atual.

Refatoração deve resolver problema concreto.

Se uma melhoria arquitetural puder esperar:

documentar e continuar.

---

# 56. REGRA PARA ABSTRAÇÕES

Não criar abstração antecipadamente.

Regra prática:

uma implementação não precisa automaticamente de trait.

Criar abstrações quando houver:

* múltiplas implementações reais;
* necessidade clara de testes;
* fronteira arquitetural importante.

---

# 57. REGRA CONTRA OVERENGINEERING

Escolher sempre a solução mais simples que:

* funcione;
* seja testável;
* preserve extensão futura razoável.

Não construir arquitetura para milhões de usuários.

Este é um aplicativo desktop individual.

---

# 58. REGRA SOBRE PERFORMANCE

Não otimizar prematuramente.

SQLite deverá suportar tranquilamente centenas de milhares de QSOs.

Primeiro garantir corretude.

Depois medir.

Depois otimizar.

---

# 59. DOCUMENTAÇÃO

Criar README.md contendo:

* objetivo;
* estado atual;
* requisitos;
* compilação;
* execução;
* localização dos dados;
* backup;
* arquitetura resumida;
* limitações conhecidas.

---

# 60. DOCUMENTAÇÃO INTERNA

Código deve ser autoexplicativo.

Comentários devem explicar:

POR QUE

e não repetir:

O QUE.

Evitar comentários óbvios.

---

# 61. NOMENCLATURA

Utilizar inglês no código.

Interface poderá posteriormente possuir tradução.

Exemplos:

Qso

DigitalMode

DigitalRoute

DmrMetadata

RadioBackend

Database

Repository

Evitar mistura de português e inglês em identificadores.

---

# 62. INTERNACIONALIZAÇÃO

Não implementar framework complexo de i18n inicialmente.

Porém evitar textos profundamente acoplados à lógica.

A arquitetura deverá permitir tradução futura.

---

# 63. FORMATO DE DATA

Banco:

UTC.

Interface:

configurável ou horário local.

Nunca armazenar apenas strings formatadas regionalmente como fonte principal da data.

---

# 64. FREQUÊNCIA

Internamente preferir representação precisa.

Avaliar:

Hz como inteiro.

Exemplo:

145500000

em vez de:

145.5

Isso evita erros de ponto flutuante.

---

# 65. BANDA

Band poderá ser derivada da frequência sempre que possível.

Não depender exclusivamente de texto inserido manualmente.

Porém permitir override quando necessário.

---

# 66. IDENTIFICADORES

Usar chave primária interna independente de callsign.

Callsign não é identificador único de QSO.

---

# 67. DUPLICATAS

Não bloquear automaticamente QSOs aparentemente duplicados.

Pode existir contato repetido com mesma estação.

Detecção futura de possíveis duplicatas poderá apenas alertar.

---

# 68. IMPORTAÇÃO ADIF

Importação deverá:

1. abrir arquivo;
2. validar;
3. analisar registros;
4. apresentar resumo;
5. inserir registros válidos;
6. reportar erros.

Não destruir banco existente em caso de erro parcial.

Idealmente utilizar transaction.

---

# 69. EXPORTAÇÃO ADIF

Exportação deverá ser determinística.

Não perder informações essenciais.

Criar testes round-trip quando possível:

ADIF
→ parser
→ estrutura
→ exporter
→ ADIF.

---

# 70. BANCO

Ativar foreign keys.

Utilizar transactions onde fizer sentido.

Criar índices somente para consultas reais.

Possíveis índices iniciais:

* callsign;
* datetime;
* mode;
* DMR ID;
* talkgroup.

Não indexar tudo.

---

# 71. SCHEMA INICIAL CONCEITUAL

Possíveis tabelas:

qsos

dmr_metadata

ft8_metadata

digital_routes

stations

schema_migrations

Não criar dezenas de tabelas antes de necessidade.

---

# 72. ESTAÇÃO LOCAL

Preparar conceito de estação local.

Possíveis dados:

* callsign;
* DMR ID;
* grid;
* QTH;
* equipamento;
* antena.

No MVP somente callsign poderá ser obrigatório.

Isso permite uso futuro de múltiplos perfis de estação.

---

# 73. EQUIPAMENTO

Não criar inventário completo de equipamentos inicialmente.

No máximo preparar campos opcionais.

---

# 74. EXPORTAÇÃO

Além de ADIF, CSV poderá ser futuramente útil.

Não implementar antes do ADIF.

---

# 75. CLI

O aplicativo principal será gráfico.

Uma CLI poderá existir futuramente para:

* import;
* export;
* backup;
* diagnostics.

Não é prioridade do primeiro MVP.

---

# 76. ROADMAP RECOMENDADO

## Fase 0

Inicialização:

* Cargo project;
* Slint;
* SQLite;
* estrutura básica;
* README;
* migrations.

## Fase 1

Modelo de domínio:

* QSO;
* modes;
* validação;
* repository;
* CRUD SQLite.

## Fase 2

GUI básica:

* tabela;
* novo QSO;
* editar;
* excluir;
* pesquisa.

## Fase 3

DMR:

* metadata;
* campos DMR;
* filtros;
* exibição.

## Fase 4

FT8:

* metadata;
* campos;
* filtros.

## Fase 5

ADIF:

* parser;
* import;
* export;
* testes.

## Fase 6

Polimento:

* configuração;
* atalhos;
* backup;
* mensagens de erro;
* documentação.

## Fase 7

Integrações futuras.

---

# 77. CRITÉRIO PARA PASSAR DE FASE

Não avançar porque "parece pronto".

Cada fase deverá:

* compilar;
* executar;
* possuir funcionalidade mínima testável;
* não quebrar fase anterior;
* ter documentação atualizada quando necessário.

---

# 78. DEFINIÇÃO DO MVP CONCLUÍDO

O MVP estará concluído quando o usuário puder:

1. iniciar o programa;
2. configurar callsign local;
3. adicionar QSO;
4. registrar QSO DMR;
5. registrar QSO FT8;
6. editar;
7. excluir;
8. pesquisar;
9. filtrar;
10. importar ADIF;
11. exportar ADIF;
12. fechar;
13. abrir novamente;
14. encontrar os dados intactos.

Tudo funcionando offline.

---

# 79. NÃO CONFUNDIR MVP COM PROTÓTIPO DESCARTÁVEL

O código do MVP deverá possuir qualidade suficiente para continuar evoluindo.

Porém isso não significa criar arquitetura corporativa.

O equilíbrio esperado é:

simples

*

limpo

*

extensível

*

testável.

---

# 80. POLÍTICA DE DECISÃO AUTÔNOMA DO AGENTE

Você possui autonomia para tomar decisões pequenas.

Exemplos:

* nome de função;
* organização local de módulo;
* detalhes de layout;
* escolha entre estruturas equivalentes.

Você NÃO possui autonomia para:

* trocar stack;
* adicionar serviço;
* criar nuvem;
* alterar objetivo;
* adicionar grandes funcionalidades;
* substituir banco;
* mudar toolkit GUI;
* adicionar sistema de plugins dinâmicos;
* adicionar dependências pesadas.

---

# 81. QUANDO ENCONTRAR AMBIGUIDADE

Primeiro procure solução conservadora.

Pergunte:

"Qual é a menor implementação que atende ao requisito?"

Utilize essa solução.

Não preencher lacunas com arquitetura imaginária.

---

# 82. PLUGINS

Não implementar sistema de plugins dinâmicos no MVP.

A arquitetura modular dos modos digitais deverá inicialmente ser feita através do próprio código Rust.

Exemplo:

modes::dmr

modes::ft8

modes::dstar

Isso já é suficiente.

Plugins carregáveis poderão ser avaliados muito futuramente.

---

# 83. THREADS

Não adicionar threads sem necessidade.

Operações rápidas podem ocorrer normalmente.

Operações potencialmente demoradas deverão futuramente sair da thread da UI.

Exemplo:

importação de arquivo ADIF gigantesco.

---

# 84. FEEDBACK DE OPERAÇÕES LONGAS

Quando necessário:

* progress indicator;
* status;
* resultado.

Não congelar silenciosamente interface.

---

# 85. RECUPERAÇÃO DE ERRO

Erro de importação não deve corromper log.

Erro de configuração não deve apagar configuração anterior.

Erro de banco deve produzir diagnóstico compreensível.

---

# 86. INTEGRIDADE DO BANCO

Utilizar atomicidade.

Evitar sequência:

INSERT QSO

falha

INSERT metadata.

Utilizar transaction para gravações relacionadas.

---

# 87. FOREIGN KEYS

Metadados específicos deverão referenciar QSO.

Quando QSO for removido:

avaliar ON DELETE CASCADE para metadados diretamente pertencentes àquele contato.

Documentar decisão.

---

# 88. CONFIGURAÇÕES DO USUÁRIO

Exemplos futuros:

* callsign;
* timezone;
* formato de hora;
* unidade;
* diretório de exportação;
* estação padrão.

Não transformar configuração em árvore gigantesca.

---

# 89. OBSERVABILIDADE

Logs técnicos poderão registrar:

* startup;
* versão;
* database migration;
* import/export;
* erros relevantes.

Nunca registrar:

* senhas;
* tokens;
* credenciais.

---

# 90. COMPATIBILIDADE FUTURA

Ao evoluir banco ou formatos:

dados antigos devem continuar legíveis através de migrations.

Nunca assumir que usuário pode simplesmente apagar banco e começar novamente.

Log de radioamador pode representar anos de registros.

Tratá-lo como informação valiosa.

---

# 91. DADOS SÃO MAIS IMPORTANTES QUE UI

Prioridade:

1. integridade dos dados;
2. exportação;
3. compatibilidade;
4. usabilidade;
5. estética.

Uma interface bonita não compensa um log corrompido.

---

# 92. ESTILO DE INTERFACE

Preferir:

* simples;
* funcional;
* compacto;
* informativo.

Evitar:

* animações desnecessárias;
* cards gigantes;
* espaços desperdiçados;
* interfaces mobile transplantadas para desktop.

É uma ferramenta.

---

# 93. DENSIDADE DE INFORMAÇÃO

Um radioamador pode possuir milhares de QSOs.

A tabela deve permitir visualizar quantidade razoável de registros simultaneamente.

Não transformar cada QSO em um cartão enorme.

---

# 94. FILTROS DMR

Planejar suporte a:

* TG;
* DMR ID;
* network;
* repeater;
* hotspot;
* timeslot.

---

# 95. FILTROS FT8

Planejar:

* grid;
* band;
* SNR;
* callsign;
* período.

---

# 96. FUTURO: INTEGRAÇÃO WSJT-X

Pode futuramente utilizar protocolo UDP do WSJT-X para capturar QSOs.

NÃO implementar durante MVP sem solicitação.

A arquitetura FT8 apenas não deve impedir isso.

---

# 97. FUTURO: BRANDMEISTER

Pode futuramente consultar APIs BrandMeister.

Usos possíveis:

* dados de repetidora;
* dados de hotspot;
* TG;
* DMR ID.

Essa integração deve ser opcional.

Nunca depender dela para abrir ou consultar o log.

---

# 98. FUTURO: RADIOID

Pode futuramente enriquecer informações DMR.

Mesmas restrições:

opcional

e

não essencial.

---

# 99. FUTURO: CAT

Pode capturar automaticamente:

* frequência;
* modo;
* rádio.

Não implementar ainda.

---

# 100. FUTURO: QSOs AUTOMÁTICOS

Mesmo com automação futura, usuário deve poder:

* revisar;
* corrigir;
* excluir;
* inserir manualmente.

Automação nunca será fonte absoluta da verdade.

---

# 101. DOCUMENTAÇÃO DE DECISÕES

Para decisões arquiteturais significativas, utilizar arquivo simples:

docs/architecture.md

ou ADRs pequenos.

Não criar burocracia exagerada.

---

# 102. QUALIDADE DE CÓDIGO

Antes de considerar tarefa concluída:

cargo fmt

cargo clippy

cargo test

deverão ser executados quando aplicáveis.

Corrigir warnings relevantes.

Não esconder warnings indiscriminadamente.

---

# 103. UNSAFE

Evitar unsafe.

Se biblioteca externa exigir, tudo bem.

Não escrever código unsafe próprio salvo necessidade técnica muito bem justificada.

---

# 104. CLONES E ALOCAÇÕES

Não sacrificar legibilidade tentando eliminar todo clone.

Primeiro corretude.

Depois profiling se houver problema.

---

# 105. ERGONOMIA RUST

Preferir:

* tipos fortes;
* enums;
* structs pequenas;
* Result;
* Option;
* ownership claro.

Evitar estruturas baseadas em HashMap<String,String> para todo o domínio.

Informação conhecida merece tipos conhecidos.

---

# 106. FLEXIBILIDADE

Ao mesmo tempo, manter mecanismo para metadata adicional quando houver campos importados desconhecidos.

Exemplo:

extra_fields

somente onde realmente necessário.

Não utilizar isso como desculpa para abandonar modelagem.

---

# 107. TESTE MANUAL MÍNIMO

Criar pelo menos estes registros de teste:

QSO DMR via repetidora.

QSO DMR via hotspot.

QSO DMR simplex.

QSO FT8.

QSO genérico digital.

Verificar persistência após reinicialização.

---

# 108. EXEMPLO DMR

Dados conceituais:

Callsign:
PU2XYZ

DMR ID:
7241234

TG:
724

Timeslot:
1

Color Code:
1

Network:
BrandMeister

Access:
Repeater

Frequency:
438.500 MHz

Não codificar esses valores como padrão.

São apenas dados de teste.

---

# 109. DEFINIÇÃO DE PRONTO PARA CADA FEATURE

Uma funcionalidade somente está pronta quando:

* implementada;
* compilando;
* integrada;
* testável;
* erros tratados;
* não quebra funcionalidades anteriores.

Código pela metade não deve ser declarado pronto.

---

# 110. POLÍTICA DE COMMIT

Se o ambiente estiver sob Git:

manter alterações logicamente agrupadas.

Não realizar commits automaticamente se não houver autorização para commits.

Nunca executar push automaticamente sem autorização explícita.

---

# 111. ALTERAÇÕES DESTRUTIVAS

Nunca:

* apagar banco;
* apagar arquivos do usuário;
* resetar repositório;
* fazer git clean;
* fazer git reset --hard;
* sobrescrever configuração;

sem autorização explícita.

---

# 112. COMANDOS DESTRUTIVOS

Proibido utilizar comandos potencialmente destrutivos apenas para solucionar erros de ambiente.

Corrigir causa.

Não eliminar evidências.

---

# 113. QUANDO UMA DEPENDÊNCIA FALHAR

Diagnosticar.

Não trocar tecnologia imediatamente.

Exemplo:

se Slint apresentar problema de compilação:

investigar primeiro.

Não substituir interface por GTK sem autorização.

---

# 114. QUANDO UMA ABORDAGEM FALHAR

Registrar:

* o que foi tentado;
* por que falhou;
* alternativa mínima.

Evitar entrar em ciclo tentando infinitamente pequenas variações da mesma solução.

---

# 115. FREIO CONTRA LOOP DE AGENTES

Caso você consulte outro agente ou ferramenta:

limite a consulta a uma questão específica.

Não permitir delegação recursiva indefinida.

Um agente secundário não deve continuar delegando indefinidamente.

Máximo conceitual:

agente principal
→ uma consulta
→ retorno
→ decisão.

Não criar ciclo:

A pergunta B

B pergunta A

A pergunta B.

---

# 116. FREIO CONTRA REESCRITA COMPLETA

Não reescrever o projeto inteiro porque encontrou pequena inconsistência.

Corrigir localmente.

Reescrita ampla exige justificativa técnica forte e autorização.

---

# 117. FREIO CONTRA "MELHORIAS"

Não adicionar feature apenas porque:

"seria legal".

Registrar em:

docs/ideas.md

se realmente merecer ser preservada.

---

# 118. FUTURE IDEAS

Arquivo opcional:

docs/ideas.md

Pode conter:

* integração BrandMeister;
* RadioID;
* Hamlib;
* WSJT-X UDP;
* QRZ;
* HamQTH;
* LoTW;
* eQSL;
* mapas;
* estatísticas;
* gráficos.

Nenhuma dessas ideias deverá contaminar o MVP.

---

# 119. ORDEM DAS PRIORIDADES

Em caso de conflito:

1. integridade dos dados;
2. corretude;
3. simplicidade;
4. interoperabilidade;
5. manutenção;
6. desempenho;
7. estética;
8. funcionalidades adicionais.

---

# 120. PERGUNTA OBRIGATÓRIA ANTES DE INVENTAR

Sempre que surgir vontade de acrescentar grande funcionalidade, faça internamente a pergunta:

"Isso foi solicitado?"

Se não:

não faça.

---

# 121. PRIMEIRA TAREFA

Antes de escrever código significativo:

1. inspecione o diretório atual;
2. verifique se já existe projeto;
3. identifique arquivos existentes;
4. não sobrescreva trabalho existente;
5. descreva resumidamente o estado encontrado;
6. proponha estrutura mínima;
7. implemente somente a primeira etapa necessária.

Se o repositório estiver vazio:

crie estrutura inicial.

---

# 122. PRIMEIRO RESULTADO ESPERADO

Primeiro marco:

um aplicativo Slint mínimo que:

* compila;
* inicia;
* abre banco SQLite;
* executa migrations;
* apresenta janela;
* lista QSOs existentes;
* permite inserir um QSO simples;
* persiste registro;
* reabre registro posteriormente.

Somente depois disso adicionar complexidade específica DMR e FT8.

---

# 123. NÃO SIMULAR CONCLUSÃO

Nunca afirmar que algo funciona sem executar teste apropriado quando o ambiente permitir.

Se não puder testar:

declare explicitamente:

"implementado, porém não validado neste ambiente."

---

# 124. SAÍDA ESPERADA DO AGENTE

Após cada bloco relevante de trabalho, informar resumidamente:

* o que foi feito;
* arquivos alterados;
* decisões tomadas;
* testes executados;
* resultado;
* problemas encontrados;
* próximo passo lógico.

Evitar relatórios gigantescos.

---

# 125. COMPORTAMENTO ESPERADO

Atue como engenheiro responsável por um projeto real.

Não como gerador indiscriminado de features.

Seu trabalho é preservar o objetivo original.

Quando houver duas soluções tecnicamente aceitáveis:

prefira a mais simples.

Quando uma feature puder esperar:

faça esperar.

Quando uma abstração não for necessária:

não crie.

Quando algo puder permanecer local:

não coloque na rede.

Quando SQLite resolver:

não invente servidor.

Quando Rust std resolver:

não adicione crate.

Quando um módulo resolver:

não crie framework.

---

# 126. RESUMO DO CONTRATO

O produto é:

um logbook desktop local especializado em modos digitais para radioamadores.

Stack:

Rust
+
Slint
+
SQLite
+
rusqlite
+
Serde.

Primeiro foco:

DMR
+
FT8
+
ADIF.

Características centrais:

QSO
+
metadados específicos do modo
+
rota digital.

Não é:

SaaS.

Não é:

rede social.

Não é:

servidor.

Não é:

dashboard corporativo.

Não é:

framework.

Não é:

projeto acadêmico de arquitetura excessiva.

É uma ferramenta desktop destinada a registrar, preservar, consultar e exportar contatos digitais de radioamador de forma confiável.

---

# 127. INSTRUÇÃO FINAL

A partir deste ponto:

1. examine o estado atual do projeto;
2. não altere tecnologias definidas;
3. não amplie o escopo;
4. apresente um plano curto de implementação;
5. execute a menor etapa funcional possível;
6. compile;
7. teste;
8. corrija problemas encontrados;
9. apresente o resultado;
10. pare no próximo marco lógico.

Não avance dez fases de uma única vez.

Não implemente funcionalidades futuras antecipadamente.

Não reorganize arquitetura sem necessidade.

Não substitua decisões fundamentais silenciosamente.

O projeto deve crescer de forma incremental, verificável e controlada.

O objetivo é construir software que funcione, e não impressionar outro agente de IA com quantidade de abstrações.

