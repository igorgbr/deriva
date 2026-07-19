# Publicando o Deriva

Checklist na ordem. Os passos 1–3 são feitos uma vez; o 4 a cada versão nova.

## 1. GitHub (uma vez)

```bash
# crie o repo vazio em https://github.com/new (nome: deriva, público, sem README)
cd deriva
git remote add origin git@github.com:igorgiamoniano/deriva.git
git push -u origin main
```

Ou com o GitHub CLI: `gh repo create igorgiamoniano/deriva --public --source . --push`

## 2. crates.io (uma vez o login)

```bash
cargo login            # token em https://crates.io/settings/tokens
cargo publish --dry-run   # confere o pacote
cargo publish
```

Depois disso qualquer pessoa instala com `cargo install deriva`
(e `cargo install deriva --features sound` para som).

## 3. AUR (uma vez, opcional)

Requer conta em https://aur.archlinux.org com chave SSH cadastrada.

```bash
git clone ssh://aur@aur.archlinux.org/deriva.git aur-deriva
cp packaging/PKGBUILD aur-deriva/
cd aur-deriva
# preencha o sha256: curl -L https://github.com/igorgiamoniano/deriva/archive/v0.1.0.tar.gz | sha256sum
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO && git commit -m "deriva 0.1.0" && git push
```

## 4. Cada release nova

```bash
# 1. suba a versão no Cargo.toml (ex.: 0.2.0) e commite
git tag v0.2.0
git push && git push --tags
```

A tag dispara o workflow `release.yml`, que compila e anexa na GitHub Release:
binários Linux/macOS (x86_64 e ARM)/Windows e o pacote `.deb`.

Depois:

```bash
cargo publish                      # atualiza crates.io
# AUR: atualize pkgver/sha256 no PKGBUILD e repita o passo 3
```

## Flatpak — por que ficou de fora

O Deriva é um jogo de terminal; Flatpak/Flathub é voltado a apps gráficos
(um flatpak de terminal roda via `flatpak run` num sandbox, sem entrar no
PATH — experiência ruim). Os canais acima cobrem Linux melhor:
`.deb`, AUR, `cargo install` e binário solto da Release.
Se um dia o Deriva ganhar interface gráfica, aí sim vale Flathub.
