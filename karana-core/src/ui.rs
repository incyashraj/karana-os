pub mod gui;
pub mod input;
pub mod render_proof;
pub mod theme;

use actix::prelude::*;
use crate::runtime::KaranaActor;
use crate::net::KaranaSwarm;
use crate::ai::KaranaAI;
use crate::gov::KaranaDAO;
use crate::market::KaranaBazaar;
use crate::pkg::AppBundle;
use std::sync::{Arc, Mutex, mpsc};
use std::process::Command;
use std::fs;
use alloy_primitives::U256;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui_logger::{TuiLoggerWidget, TuiLoggerLevelOutput};

#[derive(Message)]
#[rtype(result = "String")]
pub struct RenderIntent {
    pub input: String,
    pub proof: Vec<u8>,
}

pub struct UiState {
    pub balance: u64,
    pub block_height: u64,
    pub current_view: String,
    pub last_intent: String,
    pub active_app: Option<String>,
    pub app_output: String,
    pub cwd: String,
}

pub struct KaranaUI {
    swarm: KaranaSwarm,
    ai_render: Arc<Mutex<KaranaAI>>,
    #[allow(dead_code)]
    runtime: Arc<KaranaActor>,
    dao: Arc<Mutex<KaranaDAO>>,
    market: Arc<Mutex<KaranaBazaar>>,
    bundle: AppBundle,
    state: Arc<Mutex<UiState>>,
    intent_rx: Mutex<mpsc::Receiver<String>>,
}

impl KaranaUI {
    pub fn new(runtime: &Arc<KaranaActor>, swarm: &KaranaSwarm, ai: Arc<Mutex<KaranaAI>>) -> anyhow::Result<Self> {
        let mut dao = KaranaDAO::default();
        // Test mint for user
        dao.token.mint("user", U256::from(200u64));

        let market = Arc::new(Mutex::new(KaranaBazaar::new(ai.clone())));
        let bundle = AppBundle::new();

        let state = Arc::new(Mutex::new(UiState {
            balance: 200,
            block_height: 0,
            current_view: "System Ready".to_string(),
            last_intent: String::new(),
            active_app: None,
            app_output: String::new(),
            cwd: std::env::current_dir().unwrap_or_default().to_string_lossy().to_string(),
        }));

        let (tx, rx) = mpsc::channel();

        // Spawn TUI thread
        let state_clone = state.clone();
        std::thread::spawn(move || {
            if let Err(e) = run_tui(state_clone, tx) {
                log::error!("TUI Error: {}", e);
            }
        });

        Ok(Self {
            swarm: swarm.clone(),
            ai_render: ai,
            runtime: runtime.clone(),
            dao: Arc::new(Mutex::new(dao)),
            market,
            bundle,
            state,
            intent_rx: Mutex::new(rx),
        })
    }

    pub fn poll_intent(&self) -> Option<String> {
        if let Ok(rx) = self.intent_rx.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }

    pub fn update_height(&self, height: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.block_height = height;
        }
    }

    pub async fn render_intent(&self, input: String, proof: Vec<u8>) -> anyhow::Result<String> {
        // Step 0: Handle Active App Interaction
        {
            let mut state = self.state.lock().unwrap();
            if let Some(app) = state.active_app.clone() {
                if input == "exit" || input == "close" {
                    state.active_app = None;
                    state.current_view = "System Ready".to_string();
                    return Ok("Closed application.".to_string());
                }

                if app == "terminal" {
                    let output = match Command::new("sh").arg("-c").arg(&input).output() {
                        Ok(o) => {
                            let stdout = String::from_utf8_lossy(&o.stdout);
                            let stderr = String::from_utf8_lossy(&o.stderr);
                            format!("$ {}\n{}{}", input, stdout, stderr)
                        }
                        Err(e) => format!("$ {}\nError: {}", input, e),
                    };
                    state.app_output.push_str(&format!("\n{}", output));
                    // Keep last 20 lines
                    let lines: Vec<String> = state.app_output.lines().rev().take(20).map(String::from).collect();
                    state.app_output = lines.into_iter().rev().collect::<Vec<String>>().join("\n");
                    return Ok(output);
                } else if app == "files" {
                    if input.starts_with("cd ") {
                        let target = input.replace("cd ", "").trim().to_string();
                        let new_path = if target == ".." {
                            std::path::Path::new(&state.cwd).parent().map(|p| p.to_path_buf()).unwrap_or(std::path::PathBuf::from("/"))
                        } else {
                            std::path::Path::new(&state.cwd).join(target)
                        };
                        
                        if new_path.exists() && new_path.is_dir() {
                            state.cwd = new_path.to_string_lossy().to_string();
                        }
                    }
                    
                    if let Ok(entries) = fs::read_dir(&state.cwd) {
                        let mut list = String::new();
                        list.push_str(&format!("Directory: {}\n\n", state.cwd));
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let type_str = if entry.path().is_dir() { "[DIR]" } else { "[FILE]" };
                            list.push_str(&format!("{} {}\n", type_str, name));
                        }
                        state.app_output = list;
                    }
                    return Ok("Updated file view".to_string());
                }
            }
        }

        // Step 1: ZK-Verify Input (Stub)
        if !verify_zk_proof(&proof, input.as_bytes()) { 
            return Err(anyhow::anyhow!("Invalid intent!")); 
        }

        let view = if input == "exit" || input == "close" {
             if let Ok(mut state) = self.state.lock() {
                state.active_app = None;
                state.current_view = "System Ready".to_string();
            }
            "Closed application.".to_string()
        } else if input.starts_with("find app") || input.starts_with("search app") {
            let query = input.replace("find app", "").replace("search app", "").trim().to_string();
            let results = {
                let market = self.market.lock().unwrap();
                market.search(&query)
            };
            
            let mut result_view = format!("Bazaar Search Results for '{}':\n", query);
            for app in results {
                result_view.push_str(&format!("- {} (ID: {}) [Rating: {:.1} | Stake: {} KARA]\n  {}\n", 
                    app.name, app.id, app.rating, app.stake, app.description));
            }
            
            // Also search bundle
            let bundle_matches: Vec<String> = self.bundle.list().into_iter()
                .filter(|n| n.contains(&query))
                .collect();
            
            if !bundle_matches.is_empty() {
                result_view.push_str("\nBundled Apps (Instant Install):\n");
                for app in bundle_matches {
                    result_view.push_str(&format!("- {} [Verified Bundle]\n", app));
                }
            }

            if result_view.lines().count() <= 1 {
                result_view.push_str("No apps found matching your intent.");
            }
            result_view
        } else if input.starts_with("install") || input.starts_with("download") || input.starts_with("get") {
            let app_id = input.replace("install", "")
                              .replace("download", "")
                              .replace("get", "")
                              .trim().to_string();
            
            // Try Market first
            let market_result = {
                let mut market = self.market.lock().unwrap();
                market.install(&app_id)
            };

            match market_result {
                Ok(s) => s,
                Err(_) => {
                    // Try Bundle if Market fails
                    // Fuzzy match for bundle
                    let normalized_input = app_id.to_lowercase().replace(" ", "").replace("-", "");
                    let bundle_id = self.bundle.list().into_iter()
                        .find(|n| {
                            let normalized_name = n.to_lowercase().replace(" ", "").replace("-", "");
                            normalized_name == normalized_input || normalized_name.contains(&normalized_input) || normalized_input.contains(&normalized_name)
                        })
                        .unwrap_or(app_id.clone());

                    match self.bundle.install(&bundle_id) {
                        Ok(s) => s,
                        Err(e) => format!("Installation Failed: App '{}' not found in Bazaar or Bundle. ({})", app_id, e),
                    }
                }
            }
        } else if input.starts_with("run") || input.starts_with("open") || input.starts_with("launch") || input.starts_with("use") {
             let app_id = input.replace("run", "")
                               .replace("open", "")
                               .replace("launch", "")
                               .replace("use", "")
                               .trim().to_string();
             
             let mut state = self.state.lock().unwrap();
             state.active_app = Some(app_id.clone());

             if app_id == "terminal" {
                 state.app_output = "Karana Terminal v1.0\nType 'exit' to close.\n".to_string();
             } else if app_id == "files" {
                 state.app_output = format!("File Manager\nCWD: {}\n(Type 'cd <dir>' to navigate)", state.cwd);
                 // Initial list
                 if let Ok(entries) = fs::read_dir(&state.cwd) {
                        let mut list = String::new();
                        list.push_str(&format!("Directory: {}\n\n", state.cwd));
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let type_str = if entry.path().is_dir() { "[DIR]" } else { "[FILE]" };
                            list.push_str(&format!("{} {}\n", type_str, name));
                        }
                        state.app_output = list;
                 }
             } else if app_id.contains("code") || app_id.contains("edit") {
                 state.app_output = format!("File: main.rs\n\nfn main() {{\n    println!(\"Hello from {}!\");\n}}\n\n[TERMINAL]\n$ cargo run\nCompiling...\nFinished dev [unoptimized + debuginfo]\nRunning target/debug/hello", app_id);
             } else if app_id.contains("browser") || app_id.contains("web") {
                 state.app_output = "URL: https://karana.io\n\n[ Kāraṇa Decentralized Web ]\n\nWelcome to the new internet.\nNo servers. No censors.\n\n[Links]\n- Bazaar\n- Governance\n- Wallet".to_string();
             } else {
                 state.app_output = format!("Running {}...\n\n[App Interface Placeholder]\n\n(This is a simulated TUI view for {})", app_id, app_id);
             };
             
             format!("Launched {}", app_id)
        } else if input.contains("list apps") || input.contains("show apps") {
            let mut view = String::from("Available Apps (Bundle & Bazaar):\n");
            for app in self.bundle.list() {
                view.push_str(&format!("- {} [Bundle]\n", app));
            }
            let market = self.market.lock().unwrap();
            for app in market.search("") { 
                 view.push_str(&format!("- {} [Bazaar]\n", app.name));
            }
            view
        } else if input.contains("help") {
            let tutorial = {
                let mut ai = self.ai_render.lock().unwrap();
                ai.predict(&format!("Tutorial for: {}", input), 100).unwrap_or_else(|_| "Help unavailable.".to_string())
            };
            format!("Symbiotic Guide:\n{}", tutorial)
        } else if input.starts_with("report bug") {
             let proof = input.replace("report bug", "").trim().to_string();
             let (prop_id, bounty) = {
                 let mut dao = self.dao.lock().unwrap();
                 let id = dao.propose_bounty(&proof, 5); 
                 (id, 50)
             };
             format!("Bug Reported! Proposal ID: {}. Potential Bounty: {} KARA. DAO is voting...", prop_id, bounty)
        } else {
            // Step 2: AI Parse & Render (Symbiotic Canvas)
            let prompt = format!("Describe a UI view for this user intent: '{}'. Keep it short.", input);
            let view_desc = {
                let mut ai = self.ai_render.lock().unwrap();
                ai.predict(&prompt, 30).unwrap_or_else(|_| "Standard View".to_string())
            };
            
            let view_type = if view_desc.contains("Graph") || view_desc.contains("chart") {
                "Graph View"
            } else if view_desc.contains("List") || view_desc.contains("table") {
                "List View"
            } else {
                "Adaptive View"
            };
            
            format!("AI Generated View [{}]: {}", view_type, view_desc.trim())
        };

        log::info!("Atom 6 (UI): Symbiotic Render: {}", view);

        // Update TUI State
        if let Ok(mut state) = self.state.lock() {
            state.current_view = view.clone();
            state.last_intent = input.clone();
        }

        // Step 3: Swarm Sync Context
        self.swarm.broadcast_ui_update(&view, &proof).await?;

        // Phase 6: Live DAO Integration (Boot Sim)
        if input.contains("optimize storage") || input.contains("boot") {
             let prop_id = {
                let mut dao = self.dao.lock().unwrap();
                dao.propose(&format!("AI Suggestion: {}", view.chars().take(20).collect::<String>()), "Live vote on boot")
            };
            log::info!("BOOT DAO: Vote yes/no on Prop {}? (Sim: yes)", prop_id);
            
            let mut dao = self.dao.lock().unwrap();
            if dao.vote("user", prop_id, true).unwrap() {
                dao.execute_if_passed(prop_id, &mut |id| {
                    log::info!("Executed: Prop {} live – Runtime boosted!", id);
                });
            }
        }

        Ok(view)
    }

    #[allow(dead_code)]
    fn propose_dao_nudge(&self, view: &str, input: &str) {
        log::info!("Atom 6 (UI): DAO Proposal: Better layout '{}' for '{}'? Stake KARA to vote!", view, input);
    }
}

fn verify_zk_proof(_proof: &[u8], _data: &[u8]) -> bool {
    true
}

fn run_tui(state: Arc<Mutex<UiState>>, intent_tx: mpsc::Sender<String>) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Boot Animation
    let logo = vec![
        "  _  __   _   ___   _   _  _   _   ",
        " | |/ /  /_\ | _ \ /_\ | \| | /_\  ",
        " | ' <  / _ \|   // _ \| .` |/ _ \ ",
        " |_|\_\/_/ \_\_|_/_/ \_\_|\_/_/ \_\",
        "                                   ",
        "      The Sovereign AI-Native OS   "
    ];

    let start = std::time::Instant::now();
    let duration = std::time::Duration::from_secs(3);
    
    while start.elapsed() < duration {
        terminal.draw(|f| {
             let size = f.area();
             let block = Block::default().borders(Borders::ALL);
             f.render_widget(block, size);
             
             // Center the logo
             let v_center = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(logo.len() as u16),
                    Constraint::Percentage(40),
                ].as_ref())
                .split(size);
                
             let h_center = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10),
                ].as_ref())
                .split(v_center[1]);
                
             let logo_text = logo.join("\n");
             let p = Paragraph::new(logo_text)
                .alignment(Alignment::Center);
             f.render_widget(p, h_center[1]);
             
             // Loading bar
             let loading_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                ].as_ref())
                .split(v_center[2]);
                
             let elapsed = start.elapsed().as_millis() as f64;
             let total = duration.as_millis() as f64;
             let progress = (elapsed / total).min(1.0);
             
             let bar_width = (size.width as f64 * 0.4 * progress) as usize;
             let bar = "█".repeat(bar_width);
             let bar_p = Paragraph::new(bar).alignment(Alignment::Center);
             
             let loading_text = Paragraph::new(format!("Booting Sovereign Monad... {:.0}%", progress * 100.0))
                .alignment(Alignment::Center);
                
             f.render_widget(loading_text, loading_area[0]);
             f.render_widget(bar_p, loading_area[1]);
        })?;
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    let mut input_buffer = String::new();

    // TUI Loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),      // Header
                        Constraint::Percentage(40), // User Output
                        Constraint::Percentage(40), // Chain Logs
                        Constraint::Length(3),      // Input
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let (balance, height, view, intent, active_app, app_output) = {
                let s = state.lock().unwrap();
                (s.balance, s.block_height, s.current_view.clone(), s.last_intent.clone(), s.active_app.clone(), s.app_output.clone())
            };

            // 1. Header
            let header_text = format!(
                "Kāraṇa OS v0.1 (Sovereign) | Balance: {} KARA | Height: #{}",
                balance, height
            );
            let header = Paragraph::new(header_text)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // 2. User Output / Dashboard
            let title = if let Some(app) = active_app.clone() {
                format!(" Running: {} (Type 'close' to exit) ", app)
            } else {
                " Symbiotic Interface ".to_string()
            };

            let content = if active_app.is_some() {
                app_output
            } else {
                view
            };

            let output = Paragraph::new(content)
                .block(Block::default().title(title).borders(Borders::ALL))
                .wrap(Wrap { trim: true });
            f.render_widget(output, chunks[1]);

            // 3. Logs (using tui-logger)
            let logs = TuiLoggerWidget::default()
                .block(Block::default().title(" Consensus Stream ").borders(Borders::ALL))
                .output_separator('|')
                .output_timestamp(Some("%H:%M:%S".to_string()))
                .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
                .output_target(false)
                .output_file(false)
                .output_line(false);
            f.render_widget(logs, chunks[2]);

            // 4. Intent / Input
            let input_text = format!("Last Intent: {}\n> {}_", intent, input_buffer);
            let input = Paragraph::new(input_text)
                .block(Block::default().title(" User Intent ").borders(Borders::ALL))
                .wrap(Wrap { trim: true });
            f.render_widget(input, chunks[3]);
        })?;

        // Event Handling
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        let _ = intent_tx.send("quit".to_string());
                        break;
                    }
                    KeyCode::Char(c) => {
                        input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        input_buffer.pop();
                    }
                    KeyCode::Enter => {
                        if !input_buffer.is_empty() {
                            if let Ok(mut s) = state.lock() {
                                s.current_view = format!("Processing intent: '{}'...", input_buffer);
                                s.last_intent = input_buffer.clone();
                            }
                            let _ = intent_tx.send(input_buffer.clone());
                            input_buffer.clear();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
