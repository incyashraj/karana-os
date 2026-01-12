//! API Route handlers

use std::sync::Arc;
use axum::{
    extract::{State, ws::{WebSocket, Message}},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use base64::Engine;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::wallet::{KaranaWallet, get_device_id, SignedTransaction};
use crate::api::state::AppState;
use crate::api::types::*;

// ============================================================================
// Wallet Handlers
// ============================================================================

/// Create a new wallet with fresh mnemonic
pub async fn create_wallet(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let device_id = get_device_id();
    
    match KaranaWallet::generate(&device_id) {
        Ok(result) => {
            let response = WalletCreationResponse {
                did: result.wallet.did().to_string(),
                public_key: result.wallet.public_key_hex(),
                recovery_phrase: result.recovery_phrase.words().to_vec(),
            };
            
            // Store wallet in state
            *state.wallet.write().await = Some(result.wallet);
            
            log::info!("[API] 🔐 New wallet created: {}", response.did);
            
            (StatusCode::CREATED, Json(ApiResponse::success(response)))
        }
        Err(e) => {
            log::error!("[API] Wallet creation failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<WalletCreationResponse>::error(e.to_string())))
        }
    }
}

/// Restore wallet from mnemonic
pub async fn restore_wallet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RestoreWalletRequest>,
) -> impl IntoResponse {
    let device_id = get_device_id();
    
    match KaranaWallet::from_mnemonic(&req.mnemonic, &device_id) {
        Ok(wallet) => {
            let info = WalletInfo {
                did: wallet.did().to_string(),
                public_key: wallet.public_key_hex(),
                balance: *state.balance.read().await,
                device_id: wallet.device_id().to_string(),
            };
            
            *state.wallet.write().await = Some(wallet);
            
            log::info!("[API] 🔓 Wallet restored: {}", info.did);
            
            (StatusCode::OK, Json(ApiResponse::success(info)))
        }
        Err(e) => {
            log::error!("[API] Wallet restore failed: {}", e);
            (StatusCode::BAD_REQUEST, Json(ApiResponse::<WalletInfo>::error(e.to_string())))
        }
    }
}

/// Get wallet info
pub async fn get_wallet_info(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let wallet = state.wallet.read().await;
    
    match wallet.as_ref() {
        Some(w) => {
            let info = WalletInfo {
                did: w.did().to_string(),
                public_key: w.public_key_hex(),
                balance: *state.balance.read().await,
                device_id: w.device_id().to_string(),
            };
            (StatusCode::OK, Json(ApiResponse::success(info)))
        }
        None => {
            // Return 200 with error message - no wallet is an expected state
            (StatusCode::OK, Json(ApiResponse::<WalletInfo>::error("No wallet found. Create or restore one first.")))
        }
    }
}

/// Sign a transaction
pub async fn sign_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignTransactionRequest>,
) -> impl IntoResponse {
    // Check balance first
    let balance = *state.balance.read().await;
    if req.amount > balance {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::<SignedTransactionResponse>::error(
            format!("Insufficient balance. Have: {}, Need: {}", balance, req.amount)
        )));
    }
    
    // Get nonce
    let nonce = state.next_nonce().await;
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Build and sign transaction while holding wallet lock
    let response = {
        let wallet_guard = state.wallet.read().await;
        
        let Some(wallet) = wallet_guard.as_ref() else {
            return (StatusCode::UNAUTHORIZED, Json(ApiResponse::<SignedTransactionResponse>::error("No wallet. Create or restore one first.")));
        };
        
        // Build transaction data
        let tx_data = serde_json::json!({
            "action": req.action,
            "recipient": req.recipient,
            "amount": req.amount,
            "memo": req.memo,
        });
        let tx_bytes = serde_json::to_vec(&tx_data).unwrap();
        
        // Create signed transaction
        let signed_tx = SignedTransaction::new(wallet, &tx_bytes, nonce);
        
        SignedTransactionResponse {
            tx_hash: signed_tx.hash_hex(),
            signature: signed_tx.signature.clone(),
            sender: signed_tx.sender.clone(),
            recipient: req.recipient.clone(),
            amount: req.amount,
            timestamp,
            nonce,
        }
    }; // wallet guard dropped here
    
    // Debit balance
    state.debit(req.amount).await;
    
    // Add to transaction history
    let tx = Transaction {
        id: response.tx_hash.clone(),
        tx_type: req.action.clone(),
        amount: req.amount,
        recipient: req.recipient,
        sender: response.sender.clone(),
        timestamp,
        status: "CONFIRMED".to_string(),
        signature: Some(response.signature.clone()),
        da_tx_hash: None, // Will be set when submitted to Celestia
    };
    state.add_transaction(tx).await;
    
    // Broadcast to WebSocket subscribers
    let ws_msg = serde_json::to_string(&WsMessage::TransactionConfirmed {
        tx_hash: response.tx_hash.clone(),
        status: "CONFIRMED".to_string(),
    }).unwrap();
    state.broadcast("transactions", &ws_msg).await;
    
    log::info!("[API] ✍️ Transaction signed: {} -> {} ({} KARA)", 
        response.sender, response.recipient, response.amount);
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

/// Get transaction history
pub async fn get_transactions(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let transactions = state.transactions.read().await;
    (StatusCode::OK, Json(ApiResponse::success(transactions.clone())))
}

// ============================================================================
// AI Vision Handler
// ============================================================================

/// Analyze an image using AI vision
pub async fn analyze_vision(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VisionAnalysisRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    
    // Set mode to analyzing
    state.set_mode(OsMode::Analyzing).await;
    
    // In a full implementation, this would use the candle-based BLIP model
    // For now, we return mock data with simulated analysis
    
    // Decode base64 to verify it's valid image data
    use base64::engine::general_purpose::STANDARD;
    let clean_b64 = req.image_base64.trim_start_matches("data:image/jpeg;base64,")
            .trim_start_matches("data:image/png;base64,");
    let _image_bytes = match STANDARD.decode(clean_b64) {
        Ok(bytes) => bytes,
        Err(_) => {
            state.set_mode(OsMode::Idle).await;
            return (StatusCode::BAD_REQUEST, Json(ApiResponse::<VisionAnalysisResponse>::error("Invalid base64 image")));
        }
    };
    
    // TODO: Integrate real BLIP model from ai/blip.rs
    // For now, return simulated analysis
    let response = VisionAnalysisResponse {
        detected_object: "Object Detected".to_string(),
        category: "General".to_string(),
        description: "AI Vision analysis via Kāraṇa OS local model".to_string(),
        confidence: 85.5,
        related_tags: vec!["detected".to_string(), "analyzed".to_string(), "karana".to_string()],
        processing_time_ms: start.elapsed().as_millis() as u64,
    };
    
    state.set_mode(OsMode::Idle).await;
    
    log::info!("[API] 👁️ Vision analysis completed in {}ms", response.processing_time_ms);
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

// ============================================================================
// Oracle (NLP Intent) Handler
// ============================================================================

/// Process natural language intent using OracleVeil (preferred) or legacy Oracle (fallback)
pub async fn process_oracle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OracleIntentRequest>,
) -> impl IntoResponse {
    state.set_mode(OsMode::Oracle).await;
    
    // Broadcast: Oracle is thinking
    state.broadcast_oracle_thinking(&req.text, "parsing").await;
    
    // ════════════════════════════════════════════════════════════════════════
    // PHASE 3: Try OracleVeil first (ZK-signed intent processing via Monad)
    // ════════════════════════════════════════════════════════════════════════
    if let Some(ref veil) = state.oracle_veil {
        log::info!("[API] 🔮 Processing via OracleVeil: '{}'", req.text);
        
        let mut veil = veil.lock().await;
        
        // Set user DID if we have a wallet
        if let Some(wallet) = state.wallet.read().await.as_ref() {
            veil.set_user_did(wallet.did().to_string()).await;
        }
        
        // Use InputSource::Api for API requests
        use crate::oracle::veil::InputSource;
        
        match veil.mediate(&req.text, InputSource::Api).await {
            Ok(response) => {
                state.set_mode(OsMode::Idle).await;
                
                // Broadcast whisper and haptic
                let whisper_id = state.add_whisper(
                    response.whisper.clone(),
                    if response.needs_confirmation { "emphasized" } else { "subtle" },
                    "top_right",
                    Some(3000),
                ).await;
                
                state.broadcast_oracle_whisper(
                    &whisper_id,
                    &response.whisper,
                    if response.needs_confirmation { "emphasized" } else { "subtle" },
                    "top_right",
                    3000,
                ).await;
                
                let haptic_str = format!("{:?}", response.haptic).to_lowercase();
                state.set_haptic(&haptic_str).await;
                state.broadcast_oracle_haptic(&haptic_str, 1.0).await;
                
                // Convert OracleVeil response to API response format
                let api_response = OracleIntentResponse {
                    intent_type: infer_intent_type_from_data(&response.data),
                    content: response.whisper,
                    data: convert_command_data_to_intent_data(&response.data),
                    requires_confirmation: response.needs_confirmation,
                    suggested_actions: vec![],
                    confidence: response.confidence,
                };
                
                log::info!("[API] 🔮 OracleVeil processed: {:?} (confidence: {:.0}%)", 
                    api_response.intent_type, api_response.confidence * 100.0);
                
                return (StatusCode::OK, Json(ApiResponse::success(api_response)));
            }
            Err(e) => {
                log::warn!("[API] OracleVeil error: {} - falling back to legacy", e);
                // Fall through to legacy handler
            }
        }
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // FALLBACK: Legacy Oracle processing (direct, without Monad integration)
    // ════════════════════════════════════════════════════════════════════════
    use crate::oracle::{Oracle, OracleContext, OracleIntent};
    
    // Create Oracle instance and set context
    let mut oracle = Oracle::new();
    
    // Build context from request
    let context = if let Some(ctx_data) = &req.context {
        let mut ctx = OracleContext::default();
        ctx.vision_object = ctx_data.vision_object.clone();
        if let Some(balance) = ctx_data.wallet_balance {
            ctx.wallet_balance = balance;
        }
        ctx.current_app = ctx_data.active_app.clone();
        Some(ctx)
    } else {
        // Get balance from state for context
        let balance = *state.balance.read().await;
        let mut ctx = OracleContext::default();
        ctx.wallet_balance = balance;
        Some(ctx)
    };
    
    // Process through Oracle
    let oracle_response = oracle.process(&req.text, context);
    
    // NEW: Execute actual tools based on Oracle intent
    use crate::oracle::tool_bridge;
    use crate::assistant::ToolRegistry;
    
    // Create tool registry with default tools
    let tool_registry = state.tool_registry.as_ref()
        .or_else(|| {
            // Fallback: create temporary registry if not in state
            log::warn!("[API] No tool registry in state - tools won't execute");
            None
        });
    
    // Execute the intent if we have tools
    let execution_result = if let Some(registry) = tool_registry {
        match tool_bridge::execute_intent(&oracle_response.intent, registry).await {
            Ok(result) => {
                log::info!("[API] ✓ Tool executed: {}", result);
                Some(result)
            }
            Err(e) => {
                log::warn!("[API] Tool execution failed: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    // Use execution result if available, otherwise use Oracle's message
    let final_message = execution_result.unwrap_or_else(|| oracle_response.message.clone());
    
    // Broadcast: Oracle is executing
    state.broadcast_oracle_thinking(&req.text, "executing").await;
    
    // Convert Oracle response to API response
    let (intent_type, data, pending_action_id) = match oracle_response.intent.clone() {
        OracleIntent::Transfer { amount, recipient, memo } => {
            let data = IntentData {
                amount: Some(amount),
                recipient: Some(recipient.clone()),
                memo: memo.clone(),
                location: None,
                duration: None,
                app_type: None,
                url: None,
                query: None,
            };
            
            // Create pending action for transfers (requires confirmation)
            let pending = AppState::create_pending_action(
                IntentType::Transfer,
                format!("Send {} KARA to {}{}", amount, recipient, 
                    memo.as_ref().map(|m| format!(" ({})", m)).unwrap_or_default()),
                Some(data.clone()),
                None, // ZK proof would go here
                oracle_response.confidence,
            );
            
            let action_id = pending.id.clone();
            let expires_at = pending.expires_at;
            state.add_pending_action(pending).await;
            
            // Broadcast: Confirmation required
            state.broadcast_oracle_confirmation(
                &action_id,
                "TRANSFER",
                &format!("Send {} KARA to {}", amount, recipient),
                expires_at,
                oracle_response.confidence,
            ).await;
            
            log::info!("[API] 📝 Created pending transfer action: {}", action_id);
            
            (IntentType::Transfer, Some(data), Some(action_id))
        }
        OracleIntent::CheckBalance | OracleIntent::TransactionHistory => {
            (IntentType::Wallet, None, None)
        }
        OracleIntent::OpenApp { app_type } => {
            (IntentType::OpenApp, Some(IntentData {
                app_type: Some(app_type),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                url: None,
                query: None,
            }), None)
        }
        OracleIntent::OpenBrowser { url } => {
            (IntentType::OpenBrowser, Some(IntentData {
                url,
                app_type: Some("browser".to_string()),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                query: None,
            }), None)
        }
        OracleIntent::PlayVideo { query, url } => {
            (IntentType::PlayVideo, Some(IntentData {
                query,
                url,
                app_type: Some("video".to_string()),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
            }), None)
        }
        OracleIntent::TakeNote { content } => {
            (IntentType::TakeNote, Some(IntentData {
                query: content,
                app_type: Some("notes".to_string()),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                url: None,
            }), None)
        }
        OracleIntent::SetReminder { message, duration } => {
            (IntentType::SetReminder, Some(IntentData {
                query: Some(message),
                duration: Some(duration),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                app_type: None,
                url: None,
            }), None)
        }
        OracleIntent::PlayMusic { query } => {
            (IntentType::PlayMusic, Some(IntentData {
                query,
                app_type: Some("music".to_string()),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                url: None,
            }), None)
        }
        OracleIntent::Navigate { destination } => {
            (IntentType::Navigate, Some(IntentData {
                location: Some(destination),
                amount: None,
                recipient: None,
                memo: None,
                duration: None,
                app_type: None,
                url: None,
                query: None,
            }), None)
        }
        OracleIntent::AnalyzeVision | OracleIntent::ExplainObject { .. } => {
            (IntentType::Analyze, None, None)
        }
        OracleIntent::CloseApp { .. } => {
            (IntentType::CloseApp, None, None)
        }
        OracleIntent::Help => {
            (IntentType::Help, None, None)
        }
        _ => {
            (IntentType::Speak, None, None)
        }
    };
    
    state.set_mode(OsMode::Idle).await;
    
    // Update requires_confirmation based on whether we created a pending action
    let needs_confirmation = pending_action_id.is_some() || oracle_response.requires_confirmation;
    
    // Add pending action ID to suggested actions if present
    let mut suggested = oracle_response.suggested_actions.clone();
    if let Some(ref action_id) = pending_action_id {
        suggested.insert(0, format!("action_id:{}", action_id));
    }
    
    // Clone final message (tool result or oracle message) before moving into response
    let message = final_message.clone();
    
    let response = OracleIntentResponse {
        intent_type: intent_type.clone(),
        content: final_message,
        data,
        requires_confirmation: needs_confirmation,
        suggested_actions: suggested,
        confidence: oracle_response.confidence,
    };
    
    // Add whisper and haptic feedback for the response
    let style = if needs_confirmation { "emphasized" } else { "subtle" };
    let haptic = if needs_confirmation { "confirm" } else { "success" };
    
    // Add whisper and broadcast it
    let whisper_id = state.add_whisper(
        message.clone(),
        style,
        "top_right",
        Some(3000),
    ).await;
    
    // Broadcast whisper event
    state.broadcast_oracle_whisper(
        &whisper_id,
        &message,
        style,
        "top_right",
        3000,
    ).await;
    
    // Set haptic and broadcast it
    state.set_haptic(haptic).await;
    state.broadcast_oracle_haptic(haptic, 1.0).await;
    
    log::info!("[API] 🔮 Oracle processed: {:?} (confidence: {:.0}%)", 
        response.intent_type, response.confidence * 100.0);
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

// ============================================================================
// Celestia DA Handlers
// ============================================================================

/// Submit data to Celestia
pub async fn submit_to_da(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<DaSubmitRequest>,
) -> impl IntoResponse {
    // In production, this would use celestia_rpc
    // For now, return simulated response
    
    let namespace = req.namespace.unwrap_or_else(|| "karana".to_string());
    let tx_hash = format!("0x{}", hex::encode(&uuid::Uuid::new_v4().as_bytes()[..16]));
    
    let response = DaSubmitResponse {
        tx_hash: tx_hash.clone(),
        height: 1234567, // Mock height
        namespace,
        status: "SUBMITTED".to_string(),
    };
    
    log::info!("[API] 📡 Data submitted to Celestia: {}", tx_hash);
    
    (StatusCode::ACCEPTED, Json(ApiResponse::success(response)))
}

/// Check DA submission status
pub async fn get_da_status(
    axum::extract::Path(tx_hash): axum::extract::Path<String>,
) -> impl IntoResponse {
    // In production, query Celestia for actual status
    let response = DaStatusResponse {
        tx_hash,
        status: "confirmed".to_string(),
        confirmations: 6,
        height: Some(1234567),
    };
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

// ============================================================================
// OS State Handler
// ============================================================================

pub async fn get_os_state(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let wallet = state.wallet.read().await;
    let mode = state.mode.read().await;
    
    let info = OsStateInfo {
        mode: mode.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // TODO: Track uptime
        wallet_connected: wallet.is_some(),
        camera_active: false,
    };
    
    (StatusCode::OK, Json(ApiResponse::success(info)))
}

// ============================================================================
// WebSocket Handler
// ============================================================================

pub async fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: Arc<AppState>) {
    // Create channel for this connection
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    
    // Simple ping-pong and subscribe handling
    loop {
        tokio::select! {
            // Check for outgoing messages
            Some(msg) = rx.recv() => {
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            // Check for incoming messages
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            match ws_msg {
                                WsMessage::Subscribe { channel } => {
                                    let mut subs = state.ws_subscribers.write().await;
                                    subs.entry(channel).or_default().push(tx.clone());
                                }
                                WsMessage::Ping => {
                                    let _ = tx.send(serde_json::to_string(&WsMessage::Pong).unwrap()).await;
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
    
    log::info!("[API] WebSocket connection closed");
}

// ============================================================================
// Confirmation Handlers
// ============================================================================

/// Get all pending actions awaiting confirmation
pub async fn get_pending_actions(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let actions = state.get_all_pending_actions().await;
    
    log::info!("[API] 📋 Retrieved {} pending actions", actions.len());
    
    (StatusCode::OK, Json(ApiResponse::success(actions)))
}

/// Confirm or reject a pending action
pub async fn confirm_action(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConfirmActionRequest>,
) -> impl IntoResponse {
    // Get the pending action
    let action = match state.remove_pending_action(&req.action_id).await {
        Some(a) => a,
        None => {
            log::warn!("[API] Attempted to confirm non-existent action: {}", req.action_id);
            return (StatusCode::NOT_FOUND, Json(ApiResponse::<ConfirmActionResponse>::error(
                "Pending action not found or expired"
            )));
        }
    };
    
    if !req.approved {
        log::info!("[API] ❌ Action {} rejected by user", req.action_id);
        return (StatusCode::OK, Json(ApiResponse::success(ConfirmActionResponse {
            success: true,
            tx_hash: None,
            message: "Action rejected".to_string(),
        })));
    }
    
    // Process the confirmed action
    let result = match action.action_type {
        IntentType::Transfer => {
            if let Some(data) = &action.data {
                let amount = data.amount.unwrap_or(0);
                let recipient = data.recipient.clone().unwrap_or_default();
                
                // Check balance
                if !state.debit(amount).await {
                    return (StatusCode::BAD_REQUEST, Json(ApiResponse::<ConfirmActionResponse>::error(
                        "Insufficient balance"
                    )));
                }
                
                // Create transaction
                let tx_hash = format!("0x{}", hex::encode(&uuid::Uuid::new_v4().as_bytes()[..16]));
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                let tx = Transaction {
                    id: tx_hash.clone(),
                    tx_type: "TRANSFER".to_string(),
                    amount,
                    recipient: recipient.clone(),
                    sender: "user".to_string(),
                    timestamp: now,
                    status: "CONFIRMED".to_string(),
                    signature: None,
                    da_tx_hash: None,
                };
                
                state.add_transaction(tx).await;
                
                // Broadcast wallet update
                let balance = *state.balance.read().await;
                let ws_msg = serde_json::to_string(&WsMessage::WalletUpdate {
                    balance,
                    last_tx_hash: Some(tx_hash.clone()),
                }).unwrap();
                state.broadcast("wallet", &ws_msg).await;
                
                log::info!("[API] ✅ Transfer confirmed: {} KARA to {}", amount, recipient);
                
                Ok((tx_hash, format!("Sent {} KARA to {}", amount, recipient)))
            } else {
                Err("Transfer data missing".to_string())
            }
        }
        _ => {
            // For other action types, just acknowledge
            log::info!("[API] ✅ Action {:?} confirmed", action.action_type);
            Ok((uuid::Uuid::new_v4().to_string(), format!("{:?} action completed", action.action_type)))
        }
    };
    
    match result {
        Ok((tx_hash, message)) => {
            (StatusCode::OK, Json(ApiResponse::success(ConfirmActionResponse {
                success: true,
                tx_hash: Some(tx_hash),
                message,
            })))
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(ApiResponse::<ConfirmActionResponse>::error(e)))
        }
    }
}

// ============================================================================
// Manifest Handlers (AR Whispers / Haptic)
// ============================================================================

/// Get current manifest state (active whispers, last haptic)
pub async fn get_manifest_state(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let manifest = state.get_manifest_state().await;
    
    log::debug!("[API] 👁️ Manifest state: {} whispers, haptic: {:?}",
        manifest.whispers.len(), manifest.last_haptic);
    
    (StatusCode::OK, Json(ApiResponse::success(manifest)))
}

/// Clear all whispers
pub async fn clear_manifest(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.clear_whispers().await;
    
    log::info!("[API] 🧹 Manifest cleared");
    
    (StatusCode::OK, Json(ApiResponse::success("Manifest cleared")))
}

// ============================================================================
// Helper Functions for OracleVeil → API Response Conversion
// ============================================================================

use crate::oracle::command::CommandData;

/// Infer IntentType from CommandData
fn infer_intent_type_from_data(data: &Option<CommandData>) -> IntentType {
    match data {
        Some(CommandData::TxHash(_)) => IntentType::Transfer,
        Some(CommandData::Balance(_)) => IntentType::Wallet,
        Some(CommandData::BlockData(_)) => IntentType::Wallet,
        Some(CommandData::StoredHash(_)) => IntentType::TakeNote,
        Some(CommandData::RetrievedData(_)) => IntentType::Wallet,
        Some(CommandData::SearchResults(_)) => IntentType::Analyze,
        Some(CommandData::MessageId(_)) => IntentType::Speak,
        Some(CommandData::HapticPlayed) => IntentType::Speak,
        Some(CommandData::HardwareStatus(_)) => IntentType::Analyze,
        Some(CommandData::PipelineStatus(_)) => IntentType::Analyze,
        Some(CommandData::Text(_)) => IntentType::Speak,
        _ => IntentType::Speak,
    }
}

/// Convert CommandData to IntentData for API response
fn convert_command_data_to_intent_data(data: &Option<CommandData>) -> Option<IntentData> {
    match data {
        Some(CommandData::TxHash(hash)) => {
            Some(IntentData {
                query: Some(format!("Transaction: {}", hash)),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                app_type: None,
                url: None,
            })
        }
        Some(CommandData::Balance(balance)) => {
            Some(IntentData {
                query: Some(format!("{} KARA", balance)),
                amount: Some(*balance as u64),
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                app_type: None,
                url: None,
            })
        }
        Some(CommandData::StoredHash(hash)) => {
            Some(IntentData {
                query: Some(format!("Stored: {}", hex::encode(hash))),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                app_type: None,
                url: None,
            })
        }
        Some(CommandData::SearchResults(hits)) => {
            let results: Vec<String> = hits.iter().map(|h| h.preview.clone()).collect();
            Some(IntentData {
                query: Some(results.join(", ")),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                app_type: None,
                url: None,
            })
        }
        Some(CommandData::Text(text)) => {
            Some(IntentData {
                query: Some(text.clone()),
                amount: None,
                recipient: None,
                memo: None,
                location: None,
                duration: None,
                app_type: None,
                url: None,
            })
        }
        _ => None,
    }
}

// ============================================================================
// Use Case Handlers (Phase 2: Glasses-Ready Use Cases)
// ============================================================================

/// Execute a use case (productivity, health, social, navigation)
pub async fn execute_use_case(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UseCaseRequest>,
) -> impl IntoResponse {
    use crate::oracle::UseCaseDispatcher;
    
    log::info!("[API] 📱 Use case: {} / {}", req.category, req.intent);
    state.set_mode(OsMode::Oracle).await;
    
    let dispatcher = UseCaseDispatcher::new();
    
    match dispatcher.dispatch(&req.category, &req.intent, req.params.clone()).await {
        Ok(manifest) => {
            state.set_mode(OsMode::Idle).await;
            
            // Broadcast whisper
            let whisper_id = state.add_whisper(
                manifest.whisper.clone(),
                if manifest.needs_confirmation { "emphasized" } else { "normal" },
                "center",
                Some(3000),
            ).await;
            
            state.broadcast_oracle_whisper(
                &whisper_id,
                &manifest.whisper,
                if manifest.needs_confirmation { "emphasized" } else { "normal" },
                "center",
                3000,
            ).await;
            
            // Broadcast haptic
            let haptic_str = format!("{:?}", manifest.haptic).to_lowercase();
            state.set_haptic(&haptic_str).await;
            state.broadcast_oracle_haptic(&haptic_str, 1.0).await;
            
            // Convert overlay
            let overlay_response = manifest.overlay.map(|o| AROverlayResponse {
                content: o.content,
                position: o.position,
                duration_ms: o.duration_ms,
                overlay_type: format!("{:?}", o.overlay_type).to_lowercase(),
                style: format!("{:?}", o.style).to_lowercase(),
            });
            
            // Check for generated files
            let artifacts: Vec<String> = std::fs::read_dir("/tmp/karana")
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path().display().to_string())
                        .collect()
                })
                .unwrap_or_default();
            
            let response = UseCaseResponse {
                success: true,
                whisper: manifest.whisper,
                haptic: haptic_str,
                overlay: overlay_response,
                confidence: manifest.confidence,
                artifacts,
            };
            
            log::info!("[API] 📱 Use case complete: {} (confidence: {:.0}%)", 
                req.category, response.confidence * 100.0);
            
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => {
            state.set_mode(OsMode::Idle).await;
            log::error!("[API] Use case failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<UseCaseResponse>::error(e.to_string())))
        }
    }
}

