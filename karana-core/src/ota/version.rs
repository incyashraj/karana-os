// Version management for OTA updates

use std::cmp::Ordering;
use std::collections::HashMap;

/// Semantic version representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build_metadata: Option<String>,
}

impl SemanticVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build_metadata: None,
        }
    }
    
    pub fn with_prerelease(mut self, prerelease: &str) -> Self {
        self.prerelease = Some(prerelease.to_string());
        self
    }
    
    pub fn with_build(mut self, build: &str) -> Self {
        self.build_metadata = Some(build.to_string());
        self
    }
    
    pub fn parse(version_str: &str) -> Result<Self, VersionParseError> {
        let version_str = version_str.trim();
        let version_str = version_str.strip_prefix('v').unwrap_or(version_str);
        
        // Handle build metadata
        let (version_part, build_metadata) = if let Some(idx) = version_str.find('+') {
            let (v, b) = version_str.split_at(idx);
            (v, Some(b[1..].to_string()))
        } else {
            (version_str, None)
        };
        
        // Handle prerelease
        let (version_part, prerelease) = if let Some(idx) = version_part.find('-') {
            let (v, p) = version_part.split_at(idx);
            (v, Some(p[1..].to_string()))
        } else {
            (version_part, None)
        };
        
        let parts: Vec<&str> = version_part.split('.').collect();
        
        if parts.len() < 3 {
            return Err(VersionParseError::InvalidFormat);
        }
        
        let major = parts[0].parse()
            .map_err(|_| VersionParseError::InvalidMajor)?;
        let minor = parts[1].parse()
            .map_err(|_| VersionParseError::InvalidMinor)?;
        let patch = parts[2].parse()
            .map_err(|_| VersionParseError::InvalidPatch)?;
        
        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
            build_metadata,
        })
    }
    
    pub fn to_string_full(&self) -> String {
        let mut s = format!("{}.{}.{}", self.major, self.minor, self.patch);
        
        if let Some(ref pre) = self.prerelease {
            s.push('-');
            s.push_str(pre);
        }
        
        if let Some(ref build) = self.build_metadata {
            s.push('+');
            s.push_str(build);
        }
        
        s
    }
    
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }
    
    pub fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }
    
    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }
    
    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
    
    /// Check if this version satisfies a constraint
    pub fn satisfies(&self, constraint: &VersionConstraint) -> bool {
        match constraint {
            VersionConstraint::Exact(v) => self == v,
            VersionConstraint::GreaterThan(v) => self > v,
            VersionConstraint::GreaterOrEqual(v) => self >= v,
            VersionConstraint::LessThan(v) => self < v,
            VersionConstraint::LessOrEqual(v) => self <= v,
            VersionConstraint::Compatible(v) => {
                self.major == v.major && self >= v
            }
            VersionConstraint::Range { min, max, min_inclusive, max_inclusive } => {
                let min_ok = if *min_inclusive { self >= min } else { self > min };
                let max_ok = if *max_inclusive { self <= max } else { self < max };
                min_ok && max_ok
            }
        }
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }
        
        // Prerelease versions have lower precedence
        match (&self.prerelease, &other.prerelease) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Version constraint for dependency checking
#[derive(Debug, Clone, PartialEq)]
pub enum VersionConstraint {
    /// Exact version match
    Exact(SemanticVersion),
    /// Greater than
    GreaterThan(SemanticVersion),
    /// Greater than or equal
    GreaterOrEqual(SemanticVersion),
    /// Less than
    LessThan(SemanticVersion),
    /// Less than or equal
    LessOrEqual(SemanticVersion),
    /// Compatible (same major, >= minor.patch)
    Compatible(SemanticVersion),
    /// Version range
    Range {
        min: SemanticVersion,
        max: SemanticVersion,
        min_inclusive: bool,
        max_inclusive: bool,
    },
}

impl VersionConstraint {
    pub fn parse(constraint_str: &str) -> Result<Self, VersionParseError> {
        let constraint_str = constraint_str.trim();
        
        if constraint_str.starts_with(">=") {
            let version = SemanticVersion::parse(&constraint_str[2..])?;
            Ok(Self::GreaterOrEqual(version))
        } else if constraint_str.starts_with('>') {
            let version = SemanticVersion::parse(&constraint_str[1..])?;
            Ok(Self::GreaterThan(version))
        } else if constraint_str.starts_with("<=") {
            let version = SemanticVersion::parse(&constraint_str[2..])?;
            Ok(Self::LessOrEqual(version))
        } else if constraint_str.starts_with('<') {
            let version = SemanticVersion::parse(&constraint_str[1..])?;
            Ok(Self::LessThan(version))
        } else if constraint_str.starts_with('^') {
            let version = SemanticVersion::parse(&constraint_str[1..])?;
            Ok(Self::Compatible(version))
        } else if constraint_str.starts_with('=') {
            let version = SemanticVersion::parse(&constraint_str[1..])?;
            Ok(Self::Exact(version))
        } else {
            let version = SemanticVersion::parse(constraint_str)?;
            Ok(Self::Exact(version))
        }
    }
}

/// Version parsing errors
#[derive(Debug, Clone, PartialEq)]
pub enum VersionParseError {
    InvalidFormat,
    InvalidMajor,
    InvalidMinor,
    InvalidPatch,
    InvalidConstraint,
}

impl std::fmt::Display for VersionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "Invalid version format"),
            Self::InvalidMajor => write!(f, "Invalid major version number"),
            Self::InvalidMinor => write!(f, "Invalid minor version number"),
            Self::InvalidPatch => write!(f, "Invalid patch version number"),
            Self::InvalidConstraint => write!(f, "Invalid version constraint"),
        }
    }
}

impl std::error::Error for VersionParseError {}

/// Manages version history and comparisons
pub struct VersionManager {
    current_version: SemanticVersion,
    installed_versions: Vec<SemanticVersion>,
    version_metadata: HashMap<SemanticVersion, VersionMetadata>,
}

/// Metadata about an installed version
#[derive(Debug, Clone)]
pub struct VersionMetadata {
    pub installed_at: u64,
    pub install_source: InstallSource,
    pub size_bytes: u64,
    pub is_active: bool,
}

/// Source of version installation
#[derive(Debug, Clone, PartialEq)]
pub enum InstallSource {
    Factory,
    OTA,
    Manual,
    Rollback,
    Recovery,
}

impl VersionManager {
    pub fn new(current: SemanticVersion) -> Self {
        let mut manager = Self {
            current_version: current.clone(),
            installed_versions: vec![current.clone()],
            version_metadata: HashMap::new(),
        };
        
        manager.version_metadata.insert(current.clone(), VersionMetadata {
            installed_at: 0,
            install_source: InstallSource::Factory,
            size_bytes: 0,
            is_active: true,
        });
        
        manager
    }
    
    pub fn current(&self) -> &SemanticVersion {
        &self.current_version
    }
    
    pub fn set_current(&mut self, version: SemanticVersion) {
        // Mark old as inactive
        if let Some(meta) = self.version_metadata.get_mut(&self.current_version) {
            meta.is_active = false;
        }
        
        self.current_version = version;
        
        // Mark new as active
        if let Some(meta) = self.version_metadata.get_mut(&self.current_version) {
            meta.is_active = true;
        }
    }
    
    pub fn add_version(&mut self, version: SemanticVersion, metadata: VersionMetadata) {
        if !self.installed_versions.contains(&version) {
            self.installed_versions.push(version.clone());
        }
        self.version_metadata.insert(version, metadata);
    }
    
    pub fn get_metadata(&self, version: &SemanticVersion) -> Option<&VersionMetadata> {
        self.version_metadata.get(version)
    }
    
    pub fn installed_versions(&self) -> &[SemanticVersion] {
        &self.installed_versions
    }
    
    pub fn is_newer(&self, version: &SemanticVersion) -> bool {
        version > &self.current_version
    }
    
    pub fn is_upgrade(&self, from: &SemanticVersion, to: &SemanticVersion) -> bool {
        to > from
    }
    
    /// Get version difference type
    pub fn diff_type(&self, from: &SemanticVersion, to: &SemanticVersion) -> VersionDiffType {
        if to.major != from.major {
            VersionDiffType::Major
        } else if to.minor != from.minor {
            VersionDiffType::Minor
        } else if to.patch != from.patch {
            VersionDiffType::Patch
        } else {
            VersionDiffType::None
        }
    }
    
    /// Check if delta update is possible between versions
    pub fn can_delta_update(&self, from: &SemanticVersion, to: &SemanticVersion) -> bool {
        // Delta updates only allowed within same major version
        from.major == to.major && self.is_upgrade(from, to)
    }
    
    /// Get all versions newer than current
    pub fn available_upgrades(&self, available: &[SemanticVersion]) -> Vec<SemanticVersion> {
        available
            .iter()
            .filter(|v| self.is_newer(v))
            .cloned()
            .collect()
    }
    
    /// Cleanup old versions, keeping n most recent
    pub fn cleanup_old_versions(&mut self, keep: usize) {
        if self.installed_versions.len() <= keep {
            return;
        }
        
        // Sort versions
        let mut sorted: Vec<_> = self.installed_versions.iter().cloned().collect();
        sorted.sort();
        
        // Remove oldest, but keep current
        let to_remove: Vec<_> = sorted
            .iter()
            .take(sorted.len() - keep)
            .filter(|v| **v != self.current_version)
            .cloned()
            .collect();
        
        for version in to_remove {
            self.installed_versions.retain(|v| *v != version);
            self.version_metadata.remove(&version);
        }
    }
}

/// Type of version difference
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VersionDiffType {
    None,
    Patch,
    Minor,
    Major,
}

/// Build information
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub version: SemanticVersion,
    pub build_number: u64,
    pub build_date: String,
    pub git_commit: String,
    pub git_branch: String,
    pub compiler_version: String,
    pub target_platform: String,
    pub features: Vec<String>,
    pub debug: bool,
}

impl BuildInfo {
    pub fn current() -> Self {
        Self {
            version: SemanticVersion::new(0, 1, 0),
            build_number: 1,
            build_date: "2024-01-01".to_string(),
            git_commit: "unknown".to_string(),
            git_branch: "master".to_string(),
            compiler_version: "rustc 1.75.0".to_string(),
            target_platform: "aarch64-linux-gnu".to_string(),
            features: vec!["ar".to_string(), "voice".to_string(), "gesture".to_string()],
            debug: cfg!(debug_assertions),
        }
    }
    
    pub fn is_debug(&self) -> bool {
        self.debug
    }
    
    pub fn full_version_string(&self) -> String {
        format!(
            "{} (build {} @ {})",
            self.version.to_string_full(),
            self.build_number,
            self.git_commit.chars().take(7).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_semantic_version_new() {
        let v = SemanticVersion::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }
    
    #[test]
    fn test_semantic_version_parse() {
        let v = SemanticVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        
        let v = SemanticVersion::parse("v2.0.0").unwrap();
        assert_eq!(v.major, 2);
        
        let v = SemanticVersion::parse("1.0.0-alpha").unwrap();
        assert_eq!(v.prerelease, Some("alpha".to_string()));
        
        let v = SemanticVersion::parse("1.0.0+build123").unwrap();
        assert_eq!(v.build_metadata, Some("build123".to_string()));
    }
    
    #[test]
    fn test_semantic_version_comparison() {
        let v1 = SemanticVersion::new(1, 0, 0);
        let v2 = SemanticVersion::new(1, 0, 1);
        let v3 = SemanticVersion::new(1, 1, 0);
        let v4 = SemanticVersion::new(2, 0, 0);
        
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }
    
    #[test]
    fn test_prerelease_comparison() {
        let v1 = SemanticVersion::new(1, 0, 0).with_prerelease("alpha");
        let v2 = SemanticVersion::new(1, 0, 0);
        
        // Prerelease is less than release
        assert!(v1 < v2);
    }
    
    #[test]
    fn test_version_bump() {
        let v = SemanticVersion::new(1, 2, 3);
        
        let major = v.bump_major();
        assert_eq!(major, SemanticVersion::new(2, 0, 0));
        
        let minor = v.bump_minor();
        assert_eq!(minor, SemanticVersion::new(1, 3, 0));
        
        let patch = v.bump_patch();
        assert_eq!(patch, SemanticVersion::new(1, 2, 4));
    }
    
    #[test]
    fn test_version_constraint() {
        let v = SemanticVersion::new(1, 5, 0);
        
        assert!(v.satisfies(&VersionConstraint::GreaterOrEqual(SemanticVersion::new(1, 0, 0))));
        assert!(v.satisfies(&VersionConstraint::LessThan(SemanticVersion::new(2, 0, 0))));
        assert!(v.satisfies(&VersionConstraint::Compatible(SemanticVersion::new(1, 0, 0))));
        assert!(!v.satisfies(&VersionConstraint::Compatible(SemanticVersion::new(2, 0, 0))));
    }
    
    #[test]
    fn test_version_constraint_parse() {
        let c = VersionConstraint::parse(">=1.0.0").unwrap();
        assert!(matches!(c, VersionConstraint::GreaterOrEqual(_)));
        
        let c = VersionConstraint::parse("^1.2.0").unwrap();
        assert!(matches!(c, VersionConstraint::Compatible(_)));
    }
    
    #[test]
    fn test_version_manager() {
        let mut manager = VersionManager::new(SemanticVersion::new(1, 0, 0));
        
        assert_eq!(manager.current().major, 1);
        assert!(manager.is_newer(&SemanticVersion::new(1, 1, 0)));
        assert!(!manager.is_newer(&SemanticVersion::new(0, 9, 0)));
    }
    
    #[test]
    fn test_version_diff_type() {
        let manager = VersionManager::new(SemanticVersion::new(1, 0, 0));
        
        let from = SemanticVersion::new(1, 0, 0);
        
        assert_eq!(
            manager.diff_type(&from, &SemanticVersion::new(2, 0, 0)),
            VersionDiffType::Major
        );
        assert_eq!(
            manager.diff_type(&from, &SemanticVersion::new(1, 1, 0)),
            VersionDiffType::Minor
        );
        assert_eq!(
            manager.diff_type(&from, &SemanticVersion::new(1, 0, 1)),
            VersionDiffType::Patch
        );
    }
    
    #[test]
    fn test_can_delta_update() {
        let manager = VersionManager::new(SemanticVersion::new(1, 0, 0));
        
        assert!(manager.can_delta_update(
            &SemanticVersion::new(1, 0, 0),
            &SemanticVersion::new(1, 1, 0)
        ));
        assert!(!manager.can_delta_update(
            &SemanticVersion::new(1, 0, 0),
            &SemanticVersion::new(2, 0, 0)
        ));
    }
    
    #[test]
    fn test_build_info() {
        let info = BuildInfo::current();
        assert_eq!(info.version.major, 0);
        assert!(!info.full_version_string().is_empty());
    }
    
    #[test]
    fn test_version_to_string() {
        let v = SemanticVersion::new(1, 2, 3)
            .with_prerelease("beta")
            .with_build("456");
        
        assert_eq!(v.to_string(), "1.2.3");
        assert_eq!(v.to_string_full(), "1.2.3-beta+456");
    }
}
