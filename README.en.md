# DERIVA

[🇧🇷 Português](README.md) | 🇺🇸 English

[![CI](https://github.com/igorgbr/deriva/actions/workflows/ci.yml/badge.svg)](https://github.com/igorgbr/deriva/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/deriva.svg)](https://crates.io/crates/deriva)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A terminal visual novel written in Rust. Colorful ASCII art, truecolor
gradients, mouse support, and stories anyone can write in a plain .txt file.

The built-in story is in Brazilian Portuguese — but the engine is
language-agnostic: write your own story in any language and play it
without recompiling.

## Install

```bash
cargo install deriva                    # via crates.io
cargo install deriva --features sound   # with sound (needs ALSA headers on Linux)
```

Or grab a prebuilt binary (Linux, macOS, Windows) or the `.deb` from the
[releases page](https://github.com/igorgbr/deriva/releases).
On Arch: `deriva` on the AUR.

## Play

```bash
deriva                       # built-in story (pt-BR)
deriva my-story.txt          # play your own story
deriva --check my-story.txt  # validate without playing (for authors)
deriva --c64                 # Commodore 64 look (blue screen + frame)
```

Click a choice with the mouse or press its number. `q` / `Esc` quits.

## Writing stories

Any .txt in this format is playable:

```
=== scene_id
@art
  (optional ASCII art, cyan by default)
@text
Narrative text (typewriter effect).
@choices
Choice label -> target_scene_id
Another choice -> other_id
```

The starting scene must be named `inicio`. End a branch by replacing
`@choices` with `@ending good` or `@ending bad`.

Inline colors (work in `@art` and `@text`): `{c}` cyan, `{y}` yellow,
`{g}` green, `{r}` red, `{m}` magenta, `{w}` white, `{0}` default.
Vertical truecolor gradient: `@art #RRGGBB #RRGGBB` interpolates top to
bottom, e.g. `@art #5fd7ff #ff5fd7`.

See the [Portuguese README](README.md) for the full authoring guide.
