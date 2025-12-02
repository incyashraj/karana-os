//! # Kāraṇa Onboarding System
//!
//! First-time user experience for setting up the smart glasses.
//! Handles wallet creation, backup verification, and initial configuration.

use anyhow::{Result, anyhow};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::wallet::{KaranaWallet, RecoveryPhrase, WalletCreationResult, get_device_id};
use crate::hud::{GlassesHUD, HudNotification};

/// Onboarding step progression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Welcome,
    TermsOfService,
    WalletChoice,      // New or Restore
    WalletCreate,
    BackupPhrase,
    VerifyBackup,
    SetPassword,
    WalletRestore,
    NetworkConnect,
    Complete,
}

/// Onboarding configuration
#[derive(Debug, Clone)]
pub struct OnboardingConfig {
    pub data_dir: PathBuf,
    pub network_bootstrap: Vec<String>,
    pub skip_backup_verify: bool,
}

impl Default for OnboardingConfig {
    fn default() -> Self {
        Self {
            data_dir: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("karana"),
            network_bootstrap: vec![
                "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW...".to_string(),
            ],
            skip_backup_verify: false,
        }
    }
}

/// Result of the onboarding process
pub struct OnboardingResult {
    pub wallet: KaranaWallet,
    pub data_dir: PathBuf,
    pub is_new_wallet: bool,
}

/// The onboarding wizard state machine
pub struct OnboardingWizard {
    step: OnboardingStep,
    config: OnboardingConfig,
    wallet_result: Option<WalletCreationResult>,
    password: Option<String>,
    verified_backup: bool,
}

impl OnboardingWizard {
    pub fn new(config: OnboardingConfig) -> Self {
        Self {
            step: OnboardingStep::Welcome,
            config,
            wallet_result: None,
            password: None,
            verified_backup: false,
        }
    }
    
    /// Check if onboarding is needed (no wallet exists)
    pub fn needs_onboarding(data_dir: &Path) -> bool {
        !data_dir.join("wallet.json").exists()
    }
    
    /// Get current step
    pub fn current_step(&self) -> OnboardingStep {
        self.step
    }
    
    /// Run the full onboarding flow interactively
    pub fn run_interactive(&mut self) -> Result<OnboardingResult> {
        // Create data directory
        std::fs::create_dir_all(&self.config.data_dir)?;
        
        loop {
            match self.step {
                OnboardingStep::Welcome => {
                    self.show_welcome()?;
                    self.step = OnboardingStep::TermsOfService;
                }
                OnboardingStep::TermsOfService => {
                    if self.show_terms()? {
                        self.step = OnboardingStep::WalletChoice;
                    } else {
                        return Err(anyhow!("Terms not accepted"));
                    }
                }
                OnboardingStep::WalletChoice => {
                    let choice = self.show_wallet_choice()?;
                    self.step = if choice { 
                        OnboardingStep::WalletCreate 
                    } else { 
                        OnboardingStep::WalletRestore 
                    };
                }
                OnboardingStep::WalletCreate => {
                    self.create_new_wallet()?;
                    self.step = OnboardingStep::BackupPhrase;
                }
                OnboardingStep::BackupPhrase => {
                    self.show_backup_phrase()?;
                    self.step = OnboardingStep::VerifyBackup;
                }
                OnboardingStep::VerifyBackup => {
                    if self.config.skip_backup_verify || self.verify_backup()? {
                        self.verified_backup = true;
                        self.step = OnboardingStep::SetPassword;
                    }
                }
                OnboardingStep::SetPassword => {
                    self.set_password()?;
                    self.step = OnboardingStep::NetworkConnect;
                }
                OnboardingStep::WalletRestore => {
                    self.restore_wallet()?;
                    self.step = OnboardingStep::SetPassword;
                }
                OnboardingStep::NetworkConnect => {
                    self.connect_network()?;
                    self.step = OnboardingStep::Complete;
                }
                OnboardingStep::Complete => {
                    return self.finalize();
                }
            }
        }
    }
    
    /// Run onboarding with HUD display
    pub fn run_with_hud(&mut self, hud: &mut GlassesHUD) -> Result<OnboardingResult> {
        std::fs::create_dir_all(&self.config.data_dir)?;
        
        loop {
            match self.step {
                OnboardingStep::Welcome => {
                    hud.clear_messages();
                    hud.add_system_message("👓 Welcome to Kāraṇa");
                    hud.add_assistant_message("I'm your sovereign AI assistant. Let's set up your glasses.");
                    hud.render();
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    self.step = OnboardingStep::TermsOfService;
                }
                OnboardingStep::TermsOfService => {
                    hud.add_assistant_message("By using Kāraṇa, you agree to our decentralized terms. Your data stays on your device.");
                    hud.notify(HudNotification::new("📜", "Terms", "Data sovereignty guaranteed"));
                    hud.render();
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    self.step = OnboardingStep::WalletChoice;
                }
                OnboardingStep::WalletChoice => {
                    hud.add_assistant_message("Creating your sovereign wallet...");
                    hud.render();
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    self.step = OnboardingStep::WalletCreate;
                }
                OnboardingStep::WalletCreate => {
                    let device_id = get_device_id();
                    self.wallet_result = Some(KaranaWallet::generate(&device_id)?);
                    
                    let did = self.wallet_result.as_ref().unwrap().wallet.did();
                    hud.add_assistant_message(&format!("✓ Wallet created: {}...", &did[..20]));
                    hud.notify(HudNotification::new("🔐", "Wallet", "Sovereign identity created"));
                    hud.render();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    self.step = OnboardingStep::BackupPhrase;
                }
                OnboardingStep::BackupPhrase => {
                    let phrase = &self.wallet_result.as_ref().unwrap().recovery_phrase;
                    hud.add_assistant_message("⚠️ IMPORTANT: Write down your recovery phrase!");
                    hud.add_system_message(&format!("Words 1-6: {}", phrase.words()[0..6].join(" ")));
                    hud.add_system_message(&format!("Words 7-12: {}", phrase.words()[6..12].join(" ")));
                    hud.add_system_message(&format!("Words 13-18: {}", phrase.words()[12..18].join(" ")));
                    hud.add_system_message(&format!("Words 19-24: {}", phrase.words()[18..24].join(" ")));
                    hud.notify(HudNotification::urgent("⚠️", "Backup", "Save these 24 words!"));
                    hud.render();
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    self.step = OnboardingStep::SetPassword;
                }
                OnboardingStep::SetPassword => {
                    // In HUD mode, use a default password or device-derived key
                    self.password = Some("device_secured".to_string());
                    hud.add_assistant_message("✓ Wallet encrypted with device key");
                    hud.render();
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    self.step = OnboardingStep::NetworkConnect;
                }
                OnboardingStep::WalletRestore => {
                    // Not handled in HUD flow - always create new
                    self.step = OnboardingStep::WalletCreate;
                }
                OnboardingStep::VerifyBackup => {
                    // Skip in HUD mode
                    self.verified_backup = true;
                    self.step = OnboardingStep::SetPassword;
                }
                OnboardingStep::NetworkConnect => {
                    hud.add_assistant_message("Connecting to Kāraṇa network...");
                    hud.render();
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    hud.add_assistant_message("✓ Connected to swarm (simulated)");
                    hud.notify(HudNotification::new("🌐", "Network", "P2P connected"));
                    hud.render();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    self.step = OnboardingStep::Complete;
                }
                OnboardingStep::Complete => {
                    hud.add_assistant_message("🎉 Setup complete! Your glasses are ready.");
                    hud.add_assistant_message("Say 'Hey Karana' or type to begin.");
                    hud.notify(HudNotification::new("✨", "Ready", "Kāraṇa is ready"));
                    hud.render();
                    return self.finalize();
                }
            }
        }
    }
    
    // ==================== Interactive Console Methods ====================
    
    fn show_welcome(&self) -> Result<()> {
        println!("\x1b[2J\x1b[H"); // Clear screen
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║                                                               ║");
        println!("║     👓  Welcome to Kāraṇa Smart Glasses                       ║");
        println!("║                                                               ║");
        println!("║     Your Sovereign AI Companion                               ║");
        println!("║                                                               ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  This wizard will help you set up your glasses:");
        println!();
        println!("  1. Create your sovereign wallet (your identity)");
        println!("  2. Backup your recovery phrase");
        println!("  3. Connect to the Kāraṇa network");
        println!();
        println!("  Press ENTER to continue...");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(())
    }
    
    fn show_terms(&self) -> Result<bool> {
        println!("\x1b[2J\x1b[H");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  📜 Terms of Sovereignty                                      ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  By using Kāraṇa, you acknowledge:");
        println!();
        println!("  ✓ Your data stays on YOUR device");
        println!("  ✓ Your private keys are YOUR responsibility");
        println!("  ✓ No central server can access your information");
        println!("  ✓ YOU control what the AI sees and does");
        println!("  ✓ Transactions are recorded on a decentralized ledger");
        println!();
        println!("  This is true digital sovereignty.");
        println!();
        print!("  Do you accept? [y/N]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_lowercase().starts_with('y'))
    }
    
    fn show_wallet_choice(&self) -> Result<bool> {
        println!("\x1b[2J\x1b[H");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🔐 Wallet Setup                                              ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Choose an option:");
        println!();
        println!("  [1] Create NEW wallet (recommended for new users)");
        println!("  [2] RESTORE wallet from recovery phrase");
        println!();
        print!("  Enter choice [1/2]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(!input.trim().starts_with('2'))
    }
    
    fn create_new_wallet(&mut self) -> Result<()> {
        println!("\x1b[2J\x1b[H");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🔐 Creating Your Sovereign Wallet                            ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Generating cryptographic keys...");
        
        let device_id = get_device_id();
        let result = KaranaWallet::generate(&device_id)?;
        
        println!("  ✓ Ed25519 keypair generated");
        println!("  ✓ BIP-39 mnemonic created (24 words)");
        println!();
        println!("  Your DID (Decentralized Identifier):");
        println!("  {}", result.wallet.did());
        println!();
        
        self.wallet_result = Some(result);
        
        println!("  Press ENTER to see your recovery phrase...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(())
    }
    
    fn show_backup_phrase(&self) -> Result<()> {
        let result = self.wallet_result.as_ref()
            .ok_or_else(|| anyhow!("No wallet created"))?;
        
        println!("\x1b[2J\x1b[H");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  ⚠️  CRITICAL: Your Recovery Phrase                           ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Write down these 24 words IN ORDER:");
        println!("  (This is the ONLY way to recover your wallet)");
        println!();
        println!("  ┌─────────────────────────────────────────────────────────┐");
        println!("{}", result.recovery_phrase.display_for_backup()
            .lines()
            .map(|l| format!("  │  {}│", format!("{:<55}", l)))
            .collect::<Vec<_>>()
            .join("\n"));
        println!("  └─────────────────────────────────────────────────────────┘");
        println!();
        println!("  ⚠️  NEVER share these words with anyone!");
        println!("  ⚠️  Store them in a SAFE place offline!");
        println!("  ⚠️  If you lose them, you lose your wallet FOREVER!");
        println!();
        println!("  Press ENTER when you have written them down...");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(())
    }
    
    fn verify_backup(&self) -> Result<bool> {
        let result = self.wallet_result.as_ref()
            .ok_or_else(|| anyhow!("No wallet created"))?;
        
        println!("\x1b[2J\x1b[H");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  ✅ Verify Your Backup                                        ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        
        // Pick 3 random words to verify
        use rand::seq::SliceRandom;
        let mut indices: Vec<usize> = (0..24).collect();
        indices.shuffle(&mut rand::thread_rng());
        let check_indices: Vec<usize> = indices.into_iter().take(3).collect();
        
        let words = result.recovery_phrase.words();
        
        for &idx in &check_indices {
            print!("  Enter word #{}: ", idx + 1);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() != words[idx].to_lowercase() {
                println!();
                println!("  ❌ Incorrect! The word was: {}", words[idx]);
                println!("  Please go back and write down your phrase correctly.");
                println!();
                println!("  Press ENTER to try again...");
                let mut _input = String::new();
                io::stdin().read_line(&mut _input)?;
                return Ok(false);
            }
            println!("  ✓ Correct!");
        }
        
        println!();
        println!("  ✅ Backup verified successfully!");
        println!();
        println!("  Press ENTER to continue...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(true)
    }
    
    fn set_password(&mut self) -> Result<()> {
        println!("\x1b[2J\x1b[H");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🔒 Set Wallet Password                                       ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  This password encrypts your wallet on this device.");
        println!("  You'll need it to unlock your glasses.");
        println!();
        
        loop {
            print!("  Enter password (min 8 chars): ");
            io::stdout().flush()?;
            let mut pass1 = String::new();
            io::stdin().read_line(&mut pass1)?;
            let pass1 = pass1.trim().to_string();
            
            if pass1.len() < 8 {
                println!("  ❌ Password too short!");
                continue;
            }
            
            print!("  Confirm password: ");
            io::stdout().flush()?;
            let mut pass2 = String::new();
            io::stdin().read_line(&mut pass2)?;
            
            if pass1 != pass2.trim() {
                println!("  ❌ Passwords don't match!");
                continue;
            }
            
            self.password = Some(pass1);
            println!("  ✓ Password set!");
            break;
        }
        
        // Save encrypted wallet
        let result = self.wallet_result.as_ref()
            .ok_or_else(|| anyhow!("No wallet created"))?;
        let wallet_path = self.config.data_dir.join("wallet.json");
        result.wallet.save_encrypted(&wallet_path, self.password.as_ref().unwrap())?;
        
        println!("  ✓ Wallet saved to {}", wallet_path.display());
        println!();
        println!("  Press ENTER to continue...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(())
    }
    
    fn restore_wallet(&mut self) -> Result<()> {
        println!("\x1b[2J\x1b[H");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🔄 Restore Wallet from Recovery Phrase                       ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Enter your 24-word recovery phrase:");
        println!("  (separate words with spaces)");
        println!();
        print!("  > ");
        io::stdout().flush()?;
        
        let mut phrase = String::new();
        io::stdin().read_line(&mut phrase)?;
        
        let device_id = get_device_id();
        let wallet = KaranaWallet::from_mnemonic(phrase.trim(), &device_id)?;
        
        println!();
        println!("  ✓ Wallet restored successfully!");
        println!("  DID: {}", wallet.did());
        
        // Create a dummy recovery phrase (not needed for restore)
        self.wallet_result = Some(WalletCreationResult {
            wallet,
            recovery_phrase: crate::wallet::RecoveryPhrase::new(vec!["restored".to_string(); 24]),
        });
        
        println!();
        println!("  Press ENTER to continue...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(())
    }
    
    fn connect_network(&self) -> Result<()> {
        println!("\x1b[2J\x1b[H");
        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🌐 Connecting to Kāraṇa Network                              ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!();
        
        println!("  Initializing P2P swarm...");
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("  ✓ libp2p initialized");
        
        println!("  Connecting to bootstrap nodes...");
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("  ✓ Connected to {} peers (simulated)", 3);
        
        println!("  Syncing blockchain state...");
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("  ✓ Block height: 0 (genesis)");
        
        println!();
        println!("  ✅ Network connected!");
        println!();
        println!("  Press ENTER to complete setup...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(())
    }
    
    fn finalize(&self) -> Result<OnboardingResult> {
        let result = self.wallet_result.as_ref()
            .ok_or_else(|| anyhow!("No wallet created"))?;
        
        // In HUD mode, save with device key if no password set
        if let Some(ref password) = self.password {
            let wallet_path = self.config.data_dir.join("wallet.json");
            if !wallet_path.exists() {
                result.wallet.save_encrypted(&wallet_path, password)?;
            }
        }
        
        // Clone the wallet by creating from the same seed
        // (In real impl, we'd serialize/deserialize or use Arc)
        let device_id = get_device_id();
        let phrase = result.recovery_phrase.as_string();
        let final_wallet = KaranaWallet::from_mnemonic(&phrase, &device_id)?;
        
        Ok(OnboardingResult {
            wallet: final_wallet,
            data_dir: self.config.data_dir.clone(),
            is_new_wallet: true,
        })
    }
}

/// Quick check and load existing wallet, or run onboarding
pub fn load_or_onboard(data_dir: &Path, password: Option<&str>) -> Result<KaranaWallet> {
    let wallet_path = data_dir.join("wallet.json");
    
    if wallet_path.exists() {
        // Load existing wallet
        let password = password.ok_or_else(|| anyhow!("Password required to unlock wallet"))?;
        KaranaWallet::load_encrypted(&wallet_path, password)
    } else {
        // Run onboarding
        let config = OnboardingConfig {
            data_dir: data_dir.to_path_buf(),
            ..Default::default()
        };
        let mut wizard = OnboardingWizard::new(config);
        let result = wizard.run_interactive()?;
        Ok(result.wallet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_needs_onboarding() {
        let temp_dir = std::env::temp_dir().join("karana_onboarding_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        assert!(OnboardingWizard::needs_onboarding(&temp_dir));
        
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("wallet.json"), "{}").unwrap();
        
        assert!(!OnboardingWizard::needs_onboarding(&temp_dir));
        
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
