# pc

`pc` is a terminal matchup matrix for Pokémon Champions / VGC Regulation-M-A.

The only supported command is:

```sh
pc matrix
```

By default it loads:

- `examples/my-team.txt`
- `examples/opponents.txt`

Both files use Showdown-like team text. In this tool, `EVs:` means Pokémon Champions Stat Points.

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
- `<Nature> Nature`
- `Level:`
- `EVs:` or `SPs:`
- `Tera Type:`
- `Tera: Yes/No`
- `- Move`

`IVs:` are rejected. Missing level defaults to `50`, missing nature defaults to `Hardy`, and missing SPs default to `0`.

## Data

`pc matrix` fetches Pokémon and move data from [PokéAPI](https://pokeapi.co/docs/v2) while loading the TUI. There is no disk cache.

PokéAPI provides base stats, typing, move type, move category, power, and move target metadata. The local code handles Pokémon Champions SP stat math, speed modifiers, type chart, weather, screens, Tera basics, and damage rolls.

The TUI currently shows a terrain selector for planning context, but terrain is display-only and does not affect damage or speed calculations yet.

Current stat model:

```text
1 SP = 8 EV-equivalent, IV-equivalent baseline = 31.
```

## Development

```sh
cargo run -- matrix
cargo test
cargo build
```
