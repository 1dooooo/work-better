//! Collector configuration building and validation.

use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for a single collector source.
#[derive(Debug, Clone)]
pub struct CollectorSourceConfig {
    pub id: String,
    pub enabled: bool,
    pub health_mode: HealthMode,
}

/// How health checks are performed for a collector.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthMode {
    /// Automatic health check via tool availability / API probe.
    Auto,
    /// Manual health status set by the operator.
    Manual(bool),
}

/// Top-level collector configuration, built via [`CollectorConfigBuilder`].
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    pub sources: Vec<CollectorSourceConfig>,
    pub vault_path: PathBuf,
}

/// Builder for [`CollectorConfig`] with validation.
#[derive(Debug, Default)]
pub struct CollectorConfigBuilder {
    sources: HashMap<String, CollectorSourceConfig>,
    vault_path: Option<PathBuf>,
}

impl CollectorConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable a collector source by id. Creates it if it doesn't exist.
    pub fn enable(&mut self, id: &str) -> &mut Self {
        self.sources
            .entry(id.to_string())
            .or_insert_with(|| CollectorSourceConfig {
                id: id.to_string(),
                enabled: false,
                health_mode: HealthMode::Auto,
            })
            .enabled = true;
        self
    }

    /// Disable a collector source by id. Creates it if it doesn't exist.
    pub fn disable(&mut self, id: &str) -> &mut Self {
        self.sources
            .entry(id.to_string())
            .or_insert_with(|| CollectorSourceConfig {
                id: id.to_string(),
                enabled: false,
                health_mode: HealthMode::Auto,
            })
            .enabled = false;
        self
    }

    /// Set manual health status for a collector source.
    pub fn set_manual_health(&mut self, id: &str, healthy: bool) -> &mut Self {
        self.sources
            .entry(id.to_string())
            .or_insert_with(|| CollectorSourceConfig {
                id: id.to_string(),
                enabled: false,
                health_mode: HealthMode::Auto,
            })
            .health_mode = HealthMode::Manual(healthy);
        self
    }

    /// Set the Obsidian vault path. Must be an absolute path.
    pub fn vault_path(&mut self, path: PathBuf) -> &mut Self {
        self.vault_path = Some(path);
        self
    }

    /// Build and validate the configuration.
    ///
    /// Returns `Err` if:
    /// - The vault path is missing
    /// - The vault path is not absolute
    pub fn build(&self) -> Result<CollectorConfig, ConfigError> {
        let vault_path = self
            .vault_path
            .clone()
            .ok_or(ConfigError::MissingVaultPath)?;

        if !vault_path.is_absolute() {
            return Err(ConfigError::InvalidVaultPath(vault_path));
        }

        Ok(CollectorConfig {
            sources: self.sources.values().cloned().collect(),
            vault_path,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    MissingVaultPath,
    InvalidVaultPath(PathBuf),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVaultPath => write!(f, "vault path is required"),
            ConfigError::InvalidVaultPath(p) => {
                write!(f, "vault path must be absolute: {}", p.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    // ── A12-01: Feishu enabled ────────────────────────────────────────
    #[test]
    fn a12_01_feishu_enabled() {
        let config = CollectorConfigBuilder::new()
            .enable("feishu")
            .vault_path(PathBuf::from("/tmp/vault"))
            .build()
            .unwrap();

        assert_eq!(config.sources.len(), 1);
        let feishu = config.sources.iter().find(|s| s.id == "feishu").unwrap();
        assert!(feishu.enabled);
    }

    // ── A12-02: All disabled ──────────────────────────────────────────
    #[test]
    fn a12_02_all_disabled() {
        let config = CollectorConfigBuilder::new()
            .disable("feishu")
            .disable("browser")
            .disable("app_switch")
            .vault_path(PathBuf::from("/tmp/vault"))
            .build()
            .unwrap();

        assert_eq!(config.sources.len(), 3);
        assert!(config.sources.iter().all(|s| !s.enabled));
    }

    // ── A12-03: Path validation ───────────────────────────────────────
    #[test]
    fn a12_03_missing_path_fails() {
        let result = CollectorConfigBuilder::new().enable("feishu").build();

        assert_eq!(result.unwrap_err(), ConfigError::MissingVaultPath);
    }

    #[test]
    fn a12_03_relative_path_fails() {
        let result = CollectorConfigBuilder::new()
            .enable("feishu")
            .vault_path(PathBuf::from("relative/path"))
            .build();

        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidVaultPath(_)
        ));
    }

    #[test]
    fn a12_03_absolute_path_succeeds() {
        let result = CollectorConfigBuilder::new()
            .enable("feishu")
            .vault_path(PathBuf::from("/absolute/path"))
            .build();

        assert!(result.is_ok());
    }

    // ── A12-04: Manual health ─────────────────────────────────────────
    #[test]
    fn a12_04_manual_health_healthy() {
        let config = CollectorConfigBuilder::new()
            .enable("feishu")
            .set_manual_health("feishu", true)
            .vault_path(PathBuf::from("/tmp/vault"))
            .build()
            .unwrap();

        let feishu = config.sources.iter().find(|s| s.id == "feishu").unwrap();
        assert_eq!(feishu.health_mode, HealthMode::Manual(true));
    }

    #[test]
    fn a12_04_manual_health_unhealthy() {
        let config = CollectorConfigBuilder::new()
            .enable("feishu")
            .set_manual_health("feishu", false)
            .vault_path(PathBuf::from("/tmp/vault"))
            .build()
            .unwrap();

        let feishu = config.sources.iter().find(|s| s.id == "feishu").unwrap();
        assert_eq!(feishu.health_mode, HealthMode::Manual(false));
    }

    #[test]
    fn a12_04_default_health_is_auto() {
        let config = CollectorConfigBuilder::new()
            .enable("feishu")
            .vault_path(PathBuf::from("/tmp/vault"))
            .build()
            .unwrap();

        let feishu = config.sources.iter().find(|s| s.id == "feishu").unwrap();
        assert_eq!(feishu.health_mode, HealthMode::Auto);
    }

    // ── Additional: mixed enable/disable ──────────────────────────────
    #[test]
    fn mixed_enable_disable() {
        let config = CollectorConfigBuilder::new()
            .enable("feishu")
            .disable("browser")
            .enable("app_switch")
            .vault_path(PathBuf::from("/tmp/vault"))
            .build()
            .unwrap();

        let by_id: HashMap<&str, bool> = config
            .sources
            .iter()
            .map(|s| (s.id.as_str(), s.enabled))
            .collect();

        assert_eq!(by_id.get("feishu"), Some(&true));
        assert_eq!(by_id.get("browser"), Some(&false));
        assert_eq!(by_id.get("app_switch"), Some(&true));
    }
}
