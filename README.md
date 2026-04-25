# pc

`pc` is a terminal matchup matrix for Pokémon Champions / VGC Regulation-M-A.

It loads Showdown-like team text, fetches Pokémon and move metadata from [PokéAPI](https://pokeapi.co/docs/v2), and opens a keyboard-driven TUI for offensive, defensive, and speed comparisons.

https://github.com/user-attachments/assets/d942eae4-8a99-4eda-8106-42393f39cf6a

## Install

The first public distribution target is a custom Homebrew tap:

```sh
brew install Mario-SO/tap/pc
```

You can also install directly from source:

```sh
cargo install --git https://github.com/Mario-SO/pokemon-champions-matrix pc
```

For local development:

```sh
cargo run -- matrix --team examples/my-team.txt --opponents examples/opponents.txt
```

## Quickstart

Create editable sample files:

```sh
pc init
```

Then open the matrix:

```sh
pc matrix
```

By default, `pc matrix` reads:

- `$PC_CONFIG_DIR/my-team.txt`, when `PC_CONFIG_DIR` is set.
- `$XDG_CONFIG_HOME/pc/my-team.txt` and `$XDG_CONFIG_HOME/pc/opponents.txt`, when `XDG_CONFIG_HOME` is set.
- `~/.config/pc/my-team.txt` and `~/.config/pc/opponents.txt` otherwise.

You can always pass explicit paths:

```sh
pc matrix --team examples/my-team.txt --opponents examples/opponents.txt
```

## Matrix

The matrix has three views:

- `Offensive`: selected team Pokémon's damaging moves into each opponent.
- `Defensive`: each opponent's damaging moves into the selected team Pokémon.
- `Speed`: effective speed comparison into each opponent.

Keyboard:

- `1`, `2`, `3`: Offensive, Defensive, Speed.
- `Up`/`Down` or `k`/`j`: select your Pokémon.
- `Left`/`Right` or `h`/`l`: select opponent card.
- `PgUp`/`PgDn`: jump through opponent cards.
- `/`: search opponent Pokémon by name.
- `c`: battle conditions.
- `r`: reload files.
- `?`: help.
- `q`: quit.

## Input Format

Example:

```text
Milotic @ Leftovers
Ability: Competitive
Bold Nature
Level: 50
EVs: 30 HP / 21 Def / 1 SpA / 12 SpD / 1 Spe
- Muddy Water
- Coil
- Recover
- Hypnosis
```

Supported fields:

- `Species @ Item`
- `Ability:`
- `<Nature> Nature` or `Nature: <Nature>`
- `Level:`
- `EVs:` or `SPs:`
- `Tera Type:`
- `Tera: Yes/No`
- `- Move`

`IVs:` are rejected. In this tool, `EVs:` means Pokémon Champions Stat Points. Missing level defaults to `50`, missing nature defaults to `Hardy`, and missing SPs default to `0`.

## Data and Cache

`pc matrix` fetches Pokémon and move data from PokéAPI while loading the TUI. Raw PokéAPI responses are cached in a local SQLite database:

- `$PC_CONFIG_DIR/pc.sqlite`, when `PC_CONFIG_DIR` is set.
- `$XDG_CONFIG_HOME/pc/pc.sqlite`, when `XDG_CONFIG_HOME` is set.
- `~/.config/pc/pc.sqlite` otherwise.

The cache is read before network requests. On a cache miss, `pc matrix` fetches from PokéAPI and stores the response for later runs. Network requests use a 10 second timeout.

## Limitations

- Terrain is currently display-only and does not affect damage or speed calculations yet.
- The calculator covers the local Pokémon Champions SP stat math, speed modifiers, type chart, weather, screens, Tera basics, and damage rolls implemented in this repo.
- Data availability depends on PokéAPI names. Some Pokémon Champions-specific forms or mechanics may need local aliases or special handling.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release --locked
```

Release builds are handled by GitHub Actions for Linux and macOS. The Homebrew formula is maintained in the [`Mario-SO/homebrew-tap`](https://github.com/Mario-SO/homebrew-tap) repository.

## Attribution

Pokémon and related names are trademarks of Nintendo, Creatures Inc., and GAME FREAK. This project is unofficial and is not affiliated with or endorsed by those companies.

Pokémon and move metadata is provided by [PokéAPI](https://pokeapi.co/).
