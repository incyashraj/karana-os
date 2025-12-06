// Phase 49: Smart Defaults System
// Context-aware intelligent defaults that learn from user behavior

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context information for determining smart defaults
#[derive(Debug, Clone)]
pub struct DefaultContext {
    pub battery_level: u8,
    pub time_of_day: u8,
    pub location: String,
    pub current_activity: String,
}

/// Tracks usage patterns to learn user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDefaults {
    defaults: HashMap<String, String>,
    usage_counts: HashMap<String, HashMap<String, u32>>,
    confidence_threshold: f32,
}

impl SmartDefaults {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();
        
        defaults.insert("volume".to_string(), "50".to_string());
        defaults.insert("brightness".to_string(), "80".to_string());
        defaults.insert("notification_style".to_string(), "subtle".to_string());
        defaults.insert("voice_speed".to_string(), "normal".to_string());
        
        Self {
            defaults,
            usage_counts: HashMap::new(),
            confidence_threshold: 0.3,
        }
    }

    pub fn get_default(&self, key: &str) -> Option<String> {
        self.defaults.get(key).cloned()
    }

    pub fn set_default(&mut self, key: &str, value: String) {
        self.defaults.insert(key.to_string(), value);
    }

    pub fn record_usage(&mut self, key: &str, value: String) {
        let key_counts = self.usage_counts.entry(key.to_string()).or_insert_with(HashMap::new);
        *key_counts.entry(value.clone()).or_insert(0) += 1;
        
        let total_uses: u32 = key_counts.values().sum();
        let value_uses = key_counts.get(&value).unwrap();
        let frequency = *value_uses as f32 / total_uses as f32;
        
        if frequency >= self.confidence_threshold {
            self.set_default(key, value);
        }
    }

    pub fn get_default_with_context(&self, key: &str, context: &DefaultContext) -> Option<String> {
        match key {
            "brightness" => {
                if context.battery_level < 20 {
                    Some("40".to_string())
                } else if context.time_of_day >= 22 || context.time_of_day < 6 {
                    Some("30".to_string())
                } else {
                    self.get_default(key)
                }
            },
            "notification_style" => {
                if context.current_activity == "meeting" || context.current_activity == "presentation" {
                    Some("silent".to_string())
                } else if context.location == "home" {
                    Some("full".to_string())
                } else {
                    self.get_default(key)
                }
            },
            _ => self.get_default(key),
        }
    }
}

pub struct DefaultTemplates;

impl DefaultTemplates {
    pub fn get_template(name: &str) -> Option<HashMap<String, String>> {
        match name {
            "power_saving" => Some(HashMap::from([
                ("brightness".to_string(), "40".to_string()),
                ("volume".to_string(), "30".to_string()),
                ("notification_style".to_string(), "silent".to_string()),
                ("screen_timeout".to_string(), "30".to_string()),
            ])),
            "performance" => Some(HashMap::from([
                ("brightness".to_string(), "100".to_string()),
                ("volume".to_string(), "80".to_string()),
                ("notification_style".to_string(), "full".to_string()),
                ("screen_timeout".to_string(), "never".to_string()),
            ])),
            "privacy" => Some(HashMap::from([
                ("location_tracking".to_string(), "off".to_string()),
                ("data_sharing".to_string(), "minimal".to_string()),
                ("notification_preview".to_string(), "off".to_string()),
                ("screen_capture".to_string(), "blocked".to_string()),
            ])),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_defaults() {
        let defaults = SmartDefaults::new();
        assert_eq!(defaults.get_default("volume"), Some("50".to_string()));
        assert_eq!(defaults.get_default("brightness"), Some("80".to_string()));
    }

    #[test]
    fn test_usage_tracking() {
        let mut defaults = SmartDefaults::new();
        for _ in 0..5 {
            defaults.record_usage("volume", "75".to_string());
        }
        assert_eq!(defaults.get_default("volume"), Some("75".to_string()));
    }

    #[test]
    fn test_battery_context() {
        let defaults = SmartDefaults::new();
        let context = DefaultContext {
            battery_level: 15,
            time_of_day: 14,
            location: "office".to_string(),
            current_activity: "working".to_string(),
        };
        let brightness = defaults.get_default_with_context("brightness", &context);
        assert_eq!(brightness, Some("40".to_string()));
    }

    #[test]
    fn test_time_context() {
        let defaults = SmartDefaults::new();
        let context = DefaultContext {
            battery_level: 80,
            time_of_day: 23,
            location: "home".to_string(),
            current_activity: "browsing".to_string(),
        };
        let brightness = defaults.get_default_with_context("brightness", &context);
        assert_eq!(brightness, Some("30".to_string()));
    }

    #[test]
    fn test_location_context() {
        let defaults = SmartDefaults::new();
        let home_context = DefaultContext {
            battery_level: 80,
            time_of_day: 14,
            location: "home".to_string(),
            current_activity: "browsing".to_string(),
        };
        let style = defaults.get_default_with_context("notification_style", &home_context);
        assert_eq!(style, Some("full".to_string()));
    }

    #[test]
    fn test_templates() {
        let power_saving = DefaultTemplates::get_template("power_saving").unwrap();
        assert_eq!(power_saving.get("brightness"), Some(&"40".to_string()));
        assert_eq!(power_saving.get("volume"), Some(&"30".to_string()));
        
        let privacy = DefaultTemplates::get_template("privacy").unwrap();
        assert_eq!(privacy.get("location_tracking"), Some(&"off".to_string()));
    }

    #[test]
    fn test_confidence_threshold() {
        let mut defaults = SmartDefaults::new();
        for _ in 0..2 {
            defaults.record_usage("volume", "90".to_string());
        }
        for _ in 0..8 {
            defaults.record_usage("volume", "50".to_string());
        }
        assert_eq!(defaults.get_default("volume"), Some("50".to_string()));
    }

    #[test]
    fn test_user_preference_learning() {
        let mut defaults = SmartDefaults::new();
        for _ in 0..10 {
            defaults.record_usage("volume", "80".to_string());
        }
        assert_eq!(defaults.get_default("volume"), Some("80".to_string()));
    }
}
