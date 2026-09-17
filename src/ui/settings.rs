use crate::app::AstroFeedApp;
use crate::config::{Theme, UpdateFrequency};
use crate::i18n::{self, Lang};
use crate::sources::manager::SOURCE_NAMES;
use egui::{Context, Window};

pub fn show(app: &mut AstroFeedApp, ctx: &Context) {
    let mut open = app.show_settings;
    let mut save_and_close = false;

    let lang = app.settings.language.clone();

    // Take a snapshot of disabled_sources the first time the window opens so we
    // can restore it if the user closes without saving.
    if app.settings_sources_snapshot.is_none() {
        app.settings_sources_snapshot = Some(app.settings.disabled_sources.clone());
    }

    // Use a fixed Id so the window position is stable even when the language (and thus
    // the title string) changes. The displayed title is set via .title_bar(true) and
    // passed through a labelled frame — but egui deduces position from Id, not text.
    Window::new(i18n::t(&lang, "settings.title"))
        .id(egui::Id::new("settings_window"))
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .min_width(420.0)
        .show(ctx, |ui| {
            // ── Appearance ───────────────────────────────────────────────────
            ui.heading(i18n::t(&lang, "settings.appearance"));

            ui.horizontal_wrapped(|ui| {
                ui.label(i18n::t(&lang, "settings.theme"));
                ui.add_space(4.0);

                let was_theme = app.settings.theme.clone();

                for (theme, key) in [
                    (Theme::Dark,  "settings.dark"),
                    (Theme::Light, "settings.light"),
                    (Theme::Teal,  "settings.teal"),
                    (Theme::Pink,  "settings.pink"),
                    (Theme::Navy,  "settings.navy"),
                ] {
                    let selected = app.settings.theme == theme;
                    if ui.selectable_label(selected, i18n::t(&lang, key)).clicked() {
                        app.settings.theme = theme;
                    }
                }

                if app.settings.theme != was_theme {
                    crate::ui::theme::apply_theme(ctx, &app.settings.theme);
                    app.settings.save();
                }
            });

            ui.separator();

            // ── Language ─────────────────────────────────────────────────────
            ui.heading(i18n::t(&lang, "settings.language"));
            ui.horizontal_wrapped(|ui| {
                for l in [Lang::Fr, Lang::En, Lang::Es, Lang::Pt, Lang::De, Lang::It] {
                    let selected = app.settings.language == l;
                    if ui.selectable_label(selected, l.label()).clicked() {
                        app.settings.language = l;
                        // Persist immediately so the choice survives without clicking Save
                        app.settings.save();
                    }
                }
            });

            ui.separator();

            // ── Update frequency ─────────────────────────────────────────────
            ui.heading(i18n::t(&lang, "settings.update"));
            ui.label(i18n::t(&lang, "settings.freq_label"));
            ui.selectable_value(&mut app.settings.update_frequency, UpdateFrequency::OnStartup, i18n::t(&lang, "settings.startup"));
            ui.selectable_value(&mut app.settings.update_frequency, UpdateFrequency::Weekly,    i18n::t(&lang, "settings.weekly"));
            ui.selectable_value(&mut app.settings.update_frequency, UpdateFrequency::Monthly,   i18n::t(&lang, "settings.monthly"));

            ui.separator();

            // ── Data sources ─────────────────────────────────────────────────
            ui.heading(i18n::t(&lang, "settings.sources"));
            ui.label(
                egui::RichText::new(i18n::t(&lang, "settings.sources_hint"))
                    .small()
                    .italics(),
            );
            ui.add_space(4.0);

            for &name in SOURCE_NAMES {
                let mut enabled = !app.settings.disabled_sources.contains(&name.to_string());
                if ui.checkbox(&mut enabled, name).changed() {
                    if enabled {
                        app.settings.disabled_sources.retain(|s| s != name);
                    } else {
                        app.settings.disabled_sources.push(name.to_string());
                    }
                }
            }

            ui.separator();
            if ui.button(i18n::t(&lang, "settings.save")).clicked() {
                app.settings.positions = app.positions.clone();
                app.settings.save();
                // Clear snapshot — changes are committed
                app.settings_sources_snapshot = None;
                save_and_close = true;
            }
        });

    let was_open = app.show_settings;
    app.show_settings = open && !save_and_close;

    if save_and_close {
        // Sources selection was saved: trigger a refresh so the event list reflects
        // the new set of active sources immediately.
        app.trigger_refresh();
    } else if was_open && !open {
        // Window was closed via the ✕ button (no save): restore the snapshot.
        if let Some(snapshot) = app.settings_sources_snapshot.take() {
            app.settings.disabled_sources = snapshot;
        }
    }

    // Reset snapshot when the window is fully closed so it is taken fresh on next open.
    if !app.show_settings {
        app.settings_sources_snapshot = None;
    }
}
