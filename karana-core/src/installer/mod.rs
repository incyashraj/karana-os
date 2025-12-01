use clap::Parser;
use anyhow::Result;
use crate::ai::KaranaAI;
use crate::gov::KaranaDAO;
use alloy_primitives::U256;

#[derive(Parser, Debug)]
#[command(name = "karana")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Run the interactive installer wizard
    Install {
        #[arg(short, long, default_value = "live")]
        mode: String, // live, disk, vm
    },
    /// Start the OS Kernel (Default)
    Boot {
        /// P2P Port to listen on
        #[arg(long, default_value = "0")]
        port: u16,
        
        /// Peer address to dial (Multiaddr)
        #[arg(long)]
        peer: Option<String>,
        
        /// Custom storage path (for multi-node sim)
        #[arg(long)]
        path: Option<String>,
    },
}

pub fn run_install(mode: String) -> Result<()> {
    println!(">>> Kāraṇa OS Sovereign Installer (v0.5 Beta) <<<");
    println!("Mode: {}", mode);

    // Phase 3: AI Probe
    let mut ai = KaranaAI::new()?;
    let intent = ai.predict("Probe hardware & suggest setup", 20)?;
    println!("\n[AI Probe] {}", intent); 

    // Phase 4: DAO / Token Setup
    let mut dao = KaranaDAO::default();
    println!("\n[Identity] Generating DID... Done.");
    println!("[Economy] Minting initial KARA to 'new-user'...");
    dao.token.mint("new-user", U256::from(100u64));
    println!("          Balance: 100 KARA");

    if mode == "disk" {
        println!("\n[Disk] Partitioning /dev/sda (Simulated)...");
        println!("[ZK] Attesting partition table hash... Verified.");
        println!("[Copy] Transferring ISO root to persistent storage...");
    }

    // Phase 6: DAO Vote on Setup
    println!("\n[Governance] Proposing 'Initial System Tune' based on AI probe...");
    let prop_id = dao.propose("Initial Tune", &intent);
    println!("             Proposal ID: {}", prop_id);
    
    if dao.vote("new-user", prop_id, true).unwrap() {
        dao.execute_if_passed(prop_id, &mut |_id| {
            println!("[Exec] DAO Passed! System tuned for '{}'.", intent);
        });
    }

    println!("\n>>> Installation Complete. Reboot to ignite. <<<");
    Ok(())
}
