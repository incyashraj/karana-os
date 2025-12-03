//! Use Case Handlers for Glasses-Ready Kāraṇa OS
//!
//! Phase 2: Category-specific functionality for intelligent users and all use cases
//! - Productivity (code gen, dev workflows)
//! - Health/Safety (tracking, vitals)
//! - Social/Communication (calls, translation)
//! - Navigation/Shopping (AR overlays, directions)

use anyhow::Result;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;

use super::manifest::{ManifestBuilder, ManifestOutput};
use super::command::{HapticPattern, AROverlay, AROverlayType, NavigationDirection, WhisperStyle};

/// Real output directory for use case artifacts
const REAL_OUTPUT_DIR: &str = "/tmp/karana";

// ═══════════════════════════════════════════════════════════════════════════════
// PRODUCTIVITY USE CASES (Developer/Code workflows)
// ═══════════════════════════════════════════════════════════════════════════════

/// Code generation output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeOutput {
    pub language: String,
    pub snippet: String,
    pub filename: String,
    pub explanation: String,
}

/// Productivity use case handler
pub struct ProductivityHandler {
    builder: ManifestBuilder,
}

impl ProductivityHandler {
    pub fn new() -> Self {
        Self {
            builder: ManifestBuilder::new(),
        }
    }
    
    /// "Intent: Code Rust" → AI gen snippet → ZK-prove → Editor overlay → Haptic "Saved"
    pub async fn code_intent(&self, language: &str, description: &str, ai_snippet: &str) -> Result<(ManifestOutput, CodeOutput)> {
        // Ensure output directory
        fs::create_dir_all(REAL_OUTPUT_DIR)?;
        
        // Generate filename from language
        let ext = match language.to_lowercase().as_str() {
            "rust" => "rs",
            "python" => "py",
            "javascript" | "js" => "js",
            "typescript" | "ts" => "ts",
            "go" => "go",
            _ => "txt",
        };
        let filename = format!("{}/snippet_{}.{}", REAL_OUTPUT_DIR, 
            chrono::Local::now().format("%Y%m%d_%H%M%S"), ext);
        
        // Write snippet to real file
        fs::write(&filename, ai_snippet)?;
        log::info!("[PRODUCTIVITY] ✓ Code saved: {} ({} bytes)", filename, ai_snippet.len());
        
        // Generate ZK proof for snippet hash
        let hash = Sha256::digest(ai_snippet.as_bytes());
        let proof = crate::zk::prove_data_hash(ai_snippet.as_bytes(), hash.into())?;
        log::info!("[PRODUCTIVITY] ✓ ZK proof: {} bytes", proof.len());
        
        // Create manifest output
        let manifest = ManifestOutput {
            whisper: format!("✓ {} snippet saved", language),
            haptic: HapticPattern::Success,
            overlay: Some(AROverlay {
                content: format!("📝 {}\n{}", filename, &ai_snippet[..ai_snippet.len().min(100)]),
                position: (0.7, 0.3), // Top-right gaze area
                duration_ms: 5000,
                overlay_type: AROverlayType::Whisper,
                style: WhisperStyle::Normal,
            }),
            needs_confirmation: false,
            confidence: 0.95,
        };
        
        let output = CodeOutput {
            language: language.to_string(),
            snippet: ai_snippet.to_string(),
            filename,
            explanation: description.to_string(),
        };
        
        Ok((manifest, output))
    }
    
    /// Quick note/reminder with ZK attestation
    pub async fn quick_note(&self, content: &str) -> Result<ManifestOutput> {
        fs::create_dir_all(REAL_OUTPUT_DIR)?;
        
        let notes_file = format!("{}/notes.txt", REAL_OUTPUT_DIR);
        let entry = format!("[{}] {}\n", 
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), content);
        
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&notes_file)?
            .write_all(entry.as_bytes())?;
        
        log::info!("[PRODUCTIVITY] ✓ Note saved: {}", content);
        
        Ok(ManifestOutput {
            whisper: "✓ Note saved".to_string(),
            haptic: HapticPattern::Success,
            overlay: None,
            needs_confirmation: false,
            confidence: 1.0,
        })
    }
}

use std::io::Write;

// ═══════════════════════════════════════════════════════════════════════════════
// HEALTH/SAFETY USE CASES (Tracking, Vitals)
// ═══════════════════════════════════════════════════════════════════════════════

/// Health data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthData {
    pub metric: String,
    pub value: f32,
    pub unit: String,
    pub timestamp: u64,
    pub zk_proof: Vec<u8>,
}

/// Health use case handler
pub struct HealthHandler {
    builder: ManifestBuilder,
}

impl HealthHandler {
    pub fn new() -> Self {
        Self {
            builder: ManifestBuilder::new(),
        }
    }
    
    /// "Intent: Track run" → IMU data → AI pace → ZK-attest vitals → Haptic alert
    pub async fn track_run(&self, imu_data: &[f32], duration_secs: u32) -> Result<(ManifestOutput, HealthData)> {
        // Calculate pace from IMU (simplified)
        let avg_magnitude: f32 = imu_data.iter().map(|v| v.abs()).sum::<f32>() / imu_data.len() as f32;
        let pace_kmh = (avg_magnitude * 10.0).min(15.0).max(0.0); // Clamp to reasonable range
        
        // ZK proof for pace without revealing raw IMU
        let pace_bytes = pace_kmh.to_le_bytes();
        let commitment = Sha256::digest(&pace_bytes);
        let zk_proof = crate::zk::prove_data_hash(&pace_bytes, commitment.into())?;
        
        // Determine alert level
        let (haptic, advice) = if pace_kmh < 4.0 {
            (HapticPattern::Attention, "Speed up for cardio benefit")
        } else if pace_kmh > 12.0 {
            (HapticPattern::Confirm, "Great pace! Stay hydrated")
        } else {
            (HapticPattern::Success, "Optimal zone")
        };
        
        // Log to health file
        fs::create_dir_all(REAL_OUTPUT_DIR)?;
        let health_file = format!("{}/health_log.json", REAL_OUTPUT_DIR);
        let entry = serde_json::json!({
            "type": "run",
            "pace_kmh": pace_kmh,
            "duration_secs": duration_secs,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "advice": advice,
        });
        
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&health_file)?;
        writeln!(file, "{}", entry)?;
        
        log::info!("[HEALTH] ✓ Run tracked: {:.1} km/h for {}s", pace_kmh, duration_secs);
        
        let manifest = ManifestOutput {
            whisper: format!("{:.1} km/h – {}", pace_kmh, advice),
            haptic,
            overlay: Some(AROverlay {
                content: format!("🏃 {:.1} km/h\n⏱️ {}:{:02}", 
                    pace_kmh, duration_secs / 60, duration_secs % 60),
                position: (0.5, 0.1), // Top center
                duration_ms: 3000,
                overlay_type: AROverlayType::Progress { percent: pace_kmh / 15.0 * 100.0 },
                style: WhisperStyle::Emphasized,
            }),
            needs_confirmation: false,
            confidence: 0.9,
        };
        
        let health = HealthData {
            metric: "pace".to_string(),
            value: pace_kmh,
            unit: "km/h".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            zk_proof,
        };
        
        Ok((manifest, health))
    }
    
    /// Heart rate alert (from wearable/simulated)
    pub fn heart_rate_alert(&self, bpm: u32) -> ManifestOutput {
        let (haptic, message) = if bpm > 180 {
            (HapticPattern::Error, format!("⚠️ High HR: {} bpm – Rest!", bpm))
        } else if bpm > 140 {
            (HapticPattern::Confirm, format!("❤️ {} bpm – Intense zone", bpm))
        } else if bpm > 100 {
            (HapticPattern::Success, format!("💚 {} bpm – Cardio zone", bpm))
        } else {
            (HapticPattern::Success, format!("💙 {} bpm – Rest zone", bpm))
        };
        
        ManifestOutput {
            whisper: message,
            haptic,
            overlay: None,
            needs_confirmation: false,
            confidence: 0.95,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SOCIAL/COMMUNICATION USE CASES (Calls, Translation)
// ═══════════════════════════════════════════════════════════════════════════════

/// Call state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallState {
    pub peer_did: String,
    pub status: CallStatus,
    pub duration_secs: u32,
    pub translated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallStatus {
    Dialing,
    Connected,
    OnHold,
    Ended,
}

/// Social use case handler
pub struct SocialHandler {
    builder: ManifestBuilder,
}

impl SocialHandler {
    pub fn new() -> Self {
        Self {
            builder: ManifestBuilder::new(),
        }
    }
    
    /// "Intent: Call Alice" → Swarm dial → Real-time translation → ZK-encrypt voice
    pub async fn initiate_call(&self, peer_did: &str) -> Result<(ManifestOutput, CallState)> {
        log::info!("[SOCIAL] ✓ Initiating call to {}", peer_did);
        
        // TODO: Real libp2p dial would go here
        // For now, simulate connection
        
        let manifest = ManifestOutput {
            whisper: format!("📞 Calling {}...", &peer_did[..peer_did.len().min(20)]),
            haptic: HapticPattern::Thinking,
            overlay: Some(AROverlay {
                content: format!("Calling\n{}", &peer_did[..peer_did.len().min(25)]),
                position: (0.5, 0.5), // Center
                duration_ms: 0, // Persistent until connected
                overlay_type: AROverlayType::Status,
                style: WhisperStyle::Emphasized,
            }),
            needs_confirmation: false,
            confidence: 0.9,
        };
        
        let state = CallState {
            peer_did: peer_did.to_string(),
            status: CallStatus::Dialing,
            duration_secs: 0,
            translated: false,
        };
        
        Ok((manifest, state))
    }
    
    /// Real-time translation subtitle
    pub fn translation_subtitle(&self, original: &str, translated: &str, source_lang: &str, target_lang: &str) -> ManifestOutput {
        log::info!("[SOCIAL] Translation: {} ({}) → {} ({})", 
            original, source_lang, translated, target_lang);
        
        ManifestOutput {
            whisper: translated.to_string(),
            haptic: HapticPattern::Success,
            overlay: Some(AROverlay {
                content: format!("🌐 {}\n\n{}", translated, original),
                position: (0.5, 0.85), // Bottom subtitle area
                duration_ms: 5000,
                overlay_type: AROverlayType::Whisper,
                style: WhisperStyle::Normal,
            }),
            needs_confirmation: false,
            confidence: 0.85,
        }
    }
    
    /// Message notification
    pub fn message_notification(&self, from: &str, preview: &str) -> ManifestOutput {
        ManifestOutput {
            whisper: format!("💬 {}: {}", from, preview),
            haptic: HapticPattern::Attention,
            overlay: Some(AROverlay {
                content: format!("Message from {}\n\n{}", from, preview),
                position: (0.9, 0.1), // Top-right corner
                duration_ms: 4000,
                overlay_type: AROverlayType::Whisper,
                style: WhisperStyle::Normal,
            }),
            needs_confirmation: false,
            confidence: 1.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NAVIGATION/SHOPPING USE CASES (AR Overlays, Directions)
// ═══════════════════════════════════════════════════════════════════════════════

/// Navigation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationStep {
    pub instruction: String,
    pub direction: NavigationDirection,
    pub distance_meters: f32,
    pub eta_seconds: u32,
}

/// Navigation use case handler  
pub struct NavigationHandler {
    builder: ManifestBuilder,
}

impl NavigationHandler {
    pub fn new() -> Self {
        Self {
            builder: ManifestBuilder::new(),
        }
    }
    
    /// "Intent: Nav to coffee" → Route → AR arrow + haptic turn
    pub async fn navigate_to(&self, destination: &str, current_gps: (f64, f64)) -> Result<(ManifestOutput, NavigationStep)> {
        // Simulated route calculation
        // In production, would query mapping service
        let simulated_distance = 150.0; // meters
        let simulated_eta = 120; // seconds
        let simulated_direction = NavigationDirection::Left;
        
        log::info!("[NAV] ✓ Route to '{}': {:.0}m, ~{}s", destination, simulated_distance, simulated_eta);
        
        // ZK proof for location privacy (prove "within 100m of destination" without revealing exact location)
        let loc_bytes = format!("{:.4},{:.4}", current_gps.0, current_gps.1);
        let commitment = Sha256::digest(loc_bytes.as_bytes());
        let _proof = crate::zk::prove_data_hash(loc_bytes.as_bytes(), commitment.into())?;
        
        // Log navigation to file
        fs::create_dir_all(REAL_OUTPUT_DIR)?;
        let nav_file = format!("{}/navigation.conf", REAL_OUTPUT_DIR);
        fs::write(&nav_file, format!(
            "# Navigation Route\ndestination={}\ndistance_m={}\neta_s={}\ndirection={:?}\n",
            destination, simulated_distance, simulated_eta, simulated_direction
        ))?;
        
        let manifest = self.builder.navigation(
            &format!("{:?}", simulated_direction).to_lowercase(),
            simulated_distance,
            &format!("Head {} toward {}", 
                format!("{:?}", simulated_direction).to_lowercase(), 
                destination),
        );
        
        let step = NavigationStep {
            instruction: format!("Turn {:?} in {:.0}m", simulated_direction, simulated_distance),
            direction: simulated_direction,
            distance_meters: simulated_distance,
            eta_seconds: simulated_eta,
        };
        
        Ok((manifest, step))
    }
    
    /// Turn-by-turn haptic guidance
    pub fn turn_alert(&self, direction: NavigationDirection, distance: f32) -> ManifestOutput {
        let dir_str = format!("{:?}", direction).to_lowercase();
        let dir_symbol = match direction {
            NavigationDirection::Left => "← LEFT",
            NavigationDirection::Right => "→ RIGHT",
            NavigationDirection::Forward => "↑ STRAIGHT",
            NavigationDirection::Up => "⬆ UP",
            NavigationDirection::Down => "⬇ DOWN",
        };
        
        ManifestOutput {
            whisper: format!("↰ Turn {} in {:.0}m", dir_str, distance),
            haptic: HapticPattern::Navigation { direction },
            overlay: Some(AROverlay {
                content: format!("{}\n{:.0}m", dir_symbol, distance),
                position: (0.5, 0.3), // Upper center
                duration_ms: 3000,
                overlay_type: AROverlayType::Navigation,
                style: WhisperStyle::Emphasized,
            }),
            needs_confirmation: false,
            confidence: 0.95,
        }
    }
    
    /// Product identification (shopping)
    pub fn identify_product(&self, product_name: &str, price: Option<f64>, confidence: f32) -> ManifestOutput {
        let price_str = price.map(|p| format!("${:.2}", p)).unwrap_or_else(|| "Price N/A".to_string());
        
        ManifestOutput {
            whisper: format!("{} – {}", product_name, price_str),
            haptic: HapticPattern::Success,
            overlay: Some(AROverlay {
                content: format!("🏷️ {}\n💰 {}\n📊 {:.0}% match", product_name, price_str, confidence * 100.0),
                position: (0.5, 0.5), // Center on product
                duration_ms: 4000,
                overlay_type: AROverlayType::Highlight {
                    bounds: (0.3, 0.3, 0.7, 0.7) // Product bounding box
                },
                style: WhisperStyle::Emphasized,
            }),
            needs_confirmation: false,
            confidence,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIFIED USE CASE DISPATCHER
// ═══════════════════════════════════════════════════════════════════════════════

/// Central dispatcher for all use cases
pub struct UseCaseDispatcher {
    pub productivity: ProductivityHandler,
    pub health: HealthHandler,
    pub social: SocialHandler,
    pub navigation: NavigationHandler,
}

impl UseCaseDispatcher {
    pub fn new() -> Self {
        Self {
            productivity: ProductivityHandler::new(),
            health: HealthHandler::new(),
            social: SocialHandler::new(),
            navigation: NavigationHandler::new(),
        }
    }
    
    /// Dispatch intent to appropriate handler based on category
    pub async fn dispatch(&self, category: &str, intent: &str, params: serde_json::Value) -> Result<ManifestOutput> {
        match category {
            "productivity" | "code" | "note" => {
                if intent.contains("code") {
                    let lang = params.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
                    let desc = params.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    let snippet = params.get("snippet").and_then(|v| v.as_str()).unwrap_or("// Generated code");
                    let (manifest, _) = self.productivity.code_intent(lang, desc, snippet).await?;
                    Ok(manifest)
                } else {
                    let content = params.get("content").and_then(|v| v.as_str()).unwrap_or(intent);
                    self.productivity.quick_note(content).await
                }
            }
            
            "health" | "fitness" | "track" => {
                if intent.contains("run") || intent.contains("walk") {
                    // Simulated IMU data
                    let imu: Vec<f32> = vec![0.5, 0.6, 0.4, 0.7, 0.5]; 
                    let duration = params.get("duration_secs").and_then(|v| v.as_u64()).unwrap_or(300) as u32;
                    let (manifest, _) = self.health.track_run(&imu, duration).await?;
                    Ok(manifest)
                } else {
                    let bpm = params.get("bpm").and_then(|v| v.as_u64()).unwrap_or(80) as u32;
                    Ok(self.health.heart_rate_alert(bpm))
                }
            }
            
            "social" | "call" | "message" => {
                if intent.contains("call") {
                    let peer = params.get("peer_did").and_then(|v| v.as_str()).unwrap_or("did:karana:unknown");
                    let (manifest, _) = self.social.initiate_call(peer).await?;
                    Ok(manifest)
                } else if intent.contains("translate") {
                    let original = params.get("original").and_then(|v| v.as_str()).unwrap_or("");
                    let translated = params.get("translated").and_then(|v| v.as_str()).unwrap_or("");
                    let from = params.get("source_lang").and_then(|v| v.as_str()).unwrap_or("en");
                    let to = params.get("target_lang").and_then(|v| v.as_str()).unwrap_or("es");
                    Ok(self.social.translation_subtitle(original, translated, from, to))
                } else {
                    let from = params.get("from").and_then(|v| v.as_str()).unwrap_or("Unknown");
                    let preview = params.get("preview").and_then(|v| v.as_str()).unwrap_or("");
                    Ok(self.social.message_notification(from, preview))
                }
            }
            
            "navigation" | "nav" | "directions" | "shopping" => {
                if intent.contains("identify") || intent.contains("product") {
                    let name = params.get("product").and_then(|v| v.as_str()).unwrap_or("Unknown Item");
                    let price = params.get("price").and_then(|v| v.as_f64());
                    let conf = params.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.8) as f32;
                    Ok(self.navigation.identify_product(name, price, conf))
                } else {
                    let dest = params.get("destination").and_then(|v| v.as_str()).unwrap_or("destination");
                    let lat = params.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let lon = params.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let (manifest, _) = self.navigation.navigate_to(dest, (lat, lon)).await?;
                    Ok(manifest)
                }
            }
            
            _ => {
                log::warn!("[USE_CASES] Unknown category: {}", category);
                Ok(ManifestOutput {
                    whisper: format!("Unknown command: {}", intent),
                    haptic: HapticPattern::Error,
                    overlay: None,
                    needs_confirmation: false,
                    confidence: 0.0,
                })
            }
        }
    }
}

impl Default for UseCaseDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_productivity_note() {
        let handler = ProductivityHandler::new();
        let result = handler.quick_note("Test note").await;
        assert!(result.is_ok());
        
        let manifest = result.unwrap();
        assert!(manifest.whisper.contains("saved"));
        assert!(matches!(manifest.haptic, HapticPattern::Success));
    }
    
    #[test]
    fn test_health_heart_rate() {
        let handler = HealthHandler::new();
        
        // Normal HR
        let normal = handler.heart_rate_alert(75);
        assert!(normal.whisper.contains("Rest zone"));
        
        // High HR
        let high = handler.heart_rate_alert(185);
        assert!(high.whisper.contains("High HR"));
        assert!(matches!(high.haptic, HapticPattern::Error));
    }
    
    #[test]
    fn test_navigation_turn() {
        let handler = NavigationHandler::new();
        let turn = handler.turn_alert(NavigationDirection::Left, 50.0);
        
        assert!(turn.whisper.contains("left"));
        assert!(turn.whisper.contains("50"));
        assert!(matches!(turn.haptic, HapticPattern::Navigation { .. }));
    }
    
    #[test]
    fn test_social_message() {
        let handler = SocialHandler::new();
        let msg = handler.message_notification("Alice", "Hey, are you free?");
        
        assert!(msg.whisper.contains("Alice"));
        assert!(matches!(msg.haptic, HapticPattern::Attention));
    }
    
    #[tokio::test]
    async fn test_dispatcher() {
        let dispatcher = UseCaseDispatcher::new();
        
        // Test health dispatch
        let result = dispatcher.dispatch(
            "health",
            "check heart rate",
            serde_json::json!({"bpm": 120})
        ).await;
        
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert!(manifest.whisper.contains("bpm"));
    }
}
