//! UI Animation System
//! 
//! Smooth, performant animations for AR interfaces including:
//! - Tween animations with easing functions
//! - Spring physics animations
//! - Staggered animations
//! - Animation composition

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Animation ID
pub type AnimationId = u64;

/// Animation controller managing all active animations
#[derive(Debug, Clone)]
pub struct AnimationController {
    /// Active animations
    animations: HashMap<AnimationId, Animation>,
    /// Next animation ID
    next_id: AnimationId,
    /// Global time scale (1.0 = normal speed)
    time_scale: f32,
    /// Paused
    paused: bool,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            next_id: 1,
            time_scale: 1.0,
            paused: false,
        }
    }

    /// Create a new animation
    pub fn create(&mut self, config: AnimationConfig) -> AnimationId {
        let id = self.next_id;
        self.next_id += 1;
        
        let animation = Animation {
            id,
            config,
            state: AnimationState::Pending,
            progress: 0.0,
            elapsed: Duration::ZERO,
            started_at: None,
            value: 0.0,
            loops_completed: 0,
        };
        
        self.animations.insert(id, animation);
        id
    }

    /// Start an animation
    pub fn start(&mut self, id: AnimationId) {
        if let Some(anim) = self.animations.get_mut(&id) {
            anim.state = AnimationState::Running;
            anim.started_at = Some(Instant::now());
        }
    }

    /// Pause an animation
    pub fn pause(&mut self, id: AnimationId) {
        if let Some(anim) = self.animations.get_mut(&id) {
            if anim.state == AnimationState::Running {
                anim.state = AnimationState::Paused;
            }
        }
    }

    /// Resume an animation
    pub fn resume(&mut self, id: AnimationId) {
        if let Some(anim) = self.animations.get_mut(&id) {
            if anim.state == AnimationState::Paused {
                anim.state = AnimationState::Running;
            }
        }
    }

    /// Stop an animation
    pub fn stop(&mut self, id: AnimationId) {
        if let Some(anim) = self.animations.get_mut(&id) {
            anim.state = AnimationState::Stopped;
        }
    }

    /// Cancel and remove an animation
    pub fn cancel(&mut self, id: AnimationId) {
        self.animations.remove(&id);
    }

    /// Get animation value
    pub fn value(&self, id: AnimationId) -> Option<f32> {
        self.animations.get(&id).map(|a| a.value)
    }

    /// Get animation progress (0.0 to 1.0)
    pub fn progress(&self, id: AnimationId) -> Option<f32> {
        self.animations.get(&id).map(|a| a.progress)
    }

    /// Is animation running
    pub fn is_running(&self, id: AnimationId) -> bool {
        self.animations.get(&id)
            .map(|a| a.state == AnimationState::Running)
            .unwrap_or(false)
    }

    /// Is animation completed
    pub fn is_completed(&self, id: AnimationId) -> bool {
        self.animations.get(&id)
            .map(|a| a.state == AnimationState::Completed)
            .unwrap_or(false)
    }

    /// Update all animations
    pub fn update(&mut self, delta: Duration) {
        if self.paused {
            return;
        }

        let scaled_delta = Duration::from_secs_f32(delta.as_secs_f32() * self.time_scale);
        let mut completed = Vec::new();

        for (id, anim) in &mut self.animations {
            if anim.state != AnimationState::Running {
                continue;
            }

            anim.elapsed += scaled_delta;
            let duration = anim.config.duration;

            if anim.elapsed >= duration {
                match anim.config.repeat {
                    RepeatMode::None => {
                        anim.progress = 1.0;
                        anim.value = anim.config.to;
                        anim.state = AnimationState::Completed;
                        completed.push(*id);
                    }
                    RepeatMode::Loop(count) => {
                        anim.loops_completed += 1;
                        if count > 0 && anim.loops_completed >= count {
                            anim.progress = 1.0;
                            anim.value = anim.config.to;
                            anim.state = AnimationState::Completed;
                            completed.push(*id);
                        } else {
                            anim.elapsed = Duration::ZERO;
                            anim.progress = 0.0;
                        }
                    }
                    RepeatMode::Reverse => {
                        anim.loops_completed += 1;
                        anim.elapsed = Duration::ZERO;
                        std::mem::swap(&mut anim.config.from, &mut anim.config.to);
                    }
                    RepeatMode::Infinite => {
                        anim.elapsed = Duration::ZERO;
                        anim.progress = 0.0;
                    }
                }
            } else {
                let linear = anim.elapsed.as_secs_f32() / duration.as_secs_f32();
                anim.progress = anim.config.easing.apply(linear);
                anim.value = anim.config.from + (anim.config.to - anim.config.from) * anim.progress;
            }
        }

        // Remove completed non-persistent animations
        for id in completed {
            if !self.animations.get(&id).map(|a| a.config.persist).unwrap_or(false) {
                self.animations.remove(&id);
            }
        }
    }

    /// Set global time scale
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Pause all animations
    pub fn pause_all(&mut self) {
        self.paused = true;
    }

    /// Resume all animations
    pub fn resume_all(&mut self) {
        self.paused = false;
    }

    /// Get active animation count
    pub fn active_count(&self) -> usize {
        self.animations.values()
            .filter(|a| a.state == AnimationState::Running)
            .count()
    }
}

/// Animation configuration
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Start value
    pub from: f32,
    /// End value
    pub to: f32,
    /// Duration
    pub duration: Duration,
    /// Easing function
    pub easing: Easing,
    /// Repeat mode
    pub repeat: RepeatMode,
    /// Delay before starting
    pub delay: Duration,
    /// Keep animation after completion
    pub persist: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 1.0,
            duration: Duration::from_millis(300),
            easing: Easing::EaseInOut,
            repeat: RepeatMode::None,
            delay: Duration::ZERO,
            persist: false,
        }
    }
}

impl AnimationConfig {
    pub fn new(from: f32, to: f32, duration: Duration) -> Self {
        Self {
            from,
            to,
            duration,
            ..Default::default()
        }
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn with_repeat(mut self, repeat: RepeatMode) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn looping(mut self) -> Self {
        self.repeat = RepeatMode::Infinite;
        self
    }

    pub fn reversing(mut self) -> Self {
        self.repeat = RepeatMode::Reverse;
        self
    }
}

/// Animation instance
#[derive(Debug, Clone)]
pub struct Animation {
    /// Animation ID
    pub id: AnimationId,
    /// Configuration
    pub config: AnimationConfig,
    /// Current state
    pub state: AnimationState,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Elapsed time
    pub elapsed: Duration,
    /// When animation started
    pub started_at: Option<Instant>,
    /// Current animated value
    pub value: f32,
    /// Completed loop count
    pub loops_completed: u32,
}

/// Animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Pending,
    Running,
    Paused,
    Stopped,
    Completed,
}

/// Repeat mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    /// Play once
    None,
    /// Loop N times (0 = infinite)
    Loop(u32),
    /// Reverse direction at end
    Reverse,
    /// Loop forever
    Infinite,
}

/// Easing functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    #[default]
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

impl Easing {
    /// Apply easing function to linear progress
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn | Easing::EaseInQuad => t * t,
            Easing::EaseOut | Easing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut | Easing::EaseInOutQuad => {
                if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
            }
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOutCubic => {
                if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
            }
            Easing::EaseInQuart => t * t * t * t,
            Easing::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Easing::EaseInOutQuart => {
                if t < 0.5 { 8.0 * t.powi(4) } else { 1.0 - (-2.0 * t + 2.0).powi(4) / 2.0 }
            }
            Easing::EaseInQuint => t.powi(5),
            Easing::EaseOutQuint => 1.0 - (1.0 - t).powi(5),
            Easing::EaseInOutQuint => {
                if t < 0.5 { 16.0 * t.powi(5) } else { 1.0 - (-2.0 * t + 2.0).powi(5) / 2.0 }
            }
            Easing::EaseInExpo => {
                if t == 0.0 { 0.0 } else { (2.0f32).powf(10.0 * t - 10.0) }
            }
            Easing::EaseOutExpo => {
                if t == 1.0 { 1.0 } else { 1.0 - (2.0f32).powf(-10.0 * t) }
            }
            Easing::EaseInOutExpo => {
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else if t < 0.5 { (2.0f32).powf(20.0 * t - 10.0) / 2.0 }
                else { (2.0 - (2.0f32).powf(-20.0 * t + 10.0)) / 2.0 }
            }
            Easing::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            Easing::EaseOutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Easing::EaseInOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }
            Easing::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Easing::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Easing::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }
            Easing::EaseInElastic => {
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    -(2.0f32).powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c4).sin()
                }
            }
            Easing::EaseOutElastic => {
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    (2.0f32).powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Easing::EaseInOutElastic => {
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else {
                    let c5 = (2.0 * std::f32::consts::PI) / 4.5;
                    if t < 0.5 {
                        -((2.0f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                    } else {
                        ((2.0f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
                    }
                }
            }
            Easing::EaseInBounce => 1.0 - Easing::EaseOutBounce.apply(1.0 - t),
            Easing::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            Easing::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - Easing::EaseOutBounce.apply(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + Easing::EaseOutBounce.apply(2.0 * t - 1.0)) / 2.0
                }
            }
        }
    }
}

/// Spring physics animation
#[derive(Debug, Clone)]
pub struct SpringAnimation {
    /// Current position
    pub position: f32,
    /// Target position
    pub target: f32,
    /// Velocity
    pub velocity: f32,
    /// Stiffness (higher = faster)
    pub stiffness: f32,
    /// Damping (higher = less bouncy)
    pub damping: f32,
    /// Mass
    pub mass: f32,
    /// Velocity threshold for completion
    pub velocity_threshold: f32,
}

impl Default for SpringAnimation {
    fn default() -> Self {
        Self {
            position: 0.0,
            target: 1.0,
            velocity: 0.0,
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
            velocity_threshold: 0.001,
        }
    }
}

impl SpringAnimation {
    pub fn new(stiffness: f32, damping: f32) -> Self {
        Self {
            stiffness,
            damping,
            ..Default::default()
        }
    }

    /// Preset: Gentle spring
    pub fn gentle() -> Self {
        Self::new(80.0, 12.0)
    }

    /// Preset: Bouncy spring
    pub fn bouncy() -> Self {
        Self::new(200.0, 10.0)
    }

    /// Preset: Stiff spring
    pub fn stiff() -> Self {
        Self::new(300.0, 20.0)
    }

    /// Preset: Slow spring
    pub fn slow() -> Self {
        Self::new(50.0, 8.0)
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn update(&mut self, delta: Duration) -> bool {
        let dt = delta.as_secs_f32();
        
        // Spring force: F = -k * x
        let displacement = self.position - self.target;
        let spring_force = -self.stiffness * displacement;
        
        // Damping force: F = -c * v
        let damping_force = -self.damping * self.velocity;
        
        // Newton's second law: F = ma, so a = F/m
        let acceleration = (spring_force + damping_force) / self.mass;
        
        // Update velocity and position
        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;
        
        // Check if settled
        let is_settled = self.velocity.abs() < self.velocity_threshold 
            && (self.position - self.target).abs() < self.velocity_threshold;
        
        if is_settled {
            self.position = self.target;
            self.velocity = 0.0;
        }
        
        !is_settled
    }
}

/// Transition builder for combining animations
#[derive(Debug, Clone)]
pub struct Transition {
    /// Animation entries
    entries: Vec<TransitionEntry>,
}

#[derive(Debug, Clone)]
struct TransitionEntry {
    property: String,
    config: AnimationConfig,
}

impl Transition {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn property(mut self, name: impl Into<String>, config: AnimationConfig) -> Self {
        self.entries.push(TransitionEntry {
            property: name.into(),
            config,
        });
        self
    }

    pub fn opacity(self, duration: Duration) -> Self {
        self.property("opacity", AnimationConfig::new(0.0, 1.0, duration))
    }

    pub fn scale(self, duration: Duration) -> Self {
        self.property("scale", AnimationConfig::new(0.0, 1.0, duration))
    }

    pub fn slide_up(self, distance: f32, duration: Duration) -> Self {
        self.property("translateY", AnimationConfig::new(distance, 0.0, duration))
    }

    pub fn slide_down(self, distance: f32, duration: Duration) -> Self {
        self.property("translateY", AnimationConfig::new(-distance, 0.0, duration))
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::new()
    }
}

/// Staggered animation for lists
#[derive(Debug, Clone)]
pub struct StaggeredAnimation {
    /// Base animation config
    pub config: AnimationConfig,
    /// Delay between each item
    pub stagger_delay: Duration,
    /// Current item count
    pub item_count: usize,
}

impl StaggeredAnimation {
    pub fn new(config: AnimationConfig, stagger_delay: Duration) -> Self {
        Self {
            config,
            stagger_delay,
            item_count: 0,
        }
    }

    /// Get config for item at index
    pub fn config_for_index(&self, index: usize) -> AnimationConfig {
        let mut config = self.config.clone();
        config.delay = Duration::from_secs_f32(
            self.stagger_delay.as_secs_f32() * index as f32
        );
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_controller() {
        let mut controller = AnimationController::new();
        let id = controller.create(AnimationConfig::default());
        assert!(!controller.is_running(id));
        
        controller.start(id);
        assert!(controller.is_running(id));
    }

    #[test]
    fn test_animation_progress() {
        let mut controller = AnimationController::new();
        let config = AnimationConfig::new(0.0, 100.0, Duration::from_millis(100));
        let id = controller.create(config);
        controller.start(id);

        // Update halfway through
        controller.update(Duration::from_millis(50));
        let value = controller.value(id).unwrap();
        assert!(value > 0.0 && value < 100.0);
    }

    #[test]
    fn test_animation_completion() {
        let mut controller = AnimationController::new();
        let mut config = AnimationConfig::new(0.0, 100.0, Duration::from_millis(100));
        config.persist = true;
        let id = controller.create(config);
        controller.start(id);

        // Update past duration
        controller.update(Duration::from_millis(150));
        assert!(controller.is_completed(id));
        assert!((controller.value(id).unwrap() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_linear() {
        let linear = Easing::Linear;
        assert!((linear.apply(0.0) - 0.0).abs() < 0.001);
        assert!((linear.apply(0.5) - 0.5).abs() < 0.001);
        assert!((linear.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_in() {
        let ease_in = Easing::EaseIn;
        assert!((ease_in.apply(0.0) - 0.0).abs() < 0.001);
        assert!(ease_in.apply(0.5) < 0.5); // Slower at start
        assert!((ease_in.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_out() {
        let ease_out = Easing::EaseOut;
        assert!((ease_out.apply(0.0) - 0.0).abs() < 0.001);
        assert!(ease_out.apply(0.5) > 0.5); // Faster at start
        assert!((ease_out.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_spring_animation() {
        let mut spring = SpringAnimation::gentle();
        spring.position = 0.0;
        spring.target = 1.0;

        // Update several times
        for _ in 0..100 {
            spring.update(Duration::from_millis(16));
        }

        // Should approach target
        assert!((spring.position - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_staggered_animation() {
        let config = AnimationConfig::new(0.0, 1.0, Duration::from_millis(300));
        let stagger = StaggeredAnimation::new(config, Duration::from_millis(50));

        let config0 = stagger.config_for_index(0);
        let config2 = stagger.config_for_index(2);

        assert!(config2.delay > config0.delay);
    }

    #[test]
    fn test_transition_builder() {
        let transition = Transition::new()
            .opacity(Duration::from_millis(300))
            .scale(Duration::from_millis(200));

        assert_eq!(transition.entries.len(), 2);
    }
}
