use crate::config::Settings;
use crate::model::{event::Event, position::Position};
use crate::sources::manager::{SourceManager, SyncReport};
use crate::ui::main_window;
use std::sync::{Arc, Mutex};

/// Global application state shared between UI and background tasks.
pub struct AstroFeedApp {
    pub settings: Settings,
    pub positions: Vec<Position>,
    pub active_position_index: usize,
    pub events: Arc<Mutex<Vec<Event>>>,
    pub sync_report: Arc<Mutex<Vec<SyncReport>>>,
    pub show_sync_report: bool,
    pub show_settings: bool,
    pub show_positions: bool,
    pub category_filter: CategoryFilter,
    source_manager: SourceManager,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CategoryFilter {
    #[default]
    All,
    Astronomical,
    RadioAstronomical,
}

impl AstroFeedApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();

        // Apply theme
        if settings.dark_mode {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        let positions = settings.positions.clone();
        let events = Arc::new(Mutex::new(Vec::new()));
        let sync_report = Arc::new(Mutex::new(Vec::new()));

        let source_manager = SourceManager::new(
            Arc::clone(&events),
            Arc::clone(&sync_report),
            cc.egui_ctx.clone(),
        );

        let mut app = Self {
            settings,
            positions,
            active_position_index: 0,
            events,
            sync_report,
            show_sync_report: false,
            show_settings: false,
            show_positions: false,
            category_filter: CategoryFilter::All,
            source_manager,
        };

        // Trigger initial refresh if configured
        app.maybe_refresh_on_startup();
        app
    }

    pub fn active_position(&self) -> Option<&Position> {
        self.positions.get(self.active_position_index)
    }

    pub fn trigger_refresh(&mut self) {
        if let Some(pos) = self.active_position().cloned() {
            self.source_manager.refresh(pos);
        }
    }

    fn maybe_refresh_on_startup(&mut self) {
        use crate::config::UpdateFrequency;
        match self.settings.update_frequency {
            UpdateFrequency::OnStartup => self.trigger_refresh(),
            UpdateFrequency::Weekly => {
                if self.settings.should_refresh_weekly() {
                    self.trigger_refresh();
                }
            }
            UpdateFrequency::Monthly => {
                if self.settings.should_refresh_monthly() {
                    self.trigger_refresh();
                }
            }
        }
    }
}

impl eframe::App for AstroFeedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        main_window::show(self, ctx, frame);
    }
}
