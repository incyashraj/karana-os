use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Size, UpdateCtx, Widget, WidgetExt, Point, WidgetPod
};
use druid::widget::{Flex, Label, TextBox, Button};
use crate::state::{AppState, PanelData};
use crate::ui::theme;

pub struct ZStack<T> {
    children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
}

impl<T: Data> ZStack<T> {
    pub fn new() -> Self {
        ZStack { children: Vec::new() }
    }

    pub fn with_child(mut self, child: impl Widget<T> + 'static) -> Self {
        self.children.push(WidgetPod::new(Box::new(child)));
        self
    }
}

impl<T: Data> Widget<T> for ZStack<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // Dispatch events in reverse order (top-most first)
        for child in self.children.iter_mut().rev() {
            child.event(ctx, event, data, env);
            if ctx.is_handled() {
                break;
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let mut max_size = Size::ZERO;
        // Layout all children with the same constraints (fill the stack)
        for child in self.children.iter_mut() {
            let size = child.layout(ctx, bc, data, env);
            max_size.width = max_size.width.max(size.width);
            max_size.height = max_size.height.max(size.height);
            
            // Center the child in the stack space
            let origin = Point::new(
                (bc.max().width - size.width) / 2.0,
                (bc.max().height - size.height) / 2.0
            );
            child.set_origin(ctx, data, env, origin);
        }
        
        // If constraints are infinite, use max_size, otherwise fill
        if bc.is_width_bounded() && bc.is_height_bounded() {
            bc.max()
        } else {
            max_size
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.paint(ctx, data, env);
        }
    }
}

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
                    x: 100.0, y: 100.0, z_index: 1,
                });
            } else if data.intent_input == "tune battery" {
                 data.active_panels.push_back(PanelData {
                    id: "batt_1".to_string(),
                    title: "Battery Optimization".to_string(),
                    content: "Optimizing shards for low power...".to_string(),
                    panel_type: "graph".to_string(),
                    is_verified: true,
                    proof_hash: "0xdef456".to_string(),
                    x: 200.0, y: 150.0, z_index: 2,
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
        .background(theme::NEURAL_BLUE_START)
        .rounded(8.0)
        .border(theme::SHARD_GLOW, 1.0)
}

pub fn build_status_bar() -> impl Widget<AppState> {
    Label::new(|data: &AppState, _env: &_| data.system_status.clone())
        .with_text_size(12.0)
        .with_text_color(theme::TEXT_GRAY)
        .padding(5.0)
}
