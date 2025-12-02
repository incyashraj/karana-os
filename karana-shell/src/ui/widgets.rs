use druid::widget::{Flex, Label, TextBox, Button, Padding};
use druid::{Widget, WidgetExt, Color};
use crate::state::{AppState, PanelData};
use crate::ui::theme;

pub fn build_intent_bar() -> impl Widget<AppState> {
    let input = TextBox::new()
        .with_placeholder("Express your intent...")
        .with_text_size(18.0)
        .lens(AppState::intent_input)
        .expand_width();

    let execute_btn = Button::new("Forge")
        .on_click(|_ctx, data: &mut AppState, _env| {
            data.is_processing = true;
            
            // Use the client to send intent
            match data.client.send_intent(&data.intent_input) {
                Ok(msg) => data.system_status = msg,
                Err(e) => data.system_status = format!("Error: {}", e),
            }

            if data.intent_input == "code" {
                data.active_panels.push_back(PanelData {
                    id: "code_1".to_string(),
                    title: "Code Editor".to_string(),
                    content: "fn main() { println!(\"Hello Symbiosis\"); }".to_string(),
                    panel_type: "code".to_string(),
                    is_verified: true,
                    proof_hash: "0xabc123".to_string(),
                });
            } else if data.intent_input == "tune battery" {
                 data.active_panels.push_back(PanelData {
                    id: "batt_1".to_string(),
                    title: "Battery Optimization".to_string(),
                    content: "Optimizing shards for low power...".to_string(),
                    panel_type: "graph".to_string(),
                    is_verified: true,
                    proof_hash: "0xdef456".to_string(),
                });
                // Trigger Nudge
                data.dao_proposal_active = true;
                data.dao_proposal_text = "Apply 'Eco-Neural' Theme?".to_string();
            }

            data.intent_input.clear();
            data.is_processing = false;
        });

    Flex::row()
        .with_flex_child(input, 1.0)
        .with_spacer(10.0)
        .with_child(execute_btn)
        .padding(10.0)
        .background(theme::INTENT_BAR_BG)
        .rounded(8.0)
}

pub fn build_status_bar() -> impl Widget<AppState> {
    Label::new(|data: &AppState, _env: &_| data.system_status.clone())
        .with_text_size(12.0)
        .with_text_color(Color::grey(0.6))
        .padding(5.0)
}
