// Phase 51: Native App Support - YouTube, WhatsApp, Social Media
// Pre-configured support for popular mainstream apps

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::app_ecosystem::android_container::{AndroidContainer, AndroidApp};
use crate::app_ecosystem::intent_protocol::{IntentRouter, IntentType, NetworkAction, AIAction};
use std::path::PathBuf;

/// Popular app categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AppCategory {
    Social,
    Messaging,
    Video,
    Music,
    Productivity,
    Finance,
    Health,
    Gaming,
}

/// Native app descriptor with optimization hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeAppDescriptor {
    pub id: String,
    pub name: String,
    pub package_name: String,
    pub category: AppCategory,
    pub min_api_level: u32,
    pub permissions: Vec<String>,
    pub optimizations: AppOptimizations,
    pub karana_integrations: Vec<KaranaIntegration>,
}

/// App-specific optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppOptimizations {
    /// Enable video hardware acceleration
    pub hw_video_decode: bool,
    /// Enable audio optimization
    pub hw_audio_process: bool,
    /// Use edge AI for features
    pub edge_ai: bool,
    /// Cache strategy
    pub cache_strategy: CacheStrategy,
    /// Network optimization
    pub network_mode: NetworkMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CacheStrategy {
    Aggressive,  // Cache everything possible
    Moderate,    // Cache common data
    Minimal,     // Cache only essentials
    None,        // No caching
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkMode {
    Direct,      // Direct internet access
    P2P,         // P2P network preferred
    Hybrid,      // Mix of direct and P2P
    Optimized,   // Kāraṇa network optimization
}

/// Kāraṇa OS integrations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KaranaIntegration {
    /// Use Kāraṇa AI for content moderation
    ContentModeration,
    /// Use Kāraṇa Oracle for data
    OracleData,
    /// Use Kāraṇa Ledger for payments
    BlockchainPayments,
    /// Use Kāraṇa P2P for data transfer
    P2PTransfer,
    /// Use local AI for recommendations
    LocalRecommendations,
    /// Use privacy layer for data protection
    PrivacyProtection,
}

/// Native app registry with pre-configured popular apps
pub struct NativeAppRegistry {
    apps: HashMap<String, NativeAppDescriptor>,
}

impl NativeAppRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            apps: HashMap::new(),
        };
        
        // Pre-register popular apps
        registry.register_youtube();
        registry.register_whatsapp();
        registry.register_instagram();
        registry.register_tiktok();
        registry.register_twitter();
        registry.register_facebook();
        registry.register_telegram();
        registry.register_spotify();
        
        registry
    }
    
    /// YouTube with Kāraṇa optimizations
    fn register_youtube(&mut self) {
        let descriptor = NativeAppDescriptor {
            id: "youtube".to_string(),
            name: "YouTube".to_string(),
            package_name: "com.google.android.youtube".to_string(),
            category: AppCategory::Video,
            min_api_level: 23,
            permissions: vec![
                "INTERNET".to_string(),
                "ACCESS_NETWORK_STATE".to_string(),
                "WAKE_LOCK".to_string(),
            ],
            optimizations: AppOptimizations {
                hw_video_decode: true,
                hw_audio_process: true,
                edge_ai: true,  // Local recommendations
                cache_strategy: CacheStrategy::Aggressive,
                network_mode: NetworkMode::Hybrid,  // P2P for popular videos
            },
            karana_integrations: vec![
                KaranaIntegration::ContentModeration,
                KaranaIntegration::P2PTransfer,
                KaranaIntegration::LocalRecommendations,
            ],
        };
        
        self.apps.insert(descriptor.id.clone(), descriptor);
    }
    
    /// WhatsApp with E2E encryption + Kāraṇa P2P
    fn register_whatsapp(&mut self) {
        let descriptor = NativeAppDescriptor {
            id: "whatsapp".to_string(),
            name: "WhatsApp".to_string(),
            package_name: "com.whatsapp".to_string(),
            category: AppCategory::Messaging,
            min_api_level: 19,
            permissions: vec![
                "INTERNET".to_string(),
                "CAMERA".to_string(),
                "RECORD_AUDIO".to_string(),
                "READ_CONTACTS".to_string(),
            ],
            optimizations: AppOptimizations {
                hw_video_decode: true,
                hw_audio_process: true,
                edge_ai: false,
                cache_strategy: CacheStrategy::Moderate,
                network_mode: NetworkMode::P2P,  // Use Kāraṇa P2P
            },
            karana_integrations: vec![
                KaranaIntegration::P2PTransfer,
                KaranaIntegration::PrivacyProtection,
            ],
        };
        
        self.apps.insert(descriptor.id.clone(), descriptor);
    }
    
    /// Instagram with AI features
    fn register_instagram(&mut self) {
        let descriptor = NativeAppDescriptor {
            id: "instagram".to_string(),
            name: "Instagram".to_string(),
            package_name: "com.instagram.android".to_string(),
            category: AppCategory::Social,
            min_api_level: 21,
            permissions: vec![
                "INTERNET".to_string(),
                "CAMERA".to_string(),
                "ACCESS_FINE_LOCATION".to_string(),
            ],
            optimizations: AppOptimizations {
                hw_video_decode: true,
                hw_audio_process: false,
                edge_ai: true,  // Local filters and effects
                cache_strategy: CacheStrategy::Aggressive,
                network_mode: NetworkMode::Hybrid,
            },
            karana_integrations: vec![
                KaranaIntegration::LocalRecommendations,
                KaranaIntegration::ContentModeration,
                KaranaIntegration::P2PTransfer,
            ],
        };
        
        self.apps.insert(descriptor.id.clone(), descriptor);
    }
    
    /// TikTok with heavy AI optimization
    fn register_tiktok(&mut self) {
        let descriptor = NativeAppDescriptor {
            id: "tiktok".to_string(),
            name: "TikTok".to_string(),
            package_name: "com.zhiliaoapp.musically".to_string(),
            category: AppCategory::Video,
            min_api_level: 21,
            permissions: vec![
                "INTERNET".to_string(),
                "CAMERA".to_string(),
                "RECORD_AUDIO".to_string(),
            ],
            optimizations: AppOptimizations {
                hw_video_decode: true,
                hw_audio_process: true,
                edge_ai: true,  // Local video effects
                cache_strategy: CacheStrategy::Aggressive,
                network_mode: NetworkMode::Optimized,
            },
            karana_integrations: vec![
                KaranaIntegration::LocalRecommendations,
                KaranaIntegration::ContentModeration,
            ],
        };
        
        self.apps.insert(descriptor.id.clone(), descriptor);
    }
    
    /// Twitter/X
    fn register_twitter(&mut self) {
        let descriptor = NativeAppDescriptor {
            id: "twitter".to_string(),
            name: "X (Twitter)".to_string(),
            package_name: "com.twitter.android".to_string(),
            category: AppCategory::Social,
            min_api_level: 21,
            permissions: vec![
                "INTERNET".to_string(),
                "ACCESS_FINE_LOCATION".to_string(),
            ],
            optimizations: AppOptimizations {
                hw_video_decode: true,
                hw_audio_process: false,
                edge_ai: true,
                cache_strategy: CacheStrategy::Moderate,
                network_mode: NetworkMode::Direct,
            },
            karana_integrations: vec![
                KaranaIntegration::ContentModeration,
                KaranaIntegration::BlockchainPayments,  // For tips
            ],
        };
        
        self.apps.insert(descriptor.id.clone(), descriptor);
    }
    
    /// Facebook
    fn register_facebook(&mut self) {
        let descriptor = NativeAppDescriptor {
            id: "facebook".to_string(),
            name: "Facebook".to_string(),
            package_name: "com.facebook.katana".to_string(),
            category: AppCategory::Social,
            min_api_level: 21,
            permissions: vec![
                "INTERNET".to_string(),
                "CAMERA".to_string(),
                "ACCESS_FINE_LOCATION".to_string(),
            ],
            optimizations: AppOptimizations {
                hw_video_decode: true,
                hw_audio_process: false,
                edge_ai: false,
                cache_strategy: CacheStrategy::Moderate,
                network_mode: NetworkMode::Direct,
            },
            karana_integrations: vec![
                KaranaIntegration::PrivacyProtection,
            ],
        };
        
        self.apps.insert(descriptor.id.clone(), descriptor);
    }
    
    /// Telegram with P2P
    fn register_telegram(&mut self) {
        let descriptor = NativeAppDescriptor {
            id: "telegram".to_string(),
            name: "Telegram".to_string(),
            package_name: "org.telegram.messenger".to_string(),
            category: AppCategory::Messaging,
            min_api_level: 19,
            permissions: vec![
                "INTERNET".to_string(),
                "CAMERA".to_string(),
                "READ_CONTACTS".to_string(),
            ],
            optimizations: AppOptimizations {
                hw_video_decode: true,
                hw_audio_process: true,
                edge_ai: false,
                cache_strategy: CacheStrategy::Aggressive,
                network_mode: NetworkMode::P2P,
            },
            karana_integrations: vec![
                KaranaIntegration::P2PTransfer,
                KaranaIntegration::BlockchainPayments,
            ],
        };
        
        self.apps.insert(descriptor.id.clone(), descriptor);
    }
    
    /// Spotify
    fn register_spotify(&mut self) {
        let descriptor = NativeAppDescriptor {
            id: "spotify".to_string(),
            name: "Spotify".to_string(),
            package_name: "com.spotify.music".to_string(),
            category: AppCategory::Music,
            min_api_level: 21,
            permissions: vec![
                "INTERNET".to_string(),
                "ACCESS_NETWORK_STATE".to_string(),
            ],
            optimizations: AppOptimizations {
                hw_video_decode: false,
                hw_audio_process: true,
                edge_ai: true,  // Local recommendations
                cache_strategy: CacheStrategy::Aggressive,
                network_mode: NetworkMode::Hybrid,
            },
            karana_integrations: vec![
                KaranaIntegration::LocalRecommendations,
                KaranaIntegration::P2PTransfer,
            ],
        };
        
        self.apps.insert(descriptor.id.clone(), descriptor);
    }
    
    /// Get app descriptor
    pub fn get_app(&self, app_id: &str) -> Option<&NativeAppDescriptor> {
        self.apps.get(app_id)
    }
    
    /// Get all apps
    pub fn get_all_apps(&self) -> Vec<&NativeAppDescriptor> {
        self.apps.values().collect()
    }
    
    /// Get apps by category
    pub fn get_apps_by_category(&self, category: &AppCategory) -> Vec<&NativeAppDescriptor> {
        self.apps.values()
            .filter(|app| &app.category == category)
            .collect()
    }
    
    /// Check if app is supported
    pub fn is_supported(&self, package_name: &str) -> bool {
        self.apps.values().any(|app| app.package_name == package_name)
    }
}

/// App launcher with Kāraṇa integration
pub struct NativeAppLauncher {
    registry: NativeAppRegistry,
}

impl NativeAppLauncher {
    pub fn new() -> Self {
        Self {
            registry: NativeAppRegistry::new(),
        }
    }
    
    /// Launch app with optimizations
    pub async fn launch_app(
        &self,
        app_id: &str,
        container: &AndroidContainer,
        intent_router: &IntentRouter,
    ) -> Result<(), String> {
        let descriptor = self.registry.get_app(app_id)
            .ok_or_else(|| format!("Unknown app: {}", app_id))?;
        
        // Apply optimizations before launch
        self.apply_optimizations(descriptor, intent_router).await?;
        
        // Launch via container
        container.launch_app(&descriptor.package_name).await?;
        
        Ok(())
    }
    
    /// Apply Kāraṇa optimizations
    async fn apply_optimizations(
        &self,
        descriptor: &NativeAppDescriptor,
        _intent_router: &IntentRouter,
    ) -> Result<(), String> {
        // Enable hardware acceleration if needed
        if descriptor.optimizations.hw_video_decode {
            // Configure hardware video decode
        }
        
        // Set up P2P networking if needed
        if descriptor.optimizations.network_mode == NetworkMode::P2P {
            // Configure P2P network
        }
        
        // Enable edge AI if needed
        if descriptor.optimizations.edge_ai {
            // Load AI models for app
        }
        
        Ok(())
    }
    
    /// Get recommended apps for user
    pub fn get_recommendations(&self, user_interests: &[AppCategory]) -> Vec<&NativeAppDescriptor> {
        let mut apps: Vec<_> = self.registry.get_all_apps();
        
        // Sort by relevance to user interests
        apps.sort_by_key(|app| {
            if user_interests.contains(&app.category) {
                0
            } else {
                1
            }
        });
        
        apps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = NativeAppRegistry::new();
        
        // Check popular apps are registered
        assert!(registry.get_app("youtube").is_some());
        assert!(registry.get_app("whatsapp").is_some());
        assert!(registry.get_app("instagram").is_some());
        assert!(registry.get_app("tiktok").is_some());
        assert!(registry.get_app("twitter").is_some());
        assert!(registry.get_app("telegram").is_some());
        assert!(registry.get_app("spotify").is_some());
    }
    
    #[test]
    fn test_youtube_optimizations() {
        let registry = NativeAppRegistry::new();
        let youtube = registry.get_app("youtube").unwrap();
        
        assert_eq!(youtube.name, "YouTube");
        assert!(youtube.optimizations.hw_video_decode);
        assert!(youtube.optimizations.edge_ai);
        assert_eq!(youtube.optimizations.network_mode, NetworkMode::Hybrid);
        assert!(youtube.karana_integrations.contains(&KaranaIntegration::P2PTransfer));
    }
    
    #[test]
    fn test_whatsapp_privacy() {
        let registry = NativeAppRegistry::new();
        let whatsapp = registry.get_app("whatsapp").unwrap();
        
        assert_eq!(whatsapp.category, AppCategory::Messaging);
        assert_eq!(whatsapp.optimizations.network_mode, NetworkMode::P2P);
        assert!(whatsapp.karana_integrations.contains(&KaranaIntegration::PrivacyProtection));
    }
    
    #[test]
    fn test_category_filtering() {
        let registry = NativeAppRegistry::new();
        let social_apps = registry.get_apps_by_category(&AppCategory::Social);
        
        assert!(social_apps.len() >= 3);  // Instagram, Twitter, Facebook
        assert!(social_apps.iter().any(|app| app.id == "instagram"));
        assert!(social_apps.iter().any(|app| app.id == "twitter"));
        assert!(social_apps.iter().any(|app| app.id == "facebook"));
    }
    
    #[test]
    fn test_package_name_support() {
        let registry = NativeAppRegistry::new();
        
        assert!(registry.is_supported("com.google.android.youtube"));
        assert!(registry.is_supported("com.whatsapp"));
        assert!(!registry.is_supported("com.unknown.app"));
    }
    
    #[test]
    fn test_recommendations() {
        let launcher = NativeAppLauncher::new();
        let interests = vec![AppCategory::Video, AppCategory::Messaging];
        
        let recs = launcher.get_recommendations(&interests);
        
        // Video and messaging apps should be first
        let first_categories: Vec<_> = recs.iter()
            .take(4)
            .map(|app| app.category.clone())
            .collect();
        
        assert!(first_categories.contains(&AppCategory::Video));
        assert!(first_categories.contains(&AppCategory::Messaging));
    }
    
    #[test]
    fn test_all_apps_have_optimizations() {
        let registry = NativeAppRegistry::new();
        
        for app in registry.get_all_apps() {
            // All apps should have defined optimizations
            assert!(!app.permissions.is_empty());
            
            // Video apps should have hw decode
            if app.category == AppCategory::Video {
                assert!(app.optimizations.hw_video_decode);
            }
        }
    }
}
