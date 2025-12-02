use druid::{Widget, WidgetExt, EventCtx, Event, Env, PaintCtx, RenderContext, Color, Point};
use druid::widget::{Painter, Controller};
use crate::state::AppState;

pub struct IntentOrb;

impl IntentOrb {
    pub fn new() -> impl Widget<AppState> {
        Painter::new(|ctx, data: &AppState, _env| {
            let bounds = ctx.size().to_rect();
            let center = bounds.center();
            let radius = 40.0;

            // Neural Pulse Effect
            let pulse = if data.is_processing {
                (ctx.window().get_time() as f64 * 3.0).sin() * 5.0 + 5.0
            } else {
                0.0
            };

            // Outer Glow
            ctx.fill(
                druid::kurbo::Circle::new(center, radius + pulse + 5.0),
                &Color::rgba8(0, 100, 255, 50)
            );

            // Core
            let color = if data.is_processing {
                Color::rgb8(0, 255, 180) // Active Cyan
            } else {
                Color::rgb8(0, 100, 255) // Idle Blue
            };
            
            ctx.fill(druid::kurbo::Circle::new(center, radius), &color);
        })
        .controller(OrbController)
    }
}

struct OrbController;

impl<W: Widget<AppState>> Controller<AppState, W> for OrbController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                data.is_processing = true;
                data.system_status = "Listening... (Simulated Mic)".to_string();
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    data.is_processing = false;
                    // Simulate Voice Intent
                    data.intent_input = "tune battery".to_string(); 
                    data.system_status = "Voice Intent Captured: 'tune battery'".to_string();
                    ctx.request_paint();
                }
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}
