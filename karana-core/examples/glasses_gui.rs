//! Kāraṇa OS Smart Glasses - Modern Graphical UI Simulator
//!
//! A beautiful, interactive GUI that simulates the AR glasses experience.
//! Run with: cargo run --example glasses_gui --features gui

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("This example requires the 'gui' feature. Run with: cargo run --example glasses_gui --features gui");
}

#[cfg(feature = "gui")]
use eframe::egui;
#[cfg(feature = "gui")]
use std::time::{Duration, Instant};

#[cfg(feature = "gui")]
fn main() -> eframe::Result<()> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Kāraṇa OS - Smart Glasses Simulator")
            .with_decorations(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Kāraṇa OS Glasses Simulator",
        options,
        Box::new(|cc| {
            // Set up custom fonts and styling
            setup_custom_style(&cc.egui_ctx);
            Ok(Box::new(GlassesSimulator::new()))
        }),
    )
}

#[cfg(feature = "gui")]
fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Modern, sleek styling
    style.visuals.window_rounding = egui::Rounding::same(12.0);
    style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(20, 20, 30, 240);
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 35, 50);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 65);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 90);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 100, 140);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(100, 150, 255);
    
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    
    ctx.set_style(style);
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "gui")]
enum Scene {
    Home,
    Kitchen,
    Office,
    Street,
    Park,
}

#[cfg(feature = "gui")]
impl Scene {
    fn name(&self) -> &str {
        match self {
            Scene::Home => "Living Room",
            Scene::Kitchen => "Kitchen",
            Scene::Office => "Office",
            Scene::Street => "Street",
            Scene::Park => "Park",
        }
    }
    
    fn emoji(&self) -> &str {
        match self {
            Scene::Home => "🏠",
            Scene::Kitchen => "🍳",
            Scene::Office => "💼",
            Scene::Street => "🛣️",
            Scene::Park => "🌳",
        }
    }
    
    fn objects(&self) -> Vec<DetectedObject> {
        match self {
            Scene::Home => vec![
                DetectedObject::new("Couch", 0.96, ObjectCategory::Furniture),
                DetectedObject::new("TV", 0.94, ObjectCategory::Electronics),
                DetectedObject::new("Coffee Table", 0.92, ObjectCategory::Furniture),
                DetectedObject::new("Lamp", 0.89, ObjectCategory::Electronics),
                DetectedObject::new("Book", 0.85, ObjectCategory::Item),
            ],
            Scene::Kitchen => vec![
                DetectedObject::new("Apple", 0.97, ObjectCategory::Food),
                DetectedObject::new("Frying Pan", 0.95, ObjectCategory::Kitchenware),
                DetectedObject::new("Knife", 0.93, ObjectCategory::Kitchenware),
                DetectedObject::new("Cutting Board", 0.90, ObjectCategory::Kitchenware),
                DetectedObject::new("Coffee Maker", 0.88, ObjectCategory::Electronics),
            ],
            Scene::Office => vec![
                DetectedObject::new("Laptop", 0.98, ObjectCategory::Electronics),
                DetectedObject::new("Monitor", 0.96, ObjectCategory::Electronics),
                DetectedObject::new("Keyboard", 0.94, ObjectCategory::Electronics),
                DetectedObject::new("Coffee Mug", 0.91, ObjectCategory::Item),
                DetectedObject::new("Notebook", 0.87, ObjectCategory::Item),
            ],
            Scene::Street => vec![
                DetectedObject::new("Car", 0.95, ObjectCategory::Vehicle),
                DetectedObject::new("Traffic Light", 0.93, ObjectCategory::Infrastructure),
                DetectedObject::new("Pedestrian", 0.90, ObjectCategory::Person),
                DetectedObject::new("Store Sign", 0.88, ObjectCategory::Text),
                DetectedObject::new("Tree", 0.85, ObjectCategory::Nature),
            ],
            Scene::Park => vec![
                DetectedObject::new("Dog", 0.94, ObjectCategory::Animal),
                DetectedObject::new("Bench", 0.92, ObjectCategory::Furniture),
                DetectedObject::new("Person Jogging", 0.89, ObjectCategory::Person),
                DetectedObject::new("Fountain", 0.86, ObjectCategory::Infrastructure),
                DetectedObject::new("Flower Bed", 0.83, ObjectCategory::Nature),
            ],
        }
    }
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
#[cfg(feature = "gui")]
enum ObjectCategory {
    Furniture,
    Electronics,
    Food,
    Kitchenware,
    Item,
    Vehicle,
    Infrastructure,
    Person,
    Text,
    Nature,
    Animal,
}

#[cfg(feature = "gui")]
impl ObjectCategory {
    fn color(&self) -> egui::Color32 {
        match self {
            ObjectCategory::Furniture => egui::Color32::from_rgb(139, 90, 43),
            ObjectCategory::Electronics => egui::Color32::from_rgb(100, 149, 237),
            ObjectCategory::Food => egui::Color32::from_rgb(50, 205, 50),
            ObjectCategory::Kitchenware => egui::Color32::from_rgb(192, 192, 192),
            ObjectCategory::Item => egui::Color32::from_rgb(255, 215, 0),
            ObjectCategory::Vehicle => egui::Color32::from_rgb(255, 99, 71),
            ObjectCategory::Infrastructure => egui::Color32::from_rgb(128, 128, 128),
            ObjectCategory::Person => egui::Color32::from_rgb(255, 182, 193),
            ObjectCategory::Text => egui::Color32::from_rgb(255, 255, 255),
            ObjectCategory::Nature => egui::Color32::from_rgb(34, 139, 34),
            ObjectCategory::Animal => egui::Color32::from_rgb(255, 165, 0),
        }
    }
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
#[cfg(feature = "gui")]
struct DetectedObject {
    name: String,
    confidence: f32,
    category: ObjectCategory,
    bounding_box: Option<(f32, f32, f32, f32)>, // x, y, w, h as percentages
}

#[cfg(feature = "gui")]
impl DetectedObject {
    fn new(name: &str, confidence: f32, category: ObjectCategory) -> Self {
        // Generate random bounding box position
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        
        let x = ((hash % 60) as f32) / 100.0 + 0.1;
        let y = ((hash / 100 % 50) as f32) / 100.0 + 0.15;
        let w = 0.15 + ((hash / 10000 % 10) as f32) / 100.0;
        let h = 0.12 + ((hash / 100000 % 10) as f32) / 100.0;
        
        Self {
            name: name.to_string(),
            confidence,
            category,
            bounding_box: Some((x, y, w, h)),
        }
    }
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
#[cfg(feature = "gui")]
struct Notification {
    title: String,
    body: String,
    icon: String,
    timestamp: Instant,
    priority: NotificationPriority,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "gui")]
enum NotificationPriority {
    Low,
    Normal,
    High,
}

#[cfg(feature = "gui")]
impl Notification {
    fn new(title: &str, body: &str, icon: &str, priority: NotificationPriority) -> Self {
        Self {
            title: title.to_string(),
            body: body.to_string(),
            icon: icon.to_string(),
            timestamp: Instant::now(),
            priority,
        }
    }
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
#[cfg(feature = "gui")]
struct ChatMessage {
    is_user: bool,
    text: String,
    timestamp: Instant,
}

#[cfg(feature = "gui")]
struct GlassesSimulator {
    // Device state
    battery: f32,
    is_powered: bool,
    brightness: f32,
    volume: f32,
    
    // Current view
    current_scene: Scene,
    show_ar_overlay: bool,
    show_object_detection: bool,
    
    // AI Chat
    chat_input: String,
    chat_history: Vec<ChatMessage>,
    ai_is_thinking: bool,
    ai_think_start: Option<Instant>,
    
    // Notifications
    notifications: Vec<Notification>,
    show_notifications: bool,
    
    // Navigation
    nav_active: bool,
    nav_destination: String,
    nav_distance: f32,
    nav_eta: u32,
    
    // Memory
    memory_items: Vec<MemoryItem>,
    
    // Animation
    time: f32,
    last_update: Instant,
    
    // UI state
    settings_open: bool,
    selected_tab: Tab,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
#[cfg(feature = "gui")]
struct MemoryItem {
    what: String,
    location: String,
    when: String,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "gui")]
enum Tab {
    Camera,
    Assistant,
    Memory,
    Settings,
}

#[cfg(feature = "gui")]
impl GlassesSimulator {
    fn new() -> Self {
        let mut sim = Self {
            battery: 87.0,
            is_powered: true,
            brightness: 0.8,
            volume: 0.6,
            
            current_scene: Scene::Home,
            show_ar_overlay: true,
            show_object_detection: true,
            
            chat_input: String::new(),
            chat_history: vec![
                ChatMessage {
                    is_user: false,
                    text: "Hello! I'm Kāraṇa, your AI assistant. I can help you identify objects, navigate, remember things, and more. Try asking me something!".to_string(),
                    timestamp: Instant::now(),
                },
            ],
            ai_is_thinking: false,
            ai_think_start: None,
            
            notifications: vec![
                Notification::new(
                    "Welcome to Kāraṇa OS",
                    "Your smart glasses are ready to use",
                    "🕶️",
                    NotificationPriority::Normal,
                ),
            ],
            show_notifications: false,
            
            nav_active: false,
            nav_destination: String::new(),
            nav_distance: 0.0,
            nav_eta: 0,
            
            memory_items: vec![
                MemoryItem {
                    what: "Keys".to_string(),
                    location: "Kitchen counter".to_string(),
                    when: "2 hours ago".to_string(),
                },
                MemoryItem {
                    what: "Wallet".to_string(),
                    location: "Bedroom nightstand".to_string(),
                    when: "This morning".to_string(),
                },
                MemoryItem {
                    what: "Headphones".to_string(),
                    location: "Living room couch".to_string(),
                    when: "Yesterday".to_string(),
                },
            ],
            
            time: 0.0,
            last_update: Instant::now(),
            
            settings_open: false,
            selected_tab: Tab::Camera,
        };
        sim
    }
    
    fn process_user_message(&mut self, message: &str) {
        let response = self.generate_ai_response(message);
        self.chat_history.push(ChatMessage {
            is_user: false,
            text: response,
            timestamp: Instant::now(),
        });
    }
    
    fn generate_ai_response(&mut self, message: &str) -> String {
        let msg_lower = message.to_lowercase();
        
        if msg_lower.contains("what") && (msg_lower.contains("see") || msg_lower.contains("looking")) {
            let objects = self.current_scene.objects();
            let obj_list: Vec<String> = objects.iter().take(3).map(|o| o.name.clone()).collect();
            format!(
                "I can see several objects in the {}. The main items are: {}. Would you like more details about any of them?",
                self.current_scene.name().to_lowercase(),
                obj_list.join(", ")
            )
        } else if msg_lower.contains("navigate") || msg_lower.contains("direction") || msg_lower.contains("way to") {
            self.nav_active = true;
            self.nav_destination = "Blue Bottle Coffee".to_string();
            self.nav_distance = 350.0;
            self.nav_eta = 4;
            "I've started navigation to Blue Bottle Coffee. It's 350 meters away, about a 4-minute walk. Follow the AR arrows in your view!".to_string()
        } else if msg_lower.contains("where") && (msg_lower.contains("key") || msg_lower.contains("wallet") || msg_lower.contains("phone")) {
            if msg_lower.contains("key") {
                "I found your keys! They're on the kitchen counter, near the fruit bowl. You put them there 2 hours ago after coming back from your jog.".to_string()
            } else if msg_lower.contains("wallet") {
                "Your wallet is on your bedroom nightstand. You placed it there this morning.".to_string()
            } else {
                "I don't have a recent memory of that item. Try looking around and I'll help you search!".to_string()
            }
        } else if msg_lower.contains("cook") || msg_lower.contains("recipe") || msg_lower.contains("food") {
            "Based on what I see in the kitchen - apples, a frying pan, and some ingredients - I'd suggest:\n\n1. 🍎 Caramelized Apples (10 min)\n2. 🥞 Apple Pancakes (15 min)\n3. 🥗 Apple Walnut Salad (5 min)\n\nWould you like the full recipe for any of these?".to_string()
        } else if msg_lower.contains("time") {
            let now = chrono::Local::now();
            format!("The current time is {}.", now.format("%I:%M %p"))
        } else if msg_lower.contains("weather") {
            "It's currently 72°F (22°C) and partly cloudy. Perfect weather for a walk! There's a 10% chance of rain later this evening.".to_string()
        } else if msg_lower.contains("remind") || msg_lower.contains("timer") {
            self.notifications.push(Notification::new(
                "Reminder Set",
                "I'll remind you in 30 minutes",
                "⏰",
                NotificationPriority::Normal,
            ));
            "I've set a reminder for you. I'll notify you in 30 minutes!".to_string()
        } else if msg_lower.contains("hello") || msg_lower.contains("hi") || msg_lower.contains("hey") {
            "Hello! How can I help you today? I can identify objects, help you navigate, remember where you put things, or answer questions.".to_string()
        } else if msg_lower.contains("thank") {
            "You're welcome! Is there anything else I can help you with?".to_string()
        } else if msg_lower.contains("person") || msg_lower.contains("who is") {
            "I recognize Alex from Engineering! You last met at the project sync 3 days ago. They mentioned the launch deadline is this Friday.".to_string()
        } else {
            format!(
                "I understand you're asking about '{}'. Let me help you with that. You can ask me to:\n\n• Identify objects around you\n• Navigate to places\n• Find things you've misplaced\n• Set reminders\n• Get information",
                message
            )
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for GlassesSimulator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update time for animations
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        self.time += dt;
        
        // Drain battery slowly
        if self.is_powered {
            self.battery = (self.battery - dt * 0.01).max(0.0);
        }
        
        // Handle AI thinking delay
        if self.ai_is_thinking {
            if let Some(start) = self.ai_think_start {
                if start.elapsed() > Duration::from_millis(800) {
                    self.ai_is_thinking = false;
                    self.ai_think_start = None;
                    if let Some(last) = self.chat_history.last() {
                        if last.is_user {
                            self.process_user_message(&last.text.clone());
                        }
                    }
                }
            }
        }
        
        // Request repaint for animations
        ctx.request_repaint();
        
        // Main layout
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            self.render_status_bar(ui);
        });
        
        egui::SidePanel::left("side_panel")
            .default_width(280.0)
            .show(ctx, |ui| {
                self.render_side_panel(ui);
            });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_view(ui);
        });
        
        // Render floating notifications
        if self.show_notifications && !self.notifications.is_empty() {
            self.render_notifications(ctx);
        }
    }
}

#[cfg(feature = "gui")]
impl GlassesSimulator {
    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            
            // Logo and title
            ui.heading(egui::RichText::new("🕶️ Kāraṇa OS").strong());
            
            ui.separator();
            
            // Current scene
            ui.label(format!("{} {}", self.current_scene.emoji(), self.current_scene.name()));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                
                // Settings button
                if ui.button("⚙️").clicked() {
                    self.settings_open = !self.settings_open;
                }
                
                // Notifications button
                let notif_count = self.notifications.len();
                let notif_text = if notif_count > 0 {
                    format!("🔔 {}", notif_count)
                } else {
                    "🔔".to_string()
                };
                if ui.button(notif_text).clicked() {
                    self.show_notifications = !self.show_notifications;
                }
                
                ui.separator();
                
                // Battery indicator
                let battery_icon = if self.battery > 75.0 {
                    "🔋"
                } else if self.battery > 25.0 {
                    "🪫"
                } else {
                    "⚠️"
                };
                let battery_color = if self.battery > 50.0 {
                    egui::Color32::from_rgb(100, 200, 100)
                } else if self.battery > 25.0 {
                    egui::Color32::from_rgb(255, 200, 100)
                } else {
                    egui::Color32::from_rgb(255, 100, 100)
                };
                ui.colored_label(battery_color, format!("{} {:.0}%", battery_icon, self.battery));
                
                ui.separator();
                
                // Time
                let now = chrono::Local::now();
                ui.label(now.format("%I:%M %p").to_string());
                
                ui.separator();
                
                // Signal strength
                ui.label("📶 ●●●●");
            });
        });
    }
    
    fn render_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        
        // Tab buttons
        ui.horizontal(|ui| {
            if ui.selectable_label(self.selected_tab == Tab::Camera, "📷 View").clicked() {
                self.selected_tab = Tab::Camera;
            }
            if ui.selectable_label(self.selected_tab == Tab::Assistant, "🤖 AI").clicked() {
                self.selected_tab = Tab::Assistant;
            }
            if ui.selectable_label(self.selected_tab == Tab::Memory, "🧠 Memory").clicked() {
                self.selected_tab = Tab::Memory;
            }
            if ui.selectable_label(self.selected_tab == Tab::Settings, "⚙️").clicked() {
                self.selected_tab = Tab::Settings;
            }
        });
        
        ui.separator();
        
        match self.selected_tab {
            Tab::Camera => self.render_camera_controls(ui),
            Tab::Assistant => self.render_chat_panel(ui),
            Tab::Memory => self.render_memory_panel(ui),
            Tab::Settings => self.render_settings_panel(ui),
        }
    }
    
    fn render_camera_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scene Selection");
        ui.add_space(5.0);
        
        for scene in [Scene::Home, Scene::Kitchen, Scene::Office, Scene::Street, Scene::Park] {
            let selected = self.current_scene == scene;
            if ui.selectable_label(selected, format!("{} {}", scene.emoji(), scene.name())).clicked() {
                self.current_scene = scene;
            }
        }
        
        ui.add_space(20.0);
        ui.separator();
        
        ui.heading("AR Overlays");
        ui.add_space(5.0);
        
        ui.checkbox(&mut self.show_ar_overlay, "Show AR Elements");
        ui.checkbox(&mut self.show_object_detection, "Object Detection");
        
        if self.nav_active {
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.label(egui::RichText::new("🧭 Navigation Active").strong());
                ui.label(format!("To: {}", self.nav_destination));
                ui.label(format!("Distance: {:.0}m", self.nav_distance));
                ui.label(format!("ETA: {} min", self.nav_eta));
                if ui.button("Stop Navigation").clicked() {
                    self.nav_active = false;
                }
            });
        }
        
        ui.add_space(20.0);
        ui.separator();
        
        ui.heading("Detected Objects");
        ui.add_space(5.0);
        
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for obj in self.current_scene.objects() {
                    ui.horizontal(|ui| {
                        // Confidence bar
                        let bar_width = 60.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(bar_width, 14.0),
                            egui::Sense::hover(),
                        );
                        
                        // Background
                        ui.painter().rect_filled(
                            rect,
                            4.0,
                            egui::Color32::from_rgb(40, 40, 50),
                        );
                        
                        // Fill
                        let fill_rect = egui::Rect::from_min_size(
                            rect.min,
                            egui::vec2(bar_width * obj.confidence, 14.0),
                        );
                        ui.painter().rect_filled(
                            fill_rect,
                            4.0,
                            obj.category.color(),
                        );
                        
                        ui.label(format!("{} ({:.0}%)", obj.name, obj.confidence * 100.0));
                    });
                }
            });
    }
    
    fn render_chat_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("AI Assistant");
        ui.add_space(5.0);
        
        // Chat history
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &self.chat_history {
                    let (bg_color, align) = if msg.is_user {
                        (egui::Color32::from_rgb(60, 100, 160), egui::Align::RIGHT)
                    } else {
                        (egui::Color32::from_rgb(50, 50, 70), egui::Align::LEFT)
                    };
                    
                    ui.with_layout(egui::Layout::top_down(align), |ui| {
                        egui::Frame::none()
                            .fill(bg_color)
                            .rounding(10.0)
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.set_max_width(200.0);
                                ui.label(&msg.text);
                            });
                    });
                    ui.add_space(5.0);
                }
                
                if self.ai_is_thinking {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Kāraṇa is thinking...");
                    });
                }
            });
        
        ui.add_space(10.0);
        
        // Input area
        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.chat_input)
                    .hint_text("Ask Kāraṇa anything...")
                    .desired_width(180.0)
            );
            
            let send_clicked = ui.button("Send").clicked();
            let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            
            if (send_clicked || enter_pressed) && !self.chat_input.is_empty() {
                let msg = self.chat_input.clone();
                self.chat_history.push(ChatMessage {
                    is_user: true,
                    text: msg,
                    timestamp: Instant::now(),
                });
                self.chat_input.clear();
                self.ai_is_thinking = true;
                self.ai_think_start = Some(Instant::now());
            }
        });
        
        ui.add_space(10.0);
        
        // Quick actions
        ui.label("Quick Actions:");
        ui.horizontal_wrapped(|ui| {
            for (label, query) in [
                ("What do you see?", "What can you see around me?"),
                ("Navigate", "Navigate to the nearest coffee shop"),
                ("Find keys", "Where are my keys?"),
                ("Recipe", "What can I cook?"),
            ] {
                if ui.small_button(label).clicked() {
                    self.chat_history.push(ChatMessage {
                        is_user: true,
                        text: query.to_string(),
                        timestamp: Instant::now(),
                    });
                    self.ai_is_thinking = true;
                    self.ai_think_start = Some(Instant::now());
                }
            }
        });
    }
    
    fn render_memory_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Memory Bank");
        ui.label("Items Kāraṇa remembers for you:");
        ui.add_space(10.0);
        
        for item in &self.memory_items {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(40, 45, 60))
                .rounding(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(&item.what).strong());
                    ui.label(format!("📍 {}", item.location));
                    ui.label(egui::RichText::new(format!("🕐 {}", item.when)).weak());
                });
            ui.add_space(5.0);
        }
        
        ui.add_space(20.0);
        ui.separator();
        ui.label(egui::RichText::new("🔒 All memories stored locally on device").weak());
    }
    
    fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("Brightness:");
            ui.add(egui::Slider::new(&mut self.brightness, 0.0..=1.0).show_value(false));
            ui.label(format!("{:.0}%", self.brightness * 100.0));
        });
        
        ui.horizontal(|ui| {
            ui.label("Volume:");
            ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).show_value(false));
            ui.label(format!("{:.0}%", self.volume * 100.0));
        });
        
        ui.add_space(20.0);
        ui.separator();
        
        ui.heading("Device Info");
        ui.label("Device: XREAL Air 2 Pro");
        ui.label("Kāraṇa OS: v0.2.0");
        ui.label(format!("Battery: {:.0}%", self.battery));
        ui.label("Storage: 12.4 GB / 64 GB");
        
        ui.add_space(20.0);
        ui.separator();
        
        ui.heading("Privacy");
        ui.label("✅ Local processing only");
        ui.label("✅ No cloud uploads");
        ui.label("✅ Data encrypted");
    }
    
    fn render_main_view(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        
        // Main AR view area
        let (rect, _response) = ui.allocate_exact_size(available, egui::Sense::click());
        
        // Draw gradient background (simulating camera view)
        let bg_colors = match self.current_scene {
            Scene::Home => (
                egui::Color32::from_rgb(60, 50, 70),
                egui::Color32::from_rgb(40, 35, 50),
            ),
            Scene::Kitchen => (
                egui::Color32::from_rgb(70, 60, 50),
                egui::Color32::from_rgb(50, 45, 40),
            ),
            Scene::Office => (
                egui::Color32::from_rgb(50, 55, 70),
                egui::Color32::from_rgb(35, 40, 55),
            ),
            Scene::Street => (
                egui::Color32::from_rgb(80, 90, 100),
                egui::Color32::from_rgb(50, 55, 65),
            ),
            Scene::Park => (
                egui::Color32::from_rgb(50, 80, 60),
                egui::Color32::from_rgb(35, 55, 45),
            ),
        };
        
        // Draw background gradient
        ui.painter().rect_filled(rect, 0.0, bg_colors.0);
        
        // Draw scene description
        let scene_text = format!(
            "{} {} View",
            self.current_scene.emoji(),
            self.current_scene.name()
        );
        ui.painter().text(
            rect.center_top() + egui::vec2(0.0, 40.0),
            egui::Align2::CENTER_TOP,
            &scene_text,
            egui::FontId::proportional(24.0),
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180),
        );
        
        // Draw object bounding boxes if enabled
        if self.show_object_detection {
            for obj in self.current_scene.objects() {
                if let Some((x, y, w, h)) = obj.bounding_box {
                    let box_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(rect.width() * x, rect.height() * y),
                        egui::vec2(rect.width() * w, rect.height() * h),
                    );
                    
                    // Draw bounding box
                    ui.painter().rect_stroke(
                        box_rect,
                        4.0,
                        egui::Stroke::new(2.0, obj.category.color()),
                    );
                    
                    // Draw label background
                    let label_rect = egui::Rect::from_min_size(
                        box_rect.left_top() - egui::vec2(0.0, 22.0),
                        egui::vec2(120.0, 20.0),
                    );
                    ui.painter().rect_filled(
                        label_rect,
                        4.0,
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
                    );
                    
                    // Draw label text
                    ui.painter().text(
                        label_rect.left_center() + egui::vec2(5.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        format!("{} {:.0}%", obj.name, obj.confidence * 100.0),
                        egui::FontId::proportional(12.0),
                        obj.category.color(),
                    );
                }
            }
        }
        
        // Draw navigation overlay if active
        if self.nav_active && self.show_ar_overlay {
            self.render_navigation_overlay(ui, rect);
        }
        
        // Draw AR HUD elements
        if self.show_ar_overlay {
            self.render_ar_hud(ui, rect);
        }
        
        // Draw glasses frame overlay
        self.render_glasses_frame(ui, rect);
    }
    
    fn render_navigation_overlay(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        // Draw navigation arrow
        let arrow_center = rect.center() - egui::vec2(0.0, 50.0);
        let arrow_color = egui::Color32::from_rgb(100, 200, 255);
        
        // Animate arrow bobbing
        let bob = (self.time * 3.0).sin() * 10.0;
        
        // Draw arrow
        let arrow_points = vec![
            arrow_center + egui::vec2(0.0, -40.0 + bob),
            arrow_center + egui::vec2(-30.0, 20.0 + bob),
            arrow_center + egui::vec2(-10.0, 20.0 + bob),
            arrow_center + egui::vec2(-10.0, 60.0 + bob),
            arrow_center + egui::vec2(10.0, 60.0 + bob),
            arrow_center + egui::vec2(10.0, 20.0 + bob),
            arrow_center + egui::vec2(30.0, 20.0 + bob),
        ];
        
        ui.painter().add(egui::Shape::convex_polygon(
            arrow_points,
            arrow_color,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        ));
        
        // Draw destination label
        let dest_rect = egui::Rect::from_center_size(
            rect.center() + egui::vec2(0.0, 100.0),
            egui::vec2(200.0, 60.0),
        );
        
        ui.painter().rect_filled(
            dest_rect,
            10.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
        );
        
        ui.painter().text(
            dest_rect.center_top() + egui::vec2(0.0, 15.0),
            egui::Align2::CENTER_CENTER,
            format!("☕ {}", self.nav_destination),
            egui::FontId::proportional(16.0),
            egui::Color32::WHITE,
        );
        
        ui.painter().text(
            dest_rect.center_bottom() - egui::vec2(0.0, 15.0),
            egui::Align2::CENTER_CENTER,
            format!("{:.0}m • {} min", self.nav_distance, self.nav_eta),
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(150, 200, 255),
        );
    }
    
    fn render_ar_hud(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        // Mini-map in corner
        let minimap_rect = egui::Rect::from_min_size(
            rect.right_bottom() - egui::vec2(120.0, 120.0),
            egui::vec2(100.0, 100.0),
        );
        
        ui.painter().rect_filled(
            minimap_rect,
            10.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
        );
        
        ui.painter().rect_stroke(
            minimap_rect,
            10.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 200)),
        );
        
        // Draw "you are here" dot
        ui.painter().circle_filled(
            minimap_rect.center(),
            5.0,
            egui::Color32::from_rgb(100, 200, 255),
        );
        
        // Pulse animation
        let pulse = ((self.time * 2.0).sin() + 1.0) / 2.0;
        ui.painter().circle_stroke(
            minimap_rect.center(),
            5.0 + pulse * 10.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 200, 255, (100.0 * (1.0 - pulse)) as u8)),
        );
    }
    
    fn render_glasses_frame(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        // Draw subtle vignette/frame to simulate looking through glasses
        let painter = ui.painter();
        
        // Top frame
        painter.rect_filled(
            egui::Rect::from_min_max(
                rect.left_top(),
                rect.right_top() + egui::vec2(0.0, 20.0),
            ),
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100),
        );
        
        // Bottom frame
        painter.rect_filled(
            egui::Rect::from_min_max(
                rect.left_bottom() - egui::vec2(0.0, 20.0),
                rect.right_bottom(),
            ),
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100),
        );
        
        // Corner vignettes
        for corner in [rect.left_top(), rect.right_top(), rect.left_bottom(), rect.right_bottom()] {
            painter.circle_filled(
                corner,
                100.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 50),
            );
        }
    }
    
    fn render_notifications(&mut self, ctx: &egui::Context) {
        egui::Window::new("Notifications")
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                if self.notifications.is_empty() {
                    ui.label("No notifications");
                } else {
                    let mut to_remove = Vec::new();
                    
                    for (idx, notif) in self.notifications.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(&notif.icon);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&notif.title).strong());
                                ui.label(&notif.body);
                            });
                            if ui.small_button("✕").clicked() {
                                to_remove.push(idx);
                            }
                        });
                        ui.separator();
                    }
                    
                    for idx in to_remove.into_iter().rev() {
                        self.notifications.remove(idx);
                    }
                    
                    if ui.button("Clear All").clicked() {
                        self.notifications.clear();
                    }
                }
            });
    }
}
