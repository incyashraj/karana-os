use druid::{Color, Env, Key, FontDescriptor, FontFamily};

// --- Color Palette (AR HUD) ---
pub const VOID_BLACK: Color = Color::rgb8(0, 0, 0); // Keep black for contrast, but imagine it's transparent
pub const HUD_CYAN: Color = Color::rgb8(0, 255, 255);      // #00FFFF - Primary HUD Color
pub const HUD_GREEN: Color = Color::rgb8(0, 255, 0);       // #00FF00 - Success/Active
pub const HUD_RED: Color = Color::rgb8(255, 50, 50);       // #FF3232 - Alert
pub const HUD_DIM: Color = Color::rgba8(0, 255, 255, 100); // Dimmed Cyan for inactive elements

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
    env.set(FOREGROUND, HUD_CYAN);
    env.set(ACCENT, HUD_GREEN);
    env.set(SHARD_BG, Color::rgba8(0, 20, 20, 200)); // Dark Cyan tint

    // Druid Defaults
    env.set(druid::theme::WINDOW_BACKGROUND_COLOR, VOID_BLACK);
    env.set(druid::theme::TEXT_COLOR, HUD_CYAN);
    env.set(druid::theme::CURSOR_COLOR, HUD_GREEN);
    env.set(druid::theme::BUTTON_DARK, Color::rgba8(0, 50, 50, 100));
    env.set(druid::theme::BUTTON_LIGHT, Color::rgba8(0, 100, 100, 100));
    env.set(druid::theme::TEXTBOX_BORDER_RADIUS, 0.0); // Sharp corners for AR
    env.set(druid::theme::TEXTBOX_BORDER_WIDTH, 1.0);

    // Fonts (Monospace for HUD feel)
    env.set(FONT_HEADER, FontDescriptor::new(FontFamily::MONOSPACE).with_size(20.0).with_weight(druid::FontWeight::BOLD));
    env.set(FONT_BODY, FontDescriptor::new(FontFamily::MONOSPACE).with_size(14.0));
    env.set(FONT_CODE, FontDescriptor::new(FontFamily::MONOSPACE).with_size(12.0));
}
