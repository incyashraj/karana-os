use druid::{Color, Env, Key};

pub const BACKGROUND: Key<Color> = Key::new("karana.background");
pub const FOREGROUND: Key<Color> = Key::new("karana.foreground");
pub const ACCENT: Key<Color> = Key::new("karana.accent");
pub const INTENT_BAR_BG: Key<Color> = Key::new("karana.intent_bar_bg");

pub fn configure_env(env: &mut Env, _data: &crate::AppState) {
    env.set(BACKGROUND, Color::rgb8(10, 10, 15)); // Deep Neural Dark
    env.set(FOREGROUND, Color::rgb8(220, 220, 230)); // Soft White
    env.set(ACCENT, Color::rgb8(0, 255, 180)); // Cyan/Teal Neon
    env.set(INTENT_BAR_BG, Color::rgb8(30, 30, 40));
    
    env.set(druid::theme::WINDOW_BACKGROUND_COLOR, env.get(BACKGROUND));
    env.set(druid::theme::TEXT_COLOR, env.get(FOREGROUND));
    env.set(druid::theme::CURSOR_COLOR, env.get(ACCENT));
}
