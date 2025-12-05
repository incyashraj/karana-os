// Kāraṇa OS - Connectivity HAL
// Hardware abstraction for smart glasses wireless connectivity

use super::HalError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// WiFi state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiState {
    /// Disabled
    Disabled,
    /// Enabled but not connected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Error
    Error,
}

/// WiFi security type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    /// Open network
    Open,
    /// WEP
    Wep,
    /// WPA Personal
    WpaPsk,
    /// WPA2 Personal
    Wpa2Psk,
    /// WPA3 Personal
    Wpa3Psk,
    /// WPA Enterprise
    WpaEnterprise,
}

/// WiFi network info
#[derive(Debug, Clone)]
pub struct WifiNetwork {
    /// SSID
    pub ssid: String,
    /// BSSID (MAC address)
    pub bssid: String,
    /// Signal strength (dBm)
    pub rssi: i32,
    /// Frequency (MHz)
    pub frequency: u32,
    /// Security type
    pub security: WifiSecurity,
    /// Is currently connected
    pub connected: bool,
}

/// Bluetooth state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothState {
    /// Disabled
    Disabled,
    /// Enabled, not scanning
    Idle,
    /// Scanning for devices
    Scanning,
    /// Connected to device(s)
    Connected,
    /// Error
    Error,
}

/// Bluetooth device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothDeviceType {
    /// Unknown
    Unknown,
    /// Smartphone
    Phone,
    /// Computer
    Computer,
    /// Audio device (headphones, speaker)
    Audio,
    /// Peripheral (keyboard, mouse)
    Peripheral,
    /// Wearable
    Wearable,
    /// Health device
    Health,
}

/// Bluetooth device
#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    /// Device name
    pub name: String,
    /// MAC address
    pub address: String,
    /// Device type
    pub device_type: BluetoothDeviceType,
    /// Signal strength (dBm)
    pub rssi: i32,
    /// Is paired
    pub paired: bool,
    /// Is connected
    pub connected: bool,
    /// Supports BLE
    pub ble: bool,
}

/// Cellular state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellularState {
    /// Disabled
    Disabled,
    /// No SIM
    NoSim,
    /// Searching
    Searching,
    /// Registered (home)
    RegisteredHome,
    /// Registered (roaming)
    RegisteredRoaming,
    /// Connected (data active)
    Connected,
    /// Emergency only
    EmergencyOnly,
}

/// Cellular network type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellularType {
    /// Unknown
    Unknown,
    /// 2G (GSM/EDGE)
    G2,
    /// 3G (UMTS/HSPA)
    G3,
    /// 4G (LTE)
    G4,
    /// 5G NR
    G5,
}

/// Cellular info
#[derive(Debug, Clone)]
pub struct CellularInfo {
    /// Network state
    pub state: CellularState,
    /// Network type
    pub network_type: CellularType,
    /// Carrier name
    pub carrier: String,
    /// Signal strength (0-4 bars)
    pub signal_bars: u8,
    /// Signal strength (dBm)
    pub rssi: i32,
    /// SIM present
    pub sim_present: bool,
    /// Data roaming enabled
    pub roaming_enabled: bool,
}

/// Connectivity statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectivityStats {
    /// WiFi bytes sent
    pub wifi_tx_bytes: u64,
    /// WiFi bytes received
    pub wifi_rx_bytes: u64,
    /// Bluetooth bytes sent
    pub bt_tx_bytes: u64,
    /// Bluetooth bytes received
    pub bt_rx_bytes: u64,
    /// Cellular bytes sent
    pub cell_tx_bytes: u64,
    /// Cellular bytes received
    pub cell_rx_bytes: u64,
}

/// Connectivity HAL
#[derive(Debug)]
pub struct ConnectivityHal {
    /// WiFi state
    wifi_state: WifiState,
    /// WiFi enabled
    wifi_enabled: AtomicBool,
    /// Connected WiFi network
    connected_wifi: Option<WifiNetwork>,
    /// Known WiFi networks
    known_networks: HashMap<String, String>, // SSID -> Password
    
    /// Bluetooth state
    bt_state: BluetoothState,
    /// Bluetooth enabled
    bt_enabled: AtomicBool,
    /// Connected BT devices
    connected_devices: Vec<BluetoothDevice>,
    /// Paired devices
    paired_devices: Vec<BluetoothDevice>,
    
    /// Cellular info
    cellular: CellularInfo,
    /// Cellular enabled
    cellular_enabled: AtomicBool,
    
    /// Statistics
    stats: ConnectivityStats,
    
    /// Is initialized
    initialized: bool,
    /// Start time
    start_time: Option<Instant>,
}

impl ConnectivityHal {
    /// Create new connectivity HAL
    pub fn new() -> Result<Self, HalError> {
        Ok(Self {
            wifi_state: WifiState::Disabled,
            wifi_enabled: AtomicBool::new(false),
            connected_wifi: None,
            known_networks: HashMap::new(),
            
            bt_state: BluetoothState::Disabled,
            bt_enabled: AtomicBool::new(false),
            connected_devices: Vec::new(),
            paired_devices: Vec::new(),
            
            cellular: CellularInfo {
                state: CellularState::Disabled,
                network_type: CellularType::Unknown,
                carrier: String::new(),
                signal_bars: 0,
                rssi: -100,
                sim_present: false,
                roaming_enabled: false,
            },
            cellular_enabled: AtomicBool::new(false),
            
            stats: ConnectivityStats::default(),
            initialized: false,
            start_time: None,
        })
    }

    /// Initialize connectivity systems
    pub fn initialize(&mut self) -> Result<(), HalError> {
        self.initialized = true;
        self.start_time = Some(Instant::now());
        Ok(())
    }

    // ==================== WiFi ====================

    /// Enable/disable WiFi
    pub fn set_wifi_enabled(&mut self, enabled: bool) -> Result<(), HalError> {
        self.wifi_enabled.store(enabled, Ordering::Relaxed);
        
        if enabled {
            self.wifi_state = WifiState::Disconnected;
        } else {
            self.wifi_state = WifiState::Disabled;
            self.connected_wifi = None;
        }
        
        Ok(())
    }

    /// Is WiFi enabled
    pub fn is_wifi_enabled(&self) -> bool {
        self.wifi_enabled.load(Ordering::Relaxed)
    }

    /// Get WiFi state
    pub fn wifi_state(&self) -> WifiState {
        self.wifi_state
    }

    /// Scan for WiFi networks
    pub fn scan_wifi(&mut self) -> Result<Vec<WifiNetwork>, HalError> {
        if !self.wifi_enabled.load(Ordering::Relaxed) {
            return Err(HalError::ConfigError("WiFi disabled".into()));
        }

        // Return simulated scan results
        Ok(vec![
            WifiNetwork {
                ssid: "KaranaOS_Network".into(),
                bssid: "AA:BB:CC:DD:EE:FF".into(),
                rssi: -45,
                frequency: 5180,
                security: WifiSecurity::Wpa2Psk,
                connected: false,
            },
            WifiNetwork {
                ssid: "Guest_Network".into(),
                bssid: "11:22:33:44:55:66".into(),
                rssi: -65,
                frequency: 2437,
                security: WifiSecurity::Open,
                connected: false,
            },
        ])
    }

    /// Connect to WiFi network
    pub fn connect_wifi(&mut self, ssid: &str, password: Option<&str>) -> Result<(), HalError> {
        if !self.wifi_enabled.load(Ordering::Relaxed) {
            return Err(HalError::ConfigError("WiFi disabled".into()));
        }

        self.wifi_state = WifiState::Connecting;

        // Store credentials
        if let Some(pass) = password {
            self.known_networks.insert(ssid.to_string(), pass.to_string());
        }

        // Simulate connection
        self.connected_wifi = Some(WifiNetwork {
            ssid: ssid.to_string(),
            bssid: "AA:BB:CC:DD:EE:FF".into(),
            rssi: -45,
            frequency: 5180,
            security: if password.is_some() { WifiSecurity::Wpa2Psk } else { WifiSecurity::Open },
            connected: true,
        });
        
        self.wifi_state = WifiState::Connected;
        Ok(())
    }

    /// Disconnect from WiFi
    pub fn disconnect_wifi(&mut self) -> Result<(), HalError> {
        self.connected_wifi = None;
        self.wifi_state = WifiState::Disconnected;
        Ok(())
    }

    /// Get connected WiFi network
    pub fn connected_wifi(&self) -> Option<&WifiNetwork> {
        self.connected_wifi.as_ref()
    }

    /// Get WiFi signal strength (dBm)
    pub fn wifi_rssi(&self) -> Option<i32> {
        self.connected_wifi.as_ref().map(|n| n.rssi)
    }

    // ==================== Bluetooth ====================

    /// Enable/disable Bluetooth
    pub fn set_bluetooth_enabled(&mut self, enabled: bool) -> Result<(), HalError> {
        self.bt_enabled.store(enabled, Ordering::Relaxed);
        
        if enabled {
            self.bt_state = BluetoothState::Idle;
        } else {
            self.bt_state = BluetoothState::Disabled;
            self.connected_devices.clear();
        }
        
        Ok(())
    }

    /// Is Bluetooth enabled
    pub fn is_bluetooth_enabled(&self) -> bool {
        self.bt_enabled.load(Ordering::Relaxed)
    }

    /// Get Bluetooth state
    pub fn bluetooth_state(&self) -> BluetoothState {
        self.bt_state
    }

    /// Scan for Bluetooth devices
    pub fn scan_bluetooth(&mut self) -> Result<Vec<BluetoothDevice>, HalError> {
        if !self.bt_enabled.load(Ordering::Relaxed) {
            return Err(HalError::ConfigError("Bluetooth disabled".into()));
        }

        self.bt_state = BluetoothState::Scanning;

        // Return simulated scan results
        Ok(vec![
            BluetoothDevice {
                name: "Wireless Earbuds".into(),
                address: "AA:BB:CC:11:22:33".into(),
                device_type: BluetoothDeviceType::Audio,
                rssi: -50,
                paired: false,
                connected: false,
                ble: true,
            },
            BluetoothDevice {
                name: "Companion Phone".into(),
                address: "DD:EE:FF:44:55:66".into(),
                device_type: BluetoothDeviceType::Phone,
                rssi: -60,
                paired: true,
                connected: false,
                ble: true,
            },
        ])
    }

    /// Pair with Bluetooth device
    pub fn pair_bluetooth(&mut self, address: &str) -> Result<(), HalError> {
        if !self.bt_enabled.load(Ordering::Relaxed) {
            return Err(HalError::ConfigError("Bluetooth disabled".into()));
        }

        // Simulate pairing
        let device = BluetoothDevice {
            name: format!("Device_{}", &address[..8]),
            address: address.to_string(),
            device_type: BluetoothDeviceType::Unknown,
            rssi: -60,
            paired: true,
            connected: false,
            ble: true,
        };
        
        self.paired_devices.push(device);
        Ok(())
    }

    /// Connect to Bluetooth device
    pub fn connect_bluetooth(&mut self, address: &str) -> Result<(), HalError> {
        if !self.bt_enabled.load(Ordering::Relaxed) {
            return Err(HalError::ConfigError("Bluetooth disabled".into()));
        }

        // Find in paired devices
        if let Some(device) = self.paired_devices.iter_mut().find(|d| d.address == address) {
            device.connected = true;
            self.connected_devices.push(device.clone());
            self.bt_state = BluetoothState::Connected;
            Ok(())
        } else {
            Err(HalError::DeviceNotFound("Bluetooth device not found".into()))
        }
    }

    /// Disconnect Bluetooth device
    pub fn disconnect_bluetooth(&mut self, address: &str) -> Result<(), HalError> {
        self.connected_devices.retain(|d| d.address != address);
        
        if let Some(device) = self.paired_devices.iter_mut().find(|d| d.address == address) {
            device.connected = false;
        }
        
        if self.connected_devices.is_empty() {
            self.bt_state = BluetoothState::Idle;
        }
        
        Ok(())
    }

    /// Get connected Bluetooth devices
    pub fn connected_bluetooth_devices(&self) -> &[BluetoothDevice] {
        &self.connected_devices
    }

    /// Get paired Bluetooth devices
    pub fn paired_bluetooth_devices(&self) -> &[BluetoothDevice] {
        &self.paired_devices
    }

    // ==================== Cellular ====================

    /// Enable/disable cellular
    pub fn set_cellular_enabled(&mut self, enabled: bool) -> Result<(), HalError> {
        self.cellular_enabled.store(enabled, Ordering::Relaxed);
        
        if enabled {
            self.cellular.state = if self.cellular.sim_present {
                CellularState::Searching
            } else {
                CellularState::NoSim
            };
        } else {
            self.cellular.state = CellularState::Disabled;
        }
        
        Ok(())
    }

    /// Is cellular enabled
    pub fn is_cellular_enabled(&self) -> bool {
        self.cellular_enabled.load(Ordering::Relaxed)
    }

    /// Get cellular state
    pub fn cellular_state(&self) -> CellularState {
        self.cellular.state
    }

    /// Get cellular info
    pub fn cellular_info(&self) -> &CellularInfo {
        &self.cellular
    }

    /// Set data roaming
    pub fn set_data_roaming(&mut self, enabled: bool) -> Result<(), HalError> {
        self.cellular.roaming_enabled = enabled;
        Ok(())
    }

    /// Enable/disable airplane mode
    pub fn set_airplane_mode(&mut self, enabled: bool) -> Result<(), HalError> {
        if enabled {
            self.set_wifi_enabled(false)?;
            self.set_bluetooth_enabled(false)?;
            self.set_cellular_enabled(false)?;
        }
        Ok(())
    }

    // ==================== General ====================

    /// Get connectivity statistics
    pub fn stats(&self) -> &ConnectivityStats {
        &self.stats
    }

    /// Is any network connected
    pub fn is_connected(&self) -> bool {
        self.wifi_state == WifiState::Connected
            || self.cellular.state == CellularState::Connected
    }

    /// Get active network type
    pub fn active_network(&self) -> Option<&str> {
        if self.wifi_state == WifiState::Connected {
            Some("wifi")
        } else if self.cellular.state == CellularState::Connected {
            Some("cellular")
        } else {
            None
        }
    }

    /// Self-test
    pub fn test(&self) -> Result<(), HalError> {
        if !self.initialized {
            return Err(HalError::ConfigError("Not initialized".into()));
        }
        Ok(())
    }
}

impl Default for ConnectivityHal {
    fn default() -> Self {
        Self::new().expect("Default connectivity HAL creation should not fail")
    }
}

/// Network quality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkQuality {
    /// Unknown
    Unknown,
    /// Poor (unusable)
    Poor,
    /// Fair
    Fair,
    /// Good
    Good,
    /// Excellent
    Excellent,
}

impl NetworkQuality {
    /// From WiFi RSSI
    pub fn from_wifi_rssi(rssi: i32) -> Self {
        match rssi {
            _ if rssi >= -50 => Self::Excellent,
            _ if rssi >= -60 => Self::Good,
            _ if rssi >= -70 => Self::Fair,
            _ if rssi >= -80 => Self::Poor,
            _ => Self::Unknown,
        }
    }

    /// From cellular signal bars
    pub fn from_signal_bars(bars: u8) -> Self {
        match bars {
            4 => Self::Excellent,
            3 => Self::Good,
            2 => Self::Fair,
            1 => Self::Poor,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connectivity_hal_creation() {
        let hal = ConnectivityHal::new();
        assert!(hal.is_ok());
    }

    #[test]
    fn test_wifi_enable_disable() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();

        hal.set_wifi_enabled(true).unwrap();
        assert!(hal.is_wifi_enabled());
        assert_eq!(hal.wifi_state(), WifiState::Disconnected);

        hal.set_wifi_enabled(false).unwrap();
        assert!(!hal.is_wifi_enabled());
        assert_eq!(hal.wifi_state(), WifiState::Disabled);
    }

    #[test]
    fn test_wifi_scan() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();
        hal.set_wifi_enabled(true).unwrap();

        let networks = hal.scan_wifi().unwrap();
        assert!(!networks.is_empty());
    }

    #[test]
    fn test_wifi_connect() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();
        hal.set_wifi_enabled(true).unwrap();

        hal.connect_wifi("TestNetwork", Some("password123")).unwrap();
        assert_eq!(hal.wifi_state(), WifiState::Connected);
        assert!(hal.connected_wifi().is_some());
    }

    #[test]
    fn test_bluetooth_enable_disable() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();

        hal.set_bluetooth_enabled(true).unwrap();
        assert!(hal.is_bluetooth_enabled());
        assert_eq!(hal.bluetooth_state(), BluetoothState::Idle);

        hal.set_bluetooth_enabled(false).unwrap();
        assert!(!hal.is_bluetooth_enabled());
    }

    #[test]
    fn test_bluetooth_scan() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();
        hal.set_bluetooth_enabled(true).unwrap();

        let devices = hal.scan_bluetooth().unwrap();
        assert!(!devices.is_empty());
    }

    #[test]
    fn test_bluetooth_pair_connect() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();
        hal.set_bluetooth_enabled(true).unwrap();

        hal.pair_bluetooth("AA:BB:CC:DD:EE:FF").unwrap();
        assert!(!hal.paired_bluetooth_devices().is_empty());

        hal.connect_bluetooth("AA:BB:CC:DD:EE:FF").unwrap();
        assert!(!hal.connected_bluetooth_devices().is_empty());
    }

    #[test]
    fn test_cellular_enable() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();

        hal.set_cellular_enabled(true).unwrap();
        assert!(hal.is_cellular_enabled());
    }

    #[test]
    fn test_airplane_mode() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();

        hal.set_wifi_enabled(true).unwrap();
        hal.set_bluetooth_enabled(true).unwrap();

        hal.set_airplane_mode(true).unwrap();
        
        assert!(!hal.is_wifi_enabled());
        assert!(!hal.is_bluetooth_enabled());
        assert!(!hal.is_cellular_enabled());
    }

    #[test]
    fn test_network_quality_wifi() {
        assert_eq!(NetworkQuality::from_wifi_rssi(-45), NetworkQuality::Excellent);
        assert_eq!(NetworkQuality::from_wifi_rssi(-55), NetworkQuality::Good);
        assert_eq!(NetworkQuality::from_wifi_rssi(-65), NetworkQuality::Fair);
        assert_eq!(NetworkQuality::from_wifi_rssi(-75), NetworkQuality::Poor);
    }

    #[test]
    fn test_network_quality_cellular() {
        assert_eq!(NetworkQuality::from_signal_bars(4), NetworkQuality::Excellent);
        assert_eq!(NetworkQuality::from_signal_bars(3), NetworkQuality::Good);
        assert_eq!(NetworkQuality::from_signal_bars(2), NetworkQuality::Fair);
        assert_eq!(NetworkQuality::from_signal_bars(1), NetworkQuality::Poor);
    }

    #[test]
    fn test_is_connected() {
        let mut hal = ConnectivityHal::new().unwrap();
        hal.initialize().unwrap();

        assert!(!hal.is_connected());

        hal.set_wifi_enabled(true).unwrap();
        hal.connect_wifi("Test", None).unwrap();
        
        assert!(hal.is_connected());
        assert_eq!(hal.active_network(), Some("wifi"));
    }
}
