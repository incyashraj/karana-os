//! Settings schema migration

use std::collections::HashMap;
use super::schema::SettingValue;

/// Migration version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MigrationVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl MigrationVersion {
    /// Create new version
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
    
    /// Parse from string "major.minor.patch"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        
        Some(Self { major, minor, patch })
    }
}

impl std::fmt::Display for MigrationVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Migration step
pub struct Migration {
    /// From version
    pub from: MigrationVersion,
    /// To version
    pub to: MigrationVersion,
    /// Description
    pub description: String,
    /// Migration function
    pub migrate: Box<dyn Fn(&mut HashMap<String, SettingValue>) + Send + Sync>,
}

impl std::fmt::Debug for Migration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Migration")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("description", &self.description)
            .finish()
    }
}

/// Migration runner
#[derive(Debug)]
pub struct MigrationRunner {
    /// Registered migrations
    migrations: Vec<Migration>,
    /// Current version
    current_version: MigrationVersion,
}

impl MigrationRunner {
    /// Create new migration runner
    pub fn new(current_version: MigrationVersion) -> Self {
        let mut runner = Self {
            migrations: Vec::new(),
            current_version,
        };
        
        runner.register_default_migrations();
        runner
    }
    
    /// Register default migrations
    fn register_default_migrations(&mut self) {
        // Example: Migration from 1.0.0 to 1.1.0
        // Renamed display.brightness_level to display.brightness
        self.register(Migration {
            from: MigrationVersion::new(1, 0, 0),
            to: MigrationVersion::new(1, 1, 0),
            description: "Rename brightness_level to brightness".to_string(),
            migrate: Box::new(|settings| {
                if let Some(value) = settings.remove("display.brightness_level") {
                    settings.insert("display.brightness".to_string(), value);
                }
            }),
        });
        
        // Example: Migration from 1.1.0 to 1.2.0
        // Split audio.volume into audio.master_volume and audio.notification_volume
        self.register(Migration {
            from: MigrationVersion::new(1, 1, 0),
            to: MigrationVersion::new(1, 2, 0),
            description: "Split volume settings".to_string(),
            migrate: Box::new(|settings| {
                if let Some(value) = settings.get("audio.volume").cloned() {
                    settings.insert("audio.master_volume".to_string(), value.clone());
                    settings.insert("audio.notification_volume".to_string(), value);
                }
            }),
        });
        
        // Example: Migration from 1.2.0 to 2.0.0
        // Changed power.auto_sleep from bool to int (minutes)
        self.register(Migration {
            from: MigrationVersion::new(1, 2, 0),
            to: MigrationVersion::new(2, 0, 0),
            description: "Convert auto_sleep to duration".to_string(),
            migrate: Box::new(|settings| {
                if let Some(SettingValue::Bool(enabled)) = settings.remove("power.auto_sleep") {
                    let minutes = if enabled { 5 } else { 0 };
                    settings.insert("power.auto_sleep_minutes".to_string(), SettingValue::Int(minutes));
                }
            }),
        });
    }
    
    /// Register a migration
    pub fn register(&mut self, migration: Migration) {
        self.migrations.push(migration);
        // Sort by from version
        self.migrations.sort_by(|a, b| a.from.cmp(&b.from));
    }
    
    /// Run migrations from a version to current
    pub fn run(&self, from_version: MigrationVersion, settings: &mut HashMap<String, SettingValue>) -> Result<usize, String> {
        let applicable: Vec<_> = self.migrations
            .iter()
            .filter(|m| m.from >= from_version && m.to <= self.current_version)
            .collect();
        
        let count = applicable.len();
        
        for migration in applicable {
            (migration.migrate)(settings);
        }
        
        Ok(count)
    }
    
    /// Get pending migrations
    pub fn pending_migrations(&self, from_version: MigrationVersion) -> Vec<&Migration> {
        self.migrations
            .iter()
            .filter(|m| m.from >= from_version && m.to <= self.current_version)
            .collect()
    }
    
    /// Get current version
    pub fn current_version(&self) -> MigrationVersion {
        self.current_version
    }
    
    /// Check if migrations are needed
    pub fn needs_migration(&self, from_version: MigrationVersion) -> bool {
        from_version < self.current_version
    }
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new(MigrationVersion::new(2, 0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        let v = MigrationVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }
    
    #[test]
    fn test_version_comparison() {
        let v1 = MigrationVersion::new(1, 0, 0);
        let v2 = MigrationVersion::new(1, 1, 0);
        let v3 = MigrationVersion::new(2, 0, 0);
        
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }
    
    #[test]
    fn test_version_display() {
        let v = MigrationVersion::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }
    
    #[test]
    fn test_migration_runner() {
        let runner = MigrationRunner::default();
        
        assert!(runner.needs_migration(MigrationVersion::new(1, 0, 0)));
        assert!(!runner.needs_migration(MigrationVersion::new(2, 0, 0)));
    }
    
    #[test]
    fn test_pending_migrations() {
        let runner = MigrationRunner::default();
        
        let pending = runner.pending_migrations(MigrationVersion::new(1, 0, 0));
        assert!(!pending.is_empty());
    }
    
    #[test]
    fn test_run_migration() {
        let runner = MigrationRunner::default();
        
        let mut settings = HashMap::new();
        settings.insert("display.brightness_level".to_string(), SettingValue::Float(0.5));
        
        let count = runner.run(MigrationVersion::new(1, 0, 0), &mut settings).unwrap();
        
        assert!(count > 0);
        
        // Old key should be removed, new key should exist
        assert!(!settings.contains_key("display.brightness_level"));
        assert!(settings.contains_key("display.brightness"));
    }
}
