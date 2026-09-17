use crate::app::AstroFeedApp;
use crate::i18n;
use crate::model::position::{CoordAcquisition, Position, PositionIcon};
use egui::{Context, Window};

// ── Temporary form state stored in egui memory ────────────────────────────────

#[derive(Clone, Default)]
struct FormState {
    /// None = "add new"; Some(idx) = "editing existing position at index"
    editing_index: Option<usize>,
    icon: PositionIcon,
    name: String,
    lat_s: String,
    lon_s: String,
    /// The active_position_index that was current before the user clicked a row
    /// to edit it. Restored when Cancel is pressed.
    prev_active_index: usize,
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn show(app: &mut AstroFeedApp, ctx: &Context) {
    let mut open = app.show_positions;
    let lang = app.settings.language.clone();
    let form_id = egui::Id::new("positions_form_state");

    // Load form state from egui memory
    let mut form: FormState = ctx.data_mut(|d| d.get_temp(form_id).unwrap_or_default());

    Window::new(i18n::t(&lang, "positions.title"))
        .id(egui::Id::new("positions_window"))
        .open(&mut open)
        .resizable(true)
        .collapsible(false)
        .min_width(480.0)
        .show(ctx, |ui| {

            // ── Position list ─────────────────────────────────────────────────
            // Snapshot so we can mutate app freely after the loop.
            let rows: Vec<(usize, String, String, f64, f64, PositionIcon)> = app.positions
                .iter()
                .enumerate()
                .map(|(i, p)| (i, p.icon.emoji().to_string(), p.name.clone(),
                               p.latitude, p.longitude, p.icon.clone()))
                .collect();

            let mut to_delete: Option<usize> = None;
            let mut clicked_idx: Option<(usize, PositionIcon, String, f64, f64)> = None;

            for (i, emoji, name, lat, lon, icon) in &rows {
                let i = *i;
                ui.horizontal(|ui| {
                    // Show emoji + user-given name only (no "Home / Observatory / …")
                    let label = format!("{} {}", emoji, name);
                    let selected = i == app.active_position_index;

                    if ui.selectable_label(selected, &label).clicked() {
                        clicked_idx = Some((i, icon.clone(), name.clone(), *lat, *lon));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("🗑")
                            .on_hover_text(i18n::t(&lang, "positions.delete_tooltip"))
                            .clicked()
                        {
                            to_delete = Some(i);
                        }
                        ui.label(
                            egui::RichText::new(format!("({:.4}°, {:.4}°)", lat, lon))
                                .weak().small(),
                        );
                    });
                });
            }

            // Deferred: handle click (set active + load form)
            if let Some((i, icon, name, lat, lon)) = clicked_idx {
                form.prev_active_index = app.active_position_index;
                app.active_position_index = i;
                form.editing_index = Some(i);
                form.icon  = icon;
                form.name  = name;
                form.lat_s = format!("{}", lat);
                form.lon_s = format!("{}", lon);
                save_positions(app);
            }

            // Deferred: handle delete
            if let Some(idx) = to_delete {
                if form.editing_index == Some(idx) {
                    form = FormState::default();
                } else if let Some(ei) = form.editing_index {
                    if idx < ei { form.editing_index = Some(ei - 1); }
                }
                app.positions.remove(idx);
                if app.positions.is_empty() {
                    app.active_position_index = 0;
                } else if app.active_position_index >= app.positions.len() {
                    app.active_position_index = app.positions.len() - 1;
                }
                save_positions(app);
            }

            ui.separator();

            // ── Add / Edit form ───────────────────────────────────────────────
            let is_editing = form.editing_index.is_some();
            ui.label(if is_editing {
                i18n::t(&lang, "positions.edit_label")
            } else {
                i18n::t(&lang, "positions.add_label")
            });

            // Row 1: Icon picker
            ui.horizontal(|ui| {
                ui.label(i18n::t(&lang, "positions.icon_label"));
                egui::ComboBox::from_id_source("pos_icon_picker")
                    .selected_text(form.icon.picker_label())
                    .show_ui(ui, |ui| {
                        for icon in PositionIcon::all() {
                            ui.selectable_value(
                                &mut form.icon,
                                icon.clone(),
                                icon.picker_label(),
                            );
                        }
                    });
            });

            // Row 2: Name
            ui.horizontal(|ui| {
                ui.label(i18n::t(&lang, "positions.name"));
                ui.add(egui::TextEdit::singleline(&mut form.name).desired_width(200.0));
            });

            // Row 3: Lat / Lon
            ui.horizontal(|ui| {
                ui.label(i18n::t(&lang, "positions.lat"));
                ui.add(egui::TextEdit::singleline(&mut form.lat_s).desired_width(90.0));
                ui.label(i18n::t(&lang, "positions.lon"));
                ui.add(egui::TextEdit::singleline(&mut form.lon_s).desired_width(90.0));
            });

            // Row 4: Buttons
            ui.horizontal(|ui| {
                let action_label = if is_editing {
                    i18n::t(&lang, "positions.save_btn")
                } else {
                    i18n::t(&lang, "positions.add_btn")
                };

                if ui.button(action_label).clicked() {
                    if let (Ok(lat), Ok(lon)) = (
                        form.lat_s.trim().parse::<f64>(),
                        form.lon_s.trim().parse::<f64>(),
                    ) {
                        let name = if form.name.trim().is_empty() {
                            i18n::t(&lang, "positions.default_name").to_string()
                        } else {
                            form.name.trim().to_string()
                        };
                        let new_pos = Position {
                            name,
                            icon: form.icon.clone(),
                            latitude: lat,
                            longitude: lon,
                            acquisition: CoordAcquisition::Manual,
                        };

                        if let Some(idx) = form.editing_index {
                            if let Some(existing) = app.positions.get_mut(idx) {
                                *existing = new_pos;
                            }
                        } else {
                            app.positions.push(new_pos);
                            app.active_position_index = app.positions.len() - 1;
                        }

                        save_positions(app);
                        form = FormState::default();
                    }
                }

                if is_editing && ui.button(i18n::t(&lang, "positions.cancel_btn")).clicked() {
                    // Restore the selection that existed before the user clicked the row
                    app.active_position_index = form.prev_active_index;
                    app.settings.active_position_index = form.prev_active_index;
                    app.settings.save();
                    form = FormState::default();
                }
            });
        });

    // Persist form state back into egui memory
    ctx.data_mut(|d| d.insert_temp(form_id, form));

    app.show_positions = open;
}

// ── Save helper ───────────────────────────────────────────────────────────────

/// Sync positions + active index into settings and flush to disk immediately.
fn save_positions(app: &mut AstroFeedApp) {
    app.settings.positions = app.positions.clone();
    app.settings.active_position_index = app.active_position_index;
    app.settings.save();
}
