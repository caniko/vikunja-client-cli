use std::path::Path;

use anyhow::{Context, Result};

pub fn resolve_token(token_file: Option<&Path>, token_arg: Option<&str>) -> Result<String> {
    if let Some(token) = token_arg {
        return normalize(token);
    }
    if let Some(path) = token_file {
        let token = std::fs::read_to_string(path)
            .with_context(|| format!("reading token file {}", path.display()))?;
        return normalize(&token);
    }
    anyhow::bail!("no API token provided: use --token, --token-file, VIKUNJA_TOKEN, or vkc config");
}

fn normalize(token: &str) -> Result<String> {
    let token = token.trim();
    if token.is_empty() {
        anyhow::bail!("API token is empty after trimming surrounding whitespace");
    }
    Ok(token.to_owned())
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn trims_inline_token() {
        assert_eq!(normalize(" token\n").unwrap(), "token");
    }

    #[test]
    fn rejects_empty_token() {
        assert!(normalize(" \n\t").is_err());
    }
}
