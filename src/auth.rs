use std::path::Path;

use anyhow::{Context, Result};

pub fn resolve_token(token_file: Option<&Path>, token_arg: Option<&str>) -> Result<String> {
    if let Some(token) = token_arg {
        return Ok(token.to_owned());
    }
    if let Some(path) = token_file {
        let token = std::fs::read_to_string(path)
            .with_context(|| format!("reading token file {}", path.display()))?;
        return Ok(token.trim().to_owned());
    }
    anyhow::bail!("no API token provided: use --token, --token-file, VIKUNJA_TOKEN, or vkc config");
}
