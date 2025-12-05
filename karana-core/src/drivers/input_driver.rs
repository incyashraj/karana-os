// Kāraṇa OS - Input Driver
// Low-level driver for touch, buttons, and gesture input

use super::{Driver, DriverError, DriverInfo, DriverState, DriverStats, GpioPin};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::collections::VecDeque;
use std::time::Instant;

/// Input driver configuration
#[derive(Debug, Clone)]
pub struct InputDriverConfig {
    /// Touch device path
    pub touch_device: String,
    /// Enable touch
    pub touch_enabled: bool,
    /// Button GPIO pins
    pub button_pins: Vec<u32>,
    /// Enable buttons
    pub buttons_enabled: bool,
    /// Touch poll rate (Hz)
    pub touch_poll_rate: u32,
    /// Debounce time (ms)
    pub debounce_ms: u32,
}

impl Default for InputDriverConfig {
    fn default() -> Self {
        Self {
            touch_device: "/dev/input/event0".into(),
            touch_enabled: true,
            button_pins: vec![17, 27, 22, 23], // Power, VolUp, VolDown, Action
            buttons_enabled: true,
            touch_poll_rate: 100,
            debounce_ms: 20,
        }
    }
}

/// Input event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEventType {
    /// Key/button event
    Key,
    /// Relative movement (mouse)
    Relative,
    /// Absolute position (touch)
    Absolute,
    /// Synchronization
    Sync,
    /// Multi-touch
    MultiTouch,
}

/// Input event code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputCode {
    // Keys/Buttons
    KeyPower,
    KeyVolumeUp,
    KeyVolumeDown,
    KeyAction,
    KeyBack,

    // Touch
    TouchX,
    TouchY,
    TouchPressure,
    TouchSlot,
    TouchTrackingId,
    TouchToolType,

    // Relative
    RelX,
    RelY,
    RelWheel,

    // Sync
    SyncReport,
    SyncMtReport,

    // Unknown
    Unknown(u16),
}

/// Input event
#[derive(Debug, Clone)]
pub struct InputEvent {
    /// Event type
    pub event_type: InputEventType,
    /// Event code
    pub code: InputCode,
    /// Value
    pub value: i32,
    /// Timestamp (microseconds)
    pub timestamp: u64,
}

/// Touch point (multi-touch)
#[derive(Debug, Clone, Copy)]
pub struct TouchSlot {
    /// Slot ID
    pub slot: u8,
    /// Tracking ID (-1 = released)
    pub tracking_id: i32,
    /// X position
    pub x: i32,
    /// Y position
    pub y: i32,
    /// Pressure
    pub pressure: i32,
    /// Touch width
    pub width: i32,
    /// Touch height
    pub height: i32,
}

impl Default for TouchSlot {
    fn default() -> Self {
        Self {
            slot: 0,
            tracking_id: -1, // -1 means not tracking
            x: 0,
            y: 0,
            pressure: 0,
            width: 0,
            height: 0,
        }
    }
}

/// Button state
#[derive(Debug, Clone)]
pub struct ButtonState {
    /// GPIO pin
    pub pin: u32,
    /// Is pressed
    pub pressed: bool,
    /// Press timestamp
    pub press_time: Option<Instant>,
    /// Last state change
    pub last_change: Option<Instant>,
}

/// Input driver
#[derive(Debug)]
pub struct InputDriver {
    /// Configuration
    config: InputDriverConfig,
    /// Current state
    state: DriverState,
    /// Touch slots
    touch_slots: [TouchSlot; 10],
    /// Current touch slot
    current_slot: u8,
    /// Touch max X
    touch_max_x: i32,
    /// Touch max Y
    touch_max_y: i32,
    /// Button states
    buttons: Vec<ButtonState>,
    /// GPIO pins
    gpios: Vec<GpioPin>,
    /// Event queue
    event_queue: VecDeque<InputEvent>,
    /// Statistics
    stats: DriverStats,
    /// Event counter
    event_count: AtomicU64,
    /// Touch events
    touch_events: AtomicU64,
    /// Button events
    button_events: AtomicU64,
    /// Is polling
    polling: AtomicBool,
}

impl InputDriver {
    /// Create new input driver
    pub fn new(config: InputDriverConfig) -> Self {
        let buttons: Vec<ButtonState> = config.button_pins.iter().map(|&pin| {
            ButtonState {
                pin,
                pressed: false,
                press_time: None,
                last_change: None,
            }
        }).collect();

        Self {
            config,
            state: DriverState::Unloaded,
            touch_slots: [TouchSlot::default(); 10],
            current_slot: 0,
            touch_max_x: 1920,
            touch_max_y: 1080,
            buttons,
            gpios: Vec::new(),
            event_queue: VecDeque::with_capacity(256),
            stats: DriverStats::default(),
            event_count: AtomicU64::new(0),
            touch_events: AtomicU64::new(0),
            button_events: AtomicU64::new(0),
            polling: AtomicBool::new(false),
        }
    }

    /// Process raw input event (from evdev)
    pub fn process_event(&mut self, event_type: u16, code: u16, value: i32) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let (evt_type, evt_code) = match event_type {
            1 => { // EV_KEY
                self.button_events.fetch_add(1, Ordering::Relaxed);
                let code = match code {
                    116 => InputCode::KeyPower,
                    115 => InputCode::KeyVolumeUp,
                    114 => InputCode::KeyVolumeDown,
                    158 => InputCode::KeyBack,
                    _ => InputCode::Unknown(code),
                };
                (InputEventType::Key, code)
            }
            2 => { // EV_REL
                let code = match code {
                    0 => InputCode::RelX,
                    1 => InputCode::RelY,
                    8 => InputCode::RelWheel,
                    _ => InputCode::Unknown(code),
                };
                (InputEventType::Relative, code)
            }
            3 => { // EV_ABS
                self.touch_events.fetch_add(1, Ordering::Relaxed);
                let code = match code {
                    0 | 53 => InputCode::TouchX,
                    1 | 54 => InputCode::TouchY,
                    24 | 58 => InputCode::TouchPressure,
                    47 => InputCode::TouchSlot,
                    57 => InputCode::TouchTrackingId,
                    _ => InputCode::Unknown(code),
                };
                
                // Update touch slots
                match code {
                    InputCode::TouchSlot => self.current_slot = value as u8,
                    InputCode::TouchX => self.touch_slots[self.current_slot as usize].x = value,
                    InputCode::TouchY => self.touch_slots[self.current_slot as usize].y = value,
                    InputCode::TouchPressure => self.touch_slots[self.current_slot as usize].pressure = value,
                    InputCode::TouchTrackingId => self.touch_slots[self.current_slot as usize].tracking_id = value,
                    _ => {}
                }
                
                (InputEventType::Absolute, code)
            }
            0 => { // EV_SYN
                let code = match code {
                    0 => InputCode::SyncReport,
                    2 => InputCode::SyncMtReport,
                    _ => InputCode::Unknown(code),
                };
                (InputEventType::Sync, code)
            }
            _ => (InputEventType::Sync, InputCode::Unknown(code)),
        };

        self.event_count.fetch_add(1, Ordering::Relaxed);
        self.event_queue.push_back(InputEvent {
            event_type: evt_type,
            code: evt_code,
            value,
            timestamp,
        });
    }

    /// Poll buttons
    pub fn poll_buttons(&mut self) -> Result<(), DriverError> {
        let now = Instant::now();

        for (i, gpio) in self.gpios.iter().enumerate() {
            if i >= self.buttons.len() {
                break;
            }

            let pressed = !gpio.get_value()?; // Active low

            // Debounce
            if let Some(last_change) = self.buttons[i].last_change {
                if now.duration_since(last_change).as_millis() < self.config.debounce_ms as u128 {
                    continue;
                }
            }

            if pressed != self.buttons[i].pressed {
                self.buttons[i].pressed = pressed;
                self.buttons[i].last_change = Some(now);
                
                if pressed {
                    self.buttons[i].press_time = Some(now);
                } else {
                    self.buttons[i].press_time = None;
                }

                // Generate event
                let code = match i {
                    0 => InputCode::KeyPower,
                    1 => InputCode::KeyVolumeUp,
                    2 => InputCode::KeyVolumeDown,
                    3 => InputCode::KeyAction,
                    _ => InputCode::Unknown(i as u16),
                };

                self.event_queue.push_back(InputEvent {
                    event_type: InputEventType::Key,
                    code,
                    value: if pressed { 1 } else { 0 },
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros() as u64,
                });

                self.button_events.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    /// Get next event
    pub fn poll_event(&mut self) -> Option<InputEvent> {
        self.event_queue.pop_front()
    }

    /// Get touch slots
    pub fn touch_slots(&self) -> &[TouchSlot; 10] {
        &self.touch_slots
    }

    /// Get active touch count
    pub fn active_touch_count(&self) -> usize {
        self.touch_slots.iter().filter(|s| s.tracking_id >= 0).count()
    }

    /// Get button state
    pub fn button_state(&self, index: usize) -> Option<bool> {
        self.buttons.get(index).map(|b| b.pressed)
    }

    /// Is any button pressed
    pub fn any_button_pressed(&self) -> bool {
        self.buttons.iter().any(|b| b.pressed)
    }

    /// Get event count
    pub fn event_count(&self) -> u64 {
        self.event_count.load(Ordering::Relaxed)
    }

    /// Get touch event count
    pub fn touch_event_count(&self) -> u64 {
        self.touch_events.load(Ordering::Relaxed)
    }

    /// Get button event count
    pub fn button_event_count(&self) -> u64 {
        self.button_events.load(Ordering::Relaxed)
    }

    /// Set touch resolution
    pub fn set_touch_resolution(&mut self, max_x: i32, max_y: i32) {
        self.touch_max_x = max_x;
        self.touch_max_y = max_y;
    }

    /// Get normalized touch position
    pub fn get_normalized_touch(&self, slot: u8) -> Option<(f32, f32)> {
        let touch = &self.touch_slots[slot as usize];
        if touch.tracking_id >= 0 {
            Some((
                touch.x as f32 / self.touch_max_x as f32,
                touch.y as f32 / self.touch_max_y as f32,
            ))
        } else {
            None
        }
    }
}

impl Driver for InputDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "karana-input".into(),
            version: "1.0.0".into(),
            vendor: "KaranaOS".into(),
            device_ids: vec!["input:evdev".into(), "input:gpio".into()],
            loaded: self.state != DriverState::Unloaded,
            state: self.state,
        }
    }

    fn state(&self) -> DriverState {
        self.state
    }

    fn load(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Loading;

        // Setup button GPIOs
        if self.config.buttons_enabled {
            for &pin in &self.config.button_pins {
                let mut gpio = GpioPin::new(pin);
                gpio.export()?;
                gpio.set_direction(false)?; // Input
                self.gpios.push(gpio);
            }
        }

        self.state = DriverState::Loaded;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), DriverError> {
        for gpio in &mut self.gpios {
            gpio.unexport()?;
        }
        self.gpios.clear();
        self.state = DriverState::Unloaded;
        Ok(())
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Loaded {
            return Err(DriverError::NotLoaded);
        }

        // Would read touch device properties
        // ioctl EVIOCGABS to get ABS_MT_POSITION_X/Y max values

        self.state = DriverState::Ready;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Ready {
            return Err(DriverError::NotLoaded);
        }
        self.polling.store(true, Ordering::Relaxed);
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.polling.store(false, Ordering::Relaxed);
        self.state = DriverState::Ready;
        Ok(())
    }

    fn suspend(&mut self) -> Result<(), DriverError> {
        self.polling.store(false, Ordering::Relaxed);
        self.state = DriverState::Suspended;
        Ok(())
    }

    fn resume(&mut self) -> Result<(), DriverError> {
        self.polling.store(true, Ordering::Relaxed);
        self.state = DriverState::Running;
        Ok(())
    }

    fn stats(&self) -> DriverStats {
        DriverStats {
            interrupts: self.event_count.load(Ordering::Relaxed),
            ..self.stats.clone()
        }
    }

    fn test(&self) -> Result<(), DriverError> {
        if self.state == DriverState::Unloaded {
            return Err(DriverError::NotLoaded);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_driver_creation() {
        let driver = InputDriver::new(InputDriverConfig::default());
        assert_eq!(driver.state(), DriverState::Unloaded);
    }

    #[test]
    fn test_input_driver_lifecycle() {
        let mut driver = InputDriver::new(InputDriverConfig {
            buttons_enabled: false, // Skip GPIO for test
            ..Default::default()
        });
        
        driver.load().unwrap();
        driver.init().unwrap();
        driver.start().unwrap();
        
        driver.stop().unwrap();
        driver.unload().unwrap();
    }

    #[test]
    fn test_process_touch_event() {
        let mut driver = InputDriver::new(InputDriverConfig::default());
        
        // Simulate touch events
        driver.process_event(3, 47, 0); // Slot 0
        driver.process_event(3, 57, 1); // Tracking ID 1
        driver.process_event(3, 53, 500); // X
        driver.process_event(3, 54, 300); // Y
        driver.process_event(0, 0, 0); // Sync
        
        assert!(driver.touch_slots[0].tracking_id >= 0);
        assert_eq!(driver.touch_slots[0].x, 500);
        assert_eq!(driver.touch_slots[0].y, 300);
    }

    #[test]
    fn test_process_key_event() {
        let mut driver = InputDriver::new(InputDriverConfig::default());
        
        driver.process_event(1, 116, 1); // Power key pressed
        
        let event = driver.poll_event().unwrap();
        assert_eq!(event.event_type, InputEventType::Key);
        assert_eq!(event.code, InputCode::KeyPower);
        assert_eq!(event.value, 1);
    }

    #[test]
    fn test_active_touch_count() {
        let mut driver = InputDriver::new(InputDriverConfig::default());
        
        driver.touch_slots[0].tracking_id = 1;
        driver.touch_slots[1].tracking_id = 2;
        driver.touch_slots[2].tracking_id = -1;
        
        assert_eq!(driver.active_touch_count(), 2);
    }

    #[test]
    fn test_normalized_touch() {
        let mut driver = InputDriver::new(InputDriverConfig::default());
        driver.set_touch_resolution(1920, 1080);
        
        driver.touch_slots[0].tracking_id = 1;
        driver.touch_slots[0].x = 960;
        driver.touch_slots[0].y = 540;
        
        let (x, y) = driver.get_normalized_touch(0).unwrap();
        assert!((x - 0.5).abs() < 0.01);
        assert!((y - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_driver_info() {
        let driver = InputDriver::new(InputDriverConfig::default());
        let info = driver.info();
        
        assert_eq!(info.name, "karana-input");
        assert!(!info.loaded);
    }
}
