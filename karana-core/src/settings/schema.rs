//! Setting schema and validation

use serde::{Deserialize, Serialize};

/// Setting value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// List of values
    List(Vec<SettingValue>),
    /// Map of values
    Map(std::collections::HashMap<String, SettingValue>),
}

impl SettingValue {
    /// Get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SettingValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Get as int
    pub fn as_int(&self) -> Option<i64> {
        match self {
            SettingValue::Int(i) => Some(*i),
            SettingValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }
    
    /// Get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            SettingValue::Float(f) => Some(*f),
            SettingValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
    
    /// Get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            SettingValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Get as list
    pub fn as_list(&self) -> Option<&Vec<SettingValue>> {
        match self {
            SettingValue::List(l) => Some(l),
            _ => None,
        }
    }
    
    /// Get type name
    pub fn type_name(&self) -> &'static str {
        match self {
            SettingValue::Bool(_) => "bool",
            SettingValue::Int(_) => "int",
            SettingValue::Float(_) => "float",
            SettingValue::String(_) => "string",
            SettingValue::List(_) => "list",
            SettingValue::Map(_) => "map",
        }
    }
}

/// Value constraint
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Minimum value (for numbers)
    Min(f64),
    /// Maximum value (for numbers)
    Max(f64),
    /// Minimum length (for strings/lists)
    MinLength(usize),
    /// Maximum length (for strings/lists)
    MaxLength(usize),
    /// Pattern (regex for strings)
    Pattern(String),
    /// Allowed values (enum)
    OneOf(Vec<SettingValue>),
    /// Custom validation function name
    Custom(String),
}

/// Setting schema definition
#[derive(Debug, Clone)]
pub struct SettingSchema {
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Default value
    pub default_value: SettingValue,
    /// Value type
    pub value_type: ValueType,
    /// Constraints
    pub constraints: Vec<Constraint>,
    /// Whether setting is hidden
    pub hidden: bool,
    /// Whether setting requires restart
    pub requires_restart: bool,
    /// Category
    pub category: Option<String>,
    /// Tags for filtering
    pub tags: Vec<String>,
}

/// Value type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Bool,
    Int,
    Float,
    String,
    List,
    Map,
}

impl SettingSchema {
    /// Create bool schema
    pub fn bool(default: bool) -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            default_value: SettingValue::Bool(default),
            value_type: ValueType::Bool,
            constraints: Vec::new(),
            hidden: false,
            requires_restart: false,
            category: None,
            tags: Vec::new(),
        }
    }
    
    /// Create int schema with range
    pub fn int(min: i64, max: i64, default: i64) -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            default_value: SettingValue::Int(default),
            value_type: ValueType::Int,
            constraints: vec![
                Constraint::Min(min as f64),
                Constraint::Max(max as f64),
            ],
            hidden: false,
            requires_restart: false,
            category: None,
            tags: Vec::new(),
        }
    }
    
    /// Create float schema with range
    pub fn float(min: f64, max: f64, default: f64) -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            default_value: SettingValue::Float(default),
            value_type: ValueType::Float,
            constraints: vec![
                Constraint::Min(min),
                Constraint::Max(max),
            ],
            hidden: false,
            requires_restart: false,
            category: None,
            tags: Vec::new(),
        }
    }
    
    /// Create string schema
    pub fn string(default: String) -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            default_value: SettingValue::String(default),
            value_type: ValueType::String,
            constraints: Vec::new(),
            hidden: false,
            requires_restart: false,
            category: None,
            tags: Vec::new(),
        }
    }
    
    /// Create string enum schema
    pub fn string_enum(options: Vec<String>, default: &str) -> Self {
        let one_of: Vec<SettingValue> = options.iter()
            .map(|s| SettingValue::String(s.clone()))
            .collect();
        
        Self {
            name: String::new(),
            description: String::new(),
            default_value: SettingValue::String(default.to_string()),
            value_type: ValueType::String,
            constraints: vec![Constraint::OneOf(one_of)],
            hidden: false,
            requires_restart: false,
            category: None,
            tags: Vec::new(),
        }
    }
    
    /// Set name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    
    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }
    
    /// Set hidden
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
    
    /// Set requires restart
    pub fn requires_restart(mut self) -> Self {
        self.requires_restart = true;
        self
    }
    
    /// Validate a value against this schema
    pub fn validate(&self, value: &SettingValue) -> Result<(), ValidationError> {
        // Type check
        let actual_type = match value {
            SettingValue::Bool(_) => ValueType::Bool,
            SettingValue::Int(_) => ValueType::Int,
            SettingValue::Float(_) => ValueType::Float,
            SettingValue::String(_) => ValueType::String,
            SettingValue::List(_) => ValueType::List,
            SettingValue::Map(_) => ValueType::Map,
        };
        
        // Allow int/float interchangeability
        let type_ok = actual_type == self.value_type ||
            (actual_type == ValueType::Int && self.value_type == ValueType::Float) ||
            (actual_type == ValueType::Float && self.value_type == ValueType::Int);
        
        if !type_ok {
            return Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", self.value_type),
                actual: format!("{:?}", actual_type),
            });
        }
        
        // Check constraints
        for constraint in &self.constraints {
            match constraint {
                Constraint::Min(min) => {
                    if let Some(n) = value.as_float() {
                        if n < *min {
                            return Err(ValidationError::OutOfRange {
                                value: n,
                                min: Some(*min),
                                max: None,
                            });
                        }
                    }
                }
                Constraint::Max(max) => {
                    if let Some(n) = value.as_float() {
                        if n > *max {
                            return Err(ValidationError::OutOfRange {
                                value: n,
                                min: None,
                                max: Some(*max),
                            });
                        }
                    }
                }
                Constraint::MinLength(min) => {
                    if let Some(s) = value.as_string() {
                        if s.len() < *min {
                            return Err(ValidationError::LengthOutOfRange {
                                length: s.len(),
                                min: Some(*min),
                                max: None,
                            });
                        }
                    }
                }
                Constraint::MaxLength(max) => {
                    if let Some(s) = value.as_string() {
                        if s.len() > *max {
                            return Err(ValidationError::LengthOutOfRange {
                                length: s.len(),
                                min: None,
                                max: Some(*max),
                            });
                        }
                    }
                }
                Constraint::OneOf(allowed) => {
                    if !allowed.contains(value) {
                        return Err(ValidationError::NotInAllowedValues {
                            value: format!("{:?}", value),
                            allowed: allowed.iter().map(|v| format!("{:?}", v)).collect(),
                        });
                    }
                }
                Constraint::Pattern(_pattern) => {
                    // In real implementation, would use regex
                    // For now, skip pattern validation
                }
                Constraint::Custom(_) => {
                    // Custom validation would be handled elsewhere
                }
            }
        }
        
        Ok(())
    }
}

/// Validation error
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Type mismatch
    TypeMismatch {
        expected: String,
        actual: String,
    },
    /// Value out of range
    OutOfRange {
        value: f64,
        min: Option<f64>,
        max: Option<f64>,
    },
    /// Length out of range
    LengthOutOfRange {
        length: usize,
        min: Option<usize>,
        max: Option<usize>,
    },
    /// Not in allowed values
    NotInAllowedValues {
        value: String,
        allowed: Vec<String>,
    },
    /// Pattern mismatch
    PatternMismatch {
        value: String,
        pattern: String,
    },
    /// Unknown setting
    UnknownSetting(String),
    /// Custom error
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            ValidationError::OutOfRange { value, min, max } => {
                write!(f, "Value {} out of range [{:?}, {:?}]", value, min, max)
            }
            ValidationError::LengthOutOfRange { length, min, max } => {
                write!(f, "Length {} out of range [{:?}, {:?}]", length, min, max)
            }
            ValidationError::NotInAllowedValues { value, allowed } => {
                write!(f, "Value {} not in allowed values: {:?}", value, allowed)
            }
            ValidationError::PatternMismatch { value, pattern } => {
                write!(f, "Value {} does not match pattern {}", value, pattern)
            }
            ValidationError::UnknownSetting(key) => {
                write!(f, "Unknown setting: {}", key)
            }
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_setting_value_types() {
        let bool_val = SettingValue::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        
        let int_val = SettingValue::Int(42);
        assert_eq!(int_val.as_int(), Some(42));
        
        let float_val = SettingValue::Float(3.14);
        assert!((float_val.as_float().unwrap() - 3.14).abs() < 0.01);
        
        let string_val = SettingValue::String("hello".to_string());
        assert_eq!(string_val.as_string(), Some("hello"));
    }
    
    #[test]
    fn test_schema_validation() {
        let schema = SettingSchema::float(0.0, 1.0, 0.5);
        
        // Valid
        assert!(schema.validate(&SettingValue::Float(0.5)).is_ok());
        
        // Out of range
        assert!(schema.validate(&SettingValue::Float(1.5)).is_err());
        assert!(schema.validate(&SettingValue::Float(-0.5)).is_err());
    }
    
    #[test]
    fn test_int_schema() {
        let schema = SettingSchema::int(0, 100, 50);
        
        assert!(schema.validate(&SettingValue::Int(50)).is_ok());
        assert!(schema.validate(&SettingValue::Int(150)).is_err());
    }
    
    #[test]
    fn test_bool_schema() {
        let schema = SettingSchema::bool(false);
        
        assert!(schema.validate(&SettingValue::Bool(true)).is_ok());
        assert!(schema.validate(&SettingValue::String("true".to_string())).is_err());
    }
    
    #[test]
    fn test_enum_schema() {
        let schema = SettingSchema::string_enum(
            vec!["low".to_string(), "medium".to_string(), "high".to_string()],
            "medium",
        );
        
        assert!(schema.validate(&SettingValue::String("low".to_string())).is_ok());
        assert!(schema.validate(&SettingValue::String("invalid".to_string())).is_err());
    }
    
    #[test]
    fn test_type_name() {
        assert_eq!(SettingValue::Bool(true).type_name(), "bool");
        assert_eq!(SettingValue::Int(42).type_name(), "int");
        assert_eq!(SettingValue::Float(3.14).type_name(), "float");
        assert_eq!(SettingValue::String("test".to_string()).type_name(), "string");
    }
}
