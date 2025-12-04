//! Bluetooth device management

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Bluetooth state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothState {
    /// Bluetooth disabled
    Disabled,
    /// Enabled and idle
    Idle,
    /// Scanning for devices
    Scanning,
    /// Pairing with device
    Pairing,
    /// Connected
    Connected,
    /// Error
    Error,
}

/// Bluetooth device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    /// Unknown device
    Unknown,
    /// Phone
    Phone,
    /// Computer
    Computer,
    /// Audio device (headphones, speakers)
    Audio,
    /// Input device (keyboard, mouse)
    Input,
    /// Health device
    Health,
    /// Wearable
    Wearable,
    /// Peripheral
    Peripheral,
}

/// Bluetooth device information
#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    /// Device name
    pub name: String,
    /// MAC address
    pub address: String,
    /// Device class
    pub device_class: DeviceClass,
    /// Is paired
    pub is_paired: bool,
    /// Is connected
    pub is_connected: bool,
    /// Signal strength (RSSI)
    pub rssi: i16,
    /// Battery level (if available)
    pub battery_level: Option<u8>,
    /// Last seen
    pub last_seen: Instant,
    /// Supports tethering
    pub supports_tethering: bool,
    /// Supports audio
    pub supports_audio: bool,
    /// Services UUIDs
    pub services: Vec<String>,
}

impl BluetoothDevice {
    /// Create new device
    pub fn new(name: &str, address: &str) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
            device_class: DeviceClass::Unknown,
            is_paired: false,
            is_connected: false,
            rssi: 0,
            battery_level: None,
            last_seen: Instant::now(),
            supports_tethering: false,
            supports_audio: false,
            services: Vec::new(),
        }
    }
    
    /// Get signal quality description
    pub fn signal_description(&self) -> &str {
        match self.rssi {
            _ if self.rssi >= -50 => "Excellent",
            _ if self.rssi >= -60 => "Good",
            _ if self.rssi >= -70 => "Fair",
            _ if self.rssi >= -80 => "Weak",
            _ => "Very Weak",
        }
    }
}

/// Bluetooth manager
#[derive(Debug)]
pub struct BluetoothManager {
    /// Current state
    state: BluetoothState,
    /// Discovered devices
    devices: HashMap<String, BluetoothDevice>,
    /// Paired devices
    paired_devices: Vec<String>,
    /// Connected device address
    connected_device: Option<String>,
    /// Tethering active
    tethering_active: bool,
    /// Discoverable mode
    discoverable: bool,
    /// Discoverable timeout
    discoverable_timeout: Duration,
    /// Last scan time
    last_scan: Option<Instant>,
    /// Device name (this device)
    device_name: String,
}

impl BluetoothManager {
    /// Create new Bluetooth manager
    pub fn new() -> Self {
        Self {
            state: BluetoothState::Disabled,
            devices: HashMap::new(),
            paired_devices: Vec::new(),
            connected_device: None,
            tethering_active: false,
            discoverable: false,
            discoverable_timeout: Duration::from_secs(120),
            last_scan: None,
            device_name: "Kāraṇa Glasses".to_string(),
        }
    }
    
    /// Enable Bluetooth
    pub fn enable(&mut self) {
        if self.state == BluetoothState::Disabled {
            self.state = BluetoothState::Idle;
        }
    }
    
    /// Disable Bluetooth
    pub fn disable(&mut self) {
        self.disconnect_all();
        self.state = BluetoothState::Disabled;
        self.discoverable = false;
        self.tethering_active = false;
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.state != BluetoothState::Disabled
    }
    
    /// Get current state
    pub fn state(&self) -> BluetoothState {
        self.state
    }
    
    /// Start scanning
    pub fn scan(&mut self) {
        if !self.is_enabled() {
            return;
        }
        
        self.state = BluetoothState::Scanning;
        self.last_scan = Some(Instant::now());
        
        // In real implementation, would trigger hardware scan
        // Simulate completion
        self.state = BluetoothState::Idle;
    }
    
    /// Get discovered devices
    pub fn discovered_devices(&self) -> Vec<&BluetoothDevice> {
        self.devices.values().collect()
    }
    
    /// Get device by address
    pub fn get_device(&self, address: &str) -> Option<&BluetoothDevice> {
        self.devices.get(address)
    }
    
    /// Pair with device
    pub fn pair(&mut self, address: &str) -> Result<(), String> {
        if !self.is_enabled() {
            return Err("Bluetooth is disabled".to_string());
        }
        
        if !self.devices.contains_key(address) {
            return Err("Device not found".to_string());
        }
        
        self.state = BluetoothState::Pairing;
        
        // Simulate successful pairing
        if let Some(device) = self.devices.get_mut(address) {
            device.is_paired = true;
        }
        self.paired_devices.push(address.to_string());
        
        self.state = BluetoothState::Idle;
        Ok(())
    }
    
    /// Unpair device
    pub fn unpair(&mut self, address: &str) {
        self.paired_devices.retain(|a| a != address);
        
        if let Some(device) = self.devices.get_mut(address) {
            device.is_paired = false;
            device.is_connected = false;
        }
        
        if self.connected_device.as_deref() == Some(address) {
            self.connected_device = None;
        }
    }
    
    /// Connect to device
    pub fn connect(&mut self, address: &str) -> Result<(), String> {
        if !self.is_enabled() {
            return Err("Bluetooth is disabled".to_string());
        }
        
        if !self.paired_devices.contains(&address.to_string()) {
            return Err("Device not paired".to_string());
        }
        
        // Disconnect current device
        if let Some(current) = &self.connected_device {
            if let Some(device) = self.devices.get_mut(current) {
                device.is_connected = false;
            }
        }
        
        // Connect new device
        if let Some(device) = self.devices.get_mut(address) {
            device.is_connected = true;
        }
        self.connected_device = Some(address.to_string());
        self.state = BluetoothState::Connected;
        
        Ok(())
    }
    
    /// Disconnect from device
    pub fn disconnect(&mut self, address: &str) {
        if self.connected_device.as_deref() == Some(address) {
            if let Some(device) = self.devices.get_mut(address) {
                device.is_connected = false;
            }
            self.connected_device = None;
            self.tethering_active = false;
            self.state = BluetoothState::Idle;
        }
    }
    
    /// Disconnect all devices
    pub fn disconnect_all(&mut self) {
        for device in self.devices.values_mut() {
            device.is_connected = false;
        }
        self.connected_device = None;
        self.tethering_active = false;
        
        if self.is_enabled() {
            self.state = BluetoothState::Idle;
        }
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected_device.is_some()
    }
    
    /// Get connected device
    pub fn connected_device(&self) -> Option<&BluetoothDevice> {
        self.connected_device.as_ref()
            .and_then(|addr| self.devices.get(addr))
    }
    
    /// Start tethering
    pub fn start_tethering(&mut self) -> Result<(), String> {
        if let Some(device) = self.connected_device() {
            if !device.supports_tethering {
                return Err("Device does not support tethering".to_string());
            }
        } else {
            return Err("No device connected".to_string());
        }
        
        self.tethering_active = true;
        Ok(())
    }
    
    /// Stop tethering
    pub fn stop_tethering(&mut self) {
        self.tethering_active = false;
    }
    
    /// Check if tethering
    pub fn is_tethering(&self) -> bool {
        self.tethering_active && self.is_connected()
    }
    
    /// Set discoverable mode
    pub fn set_discoverable(&mut self, discoverable: bool) {
        if self.is_enabled() {
            self.discoverable = discoverable;
        }
    }
    
    /// Check if discoverable
    pub fn is_discoverable(&self) -> bool {
        self.discoverable
    }
    
    /// Get paired devices
    pub fn paired_devices(&self) -> Vec<&BluetoothDevice> {
        self.paired_devices
            .iter()
            .filter_map(|addr| self.devices.get(addr))
            .collect()
    }
    
    /// Set device name
    pub fn set_device_name(&mut self, name: &str) {
        self.device_name = name.to_string();
    }
    
    /// Get device name
    pub fn device_name(&self) -> &str {
        &self.device_name
    }
    
    /// Update Bluetooth state
    pub fn update(&mut self) {
        // Remove devices not seen for a while
        let threshold = Duration::from_secs(60);
        let now = Instant::now();
        
        self.devices.retain(|_, d| {
            d.is_paired || now.duration_since(d.last_seen) < threshold
        });
    }
    
    /// Add test device (for testing)
    #[cfg(test)]
    pub fn add_test_device(&mut self, device: BluetoothDevice) {
        self.devices.insert(device.address.clone(), device);
    }
}

impl Default for BluetoothManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bluetooth_manager_creation() {
        let manager = BluetoothManager::new();
        assert_eq!(manager.state(), BluetoothState::Disabled);
        assert!(!manager.is_connected());
    }
    
    #[test]
    fn test_enable_disable() {
        let mut manager = BluetoothManager::new();
        
        manager.enable();
        assert!(manager.is_enabled());
        
        manager.disable();
        assert!(!manager.is_enabled());
    }
    
    #[test]
    fn test_pair_connect() {
        let mut manager = BluetoothManager::new();
        manager.enable();
        
        let mut device = BluetoothDevice::new("My Phone", "AA:BB:CC:DD:EE:FF");
        device.supports_tethering = true;
        manager.add_test_device(device);
        
        // Pair
        assert!(manager.pair("AA:BB:CC:DD:EE:FF").is_ok());
        assert_eq!(manager.paired_devices().len(), 1);
        
        // Connect
        assert!(manager.connect("AA:BB:CC:DD:EE:FF").is_ok());
        assert!(manager.is_connected());
    }
    
    #[test]
    fn test_tethering() {
        let mut manager = BluetoothManager::new();
        manager.enable();
        
        let mut device = BluetoothDevice::new("Phone", "AA:BB:CC:DD:EE:FF");
        device.supports_tethering = true;
        manager.add_test_device(device);
        
        manager.pair("AA:BB:CC:DD:EE:FF").unwrap();
        manager.connect("AA:BB:CC:DD:EE:FF").unwrap();
        
        assert!(manager.start_tethering().is_ok());
        assert!(manager.is_tethering());
        
        manager.stop_tethering();
        assert!(!manager.is_tethering());
    }
    
    #[test]
    fn test_discoverable() {
        let mut manager = BluetoothManager::new();
        manager.enable();
        
        assert!(!manager.is_discoverable());
        
        manager.set_discoverable(true);
        assert!(manager.is_discoverable());
    }
    
    #[test]
    fn test_device_name() {
        let mut manager = BluetoothManager::new();
        
        assert_eq!(manager.device_name(), "Kāraṇa Glasses");
        
        manager.set_device_name("My Glasses");
        assert_eq!(manager.device_name(), "My Glasses");
    }
}
