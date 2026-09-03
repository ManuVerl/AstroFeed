use crate::app::{AstroFeedApp, CategoryFilter};
use crate::model::event::{Category, Event};
use egui::Ui;

pub fn show(app: &mut AstroFeedApp, ui: &mut Ui) {
    let events = app.events.lock().unwrap().clone();

    let filtered: Vec<&Event> = events
        .iter()
        .filter(|e| match &app.category_filter {
            CategoryFilter::All => true,
            CategoryFilter::Astronomical => e.category == Category::Astronomical,
            CategoryFilter::RadioAstronomical => e.category == Category::RadioAstronomical,
        })
        .collect();

    if filtered.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("Aucun événement. Appuyez sur 🔄 pour rafraîchir.");
        });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for event in &filtered {
            event_row(ui, event);
            ui.separator();
        }
    });
}

fn event_row(ui: &mut Ui, event: &Event) {
    let icon = match event.category {
        Category::Astronomical => "🔭",
        Category::RadioAstronomical => "📡",
    };

    ui.horizontal(|ui| {
        ui.label(icon);
        ui.vertical(|ui| {
            ui.strong(&event.title);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(
                        event.start_time.format("🗓 %d/%m/%Y  ⏰ %H:%M UTC").to_string(),
                    )
                    .small(),
                );
                if let Some(sky) = &event.sky_position {
                    ui.label(
                        egui::RichText::new(format!(
                            " | Az {:.0}°  El {:.0}°",
                            sky.azimuth_deg, sky.elevation_deg
                        ))
                        .small(),
                    );
                }
                if let (Some(fmin), Some(fmax)) = (event.freq_min_mhz, event.freq_max_mhz) {
                    ui.label(
                        egui::RichText::new(format!(" | {:.1}–{:.1} MHz", fmin, fmax)).small(),
                    );
                }
            });
            if let Some(desc) = &event.description {
                ui.label(egui::RichText::new(desc).small().italics());
            }
        });
    });
}
