use druid::{Widget, WidgetExt, Color, LensExt};
use druid::widget::{Flex, Label, Container, List, Scroll};
use crate::state::{AppState, PanelData};
use crate::ui::theme;

pub struct AdaptivePanel;

impl AdaptivePanel {
    pub fn build() -> impl Widget<AppState> {
        Scroll::new(
            List::new(|| {
                build_single_panel()
            })
            .lens(AppState::active_panels)
        )
        .vertical()
    }
}

fn build_single_panel() -> impl Widget<PanelData> {
    let header = Flex::row()
        .with_flex_child(
            Label::new(|data: &PanelData, _env: &_| data.title.clone())
                .with_text_size(16.0)
                .with_text_color(theme::FOREGROUND),
            1.0
        )
        .with_child(
            Label::new(|data: &PanelData, _env: &_| {
                if data.is_verified { "ZK ✓" } else { "Unverified" }
            })
            .with_text_size(10.0)
            .with_text_color(Color::rgb8(0, 255, 0))
            .padding(5.0)
            .background(Color::rgba8(0, 255, 0, 30))
            .rounded(4.0)
        );

    let content = Label::new(|data: &PanelData, _env: &_| data.content.clone())
        .with_text_size(14.0)
        .with_text_color(Color::grey(0.8))
        .padding(10.0);

    Container::new(
        Flex::column()
            .with_child(header)
            .with_spacer(5.0)
            .with_child(content)
    )
    .background(Color::rgb8(30, 30, 35))
    .rounded(8.0)
    .border(Color::grey(0.3), 1.0)
    .padding(10.0)
}
