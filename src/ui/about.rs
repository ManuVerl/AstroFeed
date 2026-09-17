use crate::app::AstroFeedApp;
use crate::i18n;
use egui::{Color32, Context, Pos2, Rect, Stroke, Vec2, Window};

/// Draw a larger vector rendering of the application logo (telescope, night sky, radio dish, stars).
fn draw_large_logo(painter: egui::Painter, rect: Rect) {
    // Background rounded rectangle
    painter.rect_filled(rect, 10.0, Color32::from_rgb(10, 14, 34));
    painter.rect_stroke(rect, 10.0, Stroke::new(1.5_f32, Color32::from_rgb(50, 70, 110)));

    let min = rect.min;
    let w = rect.width();
    let h = rect.height();

    // Small stars scattered
    let stars = [
        (0.15, 0.20, 180u8, 1.2),
        (0.25, 0.12, 220u8, 1.5),
        (0.45, 0.25, 140u8, 1.0),
        (0.80, 0.15, 255u8, 2.0),
        (0.88, 0.35, 190u8, 1.3),
        (0.70, 0.45, 160u8, 1.1),
        (0.12, 0.65, 130u8, 1.0),
        (0.30, 0.75, 150u8, 1.2),
    ];

    for (rx, ry, alpha, rad) in stars {
        painter.circle_filled(
            Pos2::new(min.x + rx * w, min.y + ry * h),
            rad,
            Color32::from_rgba_unmultiplied(210, 225, 255, alpha),
        );
    }

    // Main Sparkle star (top-right)
    let star_center = Pos2::new(min.x + 0.82 * w, min.y + 0.18 * h);
    painter.line_segment(
        [Pos2::new(star_center.x - 7.0, star_center.y), Pos2::new(star_center.x + 7.0, star_center.y)],
        Stroke::new(1.5_f32, Color32::from_rgb(255, 235, 160)),
    );
    painter.line_segment(
        [Pos2::new(star_center.x, star_center.y - 7.0), Pos2::new(star_center.x, star_center.y + 7.0)],
        Stroke::new(1.5_f32, Color32::from_rgb(255, 235, 160)),
    );
    painter.circle_filled(star_center, 2.5, Color32::from_rgb(255, 255, 240));

    // Telescope tube
    let t_start = Pos2::new(min.x + 0.18 * w, min.y + 0.32 * h);
    let t_end = Pos2::new(min.x + 0.64 * w, min.y + 0.62 * h);

    // Eyepiece
    painter.line_segment(
        [Pos2::new(t_start.x - 3.0, t_start.y - 3.0), Pos2::new(t_start.x + 3.0, t_start.y + 3.0)],
        Stroke::new(5.0_f32, Color32::from_rgb(220, 230, 245)),
    );

    // Main optical tube
    painter.line_segment([t_start, t_end], Stroke::new(6.0_f32, Color32::from_rgb(170, 195, 225)));

    // Objective ring
    painter.line_segment(
        [Pos2::new(t_end.x - 3.0, t_end.y + 4.0), Pos2::new(t_end.x + 3.0, t_end.y - 4.0)],
        Stroke::new(4.0_f32, Color32::from_rgb(140, 175, 220)),
    );

    // Tripod mount & legs
    let mount = Pos2::new(min.x + 0.44 * w, min.y + 0.49 * h);
    painter.circle_filled(mount, 3.5, Color32::from_rgb(130, 150, 180));
    painter.line_segment(
        [mount, Pos2::new(min.x + 0.28 * w, min.y + 0.88 * h)],
        Stroke::new(2.0_f32, Color32::from_rgb(110, 130, 160)),
    );
    painter.line_segment(
        [mount, Pos2::new(min.x + 0.44 * w, min.y + 0.90 * h)],
        Stroke::new(2.0_f32, Color32::from_rgb(110, 130, 160)),
    );
    painter.line_segment(
        [mount, Pos2::new(min.x + 0.58 * w, min.y + 0.88 * h)],
        Stroke::new(2.0_f32, Color32::from_rgb(110, 130, 160)),
    );

    // Radio dish antenna (lower right corner)
    let dish_base = Pos2::new(min.x + 0.78 * w, min.y + 0.85 * h);
    painter.line_segment(
        [dish_base, Pos2::new(dish_base.x, dish_base.y - 12.0)],
        Stroke::new(2.0_f32, Color32::from_rgb(90, 170, 230)),
    );
    // Dish arc
    let dish_center = Pos2::new(dish_base.x, dish_base.y - 14.0);
    for i in 0..10 {
        let a1 = -0.7 + (i as f32) * 0.14;
        let a2 = -0.7 + ((i + 1) as f32) * 0.14;
        let r = 10.0;
        let p1 = Pos2::new(dish_center.x + r * a1.sin(), dish_center.y - r * a1.cos());
        let p2 = Pos2::new(dish_center.x + r * a2.sin(), dish_center.y - r * a2.cos());
        painter.line_segment([p1, p2], Stroke::new(2.0_f32, Color32::from_rgb(90, 190, 255)));
    }
}

pub fn show(app: &mut AstroFeedApp, ctx: &Context) {
    let mut open = app.show_about;
    let mut should_close = false;
    let lang = app.settings.language.clone();

    Window::new(i18n::t(&lang, "about.title"))
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .min_width(440.0)
        .show(ctx, |ui| {
            // Header with Logo and App Name
            ui.horizontal(|ui| {
                let (rect, _response) = ui.allocate_exact_size(Vec2::new(64.0, 64.0), egui::Sense::hover());
                draw_large_logo(ui.painter_at(rect), rect);

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Cosmic Beacon").size(22.0).strong());
                    ui.label(
                        egui::RichText::new(i18n::t(&lang, "about.subtitle"))
                            .size(11.5)
                            .weak(),
                    );
                    ui.label(egui::RichText::new("Version 0.2.0").size(10.5).weak());
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Authors & Attribution
            ui.heading(i18n::t(&lang, "about.authors_heading"));
            ui.horizontal_wrapped(|ui| {
                ui.label(i18n::t(&lang, "about.developed_by"));
                ui.hyperlink_to(
                    egui::RichText::new("Emmanuel Verlynde").strong(),
                    "https://www.linkedin.com/in/verlynde/",
                );
                ui.label(i18n::t(&lang, "about.with_help"));
                ui.hyperlink_to(
                    egui::RichText::new("BOB (IBM)").strong().color(Color32::from_rgb(60, 140, 240)),
                    "https://www.ibm.com",
                );
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // License
            ui.heading(i18n::t(&lang, "about.license_heading"));
            ui.horizontal_wrapped(|ui| {
                ui.label(i18n::t(&lang, "about.license_text"));
                ui.hyperlink_to(
                    egui::RichText::new(i18n::t(&lang, "about.license_name")).strong(),
                    "https://opensource.org/licenses/MIT",
                );
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // External Sources & Data Providers
            ui.heading(i18n::t(&lang, "about.sources_heading"));
            ui.label(
                egui::RichText::new(i18n::t(&lang, "about.sources_desc"))
                    .size(11.0)
                    .weak(),
            );

            ui.add_space(2.0);

            ui.group(|ui| {
                ui.set_width(ui.available_width());

                ui.horizontal(|ui| {
                    ui.label("🛰");
                    ui.label(egui::RichText::new(i18n::t(&lang, "about.src_iss")).strong());
                    ui.hyperlink_to("CelesTrak", "https://celestrak.org");
                });

                ui.horizontal(|ui| {
                    ui.label("🪐");
                    ui.label(egui::RichText::new(i18n::t(&lang, "about.src_planets")).strong());
                    ui.hyperlink_to("NASA JPL Horizons", "https://ssd.jpl.nasa.gov/horizons/");
                });

                ui.horizontal(|ui| {
                    ui.label("🌠");
                    ui.label(egui::RichText::new(i18n::t(&lang, "about.src_meteors")).strong());
                    ui.hyperlink_to("International Meteor Organization (IMO)", "https://www.imo.net");
                });

                ui.horizontal(|ui| {
                    ui.label("☄");
                    ui.label(egui::RichText::new(i18n::t(&lang, "about.src_comets")).strong());
                    ui.hyperlink_to("IAU Minor Planet Center (MPC)", "https://www.minorplanetcenter.net");
                });

                ui.horizontal(|ui| {
                    ui.label("📡");
                    ui.label(egui::RichText::new(i18n::t(&lang, "about.src_iss_radio")).strong());
                    ui.hyperlink_to("ARISS", "https://www.ariss.org");
                });

                ui.horizontal(|ui| {
                    ui.label("☀");
                    ui.label(egui::RichText::new(i18n::t(&lang, "about.src_solar")).strong());
                    ui.hyperlink_to(
                        i18n::t(&lang, "about.src_solar_noaa"),
                        "https://gml.noaa.gov/grad/solcalc/",
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("🌌");
                    ui.label(egui::RichText::new(i18n::t(&lang, "about.src_milky_way")).strong());
                    ui.label(egui::RichText::new("Local LST calculation (Sgr A*)").weak().size(10.5));
                });
            });

            ui.add_space(8.0);
            ui.separator();

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(i18n::t(&lang, "about.close")).clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        open = false;
    }
    app.show_about = open;
}
