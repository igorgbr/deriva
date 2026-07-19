# DERIVA

[![CI](https://github.com/igorgbr/deriva/actions/workflows/ci.yml/badge.svg)](https://github.com/igorgbr/deriva/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/deriva.svg)](https://crates.io/crates/deriva)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Novel game de terminal em Rust. ASCII art colorida, degradês truecolor,
mouse, e histórias que qualquer pessoa pode escrever num .txt.

## Instalar

```bash
cargo install deriva                    # via crates.io
cargo install deriva --features sound   # com som (requer headers ALSA no Linux)
```

Ou baixe o binário pronto (Linux, macOS, Windows) ou o `.deb` na
[página de releases](https://github.com/igorgbr/deriva/releases).
No Arch: pacote `deriva` no AUR.

## Rodar

```bash
cargo run              # compila em qualquer máquina, sem dependência de sistema

# visual Commodore 64 (fundo azul, moldura, texto azul-claro):
cargo run -- --c64
```

Som (tons senoidais via rodio) é **opt-in**, para o `cargo install`
nunca falhar por falta de headers de áudio. Sem a feature, os efeitos
viram o BEL do terminal:

```bash
# no Linux, precisa dos headers do ALSA (uma vez só):
sudo dnf install alsa-lib-devel      # Fedora
sudo apt install libasound2-dev      # Debian/Ubuntu

cargo run --features sound
```

## Histórias externas

A história embutida é só o começo — qualquer .txt no formato abaixo
é jogável, sem recompilar:

```bash
deriva minha-historia.txt            # joga a sua história
deriva --check minha-historia.txt    # valida sem jogar (para autores)
deriva --help
```

O modo `--c64` usa OSC 10/11 para trocar as cores padrão do terminal
(restauradas na saída); terminais sem suporte ignoram os códigos e
mostram só a moldura.

## Testar consistência da história

```bash
cargo test
```

Falha se alguma cena apontar para destino inexistente ou for beco sem saída.

## Escrever histórias

Edite `assets/story.txt` (embutida) ou crie um .txt seu. Formato:

```
=== id_da_cena
@art
  (ascii art opcional, em ciano)
@text
Texto narrativo (efeito máquina de escrever).
@choices
Rótulo da escolha -> id_da_cena_destino
Outra escolha -> outro_id
```

Cena final: troque `@choices` por `@ending good` ou `@ending bad`.

Cores (funcionam em `@art` e `@text`): `{c}` ciano, `{y}` amarelo,
`{g}` verde, `{r}` vermelho, `{m}` magenta, `{w}` branco, `{0}` padrão.
A arte começa em ciano por padrão — use `{c}` para voltar a ele.

Degradê vertical (truecolor): `@art #RRGGBB #RRGGBB` — interpola do
topo para a base, ex. `@art #5fd7ff #ff5fd7`. Tags inline sobrescrevem
o degradê até o fim da linha, então prefira um ou outro por cena.

## Controles

Clique do mouse na opção, ou tecle o número (sem Enter).
`q` / `Esc` sai a qualquer momento.

A cena inicial deve se chamar `inicio`. Histórias externas são
validadas ao carregar; a embutida exige recompilar após editar.

## Ideias futuras

- Inventário/flags (ex.: só abre o reator se tiver o Cacto)
- Pasta `~/.config/deriva/stories/` com menu de histórias instaladas
- Salvar progresso em arquivo
