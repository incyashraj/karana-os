//! OracleVeil Integration Tests
//!
//! Tests the core Oracle → ZK-Sign → Command → Response flow

use karana_core::oracle::manifest::{ManifestBuilder, MinimalManifest, OutputMode};
use karana_core::oracle::command::{
    HapticPattern, AROverlayType, WhisperStyle,
    OracleCommand, CommandResult, CommandData,
};

#[test]
fn test_manifest_builder_balance() {
    let builder = ManifestBuilder::new();
    let manifest = builder.balance(500_000_000, 1_000_000_000); // 500 / 1000 KARA
    
    assert!(manifest.whisper.contains("500"));
    assert!(matches!(manifest.haptic, HapticPattern::Success));
    assert!(manifest.overlay.is_some());
    
    if let Some(overlay) = &manifest.overlay {
        assert!(matches!(overlay.overlay_type, AROverlayType::Progress { .. }));
    }
}

#[test]
fn test_manifest_builder_transfer_success() {
    let builder = ManifestBuilder::new();
    let manifest = builder.transfer(100_000_000, "did:karana:alice123", true);
    
    assert!(manifest.whisper.contains("✓"));
    assert!(manifest.whisper.contains("did:kara"));
    assert!(matches!(manifest.haptic, HapticPattern::Success));
    assert!(!manifest.needs_confirmation);
}

#[test]
fn test_manifest_builder_transfer_failure() {
    let builder = ManifestBuilder::new();
    let manifest = builder.transfer(100_000_000, "did:karana:bob", false);
    
    assert!(manifest.whisper.contains("✗"));
    assert!(matches!(manifest.haptic, HapticPattern::Error));
}

#[test]
fn test_manifest_builder_timer() {
    let builder = ManifestBuilder::new();
    
    // Normal timer
    let manifest = builder.timer(120, "Meeting");
    assert!(manifest.whisper.contains("⏱️"));
    assert!(manifest.whisper.contains("Meeting"));
    assert!(matches!(manifest.haptic, HapticPattern::Success));
    
    // Urgent timer (< 10 seconds)
    let manifest = builder.timer(5, "Urgent");
    assert!(matches!(manifest.haptic, HapticPattern::Confirm));
    
    // Expired timer
    let manifest = builder.timer(0, "Done");
    assert!(matches!(manifest.haptic, HapticPattern::Attention));
}

#[test]
fn test_manifest_builder_navigation() {
    let builder = ManifestBuilder::new();
    let manifest = builder.navigation("left", 50.0, "Turn left at intersection");
    
    assert!(manifest.whisper.contains("←"));
    assert!(manifest.whisper.contains("50"));
    assert!(matches!(manifest.haptic, HapticPattern::Navigation { .. }));
}

#[test]
fn test_manifest_builder_identify() {
    let builder = ManifestBuilder::new();
    let manifest = builder.identify("Coffee Mug", 0.95);
    
    assert_eq!(manifest.whisper, "Coffee Mug");
    assert!(manifest.overlay.is_some());
    
    if let Some(overlay) = &manifest.overlay {
        assert!(overlay.content.contains("95%"));
        assert!(matches!(overlay.overlay_type, AROverlayType::Highlight { .. }));
    }
}

#[test]
fn test_manifest_builder_confirmation() {
    let builder = ManifestBuilder::new();
    let manifest = builder.confirmation("Send 100 KARA", "To: alice, Amount: 100");
    
    assert!(manifest.whisper.contains("Confirm"));
    assert!(manifest.needs_confirmation);
    assert!(matches!(manifest.haptic, HapticPattern::Confirm));
}

#[test]
fn test_manifest_builder_thinking() {
    let builder = ManifestBuilder::new();
    let manifest = builder.thinking("Processing your request...");
    
    assert!(manifest.whisper.contains("💭"));
    assert!(matches!(manifest.haptic, HapticPattern::Thinking));
    assert_eq!(manifest.confidence, 0.5);
}

#[test]
fn test_minimal_manifest_output_modes() {
    let mut manifest = MinimalManifest::new();
    
    manifest.set_mode(OutputMode::Full);
    // Would render overlay + haptic + audio
    
    manifest.set_mode(OutputMode::Minimal);
    // Would render only text whisper + haptic
    
    manifest.set_mode(OutputMode::HapticOnly);
    // Would render only haptic (stealth mode)
    
    manifest.set_mode(OutputMode::Silent);
    // Would only log, no output
}

#[tokio::test]
async fn test_minimal_manifest_whisper() {
    let mut manifest = MinimalManifest::new();
    
    manifest.show_whisper("Test message", WhisperStyle::Normal).unwrap();
    
    let overlays = manifest.get_overlays();
    assert_eq!(overlays.len(), 1);
    assert!(overlays[0].content.contains("Test message"));
}

#[test]
fn test_command_result_success() {
    let result = CommandResult::success("cmd-123", CommandData::Balance(1000));
    
    assert!(result.is_success());
    assert_eq!(result.command_id(), "cmd-123");
}

#[test]
fn test_command_result_failure() {
    let result = CommandResult::failure("cmd-456", "Insufficient balance", true);
    
    assert!(!result.is_success());
    assert_eq!(result.command_id(), "cmd-456");
}

#[test]
fn test_oracle_command_variants() {
    // Test that all command variants can be created
    let _cmd1 = OracleCommand::QueryBalance { did: "alice".to_string() };
    let _cmd2 = OracleCommand::PlayHaptic { pattern: HapticPattern::Success };
    let _cmd3 = OracleCommand::GetPipelineStatus;
    let _cmd4 = OracleCommand::StoreData { 
        data: vec![1, 2, 3],
        metadata: "test".to_string(),
        zk_proof: vec![],
    };
    
    // Verify they're properly typed
    assert!(matches!(_cmd1, OracleCommand::QueryBalance { .. }));
    assert!(matches!(_cmd2, OracleCommand::PlayHaptic { .. }));
    assert!(matches!(_cmd3, OracleCommand::GetPipelineStatus));
}

#[test]
fn test_haptic_patterns() {
    // Test pattern variants
    let patterns = vec![
        HapticPattern::Success,
        HapticPattern::Confirm,
        HapticPattern::Error,
        HapticPattern::Attention,
        HapticPattern::Thinking,
        HapticPattern::Navigation { 
            direction: karana_core::oracle::command::NavigationDirection::Left 
        },
    ];
    
    for pattern in patterns {
        // Just verify they can be created and cloned
        let _cloned = pattern.clone();
    }
}
