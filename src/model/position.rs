use serde::{Deserialize, Serialize};

/// Icon representing a position.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum PositionIcon {
    #[default]
    Home,
    Observatory,
    Campsite,
    Station,
    Mountain,
}

impl PositionIcon {
    /// Just the emoji glyph — used in lists, the position selector combobox, and the sidebar.
    pub fn emoji(&self) -> &'static str {
        match self {
            PositionIcon::Home       => "🏠",
            PositionIcon::Observatory => "🔭",
            PositionIcon::Campsite   => "🏕",
            PositionIcon::Station    => "📡",
            PositionIcon::Mountain   => "🏔",
        }
    }

    /// Emoji + English description — used only inside the icon-picker dropdown.
    pub fn picker_label(&self) -> &'static str {
        match self {
            PositionIcon::Home       => "🏠 Home",
            PositionIcon::Observatory => "🔭 Observatory",
            PositionIcon::Campsite   => "🏕 Field / Campsite",
            PositionIcon::Station    => "📡 Station",
            PositionIcon::Mountain   => "🏔 Mountain",
        }
    }

    /// Kept for backwards-compatibility with any call site that still uses label().
    /// Returns the same as emoji().
    pub fn label(&self) -> &'static str {
        self.emoji()
    }

    pub fn all() -> &'static [PositionIcon] {
        &[
            PositionIcon::Home,
            PositionIcon::Observatory,
            PositionIcon::Campsite,
            PositionIcon::Station,
            PositionIcon::Mountain,
        ]
    }
}

/// How the coordinates were acquired.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoordAcquisition {
    Gps,
    Manual,
}

/// A named geographic position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub name: String,
    pub icon: PositionIcon,
    /// Latitude in decimal degrees (−90 … +90)
    pub latitude: f64,
    /// Longitude in decimal degrees (−180 … +180)
    pub longitude: f64,
    pub acquisition: CoordAcquisition,
}

impl Position {
    pub fn new_manual(name: impl Into<String>, icon: PositionIcon, lat: f64, lon: f64) -> Self {
        Self {
            name: name.into(),
            icon,
            latitude: lat,
            longitude: lon,
            acquisition: CoordAcquisition::Manual,
        }
    }

    #[allow(dead_code)]
    pub fn display_label(&self) -> String {
        format!("{} {} ({:.4}°, {:.4}°)", self.icon.emoji(), self.name, self.latitude, self.longitude)
    }
}
