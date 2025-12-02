use druid::{Widget, WidgetExt, Color};
use druid::widget::{Flex, Label, Button, Container};
use crate::state::AppState;

pub struct DaoNudge;

impl DaoNudge {
    pub fn new() -> impl Widget<AppState> {
        let container = Container::new(
            Flex::column()
                .with_child(
                    Label::new("Governance Proposal")
                        .with_text_size(12.0)
                        .with_text_color(Color::rgb8(255, 200, 0))
                )
                .with_spacer(5.0)
                .with_child(
                    Label::new(|data: &AppState, _env: &_| data.dao_proposal_text.clone())
                        .with_text_size(14.0)
                )
                .with_spacer(10.0)
                .with_child(
                    Flex::row()
                        .with_child(
                            Button::new("Vote YES (1 KARA)")
                                .on_click(|_ctx, data: &mut AppState, _env| {
                                    data.system_status = "Vote Cast: YES. Staked 1 KARA.".to_string();
                                    data.dao_proposal_active = false;
                                })
                        )
                        .with_spacer(10.0)
                        .with_child(
                            Button::new("Dismiss")
                                .on_click(|_ctx, data: &mut AppState, _env| {
                                    data.dao_proposal_active = false;
                                })
                        )
                )
        )
        .background(Color::rgba8(20, 20, 30, 240))
        .rounded(8.0)
        .border(Color::rgb8(255, 200, 0), 1.0)
        .padding(20.0);

        // Only show if active
        druid::widget::Either::new(
            |data: &AppState, _env| data.dao_proposal_active,
            container,
            Flex::column() // Empty if inactive
        )
    }
}
