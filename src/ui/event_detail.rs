use crate::model::event::{Category, Event};
use egui::{Context, Window};

/// Show the detail panel for a single event (call from event_list when an event is selected).
#[allow(dead_code)]
pub fn show(event: &Event, open: &mut bool, ctx: &Context) {
    Window::new(&event.title)
        .open(open)
        .resizable(true)
        .min_width(360.0)
        .show(ctx, |ui| {
            let icon = match event.category {
                Category::Astronomical => "🔭 Astronomie",
                Category::RadioAstronomical => "📡 Radioastronomie",
            };
            ui.label(icon);
            ui.separator();

            egui::Grid::new("event_detail_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Début");
                    ui.label(event.start_time.format("%d/%m/%Y %H:%M UTC").to_string());
                    ui.end_row();

                    ui.label("Fin");
                    ui.label(event.end_time.format("%d/%m/%Y %H:%M UTC").to_string());
                    ui.end_row();

                    if let Some(sky) = &event.sky_position {
                        ui.label("Position");
                        ui.label(format!("Az {:.1}°  El {:.1}°", sky.azimuth_deg, sky.elevation_deg));
                        ui.end_row();
                    }

                    if let Some(eq) = &event.equipment {
                        ui.label("Équipement");
                        ui.label(eq);
                        ui.end_row();
                    }

                    if let (Some(fmin), Some(fmax)) = (event.freq_min_mhz, event.freq_max_mhz) {
                        ui.label("Fréquences");
                        ui.label(format!("{:.1} – {:.1} MHz", fmin, fmax));
                        ui.end_row();
                    }

                    if let Some(dir) = &event.listen_direction {
                        ui.label("Dir. d'écoute");
                        ui.label(format!("Az {:.1}°  El {:.1}°", dir.azimuth_deg, dir.elevation_deg));
                        ui.end_row();
                    }

                    ui.label("Source");
                    ui.label(&event.source);
                    ui.end_row();
                });

            if let Some(desc) = &event.description {
                ui.separator();
                ui.label(desc);
            }
        });
}
