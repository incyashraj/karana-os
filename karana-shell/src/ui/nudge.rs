use druid::{Widget, WidgetExt, Color};
use druid::widget::{Flex, Label, Button, Container, Either};
use crate::state::AppState;
use crate::ui::theme;

pub struct DaoNudge;

impl DaoNudge {
    pub fn new() -> impl Widget<AppState> {
        let content = Flex::column()
            .with_child(
                Label::new("🏛️ DAO Governance")
                    .with_font(theme::FONT_HEADER)
                    .with_text_color(theme::SHARD_GLOW)
            )
            .with_spacer(5.0)
            .with_child(
                Label::new(|data: &AppState, _env: &_| data.dao_proposal_text.clone())
                    .with_font(theme::FONT_BODY)
                    .with_text_color(theme::SHARD_CRYSTAL)
            )
            .with_spacer(15.0)
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
            );

        let container = Container::new(content)
            .background(Color::rgba8(0, 0, 0, 220)) // Semi-transparent overlay
            .rounded(16.0)
            .border(theme::SHARD_GLOW, 2.0)
            .padding(20.0);

        // Conditional Visibility
        Either::new(
            |data: &AppState, _env| data.dao_proposal_active,
            container,
            Flex::column() // Invisible when inactive
        )
    }
}
