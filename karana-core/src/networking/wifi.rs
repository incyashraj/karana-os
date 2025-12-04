//! WiFi network management

use std::collections::HashMap;
use std::time::{Duration, Instant};
use super::NetworkQuality;

/// WiFi state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiFiState {
    /// WiFi disabled
    Disabled,
    /// Scanning for networks
    Scanning,
    /// Idle (enabled but not connected)
    Idle,
    /// Connecting to network
    Connecting,
    /// Connected
    Connected,
    /// Disconnecting
    Disconnecting,
    /// Error state
    Error,
}

/// WiFi security type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiFiSecurity {
    /// Open network
    Open,
    /// WEP encryption
    WEP,
    /// WPA/WPA2 Personal
    WPA,
    /// WPA3 Personal
    WPA3,
    /// WPA Enterprise
    WPAEnterprise,
    /// Unknown security
    Unknown,
}

/// WiFi network information
#[derive(Debug, Clone)]
pub struct WiFiNetwork {
    /// Network SSID
    pub ssid: String,
    /// BSSID (MAC address)
    pub bssid: String,
    /// Signal strength in dBm
    pub signal_dbm: i32,
    /// Security type
    pub security: WiFiSecurity,
    /// Frequency in MHz
    pub frequency_mhz: u32,
    /// Channel number
    pub channel: u8,
    /// Hidden network
    pub is_hidden: bool,
    /// Saved network (credentials stored)
    pub is_saved: bool,
    /// Last connected time
    pub last_connected: Option<Instant>,
}

impl WiFiNetwork {
    /// Get signal quality
    pub fn signal_quality(&self) -> NetworkQuality {
        NetworkQuality::from_dbm(self.signal_dbm)
    }
    
    /// Check if 5GHz network
    pub fn is_5ghz(&self) -> bool {
        self.frequency_mhz >= 5000
    }
    
    /// Check if 2.4GHz network
    pub fn is_2_4ghz(&self) -> bool {
        self.frequency_mhz < 5000
    }
}

/// WiFi manager
#[derive(Debug)]
pub struct WiFiManager {
    /// Current state
    state: WiFiState,
    /// Available networks
    networks: HashMap<String, WiFiNetwork>,
    /// Currently connected network
    connected_network: Option<String>,
    /// Saved network credentials (SSID -> password)
    saved_credentials: HashMap<String, String>,
    /// Last scan time
    last_scan: Option<Instant>,
    /// Scan interval
    scan_interval: Duration,
    /// Auto-reconnect enabled
    auto_reconnect: bool,
    /// Preferred network (SSID)
    preferred_network: Option<String>,
    /// Connection attempt count
    connection_attempts: u32,
    /// Max connection attempts
    max_attempts: u32,
}

impl WiFiManager {
    /// Create new WiFi manager
    pub fn new() -> Self {
        Self {
            state: WiFiState::Disabled,
            networks: HashMap::new(),
            connected_network: None,
            saved_credentials: HashMap::new(),
            last_scan: None,
            scan_interval: Duration::from_secs(30),
            auto_reconnect: true,
            preferred_network: None,
            connection_attempts: 0,
            max_attempts: 3,
        }
    }
    
    /// Enable WiFi
    pub fn enable(&mut self) {
        if self.state == WiFiState::Disabled {
            self.state = WiFiState::Idle;
        }
    }
    
    /// Disable WiFi
    pub fn disable(&mut self) {
        if self.connected_network.is_some() {
            self.disconnect();
        }
        self.state = WiFiState::Disabled;
    }
    
    /// Check if WiFi is enabled
    pub fn is_enabled(&self) -> bool {
        self.state != WiFiState::Disabled
    }
    
    /// Get current state
    pub fn state(&self) -> WiFiState {
        self.state
    }
    
    /// Start scanning for networks
    pub fn scan(&mut self) {
        if self.state == WiFiState::Disabled {
            return;
        }
        
        self.state = WiFiState::Scanning;
        self.last_scan = Some(Instant::now());
        
        // In real implementation, would trigger hardware scan
        // For now, simulate completion
        self.state = WiFiState::Idle;
    }
    
    /// Get available networks
    pub fn available_networks(&self) -> Vec<&WiFiNetwork> {
        let mut networks: Vec<_> = self.networks.values().collect();
        // Sort by signal strength
        networks.sort_by(|a, b| b.signal_dbm.cmp(&a.signal_dbm));
        networks
    }
    
    /// Get network by SSID
    pub fn get_network(&self, ssid: &str) -> Option<&WiFiNetwork> {
        self.networks.get(ssid)
    }
    
    /// Connect to network
    pub fn connect(&mut self, ssid: &str, password: Option<&str>) -> Result<(), String> {
        if self.state == WiFiState::Disabled {
            return Err("WiFi is disabled".to_string());
        }
        
        // Check if network exists or is hidden
        if !self.networks.contains_key(ssid) && password.is_none() {
            return Err(format!("Network '{}' not found", ssid));
        }
        
        // Save credentials if provided
        if let Some(pwd) = password {
            self.saved_credentials.insert(ssid.to_string(), pwd.to_string());
        }
        
        // Check if we have credentials
        let network = self.networks.get(ssid);
        if let Some(net) = network {
            if net.security != WiFiSecurity::Open && !self.saved_credentials.contains_key(ssid) {
                return Err("Password required for this network".to_string());
            }
        }
        
        self.state = WiFiState::Connecting;
        self.connection_attempts += 1;
        
        // Simulate successful connection
        self.connected_network = Some(ssid.to_string());
        self.state = WiFiState::Connected;
        self.connection_attempts = 0;
        
        Ok(())
    }
    
    /// Disconnect from network
    pub fn disconnect(&mut self) {
        if self.connected_network.is_some() {
            self.state = WiFiState::Disconnecting;
            self.connected_network = None;
            self.state = WiFiState::Idle;
        }
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state == WiFiState::Connected && self.connected_network.is_some()
    }
    
    /// Get connected network
    pub fn connected_network(&self) -> Option<&WiFiNetwork> {
        self.connected_network.as_ref()
            .and_then(|ssid| self.networks.get(ssid))
    }
    
    /// Get connected SSID
    pub fn connected_ssid(&self) -> Option<&str> {
        self.connected_network.as_deref()
    }
    
    /// Get signal quality
    pub fn signal_quality(&self) -> NetworkQuality {
        self.connected_network()
            .map(|n| n.signal_quality())
            .unwrap_or(NetworkQuality::None)
    }
    
    /// Forget network
    pub fn forget_network(&mut self, ssid: &str) {
        self.saved_credentials.remove(ssid);
        
        if let Some(net) = self.networks.get_mut(ssid) {
            net.is_saved = false;
        }
        
        if self.connected_ssid() == Some(ssid) {
            self.disconnect();
        }
    }
    
    /// Get saved networks
    pub fn saved_networks(&self) -> Vec<&str> {
        self.saved_credentials.keys().map(|s| s.as_str()).collect()
    }
    
    /// Set preferred network
    pub fn set_preferred_network(&mut self, ssid: Option<&str>) {
        self.preferred_network = ssid.map(|s| s.to_string());
    }
    
    /// Get preferred network
    pub fn preferred_network(&self) -> Option<&str> {
        self.preferred_network.as_deref()
    }
    
    /// Enable/disable auto-reconnect
    pub fn set_auto_reconnect(&mut self, enabled: bool) {
        self.auto_reconnect = enabled;
    }
    
    /// Check if auto-reconnect is enabled
    pub fn is_auto_reconnect(&self) -> bool {
        self.auto_reconnect
    }
    
    /// Update WiFi state
    pub fn update(&mut self) {
        // Auto-scan if needed
        if self.is_enabled() && !self.is_connected() {
            if let Some(last) = self.last_scan {
                if last.elapsed() >= self.scan_interval {
                    self.scan();
                }
            } else {
                self.scan();
            }
        }
        
        // Auto-reconnect if needed
        if self.auto_reconnect && self.is_enabled() && !self.is_connected() {
            if let Some(preferred) = &self.preferred_network.clone() {
                if self.networks.contains_key(preferred) {
                    let _ = self.connect(preferred, None);
                }
            }
        }
    }
    
    /// Add test network (for testing)
    #[cfg(test)]
    pub fn add_test_network(&mut self, network: WiFiNetwork) {
        self.networks.insert(network.ssid.clone(), network);
    }
}

impl Default for WiFiManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wifi_manager_creation() {
        let manager = WiFiManager::new();
        assert_eq!(manager.state(), WiFiState::Disabled);
        assert!(!manager.is_connected());
    }
    
    #[test]
    fn test_enable_disable() {
        let mut manager = WiFiManager::new();
        
        manager.enable();
        assert!(manager.is_enabled());
        
        manager.disable();
        assert!(!manager.is_enabled());
    }
    
    #[test]
    fn test_connect_disconnect() {
        let mut manager = WiFiManager::new();
        manager.enable();
        
        // Add test network
        manager.add_test_network(WiFiNetwork {
            ssid: "TestNetwork".to_string(),
            bssid: "00:11:22:33:44:55".to_string(),
            signal_dbm: -55,
            security: WiFiSecurity::Open,
            frequency_mhz: 2437,
            channel: 6,
            is_hidden: false,
            is_saved: false,
            last_connected: None,
        });
        
        // Connect
        assert!(manager.connect("TestNetwork", None).is_ok());
        assert!(manager.is_connected());
        assert_eq!(manager.connected_ssid(), Some("TestNetwork"));
        
        // Disconnect
        manager.disconnect();
        assert!(!manager.is_connected());
    }
    
    #[test]
    fn test_signal_quality() {
        let network = WiFiNetwork {
            ssid: "Test".to_string(),
            bssid: "00:00:00:00:00:00".to_string(),
            signal_dbm: -55,
            security: WiFiSecurity::WPA,
            frequency_mhz: 2437,
            channel: 6,
            is_hidden: false,
            is_saved: false,
            last_connected: None,
        };
        
        assert_eq!(network.signal_quality(), NetworkQuality::Good);
    }
    
    #[test]
    fn test_saved_credentials() {
        let mut manager = WiFiManager::new();
        manager.enable();
        
        manager.add_test_network(WiFiNetwork {
            ssid: "SecureNetwork".to_string(),
            bssid: "00:11:22:33:44:55".to_string(),
            signal_dbm: -60,
            security: WiFiSecurity::WPA,
            frequency_mhz: 5180,
            channel: 36,
            is_hidden: false,
            is_saved: false,
            last_connected: None,
        });
        
        // Connect with password
        assert!(manager.connect("SecureNetwork", Some("password123")).is_ok());
        
        // Check saved
        assert!(manager.saved_networks().contains(&"SecureNetwork"));
        
        // Forget
        manager.forget_network("SecureNetwork");
        assert!(!manager.saved_networks().contains(&"SecureNetwork"));
    }
    
    #[test]
    fn test_frequency_bands() {
        let network_2_4 = WiFiNetwork {
            ssid: "2.4GHz".to_string(),
            bssid: "00:00:00:00:00:00".to_string(),
            signal_dbm: -60,
            security: WiFiSecurity::Open,
            frequency_mhz: 2437,
            channel: 6,
            is_hidden: false,
            is_saved: false,
            last_connected: None,
        };
        
        let network_5 = WiFiNetwork {
            ssid: "5GHz".to_string(),
            bssid: "00:00:00:00:00:00".to_string(),
            signal_dbm: -60,
            security: WiFiSecurity::Open,
            frequency_mhz: 5180,
            channel: 36,
            is_hidden: false,
            is_saved: false,
            last_connected: None,
        };
        
        assert!(network_2_4.is_2_4ghz());
        assert!(!network_2_4.is_5ghz());
        
        assert!(network_5.is_5ghz());
        assert!(!network_5.is_2_4ghz());
    }
}
