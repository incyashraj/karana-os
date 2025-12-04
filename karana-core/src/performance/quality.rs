// Quality Manager for Kāraṇa OS
// Adaptive quality settings for AR rendering and processing

use std::collections::HashMap;

/// Quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualityLevel {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

/// Individual quality setting
#[derive(Debug, Clone)]
pub struct QualitySetting {
    pub name: String,
    pub value: QualityValue,
    pub min: QualityValue,
    pub max: QualityValue,
    pub affects_performance: f32, // 0.0 - 1.0 impact weight
}

/// Quality setting value types
#[derive(Debug, Clone, PartialEq)]
pub enum QualityValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    Enum(String),
}

impl QualityValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            QualityValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            QualityValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            QualityValue::Float(v) => Some(*v),
            _ => None,
        }
    }
}

/// Texture quality levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureQuality {
    Low,      // 256x256 max
    Medium,   // 512x512 max
    High,     // 1024x1024 max
    Ultra,    // 2048x2048 max
}

/// Anti-aliasing modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AntiAliasing {
    None,
    FXAA,
    MSAA2x,
    MSAA4x,
    TAA,
}

/// Shadow quality
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
}

/// AR overlay quality settings
#[derive(Debug, Clone)]
pub struct ArQualitySettings {
    pub resolution_scale: f32,
    pub texture_quality: TextureQuality,
    pub anti_aliasing: AntiAliasing,
    pub shadow_quality: ShadowQuality,
    pub occlusion_enabled: bool,
    pub reflections_enabled: bool,
    pub particle_density: f32,
    pub draw_distance: f32,
    pub lod_bias: f32,
}

impl Default for ArQualitySettings {
    fn default() -> Self {
        Self {
            resolution_scale: 1.0,
            texture_quality: TextureQuality::High,
            anti_aliasing: AntiAliasing::FXAA,
            shadow_quality: ShadowQuality::Medium,
            occlusion_enabled: true,
            reflections_enabled: false,
            particle_density: 0.5,
            draw_distance: 50.0,
            lod_bias: 0.0,
        }
    }
}

impl ArQualitySettings {
    pub fn for_level(level: QualityLevel) -> Self {
        match level {
            QualityLevel::Low => Self {
                resolution_scale: 0.5,
                texture_quality: TextureQuality::Low,
                anti_aliasing: AntiAliasing::None,
                shadow_quality: ShadowQuality::Off,
                occlusion_enabled: false,
                reflections_enabled: false,
                particle_density: 0.1,
                draw_distance: 20.0,
                lod_bias: 2.0,
            },
            QualityLevel::Medium => Self {
                resolution_scale: 0.75,
                texture_quality: TextureQuality::Medium,
                anti_aliasing: AntiAliasing::FXAA,
                shadow_quality: ShadowQuality::Low,
                occlusion_enabled: true,
                reflections_enabled: false,
                particle_density: 0.3,
                draw_distance: 35.0,
                lod_bias: 1.0,
            },
            QualityLevel::High => Self::default(),
            QualityLevel::Ultra => Self {
                resolution_scale: 1.0,
                texture_quality: TextureQuality::Ultra,
                anti_aliasing: AntiAliasing::MSAA4x,
                shadow_quality: ShadowQuality::High,
                occlusion_enabled: true,
                reflections_enabled: true,
                particle_density: 1.0,
                draw_distance: 100.0,
                lod_bias: -1.0,
            },
            QualityLevel::Custom => Self::default(),
        }
    }

    pub fn estimated_load(&self) -> f32 {
        let mut load = 0.0;
        
        load += self.resolution_scale * self.resolution_scale * 30.0;
        load += match self.texture_quality {
            TextureQuality::Low => 5.0,
            TextureQuality::Medium => 10.0,
            TextureQuality::High => 15.0,
            TextureQuality::Ultra => 25.0,
        };
        load += match self.anti_aliasing {
            AntiAliasing::None => 0.0,
            AntiAliasing::FXAA => 5.0,
            AntiAliasing::MSAA2x => 10.0,
            AntiAliasing::MSAA4x => 20.0,
            AntiAliasing::TAA => 8.0,
        };
        load += match self.shadow_quality {
            ShadowQuality::Off => 0.0,
            ShadowQuality::Low => 5.0,
            ShadowQuality::Medium => 10.0,
            ShadowQuality::High => 20.0,
        };
        if self.occlusion_enabled { load += 5.0; }
        if self.reflections_enabled { load += 15.0; }
        
        load
    }
}

/// Quality manager
pub struct QualityManager {
    level: QualityLevel,
    ar_settings: ArQualitySettings,
    custom_settings: HashMap<String, QualitySetting>,
    auto_adjust: bool,
    target_frame_time_ms: f32,
    adjustment_cooldown_ms: u64,
    last_adjustment_time: u64,
}

impl QualityManager {
    pub fn new() -> Self {
        Self {
            level: QualityLevel::High,
            ar_settings: ArQualitySettings::default(),
            custom_settings: HashMap::new(),
            auto_adjust: true,
            target_frame_time_ms: 16.67,
            adjustment_cooldown_ms: 1000,
            last_adjustment_time: 0,
        }
    }

    pub fn set_level(&mut self, level: QualityLevel) {
        self.level = level;
        if level != QualityLevel::Custom {
            self.ar_settings = ArQualitySettings::for_level(level);
        }
    }

    pub fn get_level(&self) -> QualityLevel {
        self.level
    }

    pub fn get_ar_settings(&self) -> &ArQualitySettings {
        &self.ar_settings
    }

    pub fn get_ar_settings_mut(&mut self) -> &mut ArQualitySettings {
        self.level = QualityLevel::Custom;
        &mut self.ar_settings
    }

    pub fn set_auto_adjust(&mut self, enabled: bool) {
        self.auto_adjust = enabled;
    }

    pub fn is_auto_adjust_enabled(&self) -> bool {
        self.auto_adjust
    }

    pub fn set_target_frame_time(&mut self, ms: f32) {
        self.target_frame_time_ms = ms;
    }

    pub fn adjust_for_performance(&mut self, current_frame_time_ms: f32, current_time_ms: u64) {
        if !self.auto_adjust {
            return;
        }

        // Skip cooldown check if this is the first adjustment (last_adjustment_time == 0)
        if self.last_adjustment_time > 0 && 
           current_time_ms - self.last_adjustment_time < self.adjustment_cooldown_ms {
            return;
        }

        let ratio = current_frame_time_ms / self.target_frame_time_ms;

        if ratio > 1.2 {
            // Performance is poor, reduce quality
            self.decrease_quality();
            self.last_adjustment_time = current_time_ms;
        } else if ratio < 0.7 && self.level != QualityLevel::Ultra {
            // Performance is great, try increasing quality
            self.increase_quality();
            self.last_adjustment_time = current_time_ms;
        }
    }

    fn decrease_quality(&mut self) {
        match self.level {
            QualityLevel::Ultra => self.set_level(QualityLevel::High),
            QualityLevel::High => self.set_level(QualityLevel::Medium),
            QualityLevel::Medium => self.set_level(QualityLevel::Low),
            QualityLevel::Low | QualityLevel::Custom => {
                // Already at lowest or custom, try reducing individual settings
                if self.ar_settings.resolution_scale > 0.5 {
                    self.ar_settings.resolution_scale -= 0.1;
                }
            }
        }
    }

    fn increase_quality(&mut self) {
        match self.level {
            QualityLevel::Low => self.set_level(QualityLevel::Medium),
            QualityLevel::Medium => self.set_level(QualityLevel::High),
            QualityLevel::High => self.set_level(QualityLevel::Ultra),
            QualityLevel::Ultra | QualityLevel::Custom => {}
        }
    }

    pub fn set_custom_setting(&mut self, name: &str, setting: QualitySetting) {
        self.custom_settings.insert(name.to_string(), setting);
    }

    pub fn get_custom_setting(&self, name: &str) -> Option<&QualitySetting> {
        self.custom_settings.get(name)
    }

    pub fn get_estimated_load(&self) -> f32 {
        self.ar_settings.estimated_load()
    }

    pub fn get_quality_score(&self) -> f32 {
        // 0.0 - 100.0 quality score
        match self.level {
            QualityLevel::Low => 25.0,
            QualityLevel::Medium => 50.0,
            QualityLevel::High => 75.0,
            QualityLevel::Ultra => 100.0,
            QualityLevel::Custom => {
                // Calculate based on settings
                let load = self.ar_settings.estimated_load();
                (load / 1.2).min(100.0)
            }
        }
    }

    pub fn apply_preset(&mut self, preset_name: &str) -> bool {
        match preset_name {
            "battery_saver" => {
                self.set_level(QualityLevel::Low);
                self.ar_settings.draw_distance = 15.0;
                true
            }
            "balanced" => {
                self.set_level(QualityLevel::Medium);
                true
            }
            "quality" => {
                self.set_level(QualityLevel::High);
                true
            }
            "performance" => {
                self.set_level(QualityLevel::Ultra);
                true
            }
            _ => false,
        }
    }
}

impl Default for QualityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_manager_creation() {
        let manager = QualityManager::new();
        assert_eq!(manager.get_level(), QualityLevel::High);
    }

    #[test]
    fn test_set_level() {
        let mut manager = QualityManager::new();
        manager.set_level(QualityLevel::Low);
        assert_eq!(manager.get_level(), QualityLevel::Low);
        assert_eq!(manager.get_ar_settings().texture_quality, TextureQuality::Low);
    }

    #[test]
    fn test_ar_quality_presets() {
        let low = ArQualitySettings::for_level(QualityLevel::Low);
        let ultra = ArQualitySettings::for_level(QualityLevel::Ultra);
        
        assert!(low.resolution_scale < ultra.resolution_scale);
        assert!(low.draw_distance < ultra.draw_distance);
    }

    #[test]
    fn test_estimated_load() {
        let low = ArQualitySettings::for_level(QualityLevel::Low);
        let ultra = ArQualitySettings::for_level(QualityLevel::Ultra);
        
        assert!(low.estimated_load() < ultra.estimated_load());
    }

    #[test]
    fn test_auto_adjust_decrease() {
        let mut manager = QualityManager::new();
        manager.set_level(QualityLevel::High);
        manager.set_auto_adjust(true);
        manager.set_target_frame_time(16.67);
        
        // Simulate poor performance
        manager.adjust_for_performance(25.0, 2000);
        
        assert_eq!(manager.get_level(), QualityLevel::Medium);
    }

    #[test]
    fn test_auto_adjust_increase() {
        let mut manager = QualityManager::new();
        manager.set_level(QualityLevel::Medium);
        manager.set_auto_adjust(true);
        manager.set_target_frame_time(16.67);
        
        // Simulate great performance
        manager.adjust_for_performance(8.0, 2000);
        
        assert_eq!(manager.get_level(), QualityLevel::High);
    }

    #[test]
    fn test_cooldown() {
        let mut manager = QualityManager::new();
        manager.set_level(QualityLevel::High);
        manager.set_auto_adjust(true);
        
        manager.adjust_for_performance(25.0, 100);
        assert_eq!(manager.get_level(), QualityLevel::Medium);
        
        // Too soon, should not adjust
        manager.adjust_for_performance(25.0, 500);
        assert_eq!(manager.get_level(), QualityLevel::Medium);
    }

    #[test]
    fn test_custom_settings() {
        let mut manager = QualityManager::new();
        let setting = QualitySetting {
            name: "bloom_intensity".to_string(),
            value: QualityValue::Float(0.5),
            min: QualityValue::Float(0.0),
            max: QualityValue::Float(1.0),
            affects_performance: 0.1,
        };
        
        manager.set_custom_setting("bloom", setting);
        let retrieved = manager.get_custom_setting("bloom");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_quality_value_conversions() {
        let bool_val = QualityValue::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_int(), None);
        
        let int_val = QualityValue::Int(42);
        assert_eq!(int_val.as_int(), Some(42));
        
        let float_val = QualityValue::Float(3.14);
        assert!((float_val.as_float().unwrap() - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_presets() {
        let mut manager = QualityManager::new();
        
        assert!(manager.apply_preset("battery_saver"));
        assert_eq!(manager.get_level(), QualityLevel::Low);
        
        assert!(!manager.apply_preset("invalid_preset"));
    }

    #[test]
    fn test_quality_score() {
        let mut manager = QualityManager::new();
        
        manager.set_level(QualityLevel::Low);
        let low_score = manager.get_quality_score();
        
        manager.set_level(QualityLevel::Ultra);
        let ultra_score = manager.get_quality_score();
        
        assert!(low_score < ultra_score);
    }

    #[test]
    fn test_modify_ar_settings() {
        let mut manager = QualityManager::new();
        manager.set_level(QualityLevel::High);
        
        {
            let settings = manager.get_ar_settings_mut();
            settings.shadow_quality = ShadowQuality::Off;
        }
        
        // Should switch to custom when settings are modified directly
        assert_eq!(manager.get_level(), QualityLevel::Custom);
        assert_eq!(manager.get_ar_settings().shadow_quality, ShadowQuality::Off);
    }
}
