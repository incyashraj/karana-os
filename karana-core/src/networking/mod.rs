//! Networking System for Kāraṇa OS AR Glasses
//!
//! Manages WiFi, Bluetooth, cloud connectivity, and device synchronization.

use std::time::Instant;

pub mod wifi;
pub mod bluetooth;
pub mod cloud;
pub mod sync;
pub mod discovery;

pub use wifi::{WiFiManager, WiFiNetwork, WiFiState, WiFiSecurity};
pub use bluetooth::{BluetoothManager, BluetoothDevice, BluetoothState, DeviceClass};
pub use cloud::{CloudService, CloudState, CloudProvider, SyncItemType};
pub use sync::{SyncManager, SyncOperation, SyncDirection, SyncPriority, ConflictStrategy};
pub use discovery::{DiscoveryManager, DiscoveredService, DiscoveredDevice, ServiceType, DiscoveryProtocol};

/// Network connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    /// No connection
    None,
    /// WiFi connection
    WiFi,
    /// Bluetooth tethering
    BluetoothTether,
    /// USB tethering
    UsbTether,
    /// Mobile data (if glasses support)
    Cellular,
}

/// Network quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetworkQuality {
    /// No signal
    None,
    /// Poor signal
    Poor,
    /// Fair signal
    Fair,
    /// Good signal
    Good,
    /// Excellent signal
    Excellent,
}

impl NetworkQuality {
    /// From signal strength in dBm
    pub fn from_dbm(dbm: i32) -> Self {
        match dbm {
            _ if dbm >= -50 => NetworkQuality::Excellent,
            _ if dbm >= -60 => NetworkQuality::Good,
            _ if dbm >= -70 => NetworkQuality::Fair,
            _ if dbm >= -80 => NetworkQuality::Poor,
            _ => NetworkQuality::None,
        }
    }
    
    /// Get approximate percentage
    pub fn percentage(&self) -> u8 {
        match self {
            NetworkQuality::None => 0,
            NetworkQuality::Poor => 25,
            NetworkQuality::Fair => 50,
            NetworkQuality::Good => 75,
            NetworkQuality::Excellent => 100,
        }
    }
}

/// Network state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
    /// Disabled
    Disabled,
    /// Disconnected but enabled
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Connection error
    Error,
}

/// Data usage statistics
#[derive(Debug, Clone, Default)]
pub struct DataUsage {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Period start
    pub period_start: Option<Instant>,
    /// Session start
    pub session_start: Option<Instant>,
}

impl DataUsage {
    /// Total bytes
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent + self.bytes_received
    }
    
    /// Add sent bytes
    pub fn add_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
    }
    
    /// Add received bytes
    pub fn add_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }
    
    /// Reset statistics
    pub fn reset(&mut self) {
        self.bytes_sent = 0;
        self.bytes_received = 0;
        self.period_start = Some(Instant::now());
    }
}

/// Network manager
#[derive(Debug)]
pub struct NetworkManager {
    /// WiFi manager
    wifi: WiFiManager,
    /// Bluetooth manager
    bluetooth: BluetoothManager,
    /// Cloud service
    cloud: CloudService,
    /// Sync manager
    sync_manager: SyncManager,
    /// Discovery manager
    discovery: DiscoveryManager,
    /// Current connection type
    connection_type: ConnectionType,
    /// Network quality
    quality: NetworkQuality,
    /// Data usage
    data_usage: DataUsage,
    /// Auto-connect enabled
    auto_connect: bool,
    /// Metered connection mode
    metered_mode: bool,
    /// Low data mode
    low_data_mode: bool,
    /// Network listeners
    listeners: Vec<usize>,
}

impl NetworkManager {
    /// Create new network manager
    pub fn new() -> Self {
        Self {
            wifi: WiFiManager::new(),
            bluetooth: BluetoothManager::new(),
            cloud: CloudService::new(),
            sync_manager: SyncManager::new(),
            discovery: DiscoveryManager::new(),
            connection_type: ConnectionType::None,
            quality: NetworkQuality::None,
            data_usage: DataUsage::default(),
            auto_connect: true,
            metered_mode: false,
            low_data_mode: false,
            listeners: Vec::new(),
        }
    }
    
    /// Get WiFi manager
    pub fn wifi(&self) -> &WiFiManager {
        &self.wifi
    }
    
    /// Get mutable WiFi manager
    pub fn wifi_mut(&mut self) -> &mut WiFiManager {
        &mut self.wifi
    }
    
    /// Get Bluetooth manager
    pub fn bluetooth(&self) -> &BluetoothManager {
        &self.bluetooth
    }
    
    /// Get mutable Bluetooth manager
    pub fn bluetooth_mut(&mut self) -> &mut BluetoothManager {
        &mut self.bluetooth
    }
    
    /// Get cloud service
    pub fn cloud(&self) -> &CloudService {
        &self.cloud
    }
    
    /// Get mutable cloud service
    pub fn cloud_mut(&mut self) -> &mut CloudService {
        &mut self.cloud
    }
    
    /// Get sync manager
    pub fn sync_manager(&self) -> &SyncManager {
        &self.sync_manager
    }
    
    /// Get mutable sync manager
    pub fn sync_manager_mut(&mut self) -> &mut SyncManager {
        &mut self.sync_manager
    }
    
    /// Get discovery manager
    pub fn discovery(&self) -> &DiscoveryManager {
        &self.discovery
    }
    
    /// Get mutable discovery manager
    pub fn discovery_mut(&mut self) -> &mut DiscoveryManager {
        &mut self.discovery
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection_type != ConnectionType::None
    }
    
    /// Get connection type
    pub fn connection_type(&self) -> ConnectionType {
        self.connection_type
    }
    
    /// Get network quality
    pub fn quality(&self) -> NetworkQuality {
        self.quality
    }
    
    /// Get data usage
    pub fn data_usage(&self) -> &DataUsage {
        &self.data_usage
    }
    
    /// Enable/disable auto-connect
    pub fn set_auto_connect(&mut self, enabled: bool) {
        self.auto_connect = enabled;
    }
    
    /// Check if auto-connect is enabled
    pub fn is_auto_connect(&self) -> bool {
        self.auto_connect
    }
    
    /// Set metered mode
    pub fn set_metered_mode(&mut self, metered: bool) {
        self.metered_mode = metered;
    }
    
    /// Check if metered mode
    pub fn is_metered(&self) -> bool {
        self.metered_mode
    }
    
    /// Set low data mode
    pub fn set_low_data_mode(&mut self, enabled: bool) {
        self.low_data_mode = enabled;
    }
    
    /// Check if low data mode
    pub fn is_low_data_mode(&self) -> bool {
        self.low_data_mode
    }
    
    /// Update network state
    pub fn update(&mut self) {
        // Update WiFi
        self.wifi.update();
        
        // Update Bluetooth
        self.bluetooth.update();
        
        // Update discovery
        self.discovery.update();
        
        // Determine connection type
        if self.wifi.is_connected() {
            self.connection_type = ConnectionType::WiFi;
            self.quality = self.wifi.signal_quality();
        } else if self.bluetooth.is_tethering() {
            self.connection_type = ConnectionType::BluetoothTether;
            self.quality = NetworkQuality::Fair;
        } else {
            self.connection_type = ConnectionType::None;
            self.quality = NetworkQuality::None;
        }
        
        // Process sync queue if connected
        if self.is_connected() && !self.low_data_mode {
            self.sync_manager.process_next();
        }
        
        // Update cloud sync
        if self.is_connected() {
            self.cloud.update();
        }
    }
    
    /// Reset data usage
    pub fn reset_data_usage(&mut self) {
        self.data_usage.reset();
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_manager_creation() {
        let manager = NetworkManager::new();
        assert!(!manager.is_connected());
        assert_eq!(manager.connection_type(), ConnectionType::None);
    }
    
    #[test]
    fn test_network_quality_from_dbm() {
        assert_eq!(NetworkQuality::from_dbm(-45), NetworkQuality::Excellent);
        assert_eq!(NetworkQuality::from_dbm(-55), NetworkQuality::Good);
        assert_eq!(NetworkQuality::from_dbm(-65), NetworkQuality::Fair);
        assert_eq!(NetworkQuality::from_dbm(-75), NetworkQuality::Poor);
        assert_eq!(NetworkQuality::from_dbm(-90), NetworkQuality::None);
    }
    
    #[test]
    fn test_data_usage() {
        let mut usage = DataUsage::default();
        
        usage.add_sent(1000);
        usage.add_received(2000);
        
        assert_eq!(usage.bytes_sent, 1000);
        assert_eq!(usage.bytes_received, 2000);
        assert_eq!(usage.total_bytes(), 3000);
    }
    
    #[test]
    fn test_auto_connect() {
        let mut manager = NetworkManager::new();
        
        assert!(manager.is_auto_connect());
        
        manager.set_auto_connect(false);
        assert!(!manager.is_auto_connect());
    }
    
    #[test]
    fn test_metered_mode() {
        let mut manager = NetworkManager::new();
        
        assert!(!manager.is_metered());
        
        manager.set_metered_mode(true);
        assert!(manager.is_metered());
    }
}
