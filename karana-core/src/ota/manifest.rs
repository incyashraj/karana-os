// Update manifest parser for OTA system

use super::{SemanticVersion, UpdateInfo, UpdateChannel, OTAError};
use std::collections::HashMap;

/// Parses update manifests
pub struct ManifestParser {
    strict_mode: bool,
}

impl ManifestParser {
    pub fn new() -> Self {
        Self { strict_mode: true }
    }
    
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }
    
    /// Parse a manifest string into UpdateManifest
    pub fn parse(&self, manifest_str: &str) -> Result<UpdateManifest, OTAError> {
        // Simulated JSON-like parsing
        // In real implementation would use serde_json
        
        if manifest_str.is_empty() {
            return Err(OTAError::ManifestParseError("Empty manifest".to_string()));
        }
        
        // Simulated manifest
        Ok(UpdateManifest {
            format_version: 1,
            updates: vec![],
            server_time: 0,
            signature: None,
        })
    }
    
    /// Parse update info from manifest entry
    pub fn parse_update_entry(&self, entry: &ManifestEntry) -> Result<UpdateInfo, OTAError> {
        let version = SemanticVersion::parse(&entry.version)
            .map_err(|e| OTAError::ManifestParseError(e.to_string()))?;
        
        let channel = self.parse_channel(&entry.channel)?;
        
        Ok(UpdateInfo {
            version,
            channel,
            release_notes: entry.release_notes.clone(),
            download_size: entry.download_size,
            installed_size: entry.installed_size,
            is_delta: entry.is_delta,
            checksum: entry.checksum.clone(),
            released_at: entry.released_at,
            mandatory: entry.mandatory,
            min_version_for_delta: entry.min_version_for_delta.as_ref()
                .and_then(|v| SemanticVersion::parse(v).ok()),
            is_security_update: entry.is_security_update,
            features: entry.features.clone(),
            fixes: entry.fixes.clone(),
        })
    }
    
    /// Parse channel string
    fn parse_channel(&self, channel_str: &str) -> Result<UpdateChannel, OTAError> {
        match channel_str.to_lowercase().as_str() {
            "stable" => Ok(UpdateChannel::Stable),
            "beta" => Ok(UpdateChannel::Beta),
            "dev" | "development" => Ok(UpdateChannel::Dev),
            "enterprise" => Ok(UpdateChannel::Enterprise),
            "custom" => Ok(UpdateChannel::Custom),
            _ => {
                if self.strict_mode {
                    Err(OTAError::ManifestParseError(
                        format!("Unknown channel: {}", channel_str)
                    ))
                } else {
                    Ok(UpdateChannel::Custom)
                }
            }
        }
    }
    
    /// Validate manifest structure
    pub fn validate(&self, manifest: &UpdateManifest) -> Result<(), OTAError> {
        // Check format version
        if manifest.format_version == 0 {
            return Err(OTAError::ManifestParseError("Invalid format version".to_string()));
        }
        
        // Validate each update entry
        for update in &manifest.updates {
            self.validate_entry(update)?;
        }
        
        // Verify signature if present
        if let Some(ref sig) = manifest.signature {
            if !self.verify_signature(manifest, sig) {
                return Err(OTAError::SignatureInvalid);
            }
        } else if self.strict_mode {
            return Err(OTAError::ManifestParseError("Missing signature in strict mode".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate a single manifest entry
    fn validate_entry(&self, entry: &ManifestEntry) -> Result<(), OTAError> {
        // Validate version
        SemanticVersion::parse(&entry.version)
            .map_err(|e| OTAError::ManifestParseError(format!("Invalid version: {}", e)))?;
        
        // Validate checksum format
        if entry.checksum.len() != 64 && self.strict_mode {
            return Err(OTAError::ManifestParseError(
                "Invalid checksum format (expected SHA256)".to_string()
            ));
        }
        
        // Validate sizes
        if entry.download_size == 0 {
            return Err(OTAError::ManifestParseError("Download size cannot be zero".to_string()));
        }
        
        // Validate delta requirements
        if entry.is_delta && entry.min_version_for_delta.is_none() {
            return Err(OTAError::ManifestParseError(
                "Delta update requires min_version_for_delta".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Verify manifest signature
    fn verify_signature(&self, _manifest: &UpdateManifest, _signature: &str) -> bool {
        // Simulated - would verify RSA/ECDSA signature
        true
    }
}

impl Default for ManifestParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Update manifest structure
#[derive(Debug, Clone)]
pub struct UpdateManifest {
    pub format_version: u32,
    pub updates: Vec<ManifestEntry>,
    pub server_time: u64,
    pub signature: Option<String>,
}

impl UpdateManifest {
    pub fn new() -> Self {
        Self {
            format_version: 1,
            updates: Vec::new(),
            server_time: 0,
            signature: None,
        }
    }
    
    /// Add an update entry
    pub fn add_update(&mut self, entry: ManifestEntry) {
        self.updates.push(entry);
    }
    
    /// Get updates for a specific channel
    pub fn updates_for_channel(&self, channel: &str) -> Vec<&ManifestEntry> {
        self.updates
            .iter()
            .filter(|u| u.channel.to_lowercase() == channel.to_lowercase())
            .collect()
    }
    
    /// Get latest update for a channel
    pub fn latest_for_channel(&self, channel: &str) -> Option<&ManifestEntry> {
        self.updates_for_channel(channel)
            .into_iter()
            .max_by(|a, b| {
                SemanticVersion::parse(&a.version)
                    .unwrap_or_else(|_| SemanticVersion::new(0, 0, 0))
                    .cmp(
                        &SemanticVersion::parse(&b.version)
                            .unwrap_or_else(|_| SemanticVersion::new(0, 0, 0))
                    )
            })
    }
    
    /// Check if any updates are security updates
    pub fn has_security_updates(&self) -> bool {
        self.updates.iter().any(|u| u.is_security_update)
    }
    
    /// Get all security updates
    pub fn security_updates(&self) -> Vec<&ManifestEntry> {
        self.updates
            .iter()
            .filter(|u| u.is_security_update)
            .collect()
    }
    
    /// Get mandatory updates
    pub fn mandatory_updates(&self) -> Vec<&ManifestEntry> {
        self.updates
            .iter()
            .filter(|u| u.mandatory)
            .collect()
    }
}

impl Default for UpdateManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual update entry in manifest
#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub version: String,
    pub channel: String,
    pub release_notes: String,
    pub download_size: u64,
    pub installed_size: u64,
    pub is_delta: bool,
    pub checksum: String,
    pub released_at: u64,
    pub mandatory: bool,
    pub min_version_for_delta: Option<String>,
    pub is_security_update: bool,
    pub features: Vec<String>,
    pub fixes: Vec<String>,
    pub download_urls: Vec<DownloadUrl>,
    pub dependencies: Vec<Dependency>,
    pub hardware_requirements: HardwareRequirements,
}

impl ManifestEntry {
    pub fn new(version: &str, channel: &str) -> Self {
        Self {
            version: version.to_string(),
            channel: channel.to_string(),
            release_notes: String::new(),
            download_size: 0,
            installed_size: 0,
            is_delta: false,
            checksum: String::new(),
            released_at: 0,
            mandatory: false,
            min_version_for_delta: None,
            is_security_update: false,
            features: Vec::new(),
            fixes: Vec::new(),
            download_urls: Vec::new(),
            dependencies: Vec::new(),
            hardware_requirements: HardwareRequirements::default(),
        }
    }
    
    pub fn with_checksum(mut self, checksum: &str) -> Self {
        self.checksum = checksum.to_string();
        self
    }
    
    pub fn with_size(mut self, download: u64, installed: u64) -> Self {
        self.download_size = download;
        self.installed_size = installed;
        self
    }
    
    pub fn as_delta(mut self, min_version: &str) -> Self {
        self.is_delta = true;
        self.min_version_for_delta = Some(min_version.to_string());
        self
    }
    
    pub fn as_security_update(mut self) -> Self {
        self.is_security_update = true;
        self.mandatory = true; // Security updates are usually mandatory
        self
    }
}

/// Download URL with mirror info
#[derive(Debug, Clone)]
pub struct DownloadUrl {
    pub url: String,
    pub region: Option<String>,
    pub priority: u32,
    pub is_mirror: bool,
}

impl DownloadUrl {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            region: None,
            priority: 0,
            is_mirror: false,
        }
    }
    
    pub fn mirror(url: &str, region: &str) -> Self {
        Self {
            url: url.to_string(),
            region: Some(region.to_string()),
            priority: 1,
            is_mirror: true,
        }
    }
}

/// Package dependency
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version_constraint: String,
    pub optional: bool,
}

impl Dependency {
    pub fn required(name: &str, constraint: &str) -> Self {
        Self {
            name: name.to_string(),
            version_constraint: constraint.to_string(),
            optional: false,
        }
    }
    
    pub fn optional(name: &str, constraint: &str) -> Self {
        Self {
            name: name.to_string(),
            version_constraint: constraint.to_string(),
            optional: true,
        }
    }
}

/// Hardware requirements for update
#[derive(Debug, Clone)]
pub struct HardwareRequirements {
    pub min_ram_mb: u64,
    pub min_storage_mb: u64,
    pub required_features: Vec<String>,
    pub supported_devices: Vec<String>,
    pub min_firmware_version: Option<String>,
}

impl Default for HardwareRequirements {
    fn default() -> Self {
        Self {
            min_ram_mb: 256,
            min_storage_mb: 512,
            required_features: Vec::new(),
            supported_devices: vec!["karana-glasses-v1".to_string()],
            min_firmware_version: None,
        }
    }
}

impl HardwareRequirements {
    pub fn check(&self, device_info: &DeviceInfo) -> RequirementsCheckResult {
        let mut result = RequirementsCheckResult {
            meets_requirements: true,
            failures: Vec::new(),
        };
        
        if device_info.ram_mb < self.min_ram_mb {
            result.meets_requirements = false;
            result.failures.push(format!(
                "Insufficient RAM: {} MB (required: {} MB)",
                device_info.ram_mb, self.min_ram_mb
            ));
        }
        
        if device_info.storage_mb < self.min_storage_mb {
            result.meets_requirements = false;
            result.failures.push(format!(
                "Insufficient storage: {} MB (required: {} MB)",
                device_info.storage_mb, self.min_storage_mb
            ));
        }
        
        for feature in &self.required_features {
            if !device_info.features.contains(feature) {
                result.meets_requirements = false;
                result.failures.push(format!("Missing required feature: {}", feature));
            }
        }
        
        if !self.supported_devices.is_empty() && 
           !self.supported_devices.contains(&device_info.model) 
        {
            result.meets_requirements = false;
            result.failures.push(format!(
                "Device {} not in supported list",
                device_info.model
            ));
        }
        
        result
    }
}

/// Device information for requirement checking
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub model: String,
    pub ram_mb: u64,
    pub storage_mb: u64,
    pub features: Vec<String>,
    pub firmware_version: String,
}

/// Result of requirements check
#[derive(Debug, Clone)]
pub struct RequirementsCheckResult {
    pub meets_requirements: bool,
    pub failures: Vec<String>,
}

/// Builds manifests for publishing updates
pub struct ManifestBuilder {
    manifest: UpdateManifest,
}

impl ManifestBuilder {
    pub fn new() -> Self {
        Self {
            manifest: UpdateManifest::new(),
        }
    }
    
    pub fn format_version(mut self, version: u32) -> Self {
        self.manifest.format_version = version;
        self
    }
    
    pub fn server_time(mut self, time: u64) -> Self {
        self.manifest.server_time = time;
        self
    }
    
    pub fn add_update(mut self, entry: ManifestEntry) -> Self {
        self.manifest.add_update(entry);
        self
    }
    
    pub fn sign(mut self, signature: &str) -> Self {
        self.manifest.signature = Some(signature.to_string());
        self
    }
    
    pub fn build(self) -> UpdateManifest {
        self.manifest
    }
}

impl Default for ManifestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manifest_parser_new() {
        let parser = ManifestParser::new();
        assert!(parser.strict_mode);
    }
    
    #[test]
    fn test_parse_empty_manifest() {
        let parser = ManifestParser::new();
        let result = parser.parse("");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_channel() {
        let parser = ManifestParser::new().with_strict_mode(false);
        
        // Use parse_update_entry through a manifest entry
        let entry = ManifestEntry::new("1.0.0", "stable")
            .with_checksum("a".repeat(64).as_str())
            .with_size(1000, 2000);
        
        let result = parser.parse_update_entry(&entry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().channel, UpdateChannel::Stable);
    }
    
    #[test]
    fn test_manifest_entry_builder() {
        let entry = ManifestEntry::new("1.0.0", "stable")
            .with_checksum("abc123")
            .with_size(1000, 2000)
            .as_security_update();
        
        assert!(entry.is_security_update);
        assert!(entry.mandatory);
        assert_eq!(entry.download_size, 1000);
    }
    
    #[test]
    fn test_manifest_entry_delta() {
        let entry = ManifestEntry::new("1.0.1", "stable")
            .as_delta("1.0.0");
        
        assert!(entry.is_delta);
        assert_eq!(entry.min_version_for_delta, Some("1.0.0".to_string()));
    }
    
    #[test]
    fn test_update_manifest() {
        let mut manifest = UpdateManifest::new();
        
        manifest.add_update(ManifestEntry::new("1.0.0", "stable"));
        manifest.add_update(ManifestEntry::new("1.1.0", "beta"));
        
        assert_eq!(manifest.updates.len(), 2);
        assert_eq!(manifest.updates_for_channel("stable").len(), 1);
    }
    
    #[test]
    fn test_latest_for_channel() {
        let mut manifest = UpdateManifest::new();
        
        manifest.add_update(ManifestEntry::new("1.0.0", "stable"));
        manifest.add_update(ManifestEntry::new("1.1.0", "stable"));
        manifest.add_update(ManifestEntry::new("1.2.0", "stable"));
        
        let latest = manifest.latest_for_channel("stable");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version, "1.2.0");
    }
    
    #[test]
    fn test_security_updates() {
        let mut manifest = UpdateManifest::new();
        
        manifest.add_update(ManifestEntry::new("1.0.0", "stable"));
        manifest.add_update(ManifestEntry::new("1.0.1", "stable").as_security_update());
        
        assert!(manifest.has_security_updates());
        assert_eq!(manifest.security_updates().len(), 1);
    }
    
    #[test]
    fn test_hardware_requirements() {
        let requirements = HardwareRequirements::default();
        
        let device = DeviceInfo {
            model: "karana-glasses-v1".to_string(),
            ram_mb: 512,
            storage_mb: 1024,
            features: vec!["ar".to_string(), "voice".to_string()],
            firmware_version: "1.0.0".to_string(),
        };
        
        let result = requirements.check(&device);
        assert!(result.meets_requirements);
    }
    
    #[test]
    fn test_hardware_requirements_fail() {
        let requirements = HardwareRequirements {
            min_ram_mb: 1024,
            min_storage_mb: 2048,
            required_features: vec!["lidar".to_string()],
            supported_devices: vec!["karana-glasses-v2".to_string()],
            min_firmware_version: None,
        };
        
        let device = DeviceInfo {
            model: "karana-glasses-v1".to_string(),
            ram_mb: 512,
            storage_mb: 1024,
            features: vec!["ar".to_string()],
            firmware_version: "1.0.0".to_string(),
        };
        
        let result = requirements.check(&device);
        assert!(!result.meets_requirements);
        assert!(!result.failures.is_empty());
    }
    
    #[test]
    fn test_download_url() {
        let url = DownloadUrl::new("https://example.com/update.pkg");
        assert!(!url.is_mirror);
        
        let mirror = DownloadUrl::mirror("https://mirror.example.com/update.pkg", "us-west");
        assert!(mirror.is_mirror);
        assert_eq!(mirror.region, Some("us-west".to_string()));
    }
    
    #[test]
    fn test_dependency() {
        let dep = Dependency::required("libfoo", ">=1.0.0");
        assert!(!dep.optional);
        
        let opt_dep = Dependency::optional("libbar", ">=2.0.0");
        assert!(opt_dep.optional);
    }
    
    #[test]
    fn test_manifest_builder() {
        let manifest = ManifestBuilder::new()
            .format_version(2)
            .server_time(12345)
            .add_update(ManifestEntry::new("1.0.0", "stable"))
            .sign("signature123")
            .build();
        
        assert_eq!(manifest.format_version, 2);
        assert_eq!(manifest.server_time, 12345);
        assert_eq!(manifest.signature, Some("signature123".to_string()));
        assert_eq!(manifest.updates.len(), 1);
    }
    
    #[test]
    fn test_validate_entry_invalid_version() {
        let parser = ManifestParser::new();
        
        let entry = ManifestEntry::new("invalid", "stable");
        let manifest = UpdateManifest {
            format_version: 1,
            updates: vec![entry],
            server_time: 0,
            signature: Some("sig".to_string()),
        };
        
        let result = parser.validate(&manifest);
        assert!(result.is_err());
    }
}
