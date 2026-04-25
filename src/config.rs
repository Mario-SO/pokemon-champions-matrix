use crate::error::PcError;
use std::path::PathBuf;

pub(crate) fn pc_config_dir() -> Result<PathBuf, PcError> {
    pc_config_dir_from_values(
        std::env::var_os("PC_CONFIG_DIR").map(PathBuf::from),
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

fn pc_config_dir_from_values(
    pc_config_dir: Option<PathBuf>,
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Result<PathBuf, PcError> {
    if let Some(value) = pc_config_dir {
        return Ok(value);
    }
    if let Some(value) = xdg_config_home {
        return Ok(value.join("pc"));
    }
    if let Some(value) = home {
        return Ok(value.join(".config").join("pc"));
    }
    Err(PcError::Validation {
        message: "could not determine a config directory; set PC_CONFIG_DIR".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc_config_dir_prefers_explicit_config_dir() {
        let path = pc_config_dir_from_values(
            Some(PathBuf::from("/tmp/pc-config")),
            Some(PathBuf::from("/tmp/xdg")),
            Some(PathBuf::from("/tmp/home")),
        )
        .unwrap();

        assert_eq!(path, PathBuf::from("/tmp/pc-config"));
    }

    #[test]
    fn pc_config_dir_uses_xdg_before_home() {
        let path = pc_config_dir_from_values(
            None,
            Some(PathBuf::from("/tmp/xdg")),
            Some(PathBuf::from("/tmp/home")),
        )
        .unwrap();

        assert_eq!(path, PathBuf::from("/tmp/xdg/pc"));
    }
}
