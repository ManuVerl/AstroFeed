use crate::model::position::{Position, PositionIcon};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILE: &str = "astrofeed_config.toml";

/// How often events are automatically refreshed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum UpdateFrequency {
    /// Refresh every time the application starts (default)
    #[default]
    OnStartup,
    /// Refresh at most once per week
    Weekly,
    /// Refresh at most once per month
    Monthly,
}

/// Persisted application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub dark_mode: bool,
    pub update_frequency: UpdateFrequency,
    pub positions: Vec<Position>,
    /// UTC timestamp of the last successful refresh
    pub last_refresh: Option<DateTime<Utc>>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            update_frequency: UpdateFrequency::OnStartup,
            positions: vec![Position::new_manual(
                "Domicile",
                PositionIcon::Home,
                48.8566,
                2.3522,
            )],
            last_refresh: None,
        }
    }
}

impl Settings {
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("astrofeed")
            .join(CONFIG_FILE)
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(contents) = std::fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(contents) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, contents);
        }
    }

    /// Returns true if a weekly refresh is due.
    pub fn should_refresh_weekly(&self) -> bool {
        self.last_refresh
            .map(|t| Utc::now() - t > chrono::Duration::weeks(1))
            .unwrap_or(true)
    }

    /// Returns true if a monthly refresh is due.
    pub fn should_refresh_monthly(&self) -> bool {
        self.last_refresh
            .map(|t| Utc::now() - t > chrono::Duration::days(30))
            .unwrap_or(true)
    }
}
