use druid::{Widget, WidgetExt, EventCtx, Event, Env, RenderContext, Color, LinearGradient, UnitPoint};
use druid::widget::{Painter, Controller, Flex, TextBox};
use crate::state::AppState;
use crate::ui::theme;

pub struct IntentOrb;

impl IntentOrb {
    pub fn new() -> impl Widget<AppState> {
        // The Orb Visuals
        let orb_painter = Painter::new(|ctx, data: &AppState, _env| {
            let bounds = ctx.size().to_rect();
            let center = bounds.center();
            let radius = 50.0;

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

            // Gradient Fill
            let gradient = LinearGradient::new(
                UnitPoint::TOP_LEFT,
                UnitPoint::BOTTOM_RIGHT,
                (theme::NEURAL_BLUE_START, theme::NEURAL_BLUE_END),
            );

            // Outer Glow (Haptic Ring)
            ctx.fill(
                druid::kurbo::Circle::new(center, radius + pulse + 10.0),
                &Color::rgba8(0, 100, 255, 30)
            );

            // Core Orb
            ctx.fill(druid::kurbo::Circle::new(center, radius), &gradient);
            
            // Status Indicator (Mic)
            if data.voice_listening {
                ctx.fill(druid::kurbo::Circle::new(center, 10.0), &theme::ALERT_RED);
            }
        });

        // Input Field Overlay
        let input = TextBox::new()
            .with_placeholder("Intent...")
            .with_text_size(14.0)
            .lens(AppState::intent_input)
            .fix_width(120.0)
            .center();

        // Composition
        Flex::column()
            .with_child(
                orb_painter
                    .fix_size(120.0, 120.0)
                    .controller(OrbController)
            )
            .with_spacer(10.0)
            .with_child(input)
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
