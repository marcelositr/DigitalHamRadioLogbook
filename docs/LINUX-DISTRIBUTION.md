# Distribuição no GNU/Linux

Este documento descreve o pacote binário user-local do Digital Ham Radio Logbook. Ele não exige privilégios de administrador, não instala serviços e não altera arquivos do sistema.

## Conteúdo do pacote

O arquivo de release contém somente:

- `bin/digital-ham-radio-logbook`: binário compilado em modo release;
- `share/applications/io.github.marcelositr.DigitalHamRadioLogbook.desktop.in`: desktop entry parametrizada;
- `share/icons/hicolor/scalable/apps/io.github.marcelositr.DigitalHamRadioLogbook.svg`: ícone escalável;
- `install.sh` e `uninstall.sh`: instalação e remoção user-local;
- `LICENSE` e este guia.

O app ID usado na integração desktop é `io.github.marcelositr.DigitalHamRadioLogbook`.

## Compatibilidade

O pacote é específico para a arquitetura indicada no nome do arquivo. Ele usa bibliotecas compartilhadas do sistema exigidas pelo binário e pelo backend gráfico do Slint. Não há garantia de compatibilidade entre distribuições com versões incompatíveis da glibc ou de outras bibliotecas nativas.

`make-release.sh` executa `ldd` quando disponível e interrompe a criação se detectar uma biblioteca marcada como `not found`. Essa verificação confirma o host de build, não todos os computadores de destino. Para maior compatibilidade, gere o pacote na distribuição mais antiga que será suportada e teste-o nas distribuições-alvo.

## Gerar uma release

Requisitos:

- toolchain Rust estável e `cargo`;
- dependências nativas de compilação do projeto;
- `tar`;
- `sha256sum` ou `shasum`;
- opcionalmente GNU `tar` e `gzip` para normalização determinística;
- opcionalmente `ldd` para validar bibliotecas compartilhadas.

Na raiz do repositório:

```sh
packaging/linux/make-release.sh
```

Um diretório de saída pode ser informado:

```sh
packaging/linux/make-release.sh artifacts
```

O script valida ferramentas, arquivos de entrada e diretório de saída antes do build, executa exatamente um build bloqueado de release (`cargo build --locked --release`) e monta um staging mínimo. O tarball e o checksum são concluídos em arquivos temporários no próprio diretório de saída; somente depois são renomeados para `digital-ham-radio-logbook-VERSAO-linux-ARQUITETURA.tar.gz` e `.tar.gz.sha256`. O checksum é publicado por último e funciona como marcador de conclusão, evitando que consumidores encontrem um checksum para um tarball ainda parcial.

Quando GNU `tar` e `gzip` estão disponíveis, o conteúdo é ordenado, proprietário e grupo são normalizados para `0`, timestamps usam `${SOURCE_DATE_EPOCH:-0}` e o cabeçalho gzip não contém nome nem timestamp. Com as mesmas entradas, arquitetura e `SOURCE_DATE_EPOCH`, isso produz tarballs reproduzíveis. Em outras implementações de `tar`, a geração continua de forma portátil, com um aviso de que os metadados não foram normalizados.

## Verificar e extrair

No diretório que contém os dois arquivos:

```sh
sha256sum -c digital-ham-radio-logbook-*.tar.gz.sha256
```

Em sistemas com `shasum`:

```sh
shasum -a 256 -c digital-ham-radio-logbook-*.tar.gz.sha256
```

Depois, extraia e entre no diretório criado:

```sh
tar -xzf digital-ham-radio-logbook-*.tar.gz
cd digital-ham-radio-logbook-*-linux-*
```

Evite executar pacotes cuja origem ou checksum não sejam confiáveis.

## Instalação user-local

Confira primeiro o plano sem modificar o sistema:

```sh
./install.sh --dry-run
```

Instale:

```sh
./install.sh
```

O instalador respeita:

- binário: `${XDG_BIN_HOME:-$HOME/.local/bin}/digital-ham-radio-logbook`;
- desktop entry: `${XDG_DATA_HOME:-$HOME/.local/share}/applications/`;
- ícone: `${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps/`;
- manifesto e desinstalador: `${XDG_DATA_HOME:-$HOME/.local/share}/io.github.marcelositr.DigitalHamRadioLogbook/`.

Cada arquivo é escrito em um temporário no próprio diretório de destino e publicado com `mv`, reduzindo o risco de arquivo parcial. O desktop entry recebe o caminho absoluto do binário. O manifesto registra os arquivos da instalação, mas o desinstalador usa ainda uma lista fixa de destinos permitidos para que um manifesto adulterado não autorize remoções arbitrárias.

Se `${XDG_BIN_HOME:-$HOME/.local/bin}` não estiver em `PATH`, inicie pelo menu do desktop ou adicione esse diretório ao `PATH` conforme a configuração do seu shell.

Alguns ambientes atualizam o menu automaticamente. Se necessário e se as ferramentas existirem, pode-se executar `update-desktop-database` no diretório `applications` e `gtk-update-icon-cache` no tema `hicolor`; isso não é exigido pelo instalador.

## Atualização

Extraia a nova release, valide o checksum e execute o novo `./install.sh`. A publicação atômica substitui somente os arquivos do aplicativo. Feche o programa antes de atualizar. O banco e a configuração não são modificados.

## Desinstalação

O caminho do desinstalador é mostrado no final da instalação. Em uma configuração XDG padrão:

```sh
~/.local/share/io.github.marcelositr.DigitalHamRadioLogbook/uninstall.sh --dry-run
~/.local/share/io.github.marcelositr.DigitalHamRadioLogbook/uninstall.sh
```

Se `XDG_DATA_HOME` ou `XDG_BIN_HOME` foram usados na instalação, mantenha os mesmos valores ao desinstalar. O processo é idempotente: executá-lo quando os arquivos já não existem não causa erro. Também é possível usar o `uninstall.sh` da release extraída, com o mesmo ambiente XDG.

## Dados e privacidade

Instalação, atualização e desinstalação **não removem nem alteram dados do usuário**. O aplicativo mantém:

- banco: `${XDG_DATA_HOME:-$HOME/.local/share}/digital-ham-log/logbook.sqlite3`;
- configuração: `${XDG_CONFIG_HOME:-$HOME/.config}/digital-ham-log/config.toml`.

A pasta `digital-ham-log` é deliberadamente separada dos arquivos instalados e sempre preservada pelo desinstalador. Para remoção manual dos dados, primeiro faça backup e confirme os caminhos XDG efetivos; essa ação é irreversível e não faz parte dos scripts de distribuição.

O aplicativo é local/offline e não instala telemetria, daemon ou atualização automática.

## Validação do packaging

O teste POSIX de distribuição não compila o aplicativo real: ele cria uma cópia mínima e isolada do layout da release com um payload controlado. O teste verifica o conteúdo exato do tarball, checksum, ausência de temporários de publicação, planos `--dry-run`, instalação, reinstalação, desinstalação e preservação de dados/configuração XDG sentinela:

```sh
sh -n packaging/linux/*.sh
packaging/linux/smoke-test.sh
```

A CI executa essas verificações em um job pequeno e independente do build Rust.

## Solução de problemas

- Use `./install.sh --help`, `./uninstall.sh --help` ou `packaging/linux/make-release.sh --help` para a sintaxe.
- Se o programa não aparecer no menu, encerre e abra novamente a sessão ou atualize o cache de desktop do ambiente.
- Se o shell não encontrar o comando, confira `XDG_BIN_HOME` e `PATH`.
- Em erro de biblioteca compartilhada, execute `ldd bin/digital-ham-radio-logbook` e instale o pacote da distribuição que fornece a dependência ausente, ou use uma release compatível.
- Não execute `install.sh` com `sudo`: o pacote foi projetado exclusivamente para o usuário atual.
