use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::output::OutputFormat;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileConfig {
    pub url: Option<String>,
    pub token: Option<String>,
    pub token_file: Option<PathBuf>,
    pub output: Option<OutputFormat>,
    pub accept_invalid_certs: Option<bool>,
}

#[derive(Debug)]
pub struct ResolvedConfig {
    pub url: String,
    pub token: Option<String>,
    pub token_file: Option<PathBuf>,
    pub output: OutputFormat,
    pub accept_invalid_certs: bool,
}

#[derive(Debug)]
pub struct CliConfig {
    pub config_path: Option<PathBuf>,
    pub url: Option<String>,
    pub token: Option<String>,
    pub token_file: Option<PathBuf>,
    pub output: Option<OutputFormat>,
    pub accept_invalid_certs: Option<bool>,
}

pub fn resolve(cli: CliConfig) -> Result<ResolvedConfig> {
    let file = load_config(cli.config_path.as_deref())?;
    let url = cli
        .url
        .or(file.url)
        .context("no Vikunja URL configured: use --url, VIKUNJA_URL, or vkc config url")?;

    Ok(ResolvedConfig {
        url,
        token: cli.token.or(file.token),
        token_file: cli.token_file.or(file.token_file),
        output: cli.output.or(file.output).unwrap_or_default(),
        accept_invalid_certs: cli
            .accept_invalid_certs
            .or(file.accept_invalid_certs)
            .unwrap_or(false),
    })
}

fn load_config(explicit: Option<&Path>) -> Result<FileConfig> {
    let Some(path) = explicit.map(PathBuf::from).or_else(default_config_path) else {
        return Ok(FileConfig::default());
    };

    if !path.exists() {
        return Ok(FileConfig::default());
    }

    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading config {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing config {}", path.display()))
}

fn default_config_path() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("vkc/config.toml"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/vkc/config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_config_file_is_empty() {
        let cfg = load_config(Some(Path::new("/definitely/missing/vkc.toml"))).unwrap();
        assert!(cfg.url.is_none());
    }

    #[test]
    fn cli_values_override_file_values() {
        let dir = tempfile_dir();
        let config_path = dir.join("config.toml");
        std::fs::write(
            &config_path,
            r#"
url = "https://file.example/api/v1"
token = "file-token"
token_file = "/run/agenix/file-token"
output = "table"
accept_invalid_certs = true
"#,
        )
        .unwrap();

        let cfg = resolve(CliConfig {
            config_path: Some(config_path),
            url: Some("https://cli.example/api/v1".to_string()),
            token: Some("cli-token".to_string()),
            token_file: Some(PathBuf::from("/run/agenix/cli-token")),
            output: Some(OutputFormat::Json),
            accept_invalid_certs: Some(false),
        })
        .unwrap();

        assert_eq!(cfg.url, "https://cli.example/api/v1");
        assert_eq!(cfg.token.as_deref(), Some("cli-token"));
        assert_eq!(
            cfg.token_file.as_deref(),
            Some(Path::new("/run/agenix/cli-token"))
        );
        assert_eq!(cfg.output, OutputFormat::Json);
        assert!(!cfg.accept_invalid_certs);
    }

    #[test]
    fn file_values_are_used_when_cli_values_are_absent() {
        let dir = tempfile_dir();
        let config_path = dir.join("config.toml");
        std::fs::write(
            &config_path,
            r#"
url = "https://file.example/api/v1"
token_file = "/run/agenix/file-token"
output = "table"
"#,
        )
        .unwrap();

        let cfg = resolve(CliConfig {
            config_path: Some(config_path),
            url: None,
            token: None,
            token_file: None,
            output: None,
            accept_invalid_certs: None,
        })
        .unwrap();

        assert_eq!(cfg.url, "https://file.example/api/v1");
        assert_eq!(
            cfg.token_file.as_deref(),
            Some(Path::new("/run/agenix/file-token"))
        );
        assert_eq!(cfg.output, OutputFormat::Table);
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vkc-config-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
