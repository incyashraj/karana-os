# 📱 Android Apps on Kāraṇa OS - Complete Guide

Kāraṇa OS includes a **Waydroid-like Android container** that allows you to run regular Android apps on your AR glasses with intelligent optimizations.

---

## 🎯 Quick Start

### Using Pre-Configured Apps (Easiest)

Kāraṇa OS comes with **15+ popular apps pre-configured** with optimizations:

**Just say to Oracle:**
```
"Open YouTube"
"Launch WhatsApp" 
"Show me Instagram"
"Start Spotify"
```

The system automatically:
- Downloads the app if not installed
- Applies AR/privacy optimizations
- Launches in optimized container

---

## 📦 Pre-Configured Apps

### Social Media
- **Instagram** - Local AI filters, content moderation
- **Twitter/X** - Privacy protection, local timeline
- **Facebook** - Ad blocking, privacy layer
- **TikTok** - Heavy AI optimization, local recommendations

### Messaging
- **WhatsApp** - P2P network, enhanced privacy
- **Telegram** - Blockchain payments integration
- **Signal** - Full privacy mode

### Video & Music
- **YouTube** - P2P video delivery, ad-free
- **Spotify** - Spatial audio, local recommendations
- **Netflix** - Optimized streaming

### Productivity
- **Gmail** - Privacy-enhanced email
- **Chrome** - Decentralized browsing
- **Maps** - Offline-first navigation

---

## 🔧 Manual APK Installation

### Method 1: Via Oracle (Voice)

```
"Install APK from /path/to/app.apk"
"Download and install Twitter"
```

### Method 2: Via API (Programmatic)

```rust
use karana_core::app_ecosystem::{AndroidContainer, AppStore};

// Initialize container
let container = AndroidContainer::new("/var/karana/android".into());
container.start().await?;

// Install APK
let apk_path = PathBuf::from("/path/to/app.apk");
let package_name = container.install_app(apk_path).await?;

// Launch app
container.launch_app(&package_name).await?;
```

### Method 3: Via App Store (UI)

```rust
let app_store = AppStore::new("/var/karana/downloads".into());

// Search apps
let results = app_store.search("twitter").await;

// Download and install
let apk_path = app_store.download_app("twitter").await?;
container.install_app(apk_path).await?;
```

---

## 🎨 AR Optimizations Applied

When you launch an Android app, Kāraṇa OS automatically applies:

### Display Adaptation
- **Screen virtualization**: Android thinks it's a phone screen
- **Gaze-to-touch**: Your eye movements = touch events
- **Hand gestures**: Swipe, pinch, tap in mid-air

### Audio Enhancement
- **Spatial audio**: Sounds positioned in 3D space
- **Noise cancellation**: Clear audio even in noisy environments
- **Voice commands**: Control apps with voice

### Privacy Features
- **Local AI**: Recommendations run on-device
- **Ad blocking**: Removes ads automatically
- **Data minimization**: Blocks unnecessary tracking
- **P2P networking**: Direct peer connections

### Performance
- **Hardware acceleration**: GPU for video/graphics
- **Smart caching**: Preload content intelligently
- **Battery optimization**: Adaptive power management

---

## 🚀 Advanced Usage

### Custom App Profile

```rust
use karana_core::app_ecosystem::native_apps::*;

let custom_app = NativeAppDescriptor {
    id: "my_custom_app".to_string(),
    name: "My App".to_string(),
    package_name: "com.example.myapp".to_string(),
    category: AppCategory::Productivity,
    min_api_level: 23,
    permissions: vec!["INTERNET".to_string()],
    optimizations: AppOptimizations {
        hw_video_decode: true,
        hw_audio_process: true,
        edge_ai: true,
        cache_strategy: CacheStrategy::Aggressive,
        network_mode: NetworkMode::P2P,
    },
    karana_integrations: vec![
        KaranaIntegration::LocalRecommendations,
        KaranaIntegration::PrivacyProtection,
    ],
};
```

### Launch with Custom Settings

```rust
let launcher = NativeAppLauncher::new();
let intent_router = IntentRouter::new();

launcher.launch_app(
    "youtube",
    &container,
    &intent_router
).await?;
```

### Monitor App State

```rust
// Check container state
let state = container.get_state().await;
println!("Container: {:?}", state);

// List installed apps
let apps = container.get_installed_apps().await;
for app in apps {
    println!("📱 {} ({})", app.app_name, app.package_name);
}
```

---

## 🛡️ Security & Sandboxing

Every Android app runs in a secure sandbox:

### Automatic Security Scan
```rust
let app_store = AppStore::new(downloads_path);
let scan = app_store.scan_app(&listing).await?;

match scan.status {
    VerificationStatus::Verified => println!("✅ Safe"),
    VerificationStatus::Suspicious => println!("⚠️  Warning"),
    VerificationStatus::Malicious => println!("🚫 Blocked"),
}
```

### Sandbox Profiles
- **Strict**: Minimal permissions, fully isolated
- **Moderate**: Standard permissions (default)
- **Relaxed**: More permissions (verified apps only)

### Permission Analysis
```rust
let analysis = scan.permission_analysis;
println!("Risk score: {}", analysis.risk_score);
println!("Sensitive permissions: {:?}", analysis.sensitive_permissions);
```

---

## 🎮 Popular Use Cases

### 1. Social Media Browsing
```
"Show me Instagram reels"
→ Launches Instagram with:
  - Local AI recommendations
  - Content moderation
  - Ad blocking
  - Spatial interface
```

### 2. Video Watching
```
"Play trending YouTube videos"
→ Launches YouTube with:
  - P2P video streaming
  - Hardware acceleration
  - Offline caching
  - Voice control
```

### 3. Messaging
```
"Open WhatsApp messages"
→ Launches WhatsApp with:
  - P2P network routing
  - Enhanced privacy
  - Voice typing
  - Hand-free control
```

### 4. Music Streaming
```
"Play my Spotify playlist"
→ Launches Spotify with:
  - Spatial audio
  - Local recommendations
  - Gesture controls
  - Background mode
```

---

## 🔄 App Lifecycle Management

### Automatic Management
```rust
// Apps automatically:
// - Suspend when not in use
// - Resume when focused
// - Close when memory pressure high
// - Update in background
```

### Manual Control
```rust
use karana_core::apps::AppManager;

let mut app_manager = AppManager::new();

// Launch app
app_manager.launch("youtube")?;

// Suspend (keep in memory)
app_manager.suspend("youtube");

// Resume
app_manager.resume("youtube");

// Stop completely
app_manager.stop("youtube");

// Uninstall
app_manager.uninstall("youtube")?;
```

---

## 📊 System Requirements

- **OS**: Linux-based (Ubuntu 20.04+)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 10GB for Android container + apps
- **GPU**: OpenGL ES 3.0+ for hardware acceleration
- **Network**: Internet for app downloads

---

## 🐛 Troubleshooting

### App Won't Launch
```bash
# Check container status
curl http://localhost:8080/api/os/state

# Restart container
# (via API or voice command: "Restart Android container")
```

### Performance Issues
```
"Optimize performance"
"Clear app cache"
"Close background apps"
```

### Permission Denied
```
"Grant camera permission to Instagram"
"Allow WhatsApp to access contacts"
```

---

## 🎯 API Endpoints

### Install App
```bash
POST /api/apps/install
{
  "apk_path": "/path/to/app.apk"
}
```

### Launch App
```bash
POST /api/apps/launch
{
  "package_name": "com.instagram.android"
}
```

### List Apps
```bash
GET /api/apps/installed
```

### Uninstall App
```bash
DELETE /api/apps/{package_name}
```

---

## 🚀 Future Features

- [ ] Multi-window support (run multiple apps side-by-side)
- [ ] Gesture recorder (custom gestures per app)
- [ ] App cloning (run multiple instances)
- [ ] Desktop mode (Linux apps + Android apps together)
- [ ] Cloud sync (sync app data across devices)

---

## 📚 Additional Resources

- **Architecture**: See `ARCHITECTURE.md` Phase 51
- **Source Code**: `karana-core/src/app_ecosystem/`
- **API Docs**: `karana-core/src/api/handlers.rs`
- **Examples**: `karana-core/examples/android_apps.rs`

---

## 💡 Pro Tips

1. **Voice is fastest**: Just say the app name
2. **Pre-installed = optimized**: Use pre-configured apps when possible
3. **P2P saves data**: Let popular content come from peers
4. **Privacy first**: Check permissions before installing
5. **Gesture shortcuts**: Create custom gestures for frequent actions

---

**Questions?** Ask the Oracle: `"How do I use Android apps?"`
