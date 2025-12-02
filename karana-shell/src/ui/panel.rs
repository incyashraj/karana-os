use druid::{Widget, WidgetExt, Color, RenderContext};
use druid::widget::{Flex, Label, Container, List, Scroll, Painter, Button};
use crate::state::{AppState, PanelData};
use crate::ui::theme;

pub struct AdaptivePanel;

impl AdaptivePanel {
    pub fn build() -> impl Widget<AppState> {
        // In a real spatial UI, this would be a custom layout widget.
        // For Druid Flex, we simulate a vertical stack of floating cards.
        Scroll::new(
            List::new(|| {
                build_single_panel().padding(10.0)
            })
            .lens(AppState::active_panels)
        )
        .vertical()
        .expand_width()
    }
}

fn build_single_panel() -> impl Widget<PanelData> {
    // 1. Header (Title + Close)
    let header = Flex::row()
        .with_flex_child(
            Label::new(|data: &PanelData, _env: &_| data.title.clone())
                .with_font(theme::FONT_HEADER)
                .with_text_color(theme::HUD_CYAN),
            1.0
        )
        .with_child(
            Label::new("✕").with_text_color(theme::HUD_DIM).padding(5.0)
        )
        .padding((10.0, 10.0, 10.0, 5.0));

    // 2. Body (Content - Graph/Code/Text)
    let body = Container::new(
        Label::new(|data: &PanelData, _env: &_| data.content.clone())
            .with_font(theme::FONT_CODE)
            .with_text_color(theme::HUD_DIM)
            .with_line_break_mode(druid::widget::LineBreaking::WordWrap)
    )
    .padding(10.0)
    .expand_width();

    // 3. Footer (ZK Badge + DAO Action)
    let footer = Flex::row()
        .with_child(
            Label::new(|data: &PanelData, _env: &_| {
                if data.is_verified { "ZK-SNARK ✓" } else { "Unverified" }
            })
            .with_text_size(10.0)
            .with_text_color(if true { theme::HUD_GREEN } else { theme::HUD_RED })
            .padding(5.0)
            .background(theme::HUD_GREEN.with_alpha(0.1)) // Glow bg
        )
        .with_spacer(10.0)
        .with_child(
            Button::new("DAO Vote")
                .on_click(|_ctx, _data, _env| {
                    // Trigger Nudge
                })
                .fix_height(24.0)
        )
        .padding(10.0);

    // 4. Card Container (Shadow + Depth)
    Container::new(
        Flex::column()
            .with_child(header)
            .with_child(druid::widget::Painter::new(|ctx, _, _| {
                let rect = ctx.size().to_rect();
                ctx.fill(rect, &theme::HUD_CYAN.with_alpha(0.2)); // Separator line
            }).fix_height(1.0))
            .with_child(body)
            .with_child(footer)
    )
    .background(theme::SHARD_BG)
    .border(theme::HUD_CYAN.with_alpha(0.5), 1.0)
    .padding(10.0) // Margin
}
