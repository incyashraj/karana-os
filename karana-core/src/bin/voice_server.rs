// Kāraṇa OS - Voice AI Server
// Standalone WebSocket server for real-time voice AI communication

use karana_core::network::WsServer;
use karana_core::assistant::{create_default_registry, StateContext, TtsService};
use karana_core::ai::KaranaAI;
use karana_core::voice_pipeline::{VoicePipeline, VoiceConfig, VoiceToIntent};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("🚀 Kāraṇa Voice AI Server Starting...");
    log::info!("=====================================");

    // Create WebSocket server
    let ws_server = Arc::new(WsServer::new());
    log::info!("✓ WebSocket server initialized");

    // Create tool registry with default tools
    let tool_registry = Arc::new(create_default_registry());
    log::info!("✓ Tool registry created with {} tools", tool_registry.list_tools().len());
    log::info!("  Available tools: {:?}", tool_registry.list_tools());

    // Create state context for UI awareness
    let state_context = Arc::new(StateContext::new());
    log::info!("✓ State context initialized");

    // Create TTS service
    let tts_service = Arc::new(TtsService::new());
    if tts_service.is_available().await {
        log::info!("✓ TTS service initialized");
        let voices = tts_service.get_voices().await;
        log::info!("  Available voices: {}", voices.len());
    } else {
        log::warn!("⚠ TTS service unavailable (fallback mode)");
    }

    // Initialize AI (for voice transcription)
    log::info!("⏳ Loading Whisper model for transcription...");
    let ai = Arc::new(Mutex::new(KaranaAI::new()?));
    log::info!("✓ AI system initialized");

    // Create voice pipeline
    let voice_config = VoiceConfig {
        sample_rate: 16000,
        wake_word: "karana".to_string(),
        continuous_mode: true,
        max_duration_secs: 30,
    };
    let mut voice_pipeline = VoiceToIntent::new(ai.clone(), voice_config);
    log::info!("✓ Voice pipeline ready");

    // Server info
    log::info!("=====================================");
    log::info!("📡 WebSocket: ws://0.0.0.0:8080");
    log::info!("🎤 Voice: Whisper STT + VAD");
    log::info!("🔧 Tools: {} registered", tool_registry.list_tools().len());
    log::info!("🧠 Context: State tracking enabled");
    log::info!("🔊 TTS: {}", if tts_service.is_available().await { "Enabled" } else { "Mock mode" });
    log::info!("=====================================");
    log::info!("✨ Voice AI Server ready for connections!");
    log::info!("");

    // Spawn voice command handler
    let ws_clone = ws_server.clone();
    let tools_clone = tool_registry.clone();
    let context_clone = state_context.clone();
    let tts_clone = tts_service.clone();
    
    tokio::spawn(async move {
        log::info!("[HANDLER] Voice command handler started");
        // This will be implemented in voice_handler.rs
        // For now, just keep the task alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    // Start WebSocket server (blocks)
    log::info!("[SERVER] Starting WebSocket listener...");
    ws_server.start("0.0.0.0:8080").await?;

    Ok(())
}
