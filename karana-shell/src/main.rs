mod state;
mod ui;
mod client;

use druid::{AppLauncher, WindowDesc, PlatformError};
use state::AppState;
use ui::build_root_ui;
use ui::theme;

fn main() -> Result<(), PlatformError> {
    // Initialize Logger
    env_logger::init();
    log::info!("Igniting Kāraṇa Symbiotic Shell...");

    // Define the main window
    let main_window = WindowDesc::new(build_root_ui())
        .title("Kāraṇa OS - Symbiotic Horizon")
        .window_size((800.0, 600.0))
        .resizable(true);

    // Launch the app
    let initial_state = AppState::new();
    
    AppLauncher::with_window(main_window)
        .configure_env(|env, data| theme::configure_env(env, data))
        .launch(initial_state)
}

