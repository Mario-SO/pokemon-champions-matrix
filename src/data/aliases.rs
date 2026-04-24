pub(super) fn normalize_name(name: &str) -> String {
    name.trim()
        .to_ascii_lowercase()
        .replace(['’', '\''], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

pub(super) fn pokemon_lookup_name(name: &str) -> String {
    let normalized = normalize_name(name);
    match normalized.as_str() {
        "aegislash-shield" => "aegislash-shield".to_string(),
        "aegislash" => "aegislash-shield".to_string(),
        "aegislash-blade" => "aegislash-blade".to_string(),
        "basculegion" => "basculegion-male".to_string(),
        "maushold" | "maushold-three" | "maushold-family-of-three" => {
            "maushold-family-of-three".to_string()
        }
        "maushold-four" | "maushold-family-of-four" => "maushold-family-of-four".to_string(),
        "mimikyu" | "mimikyu-disguised" => "mimikyu-disguised".to_string(),
        "palafin" => "palafin-zero".to_string(),
        "mega-charizard-x" | "charizard-mega-x" => "charizard-mega-x".to_string(),
        "mega-gengar" | "gengar-mega" => "gengar-mega".to_string(),
        "mega-charizard-y" | "charizard-mega-y" => "charizard-mega-y".to_string(),
        normalized if normalized.starts_with("mega-") => {
            format!("{}-mega", normalized.trim_start_matches("mega-"))
        }
        normalized => normalized.to_string(),
    }
}

pub(super) fn move_lookup_name(name: &str) -> String {
    normalize_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_common_names() {
        assert_eq!(move_lookup_name("Sludge Bomb"), "sludge-bomb");
        assert_eq!(pokemon_lookup_name("Aegislash-Shield"), "aegislash-shield");
        assert_eq!(pokemon_lookup_name("Aegislash Shield"), "aegislash-shield");
        assert_eq!(pokemon_lookup_name("Aegislash"), "aegislash-shield");
        assert_eq!(pokemon_lookup_name("Mega Gengar"), "gengar-mega");
        assert_eq!(pokemon_lookup_name("Mega Charizard Y"), "charizard-mega-y");
        assert_eq!(pokemon_lookup_name("Mega Charizard X"), "charizard-mega-x");
        assert_eq!(pokemon_lookup_name("Mega Tyranitar"), "tyranitar-mega");
        assert_eq!(pokemon_lookup_name("Basculegion"), "basculegion-male");
        assert_eq!(pokemon_lookup_name("Palafin"), "palafin-zero");
        assert_eq!(pokemon_lookup_name("Maushold"), "maushold-family-of-three");
        assert_eq!(
            pokemon_lookup_name("Maushold Four"),
            "maushold-family-of-four"
        );
        assert_eq!(pokemon_lookup_name("Mimikyu"), "mimikyu-disguised");
    }
}
