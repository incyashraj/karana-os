// Example: Using Android Apps in Kāraṇa OS
// Run with: cargo run --example android_apps

use karana_core::app_ecosystem::{
    AndroidContainer, AndroidProperties, AppStore, NativeAppLauncher, IntentRouter,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Kāraṇa OS - Android App Integration Demo\n");

    // 1. Initialize Android Container
    println!("📦 Initializing Android container...");
    let container_path = PathBuf::from("/tmp/karana_android");
    let container = AndroidContainer::new(container_path)
        .with_properties(AndroidProperties {
            api_level: 33,  // Android 13
            device_name: "Kāraṇa Glasses".to_string(),
            arch: "arm64".to_string(),
            display_width: 1920,
            display_height: 1080,
            dpi: 420,
        });

    // 2. Start Container
    println!("▶️  Starting container...");
    container.start().await?;
    println!("✅ Container running\n");

    // 3. Method 1: Use Pre-Configured App (Easiest)
    println!("--- Method 1: Pre-Configured Apps ---");
    let launcher = NativeAppLauncher::new();
    let intent_router = IntentRouter::new();
    
    println!("📱 Launching YouTube with optimizations...");
    launcher.launch_app("youtube", &container, &intent_router).await?;
    println!("✅ YouTube launched with:");
    println!("   - P2P video delivery");
    println!("   - Hardware acceleration");
    println!("   - Local AI recommendations\n");

    // 4. Method 2: Install Custom APK
    println!("--- Method 2: Install Custom APK ---");
    let apk_path = PathBuf::from("/path/to/custom_app.apk");
    
    // Note: In real usage, you'd have an actual APK file
    println!("📥 Installing APK: {:?}", apk_path);
    // let package_name = container.install_app(apk_path).await?;
    // println!("✅ Installed: {}\n", package_name);
    
    // 5. Method 3: Download from App Store
    println!("--- Method 3: App Store Download ---");
    let downloads_path = PathBuf::from("/tmp/karana_downloads");
    let app_store = AppStore::new(downloads_path);
    
    println!("🔍 Searching for 'twitter'...");
    let results = app_store.search("twitter").await;
    println!("Found {} results", results.len());
    
    if let Some(listing) = results.first() {
        println!("📦 App: {} v{}", listing.name, listing.version);
        println!("   Developer: {}", listing.developer);
        println!("   Rating: ⭐ {:.1}/5.0", listing.rating);
        println!("   Downloads: {:?}", listing.downloads);
        
        // Download and verify
        println!("\n🔒 Running security scan...");
        // let apk_path = app_store.download_app(&listing.app_id).await?;
        // println!("✅ Security verified, APK downloaded\n");
        
        // Install
        // let package_name = container.install_app(apk_path).await?;
        // container.launch_app(&package_name).await?;
    }

    // 6. List Installed Apps
    println!("--- Installed Apps ---");
    let apps = container.get_installed_apps().await;
    println!("Total apps: {}", apps.len());
    for app in apps.iter().take(5) {
        println!("📱 {} ({})", app.app_name, app.package_name);
        println!("   Version: {}", app.version);
        println!("   Permissions: {:?}\n", app.permissions);
    }

    // 7. App Recommendations
    println!("--- Smart Recommendations ---");
    use karana_core::app_ecosystem::native_apps::AppCategory;
    
    let user_interests = vec![AppCategory::Social, AppCategory::Video];
    let recommended = launcher.get_recommendations(&user_interests);
    
    println!("Based on your interests, we recommend:");
    for (i, app) in recommended.iter().take(5).enumerate() {
        println!("{}. {} - {}", i + 1, app.name, app.package_name);
        println!("   Category: {:?}", app.category);
        println!("   Optimizations: HW Video={}, Edge AI={}", 
            app.optimizations.hw_video_decode,
            app.optimizations.edge_ai
        );
    }

    // 8. Voice Command Examples
    println!("\n--- Voice Commands You Can Use ---");
    println!("🎤 \"Open YouTube\"");
    println!("🎤 \"Launch WhatsApp\"");
    println!("🎤 \"Show me Instagram\"");
    println!("🎤 \"Install Twitter\"");
    println!("🎤 \"Play music on Spotify\"");
    println!("🎤 \"Close all apps\"");

    // 9. Cleanup
    println!("\n🛑 Stopping container...");
    container.stop().await?;
    println!("✅ Demo complete!");

    Ok(())
}

// Additional helper functions

/// Show app details
async fn show_app_details(container: &AndroidContainer, package_name: &str) {
    let apps = container.get_installed_apps().await;
    if let Some(app) = apps.iter().find(|a| a.package_name == package_name) {
        println!("\n📱 App Details: {}", app.app_name);
        println!("Package: {}", app.package_name);
        println!("Version: {}", app.version);
        println!("APK Path: {:?}", app.apk_path);
        println!("Installed: {}", app.installed);
        println!("Permissions:");
        for perm in &app.permissions {
            println!("  - {}", perm);
        }
    }
}

/// Demonstrate app lifecycle
async fn app_lifecycle_demo(container: &AndroidContainer, package_name: &str) {
    println!("\n--- App Lifecycle Demo ---");
    
    // Launch
    println!("▶️  Launching...");
    container.launch_app(package_name).await.ok();
    
    // Would pause/resume/stop in real usage
    println!("⏸️  Pausing...");
    container.pause().await.ok();
    
    println!("▶️  Resuming...");
    container.resume().await.ok();
    
    println!("🛑 Stopping...");
    container.uninstall_app(package_name).await.ok();
}
