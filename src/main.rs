mod app;
mod config;
mod model;
mod sources;
mod ui;
mod utils;

use eframe::NativeOptions;
use std::sync::OnceLock;

/// Global tokio runtime handle, accessible from anywhere (source manager, etc.)
pub static TOKIO_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

/// Build a 32×32 RGBA icon: dark background, a simple telescope silhouette + star sparkle.
fn build_app_icon() -> egui::IconData {
    const SIZE: usize = 32;
    let mut rgba = vec![0u8; SIZE * SIZE * 4];

    // Helper: set pixel (x, y) to RGBA
    let mut set = |x: usize, y: usize, r: u8, g: u8, b: u8, a: u8| {
        if x < SIZE && y < SIZE {
            let i = (y * SIZE + x) * 4;
            rgba[i]     = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = a;
        }
    };

    // Background: deep navy
    for y in 0..SIZE {
        for x in 0..SIZE {
            set(x, y, 10, 12, 30, 255);
        }
    }

    // ── Telescope tube (diagonal, upper-left to lower-right) ─────────────────
    let tube_color = (180u8, 200u8, 220u8, 255u8);
    // Main tube: thick diagonal line from (3,8) to (20,20)
    for i in 0i32..18 {
        let x = (3 + i) as usize;
        let y = (8 + i * 12 / 17) as usize;
        set(x, y, tube_color.0, tube_color.1, tube_color.2, tube_color.3);
        set(x, y + 1, tube_color.0, tube_color.1, tube_color.2, 200);
        set(x + 1, y, tube_color.0, tube_color.1, tube_color.2, 200);
    }
    // Eyepiece (wider end, top-left)
    for dx in 0usize..4 {
        for dy in 0usize..3 {
            set(1 + dx, 6 + dy, 220, 230, 240, 255);
        }
    }
    // Objective lens (narrow end, bottom-right)
    for dy in 0usize..2 {
        set(21, 20 + dy, 160, 200, 240, 255);
        set(22, 20 + dy, 160, 200, 240, 255);
    }
    // Tripod legs
    for i in 0i32..8 {
        set((14 - i) as usize, (22 + i) as usize, 120, 140, 160, 220); // left leg
        set((18 + i / 2) as usize, (22 + i) as usize, 120, 140, 160, 220); // right leg
    }
    set(16, 30, 100, 120, 140, 255); // foot

    // ── Stars / sparkles ─────────────────────────────────────────────────────
    // Bright star (4-point sparkle) at top-right
    let star_cx = 25usize;
    let star_cy = 5usize;
    // Core
    set(star_cx, star_cy, 255, 255, 220, 255);
    // Arms
    for d in 1usize..=3 {
        let bright = 255u8.saturating_sub((d as u8) * 60);
        set(star_cx + d, star_cy, 255, 255, 180, bright);
        if star_cx >= d { set(star_cx - d, star_cy, 255, 255, 180, bright); }
        set(star_cx, star_cy + d, 255, 255, 180, bright);
        if star_cy >= d { set(star_cx, star_cy - d, 255, 255, 180, bright); }
    }
    // Diagonal sparkle arms (shorter)
    set(star_cx + 1, star_cy + 1, 255, 255, 200, 180);
    set(star_cx + 1, star_cy.wrapping_sub(1), 255, 255, 200, 180);
    if star_cx >= 1 {
        set(star_cx - 1, star_cy + 1, 255, 255, 200, 180);
        set(star_cx - 1, star_cy.wrapping_sub(1), 255, 255, 200, 180);
    }

    // Small background stars (scattered)
    let small_stars: &[(usize, usize, u8)] = &[
        (28, 12, 180), (26, 18, 140), (29, 24, 160),
        (24, 28, 120), (27, 3,  200), (30, 8,  150),
        (2,  2,  160), (5,  4,  130), (1,  15, 100),
        (3,  25, 140), (8,  3,  120), (12, 2,  110),
    ];
    for &(x, y, a) in small_stars {
        set(x, y, 200, 210, 255, a);
    }

    // ── Radio dish (small, bottom-right corner) ───────────────────────────────
    // Arc of the parabola
    let dish_pairs: &[(usize, usize)] = &[
        (24,22),(25,21),(26,21),(27,21),(28,22),
        (25,23),(26,22),(27,22),
        (26,24), // feed
    ];
    for &(x, y) in dish_pairs {
        set(x, y, 80, 180, 255, 200);
    }
    // Mast
    set(26, 25, 80, 160, 220, 180);
    set(26, 26, 80, 160, 220, 160);

    egui::IconData {
        rgba,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    // Build a multi-thread tokio runtime for background source fetches.
    // We keep `_rt` alive for the entire process lifetime.
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    TOKIO_HANDLE.set(rt.handle().clone()).ok();

    let icon = build_app_icon();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AstroFeed  ·  by Emmanuel V. with BOB")
            .with_inner_size([1024.0, 700.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(icon),
        ..Default::default()
    };

    let result = eframe::run_native(
        "AstroFeed  ·  by Emmanuel V. with BOB",
        options,
        Box::new(|cc| Box::new(app::AstroFeedApp::new(cc))),
    );

    // runtime is dropped here, after eframe exits
    drop(rt);
    result
}
