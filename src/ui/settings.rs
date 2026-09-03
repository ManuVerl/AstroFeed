use crate::app::AstroFeedApp;
use crate::config::UpdateFrequency;
use egui::{Context, Window};

pub fn show(app: &mut AstroFeedApp, ctx: &Context) {
    let mut open = app.show_settings;
    // Flag set inside the closure to close after save (avoids double-borrow of `open`)
    let mut save_and_close = false;

    Window::new("⚙️ Paramètres")
        .open(&mut open)
        .resizable(false)
        .min_width(320.0)
        .show(ctx, |ui| {
            ui.heading("Apparence");
            ui.horizontal(|ui| {
                ui.label("Thème :");
                let was_dark = app.settings.dark_mode;
                ui.selectable_value(&mut app.settings.dark_mode, true, "🌑 Sombre");
                ui.selectable_value(&mut app.settings.dark_mode, false, "☀️ Clair");
                if app.settings.dark_mode != was_dark {
                    if app.settings.dark_mode {
                        ctx.set_visuals(egui::Visuals::dark());
                    } else {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                }
            });

            ui.separator();
            ui.heading("Mise à jour");
            ui.label("Fréquence de rafraîchissement des événements :");
            ui.selectable_value(&mut app.settings.update_frequency, UpdateFrequency::OnStartup, "À chaque démarrage");
            ui.selectable_value(&mut app.settings.update_frequency, UpdateFrequency::Weekly, "Une fois par semaine");
            ui.selectable_value(&mut app.settings.update_frequency, UpdateFrequency::Monthly, "Une fois par mois");

            ui.separator();
            if ui.button("💾 Sauvegarder").clicked() {
                app.settings.positions = app.positions.clone();
                app.settings.save();
                save_and_close = true;
            }
        });

    // Apply close: either user clicked ✕ or clicked Save
    app.show_settings = open && !save_and_close;
}
