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
    let workspace_layer = Flex::column()
        .with_flex_child(panels, 1.0);

    // Layer 2: HUD Elements
    // Top Left: Status
    let top_left = Align::left(
        Align::vertical(druid::UnitPoint::TOP,
            status_bar.padding(10.0)
        )
    );

    // Top Right: Nudge
    let top_right = Align::right(
        Align::vertical(druid::UnitPoint::TOP,
            nudge.padding(10.0)
        )
    );

    // Bottom Center: Intent Bar + Orb
    let bottom_hud = Align::centered(
        Align::vertical(druid::UnitPoint::BOTTOM,
            Flex::column()
                .with_child(orb.fix_size(50.0, 50.0)) // Smaller Orb
                .with_spacer(5.0)
                .with_child(intent_bar.fix_width(500.0))
                .padding(20.0)
        )
    );

    // Root Z-Stack
    widgets::ZStack::new()
        .with_child(workspace_layer) // Bottom
        .with_child(top_left)        // HUD TL
        .with_child(top_right)       // HUD TR
        .with_child(bottom_hud)      // HUD Bottom
        .background(theme::BACKGROUND)
}
