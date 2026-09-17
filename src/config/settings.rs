use crate::i18n::Lang;
use crate::model::position::{Position, PositionIcon};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILE: &str = "cosmic_beacon_config.toml";

/// Visual theme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    Teal,
    Pink,
    Navy,
}

impl Theme {
    pub fn is_dark_based(&self) -> bool {
        !matches!(self, Theme::Light)
    }
}

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
    #[serde(default)]
    pub theme: Theme,
    pub update_frequency: UpdateFrequency,
    pub positions: Vec<Position>,
    /// Index of the last active position (restored on next launch)
    #[serde(default)]
    pub active_position_index: usize,
    /// UTC timestamp of the last successful refresh
    pub last_refresh: Option<DateTime<Utc>>,
    /// UI language; defaults to system locale
    #[serde(default)]
    pub language: Lang,
    /// Names of external data sources that the user has disabled.
    /// An empty vec means all sources are active (default behaviour).
    #[serde(default)]
    pub disabled_sources: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            update_frequency: UpdateFrequency::OnStartup,
            positions: vec![Position::new_manual(
                "Home",
                PositionIcon::Home,
                48.8566,
                2.3522,
            )],
            active_position_index: 0,
            last_refresh: None,
            language: Lang::detect_system(),
            disabled_sources: Vec::new(),
        }
    }
}

impl Settings {
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cosmic-beacon")
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
