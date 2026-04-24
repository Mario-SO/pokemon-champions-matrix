use crate::error::PcError;
use crate::model::{Nature, PokemonSet, StatPoints};

const EV_IV_ERROR: &str = "IVs are not valid in Pokémon Champions Regulation-M. In matrix Showdown input, EVs are interpreted as Pokémon Champions SPs.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShowdownPokemon {
    pub set: PokemonSet,
    pub moves: Vec<String>,
}

pub(super) fn parse_showdown_team(input: &str) -> Result<Vec<ShowdownPokemon>, PcError> {
    let mut team = Vec::new();
    let mut current: Option<ShowdownPokemon> = None;

    for (index, raw_line) in input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line
            .split_once('#')
            .map_or(raw_line, |(before, _)| before)
            .trim();
        if line.is_empty() {
            finish_set(&mut team, &mut current)?;
            continue;
        }

        if let Some(move_name) = line.strip_prefix('-') {
            let pokemon = current.as_mut().ok_or_else(|| PcError::Parse {
                line: line_no,
                message: "move line appeared before a Pokémon identity line".to_string(),
            })?;
            let move_name = move_name.trim();
            if !move_name.is_empty() {
                pokemon.moves.push(move_name.to_string());
            }
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let pokemon = current.as_mut().ok_or_else(|| PcError::Parse {
                line: line_no,
                message: "key-value line appeared before a Pokémon identity line".to_string(),
            })?;
            parse_key_value(&mut pokemon.set, key.trim(), value.trim(), line_no)?;
            continue;
        }

        if let Some(nature_text) = line.strip_suffix(" Nature") {
            let pokemon = current.as_mut().ok_or_else(|| PcError::Parse {
                line: line_no,
                message: "nature line appeared before a Pokémon identity line".to_string(),
            })?;
            pokemon.set.nature =
                nature_text
                    .trim()
                    .parse::<Nature>()
                    .map_err(|message| PcError::Parse {
                        line: line_no,
                        message,
                    })?;
            continue;
        }

        finish_set(&mut team, &mut current)?;
        current = Some(ShowdownPokemon {
            set: parse_identity_line(line, line_no)?,
            moves: Vec::new(),
        });
    }

    finish_set(&mut team, &mut current)?;
    Ok(team)
}

fn finish_set(
    team: &mut Vec<ShowdownPokemon>,
    current: &mut Option<ShowdownPokemon>,
) -> Result<(), PcError> {
    if let Some(pokemon) = current.take() {
        pokemon
            .set
            .stat_points
            .validate()
            .map_err(|errors| PcError::Validation {
                message: errors.join("; "),
            })?;
        team.push(pokemon);
    }
    Ok(())
}

fn parse_identity_line(line: &str, line_no: usize) -> Result<PokemonSet, PcError> {
    let (species, item) = if let Some((species, item)) = line.split_once('@') {
        let item = item.trim();
        (
            species.trim(),
            if item.is_empty() {
                None
            } else {
                Some(item.to_string())
            },
        )
    } else {
        (line.trim(), None)
    };

    if species.is_empty() {
        return Err(PcError::Parse {
            line: line_no,
            message: "Pokémon species cannot be empty".to_string(),
        });
    }

    Ok(PokemonSet::new(species.to_string(), item))
}

fn parse_key_value(
    set: &mut PokemonSet,
    key: &str,
    value: &str,
    line_no: usize,
) -> Result<(), PcError> {
    match normalize_key(key).as_str() {
        "ability" => set.ability = Some(value.to_string()),
        "level" => {
            set.level = value.parse::<u8>().map_err(|_| PcError::Parse {
                line: line_no,
                message: format!("invalid level '{value}'"),
            })?
        }
        "evs" | "sps" => {
            set.stat_points = StatPoints::parse(value).map_err(|message| PcError::Parse {
                line: line_no,
                message,
            })?
        }
        "ivs" => {
            return Err(PcError::Parse {
                line: line_no,
                message: EV_IV_ERROR.to_string(),
            });
        }
        "teratype" => {
            set.tera_type = Some(value.parse().map_err(|message| PcError::Parse {
                line: line_no,
                message,
            })?)
        }
        "tera" => set.tera_active = parse_bool(value, line_no)?,
        "nature" => {
            set.nature = value.parse::<Nature>().map_err(|message| PcError::Parse {
                line: line_no,
                message,
            })?
        }
        "happiness" | "shiny" | "gender" => {}
        _ => {}
    }
    Ok(())
}

fn parse_bool(value: &str, line_no: usize) -> Result<bool, PcError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" | "on" => Ok(true),
        "no" | "false" | "off" => Ok(false),
        _ => Err(PcError::Parse {
            line: line_no,
            message: format!("expected boolean value, found '{value}'"),
        }),
    }
}

fn normalize_key(key: &str) -> String {
    key.chars()
        .filter(|character| !character.is_whitespace() && *character != '-' && *character != '_')
        .collect::<String>()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_existing_team_file() {
        let input = include_str!("../../examples/my-team.txt");
        let team = parse_showdown_team(input).unwrap();
        assert_eq!(team.len(), 6);
        assert_eq!(team[0].set.species, "Milotic");
        assert_eq!(team[0].set.stat_points.hp, 30);
        assert_eq!(team[0].set.stat_points.def, 21);
        assert_eq!(
            team[0].moves,
            ["Muddy Water", "Coil", "Recover", "Hypnosis"]
        );
        assert_eq!(team[5].set.species, "Sneasler");
        assert_eq!(team[5].set.stat_points.atk, 32);
    }

    #[test]
    fn evs_are_matrix_sps() {
        let team = parse_showdown_team(
            "Garchomp @ Sitrus Berry\nAbility: Rough Skin\nAdamant Nature\nEVs: 24 HP / 19 Atk / 1 SpD / 22 Spe\n- Earthquake\n",
        )
        .unwrap();
        assert_eq!(team[0].set.stat_points.hp, 24);
        assert_eq!(team[0].set.stat_points.atk, 19);
        assert_eq!(team[0].set.stat_points.spe, 22);
    }

    #[test]
    fn rejects_ivs() {
        let err = parse_showdown_team("Milotic\nIVs: 0 Atk\n- Recover\n").unwrap_err();
        assert!(err.to_string().contains("IVs"));
    }

    #[test]
    fn defaults_missing_fields() {
        let team = parse_showdown_team("Milotic\n- Recover\n").unwrap();
        assert_eq!(team[0].set.level, 50);
        assert_eq!(team[0].set.nature, Nature::Hardy);
        assert_eq!(team[0].set.stat_points.total(), 0);
    }
}
