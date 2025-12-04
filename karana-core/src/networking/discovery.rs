//! Device and service discovery

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Discovery protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryProtocol {
    /// mDNS/Bonjour
    MDNS,
    /// SSDP (UPnP)
    SSDP,
    /// Bluetooth LE advertising
    BLE,
    /// WiFi Direct
    WiFiDirect,
    /// Custom protocol
    Custom,
}

/// Service type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    /// Display mirroring
    Display,
    /// Audio streaming
    Audio,
    /// File sharing
    FileSharing,
    /// Companion app
    Companion,
    /// IoT device
    IoT,
    /// Media server
    MediaServer,
    /// Printer
    Printer,
    /// Custom service
    Other,
}

/// Discovered service
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    /// Service identifier
    pub id: String,
    /// Service name
    pub name: String,
    /// Service type
    pub service_type: ServiceType,
    /// Discovery protocol used
    pub protocol: DiscoveryProtocol,
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Service metadata
    pub metadata: HashMap<String, String>,
    /// Last seen timestamp
    pub last_seen: Instant,
    /// Signal strength (if applicable)
    pub signal_strength: Option<i32>,
    /// Available
    pub available: bool,
}

impl DiscoveredService {
    /// Create new discovered service
    pub fn new(id: &str, name: &str, service_type: ServiceType, protocol: DiscoveryProtocol) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            service_type,
            protocol,
            host: String::new(),
            port: 0,
            metadata: HashMap::new(),
            last_seen: Instant::now(),
            signal_strength: None,
            available: true,
        }
    }
    
    /// Set host and port
    pub fn with_address(mut self, host: &str, port: u16) -> Self {
        self.host = host.to_string();
        self.port = port;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if service is stale
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }
    
    /// Update last seen
    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
    }
}

/// Discovered device (physical device that may offer services)
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    /// Device identifier
    pub id: String,
    /// Device name
    pub name: String,
    /// Device type/model
    pub device_type: String,
    /// Manufacturer
    pub manufacturer: Option<String>,
    /// Discovery protocol
    pub protocol: DiscoveryProtocol,
    /// IP address
    pub ip_address: Option<String>,
    /// MAC address
    pub mac_address: Option<String>,
    /// Services offered
    pub services: Vec<String>,
    /// Last seen
    pub last_seen: Instant,
    /// Signal strength
    pub signal_strength: Option<i32>,
    /// Paired/connected
    pub connected: bool,
}

impl DiscoveredDevice {
    /// Create new discovered device
    pub fn new(id: &str, name: &str, device_type: &str, protocol: DiscoveryProtocol) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            device_type: device_type.to_string(),
            manufacturer: None,
            protocol,
            ip_address: None,
            mac_address: None,
            services: Vec::new(),
            last_seen: Instant::now(),
            signal_strength: None,
            connected: false,
        }
    }
    
    /// Add service
    pub fn add_service(&mut self, service_id: &str) {
        if !self.services.contains(&service_id.to_string()) {
            self.services.push(service_id.to_string());
        }
    }
    
    /// Update last seen
    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
    }
}

/// Discovery state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryState {
    /// Idle
    Idle,
    /// Scanning
    Scanning,
    /// Paused
    Paused,
}

/// Discovery manager
#[derive(Debug)]
pub struct DiscoveryManager {
    /// Current state
    state: DiscoveryState,
    /// Enabled protocols
    enabled_protocols: Vec<DiscoveryProtocol>,
    /// Discovered services
    services: HashMap<String, DiscoveredService>,
    /// Discovered devices
    devices: HashMap<String, DiscoveredDevice>,
    /// Service timeout
    service_timeout: Duration,
    /// Scan interval
    scan_interval: Duration,
    /// Last scan time
    last_scan: Option<Instant>,
    /// Auto-discovery enabled
    auto_discovery: bool,
    /// Service type filter
    type_filter: Option<Vec<ServiceType>>,
    /// Discovery listeners count (for reference counting)
    listener_count: usize,
}

impl DiscoveryManager {
    /// Create new discovery manager
    pub fn new() -> Self {
        Self {
            state: DiscoveryState::Idle,
            enabled_protocols: vec![
                DiscoveryProtocol::MDNS,
                DiscoveryProtocol::BLE,
            ],
            services: HashMap::new(),
            devices: HashMap::new(),
            service_timeout: Duration::from_secs(60),
            scan_interval: Duration::from_secs(30),
            last_scan: None,
            auto_discovery: true,
            type_filter: None,
            listener_count: 0,
        }
    }
    
    /// Start discovery
    pub fn start_discovery(&mut self) -> Result<(), String> {
        if self.state == DiscoveryState::Scanning {
            return Ok(());
        }
        
        self.state = DiscoveryState::Scanning;
        self.last_scan = Some(Instant::now());
        
        Ok(())
    }
    
    /// Stop discovery
    pub fn stop_discovery(&mut self) {
        self.state = DiscoveryState::Idle;
    }
    
    /// Pause discovery
    pub fn pause_discovery(&mut self) {
        if self.state == DiscoveryState::Scanning {
            self.state = DiscoveryState::Paused;
        }
    }
    
    /// Resume discovery
    pub fn resume_discovery(&mut self) {
        if self.state == DiscoveryState::Paused {
            self.state = DiscoveryState::Scanning;
        }
    }
    
    /// Get current state
    pub fn state(&self) -> DiscoveryState {
        self.state
    }
    
    /// Check if scanning
    pub fn is_scanning(&self) -> bool {
        self.state == DiscoveryState::Scanning
    }
    
    /// Enable protocol
    pub fn enable_protocol(&mut self, protocol: DiscoveryProtocol) {
        if !self.enabled_protocols.contains(&protocol) {
            self.enabled_protocols.push(protocol);
        }
    }
    
    /// Disable protocol
    pub fn disable_protocol(&mut self, protocol: DiscoveryProtocol) {
        self.enabled_protocols.retain(|p| *p != protocol);
    }
    
    /// Check if protocol enabled
    pub fn is_protocol_enabled(&self, protocol: DiscoveryProtocol) -> bool {
        self.enabled_protocols.contains(&protocol)
    }
    
    /// Add discovered service
    pub fn add_service(&mut self, service: DiscoveredService) {
        // Apply filter
        if let Some(ref filter) = self.type_filter {
            if !filter.contains(&service.service_type) {
                return;
            }
        }
        
        self.services.insert(service.id.clone(), service);
    }
    
    /// Get service
    pub fn get_service(&self, id: &str) -> Option<&DiscoveredService> {
        self.services.get(id)
    }
    
    /// Get all services
    pub fn services(&self) -> Vec<&DiscoveredService> {
        self.services.values().collect()
    }
    
    /// Get services by type
    pub fn services_by_type(&self, service_type: ServiceType) -> Vec<&DiscoveredService> {
        self.services
            .values()
            .filter(|s| s.service_type == service_type)
            .collect()
    }
    
    /// Add discovered device
    pub fn add_device(&mut self, device: DiscoveredDevice) {
        self.devices.insert(device.id.clone(), device);
    }
    
    /// Get device
    pub fn get_device(&self, id: &str) -> Option<&DiscoveredDevice> {
        self.devices.get(id)
    }
    
    /// Get all devices
    pub fn devices(&self) -> Vec<&DiscoveredDevice> {
        self.devices.values().collect()
    }
    
    /// Remove stale entries
    pub fn cleanup_stale(&mut self) {
        let timeout = self.service_timeout;
        
        self.services.retain(|_, s| !s.is_stale(timeout));
        self.devices.retain(|_, d| d.last_seen.elapsed() <= timeout);
    }
    
    /// Set service timeout
    pub fn set_service_timeout(&mut self, timeout: Duration) {
        self.service_timeout = timeout;
    }
    
    /// Set scan interval
    pub fn set_scan_interval(&mut self, interval: Duration) {
        self.scan_interval = interval;
    }
    
    /// Set auto-discovery
    pub fn set_auto_discovery(&mut self, enabled: bool) {
        self.auto_discovery = enabled;
    }
    
    /// Check if auto-discovery enabled
    pub fn is_auto_discovery(&self) -> bool {
        self.auto_discovery
    }
    
    /// Set service type filter
    pub fn set_type_filter(&mut self, types: Vec<ServiceType>) {
        self.type_filter = Some(types);
    }
    
    /// Clear service type filter
    pub fn clear_type_filter(&mut self) {
        self.type_filter = None;
    }
    
    /// Get service count
    pub fn service_count(&self) -> usize {
        self.services.len()
    }
    
    /// Get device count
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
    
    /// Check if needs scan
    pub fn needs_scan(&self) -> bool {
        if !self.auto_discovery {
            return false;
        }
        
        match self.last_scan {
            Some(last) => last.elapsed() >= self.scan_interval,
            None => true,
        }
    }
    
    /// Update discovery
    pub fn update(&mut self) {
        // Remove stale entries
        self.cleanup_stale();
        
        // Auto-scan if needed
        if self.needs_scan() && self.state == DiscoveryState::Idle {
            let _ = self.start_discovery();
        }
    }
    
    /// Clear all discovered items
    pub fn clear(&mut self) {
        self.services.clear();
        self.devices.clear();
    }
    
    /// Add listener
    pub fn add_listener(&mut self) {
        self.listener_count += 1;
    }
    
    /// Remove listener
    pub fn remove_listener(&mut self) {
        self.listener_count = self.listener_count.saturating_sub(1);
    }
}

impl Default for DiscoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_discovery_manager_creation() {
        let manager = DiscoveryManager::new();
        assert_eq!(manager.state(), DiscoveryState::Idle);
        assert!(manager.is_protocol_enabled(DiscoveryProtocol::MDNS));
    }
    
    #[test]
    fn test_start_stop_discovery() {
        let mut manager = DiscoveryManager::new();
        
        assert!(manager.start_discovery().is_ok());
        assert!(manager.is_scanning());
        
        manager.stop_discovery();
        assert!(!manager.is_scanning());
    }
    
    #[test]
    fn test_add_service() {
        let mut manager = DiscoveryManager::new();
        
        let service = DiscoveredService::new(
            "svc1",
            "My TV",
            ServiceType::Display,
            DiscoveryProtocol::SSDP,
        ).with_address("192.168.1.100", 8080);
        
        manager.add_service(service);
        
        assert_eq!(manager.service_count(), 1);
        
        let retrieved = manager.get_service("svc1").unwrap();
        assert_eq!(retrieved.name, "My TV");
        assert_eq!(retrieved.port, 8080);
    }
    
    #[test]
    fn test_service_filter() {
        let mut manager = DiscoveryManager::new();
        
        manager.set_type_filter(vec![ServiceType::Display]);
        
        let display = DiscoveredService::new("d1", "Display", ServiceType::Display, DiscoveryProtocol::MDNS);
        let audio = DiscoveredService::new("a1", "Audio", ServiceType::Audio, DiscoveryProtocol::MDNS);
        
        manager.add_service(display);
        manager.add_service(audio);
        
        // Only display should be added
        assert_eq!(manager.service_count(), 1);
        assert!(manager.get_service("d1").is_some());
        assert!(manager.get_service("a1").is_none());
    }
    
    #[test]
    fn test_add_device() {
        let mut manager = DiscoveryManager::new();
        
        let mut device = DiscoveredDevice::new(
            "dev1",
            "Smart Speaker",
            "Speaker",
            DiscoveryProtocol::BLE,
        );
        device.add_service("audio_streaming");
        
        manager.add_device(device);
        
        assert_eq!(manager.device_count(), 1);
        
        let dev = manager.get_device("dev1").unwrap();
        assert!(dev.services.contains(&"audio_streaming".to_string()));
    }
    
    #[test]
    fn test_protocol_management() {
        let mut manager = DiscoveryManager::new();
        
        assert!(manager.is_protocol_enabled(DiscoveryProtocol::BLE));
        
        manager.disable_protocol(DiscoveryProtocol::BLE);
        assert!(!manager.is_protocol_enabled(DiscoveryProtocol::BLE));
        
        manager.enable_protocol(DiscoveryProtocol::SSDP);
        assert!(manager.is_protocol_enabled(DiscoveryProtocol::SSDP));
    }
    
    #[test]
    fn test_pause_resume() {
        let mut manager = DiscoveryManager::new();
        
        manager.start_discovery().unwrap();
        
        manager.pause_discovery();
        assert_eq!(manager.state(), DiscoveryState::Paused);
        
        manager.resume_discovery();
        assert_eq!(manager.state(), DiscoveryState::Scanning);
    }
    
    #[test]
    fn test_services_by_type() {
        let mut manager = DiscoveryManager::new();
        
        manager.add_service(DiscoveredService::new("d1", "TV1", ServiceType::Display, DiscoveryProtocol::MDNS));
        manager.add_service(DiscoveredService::new("d2", "TV2", ServiceType::Display, DiscoveryProtocol::MDNS));
        manager.add_service(DiscoveredService::new("a1", "Speaker", ServiceType::Audio, DiscoveryProtocol::MDNS));
        
        let displays = manager.services_by_type(ServiceType::Display);
        assert_eq!(displays.len(), 2);
    }
}
