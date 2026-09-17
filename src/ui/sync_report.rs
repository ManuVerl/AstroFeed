use crate::app::AstroFeedApp;
use crate::i18n;
use crate::sources::manager::SyncStatus;
use egui::{Context, Window};

pub fn show(app: &mut AstroFeedApp, ctx: &Context) {
    let mut open = app.show_sync_report;
    let lang = app.settings.language.clone();

    Window::new(i18n::t(&lang, "sync.title"))
        .open(&mut open)
        .resizable(true)
        .collapsible(false)
        .min_width(480.0)
        .show(ctx, |ui| {
            let report = app.sync_report.lock().unwrap().clone();

            if report.is_empty() {
                ui.label(i18n::t(&lang, "sync.none"));
                return;
            }

            egui::Grid::new("sync_report_grid")
                .num_columns(3)
                .striped(true)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.strong(i18n::t(&lang, "sync.col_source"));
                    ui.strong(i18n::t(&lang, "sync.col_status"));
                    ui.strong(i18n::t(&lang, "sync.col_last"));
                    ui.end_row();

                    for entry in &report {
                        ui.label(&entry.source_name);

                        match entry.status {
                            SyncStatus::Ok => {
                                ui.colored_label(egui::Color32::from_rgb(80, 200, 80), i18n::t(&lang, "sync.ok"));
                            }
                            SyncStatus::Error => {
                                ui.colored_label(egui::Color32::from_rgb(220, 80, 80), i18n::t(&lang, "sync.error"));
                            }
                            SyncStatus::Pending => {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(i18n::t(&lang, "sync.pending"));
                                });
                            }
                            SyncStatus::Disabled => {
                                ui.colored_label(egui::Color32::from_rgb(140, 140, 140), i18n::t(&lang, "sync.disabled"));
                            }
                        }

                        if let Some(ts) = entry.last_sync {
                            // Use localised date/time format
                            let formatted = i18n::format_event_datetime(&lang, &ts);
                            ui.label(formatted);
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
            if ui.button(i18n::t(&lang, "sync.refresh_now")).clicked() {
                app.trigger_refresh();
            }
        });

    app.show_sync_report = open;
}
