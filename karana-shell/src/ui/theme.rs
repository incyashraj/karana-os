use druid::{Color, Env, Key, FontDescriptor, FontFamily};

// --- Color Palette (Neural Void) ---
pub const VOID_BLACK: Color = Color::rgb8(0, 0, 0);
pub const NEURAL_BLUE_START: Color = Color::rgb8(0, 30, 60);
pub const NEURAL_BLUE_END: Color = Color::rgb8(75, 0, 130);
pub const SHARD_CRYSTAL: Color = Color::rgb8(224, 247, 250); // #E0F7FA
pub const SHARD_GLOW: Color = Color::rgb8(0, 188, 212);      // #00BCD4
pub const ALERT_RED: Color = Color::rgb8(255, 23, 68);       // #FF1744
pub const TEXT_GRAY: Color = Color::rgb8(176, 190, 197);     // #B0BEC5

// --- Keys ---
pub const BACKGROUND: Key<Color> = Key::new("karana.background");
pub const FOREGROUND: Key<Color> = Key::new("karana.foreground");
pub const ACCENT: Key<Color> = Key::new("karana.accent");
pub const SHARD_BG: Key<Color> = Key::new("karana.shard_bg");

// --- Fonts ---
pub const FONT_HEADER: Key<FontDescriptor> = Key::new("karana.font.header");
pub const FONT_BODY: Key<FontDescriptor> = Key::new("karana.font.body");
pub const FONT_CODE: Key<FontDescriptor> = Key::new("karana.font.code");

pub fn configure_env(env: &mut Env, _data: &crate::AppState) {
    // Colors
    env.set(BACKGROUND, VOID_BLACK);
    env.set(FOREGROUND, SHARD_CRYSTAL);
    env.set(ACCENT, SHARD_GLOW);
    env.set(SHARD_BG, Color::rgba8(20, 20, 30, 200)); // Semi-transparent panel bg

    // Druid Defaults
    env.set(druid::theme::WINDOW_BACKGROUND_COLOR, VOID_BLACK);
    env.set(druid::theme::TEXT_COLOR, SHARD_CRYSTAL);
    env.set(druid::theme::CURSOR_COLOR, SHARD_GLOW);
    env.set(druid::theme::BUTTON_DARK, NEURAL_BLUE_START);
    env.set(druid::theme::BUTTON_LIGHT, NEURAL_BLUE_END);

    // Fonts (Simulated mapping to system fonts)
    env.set(FONT_HEADER, FontDescriptor::new(FontFamily::SANS_SERIF).with_size(24.0).with_weight(druid::FontWeight::BOLD));
    env.set(FONT_BODY, FontDescriptor::new(FontFamily::SANS_SERIF).with_size(14.0));
    env.set(FONT_CODE, FontDescriptor::new(FontFamily::MONOSPACE).with_size(12.0));
}
