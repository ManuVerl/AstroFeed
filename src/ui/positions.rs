use crate::app::AstroFeedApp;
use crate::model::position::{CoordAcquisition, Position, PositionIcon};
use egui::{Context, Window};

pub fn show(app: &mut AstroFeedApp, ctx: &Context) {
    let mut open = app.show_positions;

    Window::new("📍 Gestion des positions")
        .open(&mut open)
        .resizable(true)
        .min_width(420.0)
        .show(ctx, |ui| {
            // List existing positions
            let mut to_delete: Option<usize> = None;

            for (i, pos) in app.positions.iter().enumerate() {
                ui.horizontal(|ui| {
                    let selected = i == app.active_position_index;
                    if ui.selectable_label(selected, format!("{} {}", pos.icon.label(), pos.name)).clicked() {
                        app.active_position_index = i;
                    }
                    ui.label(format!("({:.4}°, {:.4}°)", pos.latitude, pos.longitude));
                    if ui.small_button("🗑").on_hover_text("Supprimer").clicked() {
                        to_delete = Some(i);
                    }
                });
            }

            if let Some(idx) = to_delete {
                app.positions.remove(idx);
                if app.active_position_index >= app.positions.len() && !app.positions.is_empty() {
                    app.active_position_index = app.positions.len() - 1;
                }
            }

            ui.separator();
            ui.label("Ajouter une position :");

            // Simple inline form stored in a temporary state via egui memory
            ui.horizontal(|ui| {
                // We store form state in egui's Id-based memory
                let id = ui.id().with("new_pos");
                let mut name = ui.data_mut(|d| d.get_temp::<String>(id.with("name")).unwrap_or_default());
                let mut lat_s = ui.data_mut(|d| d.get_temp::<String>(id.with("lat")).unwrap_or_default());
                let mut lon_s = ui.data_mut(|d| d.get_temp::<String>(id.with("lon")).unwrap_or_default());

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Nom :");
                        ui.text_edit_singleline(&mut name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Lat :");
                        ui.add(egui::TextEdit::singleline(&mut lat_s).desired_width(80.0));
                        ui.label("Lon :");
                        ui.add(egui::TextEdit::singleline(&mut lon_s).desired_width(80.0));
                    });
                    if ui.button("➕ Ajouter").clicked() {
                        if let (Ok(lat), Ok(lon)) = (lat_s.parse::<f64>(), lon_s.parse::<f64>()) {
                            app.positions.push(Position {
                                name: if name.is_empty() { "Nouvelle position".to_string() } else { name.clone() },
                                icon: PositionIcon::Home,
                                latitude: lat,
                                longitude: lon,
                                acquisition: CoordAcquisition::Manual,
                            });
                            name.clear(); lat_s.clear(); lon_s.clear();
                        }
                    }
                });

                ui.data_mut(|d| {
                    d.insert_temp(id.with("name"), name);
                    d.insert_temp(id.with("lat"), lat_s);
                    d.insert_temp(id.with("lon"), lon_s);
                });
            });
        });

    app.show_positions = open;
}
