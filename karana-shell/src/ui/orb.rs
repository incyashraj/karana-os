use druid::{Widget, WidgetExt, EventCtx, Event, Env, RenderContext, Color, LinearGradient, UnitPoint};
use druid::widget::{Painter, Controller};
use crate::state::AppState;
use crate::ui::theme;

pub struct IntentOrb;

impl IntentOrb {
    pub fn new() -> impl Widget<AppState> {
        // The Orb Visuals
        let orb_painter = Painter::new(|ctx, data: &AppState, _env| {
            let bounds = ctx.size().to_rect();
            let center = bounds.center();
            let radius = bounds.width().min(bounds.height()) / 2.0 - 10.0;

            // Neural Pulse Effect (Simulated via time)
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64;
            
            let pulse = if data.is_processing || data.voice_listening {
                (time * 0.005).sin() * 5.0 + 5.0
            } else {
                (time * 0.001).sin() * 2.0
            };

            // Gradient Fill (Cyan to Green)
            let gradient = LinearGradient::new(
                UnitPoint::TOP_LEFT,
                UnitPoint::BOTTOM_RIGHT,
                (theme::HUD_CYAN, theme::HUD_GREEN),
            );

            // Outer Glow (Haptic Ring)
            ctx.fill(
                druid::kurbo::Circle::new(center, radius + pulse),
                &Color::rgba8(0, 255, 255, 50)
            );

            // Core Orb
            ctx.fill(druid::kurbo::Circle::new(center, radius), &gradient);
            
            // Status Indicator (Mic)
            if data.voice_listening {
                ctx.fill(druid::kurbo::Circle::new(center, radius * 0.3), &theme::HUD_RED);
            }
        });

        // Just the Orb, no text box
        orb_painter.controller(OrbController)
    }
}

struct OrbController;

impl<W: Widget<AppState>> Controller<AppState, W> for OrbController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                data.voice_listening = true;
                data.system_status = "Listening... (Hold to Speak)".to_string();
                ctx.request_anim_frame();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    data.voice_listening = false;
                    // Simulate Voice Parse
                    data.intent_input = "tune battery".to_string(); 
                    data.system_status = "Voice Intent Captured: 'tune battery'".to_string();
                    ctx.request_paint();
                }
            }
            Event::AnimFrame(_) => {
                if data.voice_listening || data.is_processing {
                    ctx.request_anim_frame();
                    ctx.request_paint();
                }
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}
