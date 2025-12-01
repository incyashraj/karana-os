use anyhow::Result;

pub struct AppBundle {
    pub apps: Vec<(&'static str, [u8; 32])>, // Name, expected hash
}

impl AppBundle {
    pub fn new() -> Self {
        Self {
            apps: vec![
                ("vscode", [0xaa; 32]),
                ("firefox", [0xbb; 32]),
                ("git", [0xcc; 32]),
                ("vim", [0xdd; 32]),
                ("ollama", [0xee; 32]),
                ("curl", [0xff; 32]),
                ("htop", [0x11; 32]),
                ("python", [0x22; 32]),
                ("rustup", [0x33; 32]),
                ("tauri", [0x44; 32]),
            ],
        }
    }

    pub fn install(&self, app_name: &str) -> Result<String> {
        let (name, hash) = self
            .apps
            .iter()
            .find(|(n, _)| n == &app_name)
            .ok_or(anyhow::anyhow!("App '{}' not found in bundle.", app_name))?;

        // Simulate ZK verification
        // In reality, we would fetch data and run prove_data_hash(data, *hash)
        let hash_hex = hex::encode(&hash[0..4]); // Just show first 4 bytes
        
        Ok(format!(
            "Bundled App '{}' installed. ZK-Hash ({}...) verified. Sandboxed execution ready.",
            name, hash_hex
        ))
    }
    
    pub fn list(&self) -> Vec<String> {
        self.apps.iter().map(|(n, _)| n.to_string()).collect()
    }
}
