pub mod theme;
pub mod widgets;
pub mod orb;
pub mod panel;
pub mod nudge;

use druid::{Widget, WidgetExt};
use druid::widget::{Flex, Align};
use crate::state::AppState;

pub fn build_root_ui() -> impl Widget<AppState> {
    let intent_bar = widgets::build_intent_bar();
    let status_bar = widgets::build_status_bar();
    let panels = panel::AdaptivePanel::build();
    let orb = orb::IntentOrb::new();
    let nudge = nudge::DaoNudge::new();

    // Layer 1: Panels (Background/Workspace)
    // We wrap panels in a Flex to ensure they take up space properly if needed, 
    // but AdaptivePanel should handle its own layout.
    let workspace_layer = Flex::column()
        .with_flex_child(panels, 1.0);

    // Layer 2: HUD (Orb + Intent Bar + Status)
    // Positioned at the bottom
    let hud_layer = Align::centered(
        Flex::column()
            .with_child(
                Flex::row()
                    .with_child(orb)
                    .with_spacer(20.0)
                    .with_child(intent_bar.fix_width(400.0))
            )
            .with_spacer(10.0)
            .with_child(status_bar)
            .padding(20.0)
    );

    // Layer 3: Notifications / Nudges
    // Positioned at the top-right or center-top
    let overlay_layer = Align::right(
        Align::top(
            nudge.padding(20.0)
        )
    );

    // Root Z-Stack
    widgets::ZStack::new()
        .with_child(workspace_layer) // Bottom
        .with_child(hud_layer)       // Middle
        .with_child(overlay_layer)   // Top
        .background(theme::BACKGROUND)
}
