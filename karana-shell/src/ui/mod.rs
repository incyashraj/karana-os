pub mod theme;
pub mod widgets;
pub mod orb;
pub mod panel;
pub mod nudge;

use druid::{Widget, WidgetExt};
use druid::widget::{Flex, Align, Split};
use crate::state::AppState;

pub fn build_root_ui() -> impl Widget<AppState> {
    let intent_bar = widgets::build_intent_bar();
    let status_bar = widgets::build_status_bar();
    let panels = panel::AdaptivePanel::build();
    let orb = orb::IntentOrb::new();
    let nudge = nudge::DaoNudge::new();

    let main_layout = Flex::column()
        .with_child(
            Flex::row()
                .with_child(orb.padding(10.0))
                .with_flex_child(intent_bar, 1.0)
                .padding(20.0)
        )
        .with_flex_child(panels, 1.0)
        .with_child(Align::right(status_bar));

    // Overlay Nudge on top (using Z-stack simulation via container if Druid supported it natively easily, 
    // but here we'll just append it for simplicity or use a specialized layout in real impl)
    // For this stub, we put it at the bottom floating.
    
    Flex::column()
        .with_flex_child(main_layout, 1.0)
        .with_child(nudge)
        .background(theme::BACKGROUND)
}
