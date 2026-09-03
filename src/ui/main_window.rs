use crate::app::{AstroFeedApp, CategoryFilter};
use egui::{Context, TopBottomPanel, CentralPanel, SidePanel};

// ── Starfield ─────────────────────────────────────────────────────────────────

/// Number of background stars to paint.
const STAR_COUNT: usize = 160;

/// Simple LCG to generate stable pseudo-random star positions (no external dep).
fn lcg_next(state: &mut u64) -> f32 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    // Map to [0, 1)
    (*state >> 33) as f32 / (u32::MAX as f32)
}

/// Paint a subtle starfield that fills `rect` using the egui Painter.
fn paint_starfield(painter: &egui::Painter, rect: egui::Rect, dark_mode: bool) {
    if !dark_mode {
        // Stars don't make sense on a light background
        return;
    }

    let mut rng: u64 = 0xDEAD_CAFE_1234_5678;

    for _ in 0..STAR_COUNT {
        let rx = lcg_next(&mut rng);
        let ry = lcg_next(&mut rng);
        let size_r = lcg_next(&mut rng);
        let alpha_r = lcg_next(&mut rng);

        let x = rect.min.x + rx * rect.width();
        let y = rect.min.y + ry * rect.height();

        // Radius: mostly tiny (0.5–1 px), occasionally slightly larger
        let radius = if size_r > 0.95 { 1.5 } else if size_r > 0.85 { 1.0 } else { 0.5 };

        // Alpha: very faint (15–50) so they don't compete with the content
        let alpha = (15.0 + alpha_r * 35.0) as u8;

        // Colour: slightly warm white or cold blue
        let (r, g, b) = if alpha_r > 0.6 { (200u8, 210u8, 255u8) } else { (255u8, 250u8, 230u8) };

        painter.circle_filled(
            egui::pos2(x, y),
            radius,
            egui::Color32::from_rgba_unmultiplied(r, g, b, alpha),
        );
    }
}

pub fn show(app: &mut AstroFeedApp, ctx: &Context, _frame: &mut eframe::Frame) {
    // ── Top menu bar ──────────────────────────────────────────────────────────
    TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Fichier", |ui| {
                if ui.button("🔄  Rafraîchir").clicked() {
                    app.trigger_refresh();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Quitter").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Édition", |ui| {
                if ui.button("📍  Positions").clicked() {
                    app.show_positions = true;
                    ui.close_menu();
                }
                if ui.button("⚙️  Paramètres").clicked() {
                    app.show_settings = true;
                    ui.close_menu();
                }
            });
            ui.menu_button("Aide", |ui| {
                if ui.button("À propos").clicked() {
                    // TODO: about dialog
                    ui.close_menu();
                }
            });

            // Spacer then position selector
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄").on_hover_text("Rafraîchir les événements").clicked() {
                    app.trigger_refresh();
                }
                ui.separator();
                egui::ComboBox::from_id_source("position_selector")
                    .selected_text(
                        app.active_position()
                            .map(|p| p.name.clone())
                            .unwrap_or_else(|| "— aucune position —".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for (i, pos) in app.positions.iter().enumerate() {
                            ui.selectable_value(
                                &mut app.active_position_index,
                                i,
                                format!("{} {}", pos.icon.label(), pos.name),
                            );
                        }
                    });
                ui.label("📍");
            });
        });
    });

    // ── Status bar ────────────────────────────────────────────────────────────
    TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let report = app.sync_report.lock().unwrap();
            let errors = report.iter().filter(|r| {
                r.status == crate::sources::manager::SyncStatus::Error
            }).count();
            let pending = report.iter().filter(|r| {
                r.status == crate::sources::manager::SyncStatus::Pending
            }).count();

            if pending > 0 {
                ui.spinner();
                ui.label(format!("Synchronisation en cours ({} source(s))…", pending));
            } else if errors > 0 {
                ui.colored_label(egui::Color32::from_rgb(220, 80, 80),
                    format!("⚠ {} source(s) en erreur", errors));
            } else if !report.is_empty() {
                ui.label("✅ Synchronisation OK");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("📋 Rapport sync").clicked() {
                    app.show_sync_report = !app.show_sync_report;
                }
            });
        });
    });

    // ── Filter sidebar ────────────────────────────────────────────────────────
    SidePanel::left("filter_panel").resizable(false).min_width(160.0).show(ctx, |ui| {
        ui.separator();
        ui.selectable_value(&mut app.category_filter, CategoryFilter::All, "Tous");
        ui.selectable_value(&mut app.category_filter, CategoryFilter::Astronomical, "🔭 Astronomie");
        ui.selectable_value(&mut app.category_filter, CategoryFilter::RadioAstronomical, "📡 Radioastronomie");
        ui.separator();
        // TODO: sub-type filters, time filters
    });

    // ── Main event list (with starfield background) ───────────────────────────
    CentralPanel::default().show(ctx, |ui| {
        // Paint stars behind everything else
        let rect = ui.max_rect();
        paint_starfield(ui.painter(), rect, app.settings.dark_mode);

        crate::ui::event_list::show(app, ui);
    });

    // ── Modal panels ─────────────────────────────────────────────────────────
    if app.show_sync_report {
        crate::ui::sync_report::show(app, ctx);
    }
    if app.show_settings {
        crate::ui::settings::show(app, ctx);
    }
    if app.show_positions {
        crate::ui::positions::show(app, ctx);
    }
}
