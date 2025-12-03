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
            (StatusCode::NOT_FOUND, Json(ApiResponse::<WalletInfo>::error("No wallet found. Create or restore one first.")))
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

/// Process natural language intent using the enhanced Oracle engine
pub async fn process_oracle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OracleIntentRequest>,
) -> impl IntoResponse {
    use crate::oracle::{Oracle, OracleContext, OracleIntent};
    
    state.set_mode(OsMode::Oracle).await;
    
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
    
    // Convert Oracle response to API response
    let (intent_type, data) = match oracle_response.intent {
        OracleIntent::Transfer { amount, recipient, memo } => {
            (IntentType::Transfer, Some(IntentData {
                amount: Some(amount),
                recipient: Some(recipient),
                memo,
                location: None,
                duration: None,
                app_type: None,
                url: None,
                query: None,
            }))
        }
        OracleIntent::CheckBalance | OracleIntent::TransactionHistory => {
            (IntentType::Wallet, None)
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
            }))
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
            }))
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
            }))
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
            }))
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
            }))
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
            }))
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
            }))
        }
        OracleIntent::AnalyzeVision | OracleIntent::ExplainObject { .. } => {
            (IntentType::Analyze, None)
        }
        OracleIntent::CloseApp { .. } => {
            (IntentType::CloseApp, None)
        }
        OracleIntent::Help => {
            (IntentType::Help, None)
        }
        _ => {
            (IntentType::Speak, None)
        }
    };
    
    state.set_mode(OsMode::Idle).await;
    
    let response = OracleIntentResponse {
        intent_type,
        content: oracle_response.message,
        data,
        requires_confirmation: oracle_response.requires_confirmation,
        suggested_actions: oracle_response.suggested_actions,
        confidence: oracle_response.confidence,
    };
    
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
