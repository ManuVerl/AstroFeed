use crate::app::AstroFeedApp;
use crate::sources::manager::SyncStatus;
use egui::{Context, Window};

pub fn show(app: &mut AstroFeedApp, ctx: &Context) {
    let mut open = app.show_sync_report;

    Window::new("📋 Rapport de synchronisation")
        .open(&mut open)
        .resizable(true)
        .min_width(480.0)
        .show(ctx, |ui| {
            let report = app.sync_report.lock().unwrap().clone();

            if report.is_empty() {
                ui.label("Aucune synchronisation effectuée.");
                return;
            }

            egui::Grid::new("sync_report_grid")
                .num_columns(3)
                .striped(true)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.strong("Source");
                    ui.strong("Statut");
                    ui.strong("Dernière sync");
                    ui.end_row();

                    for entry in &report {
                        ui.label(&entry.source_name);

                        match entry.status {
                            SyncStatus::Ok => {
                                ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "✅ OK");
                            }
                            SyncStatus::Error => {
                                ui.colored_label(egui::Color32::from_rgb(220, 80, 80), "❌ Erreur");
                            }
                            SyncStatus::Pending => {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label("En cours…");
                                });
                            }
                        }

                        if let Some(ts) = entry.last_sync {
                            ui.label(ts.format("%d/%m/%Y %H:%M UTC").to_string());
                        } else {
                            ui.label("—");
                        }
                        ui.end_row();

                        // Show error message on next row if present
                        if let Some(err) = &entry.error_message {
                            ui.label(""); // source col
                            ui.colored_label(
                                egui::Color32::from_rgb(220, 80, 80),
                                egui::RichText::new(err).small().italics(),
                            );
                            ui.label(""); // date col
                            ui.end_row();
                        }
                    }
                });

            ui.separator();
            if ui.button("🔄 Rafraîchir maintenant").clicked() {
                app.trigger_refresh();
            }
        });

    app.show_sync_report = open;
}
