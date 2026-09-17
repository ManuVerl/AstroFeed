use crate::app::AstroFeedApp;
use crate::i18n::{self, Lang};
use crate::utils::astro::{azimuth_to_compass_key, calculate_moon_info, calculate_sun_times};
use chrono::Local;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use std::f64::consts::PI;

/// Render the bottom sidebar information section (rendered from top-to-bottom within its container)
pub fn show_sidebar_info(app: &AstroFeedApp, ui: &mut Ui) {
    let lang = &app.settings.language;
    let now = Local::now();

    ui.separator();

    // ── 1. Date & Real-time Clock ──────────────────────────────────────────
    ui.vertical(|ui| {
        // Time display with large bold font
        let time_str = now.format("%H:%M:%S").to_string();
        ui.label(egui::RichText::new(time_str).size(20.0).strong().color(
            if app.settings.theme.is_dark_based() {
                Color32::from_rgb(240, 240, 245)
            } else {
                Color32::from_rgb(30, 30, 40)
            },
        ));

        // Date display — localised
        let date_str = i18n::format_date(lang, &now);
        ui.label(egui::RichText::new(date_str).size(11.5).weak());
    });

    ui.add_space(4.0);

    // ── 2. Active Position Info ────────────────────────────────────────────
    if let Some(pos) = app.active_position() {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                // Large icon — same technique as in the event list
                let icon_size = 28.0;
                let icon_str = pos.icon.emoji().to_string();
                let (icon_rect, _) = ui.allocate_exact_size(
                    egui::vec2(icon_size + 6.0, icon_size + 6.0),
                    egui::Sense::hover(),
                );
                let galley = ui.painter().layout_no_wrap(
                    icon_str,
                    egui::FontId::proportional(icon_size),
                    ui.visuals().text_color(),
                );
                let text_pos = icon_rect.center() - galley.size() / 2.0;
                ui.painter().galley(text_pos, galley, ui.visuals().text_color());

                // Name + coordinates stacked vertically
                let lat_card = if pos.latitude >= 0.0 { 'N' } else { 'S' };
                let lon_card_str = if pos.longitude >= 0.0 {
                    "E"
                } else {
                    i18n::t(lang, "sidebar.lon_card_w")
                };
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&pos.name).strong().size(12.0));
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.3}° {} · {:.3}° {}",
                            pos.latitude.abs(),
                            lat_card,
                            pos.longitude.abs(),
                            lon_card_str
                        ))
                        .size(10.5)
                        .weak(),
                    );
                });
            });
        });
    } else {
        ui.label(egui::RichText::new(i18n::t(lang, "sidebar.no_position")).weak().size(11.0));
    }

    ui.add_space(4.0);

    // ── 3. Ephemerides (Sun & Moon) ─────────────────────────────────────────
    if let Some(pos) = app.active_position() {
        let sun_times = calculate_sun_times(pos.latitude, pos.longitude, now);
        let moon_info = calculate_moon_info(
            now.with_timezone(&chrono::Utc),
            pos.latitude,
            pos.longitude,
        );

        // Sun ephemeris & graphic widget
        render_sun_widget(ui, &sun_times, lang, app.settings.theme.is_dark_based());

        ui.add_space(4.0);

        // Moon phase & graphic widget
        render_moon_widget(ui, &moon_info, lang, app.settings.theme.is_dark_based());
    }
}

/// Sun widget: arc graphic + daylight duration, rise/set, altitude, direction.
fn render_sun_widget(ui: &mut Ui, sun: &crate::utils::astro::SunTimes, lang: &Lang, dark_mode: bool) {
    use crate::utils::astro::azimuth_to_compass_key;

    ui.group(|ui| {
        ui.set_width(ui.available_width());

        // ── Arc graphic ───────────────────────────────────────────────────────
        let desired_size = Vec2::new(ui.available_width(), 32.0);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        let painter = ui.painter_at(rect);

        let horizon_color = if dark_mode {
            Color32::from_rgb(80, 85, 100)
        } else {
            Color32::from_rgb(180, 185, 200)
        };
        let arc_night_color = if dark_mode {
            Color32::from_rgb(45, 50, 70)
        } else {
            Color32::from_rgb(215, 220, 230)
        };
        let arc_day_color = Color32::from_rgb(230, 165, 40);

        let horizon_y = rect.max.y - 6.0;
        let padding_x = 10.0;
        let left_x = rect.min.x + padding_x;
        let right_x = rect.max.x - padding_x;
        let arc_width = right_x - left_x;
        let arc_height = 20.0;

        painter.line_segment(
            [Pos2::new(left_x - 4.0, horizon_y), Pos2::new(right_x + 4.0, horizon_y)],
            Stroke::new(1.0_f32, horizon_color),
        );

        let points_count = 24;
        let mut arc_pts = Vec::with_capacity(points_count);
        for i in 0..points_count {
            let t = i as f32 / (points_count - 1) as f32;
            let x = left_x + t * arc_width;
            let y = horizon_y - (t * std::f32::consts::PI).sin() * arc_height;
            arc_pts.push(Pos2::new(x, y));
        }

        for w in arc_pts.windows(2) {
            painter.line_segment([w[0], w[1]], Stroke::new(1.5_f32, arc_night_color));
        }

        if let Some(progress) = sun.daylight_progress {
            if progress >= 0.0 && progress <= 1.0 {
                let mut traversed_pts = Vec::new();
                for pt in &arc_pts {
                    let pt_prog = (pt.x - left_x) / arc_width;
                    if pt_prog <= progress {
                        traversed_pts.push(*pt);
                    }
                }
                let sun_x = left_x + progress * arc_width;
                let sun_y = horizon_y - (progress * std::f32::consts::PI).sin() * arc_height;
                let sun_pos = Pos2::new(sun_x, sun_y);
                traversed_pts.push(sun_pos);
                for w in traversed_pts.windows(2) {
                    painter.line_segment([w[0], w[1]], Stroke::new(2.0_f32, arc_day_color));
                }
                painter.circle_filled(sun_pos, 5.0, Color32::from_rgba_unmultiplied(255, 200, 50, 80));
                painter.circle_filled(sun_pos, 3.2, Color32::from_rgb(255, 220, 80));
                painter.circle_stroke(sun_pos, 3.2, Stroke::new(1.0_f32, Color32::from_rgb(255, 255, 255)));
            } else {
                let p = Pos2::new(rect.center().x, horizon_y + 4.0);
                painter.circle_filled(p, 2.5, Color32::from_rgb(100, 110, 140));
            }
        } else {
            let p = Pos2::new(rect.center().x, horizon_y + 4.0);
            painter.circle_filled(p, 2.5, Color32::from_rgb(100, 110, 140));
        }

        ui.add_space(2.0);

        // ── Daylight duration ─────────────────────────────────────────────────
        let daylight_str = if let Some(secs) = sun.daylight_duration_secs {
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            format!(
                "{}: {} {} {} {}",
                i18n::t(lang, "sun.daylight"),
                h, i18n::t(lang, "sun.hour"),
                m, i18n::t(lang, "sun.minute"),
            )
        } else {
            format!("{}: {}", i18n::t(lang, "sun.daylight"), i18n::t(lang, "sun.no_rise"))
        };
        ui.label(egui::RichText::new(daylight_str).size(10.5));

        // ── Rise | Set ────────────────────────────────────────────────────────
        let rise_str = sun.sunrise_local
            .as_ref()
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| i18n::t(lang, "sun.no_rise").to_string());
        let set_str = sun.sunset_local
            .as_ref()
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string());
        ui.label(
            egui::RichText::new(format!(
                "{}: {}  |  {}: {}",
                i18n::t(lang, "sun.rise"), rise_str,
                i18n::t(lang, "sun.set"),  set_str,
            ))
            .size(10.5),
        );

        // ── Altitude ──────────────────────────────────────────────────────────
        ui.label(
            egui::RichText::new(format!(
                "{}: {:.2}°",
                i18n::t(lang, "sun.altitude"),
                sun.current_elevation,
            ))
            .size(10.5),
        );

        // ── Direction ─────────────────────────────────────────────────────────
        let compass_key = azimuth_to_compass_key(sun.current_azimuth);
        let compass_label = i18n::t(lang, compass_key);
        ui.label(
            egui::RichText::new(format!(
                "{}: {:.2}° ({})",
                i18n::t(lang, "sun.direction"),
                sun.current_azimuth,
                compass_label,
            ))
            .size(10.5),
        );
    });
}

/// Moon widget: disc graphic + phase name + illumination, rise/set, altitude, direction.
fn render_moon_widget(ui: &mut Ui, moon: &crate::utils::astro::MoonPhaseInfo, lang: &Lang, dark_mode: bool) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());

        // ── Header row: disc + phase name ─────────────────────────────────────
        ui.horizontal(|ui| {
            let moon_size = Vec2::new(26.0, 26.0);
            let (rect, _response) = ui.allocate_exact_size(moon_size, egui::Sense::hover());
            paint_moon_disc(ui.painter_at(rect), rect, moon.phase, dark_mode);

            ui.vertical(|ui| {
                let phase_key = i18n::moon_phase_key(moon.phase);
                ui.label(egui::RichText::new(i18n::t(lang, phase_key)).strong().size(11.0));
            });
        });

        ui.add_space(2.0);

        // ── Illumination ──────────────────────────────────────────────────────
        let illum_pct = (moon.illumination * 100.0 * 10.0).round() / 10.0;
        ui.label(
            egui::RichText::new(format!("{}: {:.1}%", i18n::t(lang, "moon.illumination"), illum_pct))
                .size(10.5),
        );

        // ── Rise | Set ────────────────────────────────────────────────────────
        let rise_str = moon.rise_local
            .as_ref()
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| i18n::t(lang, "moon.no_rise").to_string());
        let set_str = moon.set_local
            .as_ref()
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string());

        ui.label(
            egui::RichText::new(format!(
                "{}: {}  |  {}: {}",
                i18n::t(lang, "moon.rise"), rise_str,
                i18n::t(lang, "moon.set"),  set_str,
            ))
            .size(10.5),
        );

        // ── Altitude ──────────────────────────────────────────────────────────
        let horizon_key = if moon.altitude_deg >= 0.0 {
            "moon.above_horizon"
        } else {
            "moon.below_horizon"
        };
        ui.label(
            egui::RichText::new(format!(
                "{}: {:.1}° ({})",
                i18n::t(lang, "moon.altitude"),
                moon.altitude_deg.abs(),
                i18n::t(lang, horizon_key),
            ))
            .size(10.5),
        );

        // ── Direction ─────────────────────────────────────────────────────────
        let compass_key = azimuth_to_compass_key(moon.azimuth_deg);
        let compass_label = i18n::t(lang, compass_key);
        ui.label(
            egui::RichText::new(format!(
                "{}: {:.1}° ({})",
                i18n::t(lang, "moon.direction"),
                moon.azimuth_deg,
                compass_label,
            ))
            .size(10.5),
        );
    });
}

/// Draw a minimal vector representation of the lunar disc according to phase (0.0 to 1.0)
fn paint_moon_disc(painter: egui::Painter, rect: Rect, phase: f64, dark_mode: bool) {
    let center = rect.center();
    let radius = (rect.width().min(rect.height()) / 2.0) - 1.5;

    let dark_moon_color = if dark_mode {
        Color32::from_rgb(32, 36, 48)
    } else {
        Color32::from_rgb(190, 195, 205)
    };
    let lit_moon_color = Color32::from_rgb(245, 240, 215);
    let border_color = if dark_mode {
        Color32::from_rgb(80, 85, 110)
    } else {
        Color32::from_rgb(150, 155, 170)
    };

    // Base circle (dark side)
    painter.circle_filled(center, radius, dark_moon_color);

    // Draw illuminated crescent / gibbous geometry
    let is_waxing = phase < 0.5;
    let k = (2.0 * PI * phase).cos() as f32;

    let num_steps = 32;
    let mut lit_points = Vec::with_capacity(num_steps * 2);

    let (limb_start_angle, limb_dir) = if is_waxing {
        (-std::f32::consts::FRAC_PI_2, 1.0)
    } else {
        (std::f32::consts::FRAC_PI_2, 1.0)
    };

    for i in 0..=num_steps {
        let frac = i as f32 / num_steps as f32;
        let theta = limb_start_angle + frac * std::f32::consts::PI * limb_dir;
        let x = center.x + radius * theta.cos();
        let y = center.y + radius * theta.sin();
        lit_points.push(Pos2::new(x, y));
    }

    for i in (0..=num_steps).rev() {
        let frac = i as f32 / num_steps as f32;
        let theta = limb_start_angle + frac * std::f32::consts::PI * limb_dir;
        let y = center.y + radius * theta.sin();
        let term_x_offset = -k * radius * theta.cos();
        let x = center.x + term_x_offset;
        lit_points.push(Pos2::new(x, y));
    }

    if lit_points.len() >= 3 {
        painter.add(egui::Shape::convex_polygon(
            lit_points,
            lit_moon_color,
            Stroke::NONE,
        ));
    }

    // Subtle disc outline
    painter.circle_stroke(center, radius, Stroke::new(1.0_f32, border_color));
}
