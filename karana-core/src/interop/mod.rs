// Kāraṇa OS - Phase 62: Interoperability
// Cross-device companion protocol, desktop bridge

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    Glasses,
    Phone,
    Tablet,
    Desktop,
    Watch,
}

/// Device info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_type: DeviceType,
    pub name: String,
    pub os_version: String,
    pub capabilities: Vec<String>,
}

/// Sync message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Sync clipboard
    Clipboard { content: String },
    
    /// Sync notification
    Notification { title: String, body: String },
    
    /// Sync file
    File { name: String, data: Vec<u8> },
    
    /// Sync state
    State { key: String, value: Vec<u8> },
    
    /// Call handoff
    Handoff { session_id: String, state: Vec<u8> },
}

/// Companion protocol
pub struct CompanionProtocol {
    devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    paired_devices: Arc<RwLock<HashMap<String, String>>>, // device_id -> pairing_code
}

impl CompanionProtocol {
    /// Create new companion protocol
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            paired_devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start pairing
    pub async fn start_pairing(&self, device_info: DeviceInfo) -> Result<String> {
        let pairing_code = format!("{:06}", rand::random::<u32>() % 1_000_000);
        
        self.devices.write().await.insert(device_info.device_id.clone(), device_info.clone());
        self.paired_devices.write().await.insert(device_info.device_id, pairing_code.clone());
        
        Ok(pairing_code)
    }
    
    /// Complete pairing
    pub async fn complete_pairing(&self, device_id: &str, code: &str) -> Result<()> {
        let paired = self.paired_devices.read().await;
        
        if let Some(expected_code) = paired.get(device_id) {
            if expected_code == code {
                return Ok(());
            }
        }
        
        Err(anyhow!("Invalid pairing code"))
    }
    
    /// Send sync message
    pub async fn send_sync(&self, from_device: &str, to_device: &str, message: SyncMessage) -> Result<()> {
        // Verify devices are paired
        let paired = self.paired_devices.read().await;
        if !paired.contains_key(from_device) || !paired.contains_key(to_device) {
            return Err(anyhow!("Devices not paired"));
        }
        
        // In real implementation, would send over network
        println!("Syncing {:?} from {} to {}", message, from_device, to_device);
        
        Ok(())
    }
    
    /// List paired devices
    pub async fn list_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read().await;
        let paired = self.paired_devices.read().await;
        
        devices.iter()
            .filter(|(id, _)| paired.contains_key(*id))
            .map(|(_, info)| info.clone())
            .collect()
    }
}

/// Desktop bridge for file sync and notifications
pub struct DesktopBridge {
    companion: Arc<CompanionProtocol>,
    desktop_device_id: Option<String>,
}

impl DesktopBridge {
    /// Create new desktop bridge
    pub fn new(companion: Arc<CompanionProtocol>) -> Self {
        Self {
            companion,
            desktop_device_id: None,
        }
    }
    
    /// Connect to desktop
    pub async fn connect(&mut self, device_id: String) -> Result<()> {
        // Verify desktop device exists
        let devices = self.companion.devices.read().await;
        let device = devices.get(&device_id)
            .ok_or_else(|| anyhow!("Device not found"))?;
        
        if device.device_type != DeviceType::Desktop {
            return Err(anyhow!("Device is not a desktop"));
        }
        
        self.desktop_device_id = Some(device_id);
        Ok(())
    }
    
    /// Send file to desktop
    pub async fn send_file(&self, name: String, data: Vec<u8>) -> Result<()> {
        let desktop_id = self.desktop_device_id.as_ref()
            .ok_or_else(|| anyhow!("Not connected to desktop"))?;
        
        self.companion.send_sync(
            "glasses",
            desktop_id,
            SyncMessage::File { name, data },
        ).await
    }
    
    /// Push notification to desktop
    pub async fn push_notification(&self, title: String, body: String) -> Result<()> {
        let desktop_id = self.desktop_device_id.as_ref()
            .ok_or_else(|| anyhow!("Not connected to desktop"))?;
        
        self.companion.send_sync(
            "glasses",
            desktop_id,
            SyncMessage::Notification { title, body },
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_device_pairing() {
        let protocol = CompanionProtocol::new();
        
        let device = DeviceInfo {
            device_id: "phone_1".to_string(),
            device_type: DeviceType::Phone,
            name: "My Phone".to_string(),
            os_version: "1.0".to_string(),
            capabilities: vec!["sync".to_string()],
        };
        
        let code = protocol.start_pairing(device).await.unwrap();
        assert_eq!(code.len(), 6);
        
        protocol.complete_pairing("phone_1", &code).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_sync_message() {
        let protocol = CompanionProtocol::new();
        
        // Pair two devices
        let device1 = DeviceInfo {
            device_id: "device_1".to_string(),
            device_type: DeviceType::Glasses,
            name: "Glasses".to_string(),
            os_version: "1.0".to_string(),
            capabilities: vec![],
        };
        
        let code1 = protocol.start_pairing(device1).await.unwrap();
        protocol.complete_pairing("device_1", &code1).await.unwrap();
        
        let device2 = DeviceInfo {
            device_id: "device_2".to_string(),
            device_type: DeviceType::Phone,
            name: "Phone".to_string(),
            os_version: "1.0".to_string(),
            capabilities: vec![],
        };
        
        let code2 = protocol.start_pairing(device2).await.unwrap();
        protocol.complete_pairing("device_2", &code2).await.unwrap();
        
        // Send sync message
        protocol.send_sync(
            "device_1",
            "device_2",
            SyncMessage::Clipboard { content: "Hello".to_string() },
        ).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_desktop_bridge() {
        let protocol = Arc::new(CompanionProtocol::new());
        let mut bridge = DesktopBridge::new(protocol.clone());
        
        let desktop = DeviceInfo {
            device_id: "desktop_1".to_string(),
            device_type: DeviceType::Desktop,
            name: "My Desktop".to_string(),
            os_version: "1.0".to_string(),
            capabilities: vec!["file_sync".to_string()],
        };
        
        let code = protocol.start_pairing(desktop).await.unwrap();
        protocol.complete_pairing("desktop_1", &code).await.unwrap();
        
        bridge.connect("desktop_1".to_string()).await.unwrap();
        bridge.send_file("test.txt".to_string(), vec![1, 2, 3]).await.unwrap();
    }
}
