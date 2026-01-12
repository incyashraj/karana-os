# API Reference Guide

Complete reference for Kāraṇa OS HTTP/WebSocket API.

## Overview

The Kāraṇa OS API server exposes the entire system functionality via REST and WebSocket endpoints for integration with simulators, mobile apps, and external tools.

**Base URL**: `http://localhost:8080/api`
**WebSocket**: `ws://localhost:8080/ws`

---

## Table of Contents

1. [Authentication](#authentication)
2. [HTTP Endpoints](#http-endpoints)
3. [WebSocket Events](#websocket-events)
4. [Data Models](#data-models)
5. [Error Handling](#error-handling)
6. [Rate Limiting](#rate-limiting)
7. [Code Examples](#code-examples)

---

## Authentication

Currently, the API is **unauthenticated** for development. Production will use:
- **DID-based authentication** (Decentralized Identifiers)
- **Ed25519 signature verification**
- **JWT tokens** for session management

---

## HTTP Endpoints

### Health Check

#### `GET /health`
Check if the API server is running.

**Response:**
```json
{
  "status": "ok",
  "version": "1.0.0",
  "uptime_seconds": 3600
}
```

---

### System Commands

#### `POST /api/command`
Execute natural language command.

**Request:**
```json
{
  "command": "take a photo"
}
```

**Response:**
```json
{
  "success": true,
  "action": "hardware.camera.capture",
  "result": {
    "filename": "photo_20250108_143022.jpg",
    "path": "/storage/photos/photo_20250108_143022.jpg",
    "timestamp": "2025-01-08T14:30:22Z"
  },
  "message": "Photo captured successfully"
}
```

**Commands:**
- `"take a photo"` → Camera capture
- `"brightness 80%"` → Display brightness
- `"battery status"` → Power info
- `"create wallet"` → Wallet generation
- `"send 50 KARA to Alice"` → Transaction
- `"what am I looking at"` → Scene analysis
- `"open youtube"` → App launch
- `"run diagnostics"` → System health check

---

### Hardware Control

#### `GET /api/hardware/camera/status`
Get camera status.

**Response:**
```json
{
  "active": false,
  "resolution": "1280x720",
  "fps": 30,
  "format": "YUYV"
}
```

#### `POST /api/hardware/camera/capture`
Capture photo from camera.

**Response:**
```json
{
  "success": true,
  "image": "data:image/jpeg;base64,/9j/4AAQSkZJRg...",
  "timestamp": "2025-01-08T14:30:22Z"
}
```

#### `POST /api/hardware/brightness`
Set display brightness.

**Request:**
```json
{
  "level": 80
}
```

**Response:**
```json
{
  "success": true,
  "brightness": 80
}
```

#### `GET /api/hardware/battery`
Get battery status.

**Response:**
```json
{
  "level": 85,
  "charging": false,
  "temperature": 32.5,
  "voltage": 3.85,
  "health": "good",
  "estimated_minutes": 137
}
```

#### `POST /api/hardware/power_mode`
Change power management mode.

**Request:**
```json
{
  "mode": "balanced"
}
```

Modes: `"performance"`, `"balanced"`, `"power_saver"`

---

### Blockchain & Wallet

#### `POST /api/wallet/create`
Create new wallet.

**Response:**
```json
{
  "success": true,
  "did": "did:karana:user_001",
  "public_key": "0x1234...abcd",
  "mnemonic": "word1 word2 word3 ... word24"
}
```

#### `GET /api/wallet/balance`
Get wallet balance.

**Response:**
```json
{
  "balance": "1000.50",
  "symbol": "KARA",
  "staked": "500.00",
  "available": "500.50"
}
```

#### `GET /api/wallet/transactions`
List recent transactions.

**Response:**
```json
{
  "transactions": [
    {
      "hash": "0xabc123...",
      "from": "did:karana:user_001",
      "to": "did:karana:user_002",
      "amount": "50.00",
      "type": "transfer",
      "timestamp": "2025-01-08T14:25:00Z",
      "block": 12345,
      "status": "confirmed"
    }
  ]
}
```

#### `POST /api/wallet/send`
Send tokens.

**Request:**
```json
{
  "to": "did:karana:alice",
  "amount": "50.00",
  "memo": "Payment for coffee"
}
```

**Response:**
```json
{
  "success": true,
  "tx_hash": "0xdef456...",
  "status": "pending"
}
```

#### `POST /api/wallet/stake`
Stake tokens.

**Request:**
```json
{
  "amount": "100.00"
}
```

#### `POST /api/wallet/unstake`
Unstake tokens.

**Request:**
```json
{
  "amount": "50.00"
}
```

---

### Blockchain Info

#### `GET /api/blockchain/status`
Get blockchain status.

**Response:**
```json
{
  "height": 12345,
  "latest_block_hash": "0x789abc...",
  "peers": 8,
  "syncing": false,
  "validator_count": 21,
  "avg_block_time": 12.3
}
```

#### `GET /api/blockchain/block/:height`
Get block by height.

**Response:**
```json
{
  "height": 12345,
  "hash": "0x789abc...",
  "timestamp": "2025-01-08T14:20:00Z",
  "validator": "did:karana:validator_001",
  "transactions": 42,
  "size": 1024
}
```

---

### AI & Intelligence

#### `POST /api/ai/chat`
Chat with AI assistant.

**Request:**
```json
{
  "message": "What's the weather today?",
  "context": {
    "location": "San Francisco",
    "time": "2025-01-08T14:30:00Z"
  }
}
```

**Response:**
```json
{
  "response": "Currently 62°F (17°C) in San Francisco with partly cloudy skies. Light breeze from the west at 8 mph.",
  "intent": "weather_query",
  "confidence": 0.95,
  "actions": [
    {
      "type": "oracle_request",
      "tool": "web_api",
      "status": "completed"
    }
  ]
}
```

#### `POST /api/vision/analyze`
Analyze scene from camera.

**Response:**
```json
{
  "objects": [
    {"label": "person", "confidence": 0.92, "bbox": [100, 150, 300, 450]},
    {"label": "laptop", "confidence": 0.88, "bbox": [400, 200, 600, 400]},
    {"label": "cup", "confidence": 0.85, "bbox": [350, 300, 400, 380]}
  ],
  "scene": "indoor office",
  "lighting": "bright",
  "depth_map": "data:image/png;base64,..."
}
```

---

### Applications

#### `GET /api/apps/list`
List installed applications.

**Response:**
```json
{
  "apps": [
    {
      "id": "timer",
      "name": "Timer",
      "version": "1.0.0",
      "status": "stopped",
      "memory_mb": 5
    },
    {
      "id": "navigation",
      "name": "Navigation",
      "version": "1.2.0",
      "status": "running",
      "memory_mb": 50
    }
  ]
}
```

#### `POST /api/apps/launch`
Launch application.

**Request:**
```json
{
  "app_id": "youtube"
}
```

**Response:**
```json
{
  "success": true,
  "app_id": "youtube",
  "status": "running",
  "pid": 12345
}
```

#### `POST /api/apps/close`
Close application.

**Request:**
```json
{
  "app_id": "youtube"
}
```

---

### AR & Spatial

#### `GET /api/ar/anchors`
List spatial anchors.

**Response:**
```json
{
  "anchors": [
    {
      "id": "anchor_001",
      "position": {"x": 1.5, "y": 0.8, "z": -2.0},
      "rotation": {"x": 0, "y": 0, "z": 0, "w": 1},
      "content_type": "browser_tab",
      "url": "https://news.ycombinator.com",
      "timestamp": "2025-01-08T14:15:00Z"
    }
  ]
}
```

#### `POST /api/ar/anchor/create`
Create spatial anchor.

**Request:**
```json
{
  "position": {"x": 1.5, "y": 0.8, "z": -2.0},
  "rotation": {"x": 0, "y": 0, "z": 0, "w": 1},
  "content_type": "browser_tab",
  "url": "https://example.com"
}
```

#### `DELETE /api/ar/anchor/:id`
Delete spatial anchor.

---

### System Services

#### `POST /api/system/diagnostics`
Run system diagnostics.

**Response:**
```json
{
  "cpu": {
    "usage": 45.2,
    "temperature": 52.0,
    "cores": 4
  },
  "memory": {
    "total_mb": 8192,
    "used_mb": 4096,
    "available_mb": 4096
  },
  "storage": {
    "total_gb": 128,
    "used_gb": 64,
    "available_gb": 64
  },
  "network": {
    "peers": 8,
    "bandwidth_mbps": 100.0
  },
  "health": "good"
}
```

#### `POST /api/system/security/scan`
Run security threat scan.

**Response:**
```json
{
  "threats_detected": 0,
  "scan_duration_ms": 350,
  "last_scan": "2025-01-08T14:25:00Z",
  "security_level": "normal"
}
```

#### `GET /api/system/updates/check`
Check for system updates.

**Response:**
```json
{
  "update_available": true,
  "current_version": "1.0.0",
  "latest_version": "1.1.0",
  "download_size_mb": 45,
  "release_notes": "Bug fixes and performance improvements"
}
```

---

## WebSocket Events

### Connection

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('Connected to Kāraṇa OS');
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event:', data);
};
```

### Event Types

#### Camera Frame
```json
{
  "type": "camera_frame",
  "timestamp": "2025-01-08T14:30:22Z",
  "image": "data:image/jpeg;base64,..."
}
```

#### Voice Input
```json
{
  "type": "voice_input",
  "text": "take a photo",
  "confidence": 0.92,
  "language": "en-US"
}
```

#### System Action
```json
{
  "type": "action",
  "action": "hardware.camera.capture",
  "status": "completed",
  "result": {
    "filename": "photo_20250108_143022.jpg"
  }
}
```

#### Notification
```json
{
  "type": "notification",
  "priority": "high",
  "title": "Timer Complete",
  "message": "Your 5-minute timer has finished",
  "timestamp": "2025-01-08T14:35:00Z"
}
```

#### Battery Status
```json
{
  "type": "battery_update",
  "level": 20,
  "charging": false,
  "warning": "Low battery - enable power saver mode?"
}
```

#### Blockchain Event
```json
{
  "type": "blockchain_event",
  "event": "new_block",
  "height": 12346,
  "hash": "0xabc123...",
  "transactions": 35
}
```

---

## Data Models

### User DID (Decentralized Identifier)
```
Format: did:karana:<identifier>
Example: did:karana:user_001
```

### Transaction Hash
```
Format: 0x<64 hexadecimal characters>
Example: 0xabc123def456...
```

### Spatial Position
```json
{
  "x": 1.5,  // meters right (negative = left)
  "y": 0.8,  // meters up (negative = down)
  "z": -2.0  // meters forward (negative = backward)
}
```

### Quaternion Rotation
```json
{
  "x": 0.0,
  "y": 0.0,
  "z": 0.0,
  "w": 1.0
}
```

---

## Error Handling

### Error Response Format
```json
{
  "success": false,
  "error": {
    "code": "WALLET_NOT_FOUND",
    "message": "Wallet does not exist. Create one with POST /api/wallet/create",
    "details": {
      "attempted_did": "did:karana:user_999"
    }
  }
}
```

### Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `INVALID_REQUEST` | Malformed request | 400 |
| `WALLET_NOT_FOUND` | Wallet doesn't exist | 404 |
| `INSUFFICIENT_BALANCE` | Not enough tokens | 400 |
| `CAMERA_UNAVAILABLE` | Camera hardware error | 503 |
| `AI_SERVICE_ERROR` | AI inference failed | 500 |
| `BLOCKCHAIN_SYNC` | Chain is syncing | 503 |
| `RATE_LIMIT_EXCEEDED` | Too many requests | 429 |
| `UNAUTHORIZED` | Authentication failed | 401 |

---

## Rate Limiting

**Current limits** (per IP address):
- General endpoints: 100 requests/minute
- AI chat: 30 requests/minute
- Camera capture: 10 requests/minute
- Wallet transactions: 10 requests/minute

**Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1704726000
```

---

## Code Examples

### JavaScript (Browser/Node.js)

#### Basic Command
```javascript
async function sendCommand(command) {
  const response = await fetch('http://localhost:8080/api/command', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ command })
  });
  
  const result = await response.json();
  console.log(result);
  return result;
}

// Usage
await sendCommand('take a photo');
await sendCommand('brightness 80%');
await sendCommand('battery status');
```

#### WebSocket Events
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch (data.type) {
    case 'camera_frame':
      updateCameraView(data.image);
      break;
    case 'voice_input':
      displayTranscript(data.text);
      break;
    case 'notification':
      showNotification(data.title, data.message);
      break;
  }
};
```

### Python

#### Basic API Client
```python
import requests

BASE_URL = 'http://localhost:8080/api'

class KaranaClient:
    def send_command(self, command):
        response = requests.post(
            f'{BASE_URL}/command',
            json={'command': command}
        )
        return response.json()
    
    def get_battery_status(self):
        response = requests.get(f'{BASE_URL}/hardware/battery')
        return response.json()
    
    def create_wallet(self):
        response = requests.post(f'{BASE_URL}/wallet/create')
        return response.json()

# Usage
client = KaranaClient()
result = client.send_command('take a photo')
print(result)
```

### Rust

#### Using reqwest
```rust
use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Send command
    let response = client
        .post("http://localhost:8080/api/command")
        .json(&json!({
            "command": "take a photo"
        }))
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("Result: {:?}", result);
    
    Ok(())
}
```

### cURL

```bash
# Send command
curl -X POST http://localhost:8080/api/command \
  -H "Content-Type: application/json" \
  -d '{"command": "take a photo"}'

# Get battery status
curl http://localhost:8080/api/hardware/battery

# Create wallet
curl -X POST http://localhost:8080/api/wallet/create

# Send transaction
curl -X POST http://localhost:8080/api/wallet/send \
  -H "Content-Type: application/json" \
  -d '{
    "to": "did:karana:alice",
    "amount": "50.00",
    "memo": "Payment"
  }'
```

---

## WebSocket Client Examples

### JavaScript
```javascript
class KaranaWebSocket {
  constructor() {
    this.ws = new WebSocket('ws://localhost:8080/ws');
    this.handlers = {};
    
    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      const handler = this.handlers[data.type];
      if (handler) handler(data);
    };
  }
  
  on(eventType, handler) {
    this.handlers[eventType] = handler;
  }
}

// Usage
const karana = new KaranaWebSocket();

karana.on('camera_frame', (data) => {
  console.log('Camera frame received');
});

karana.on('notification', (data) => {
  console.log(`Notification: ${data.title}`);
});
```

### Python (websockets library)
```python
import asyncio
import websockets
import json

async def listen():
    async with websockets.connect('ws://localhost:8080/ws') as websocket:
        while True:
            message = await websocket.recv()
            data = json.loads(message)
            
            if data['type'] == 'camera_frame':
                print('Camera frame received')
            elif data['type'] == 'notification':
                print(f"Notification: {data['title']}")

asyncio.run(listen())
```

---

## Testing the API

### Postman Collection

Import this JSON into Postman:

```json
{
  "info": {
    "name": "Kāraṇa OS API",
    "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
  },
  "item": [
    {
      "name": "Health Check",
      "request": {
        "method": "GET",
        "url": "http://localhost:8080/health"
      }
    },
    {
      "name": "Send Command",
      "request": {
        "method": "POST",
        "url": "http://localhost:8080/api/command",
        "body": {
          "mode": "raw",
          "raw": "{\"command\": \"take a photo\"}"
        }
      }
    }
  ]
}
```

---

## API Versioning

Current version: **v1** (implicit in `/api/...`)

Future versions will use explicit paths:
- `/api/v2/command`
- `/api/v3/wallet/send`

---

## Support

For API issues:
1. Check server logs: `RUST_LOG=info cargo run --bin karana-api-server`
2. Test with cURL first
3. Check firewall/network settings
4. Review error codes above
5. File GitHub issue with request/response logs
