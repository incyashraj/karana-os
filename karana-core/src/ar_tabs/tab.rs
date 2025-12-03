//! # AR Tab Core Structures
//!
//! Defines the `ARTab` struct and related types for persistent AR tabs
//! that are pinned in physical space via spatial anchors.

use crate::spatial::{SpatialAnchor, WorldPosition, AnchorId};
use std::collections::HashMap;

/// Unique identifier for a tab
pub type TabId = uuid::Uuid;

/// A persistent AR tab pinned in space
#[derive(Debug, Clone)]
pub struct ARTab {
    /// Unique identifier
    pub id: TabId,
    /// Spatial anchor that pins this tab in the world
    pub anchor: SpatialAnchor,
    /// What content the tab displays
    pub content: TabContent,
    /// Size in real-world units
    pub size: TabSize,
    /// Current state
    pub state: TabState,
    /// Visual style
    pub style: TabStyle,
    /// Interaction zone for gaze/gesture targeting
    pub interaction_zone: InteractionZone,
    /// Metadata
    pub metadata: TabMetadata,
    /// Permissions for this tab
    pub permissions: TabPermissions,
}

impl ARTab {
    /// Create a new AR tab with the given content and anchor
    pub fn new(content: TabContent, anchor: SpatialAnchor) -> Self {
        let size = TabSize::default();
        let position = anchor.position.clone();
        
        Self {
            id: uuid::Uuid::new_v4(),
            anchor,
            content,
            size: size.clone(),
            state: TabState::Active,
            style: TabStyle::default(),
            interaction_zone: InteractionZone::from_size_and_position(&size, &position),
            metadata: TabMetadata::new(),
            permissions: TabPermissions::default(),
        }
    }
    
    /// Create with custom size
    pub fn with_size(mut self, size: TabSize) -> Self {
        let position = self.anchor.position.clone();
        self.interaction_zone = InteractionZone::from_size_and_position(&size, &position);
        self.size = size;
        self
    }
    
    /// Create with custom style
    pub fn with_style(mut self, style: TabStyle) -> Self {
        self.style = style;
        self
    }
    
    /// Get the world position of this tab
    pub fn position(&self) -> &WorldPosition {
        &self.anchor.position
    }
    
    /// Update the tab's state
    pub fn set_state(&mut self, state: TabState) {
        self.state = state;
        self.metadata.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    /// Check if tab is visible (not minimized or hidden)
    pub fn is_visible(&self) -> bool {
        matches!(self.state, TabState::Active | TabState::Background)
    }
    
    /// Check if tab is interactive (focused)
    pub fn is_interactive(&self) -> bool {
        matches!(self.state, TabState::Active)
    }
    
    /// Get the tab title based on content
    pub fn title(&self) -> String {
        match &self.content {
            TabContent::Browser(browser) => browser.title.clone(),
            TabContent::VideoPlayer(video) => video.title.clone(),
            TabContent::CodeEditor(editor) => format!("Code: {}", editor.file_name),
            TabContent::Document(doc) => doc.title.clone(),
            TabContent::Game(game) => game.name.clone(),
            TabContent::Widget(widget) => widget.name.clone(),
            TabContent::Custom(custom) => custom.name.clone(),
        }
    }
    
    /// Get the tab icon based on content type
    pub fn icon(&self) -> &str {
        match &self.content {
            TabContent::Browser(_) => "🌐",
            TabContent::VideoPlayer(_) => "🎬",
            TabContent::CodeEditor(_) => "💻",
            TabContent::Document(_) => "📄",
            TabContent::Game(_) => "🎮",
            TabContent::Widget(_) => "⚡",
            TabContent::Custom(_) => "📦",
        }
    }
    
    /// Resize the tab
    pub fn resize(&mut self, scale: f32) {
        self.size.width_m *= scale;
        self.size.height_m *= scale;
        let position = self.anchor.position.clone();
        self.interaction_zone = InteractionZone::from_size_and_position(&self.size, &position);
    }
    
    /// Move the tab by offset
    pub fn move_by(&mut self, dx: f32, dy: f32, dz: f32) {
        self.anchor.position.local.x += dx;
        self.anchor.position.local.y += dy;
        self.anchor.position.local.z += dz;
        let position = self.anchor.position.clone();
        self.interaction_zone = InteractionZone::from_size_and_position(&self.size, &position);
    }
}

/// Content types a tab can display
#[derive(Debug, Clone)]
pub enum TabContent {
    /// Web browser
    Browser(BrowserState),
    /// Video player (YouTube, Netflix, etc.)
    VideoPlayer(VideoState),
    /// Code editor
    CodeEditor(EditorState),
    /// Document viewer (PDF, notes, etc.)
    Document(DocumentState),
    /// Spatial game
    Game(GameState),
    /// Widget (clock, weather, etc.)
    Widget(WidgetState),
    /// Custom AR app
    Custom(CustomAppState),
}

impl TabContent {
    /// Create browser content
    pub fn browser(url: &str) -> Self {
        TabContent::Browser(BrowserState {
            url: url.to_string(),
            title: "Loading...".to_string(),
            favicon: None,
            scroll_position: 0.0,
            can_go_back: false,
            can_go_forward: false,
            is_loading: true,
            history: vec![url.to_string()],
            history_index: 0,
        })
    }
    
    /// Create video content
    pub fn video(url: &str, title: &str) -> Self {
        TabContent::VideoPlayer(VideoState {
            url: url.to_string(),
            title: title.to_string(),
            duration_secs: 0,
            position_secs: 0,
            is_playing: false,
            volume: 1.0,
            playback_rate: 1.0,
            thumbnail: None,
        })
    }
    
    /// Create code editor content
    pub fn code_editor(file_path: &str, content: &str) -> Self {
        TabContent::CodeEditor(EditorState {
            file_path: file_path.to_string(),
            file_name: file_path.split('/').last().unwrap_or(file_path).to_string(),
            content: content.to_string(),
            cursor_line: 1,
            cursor_column: 1,
            selection: None,
            language: detect_language(file_path),
            is_modified: false,
        })
    }
    
    /// Create document content
    pub fn document(title: &str, path: &str) -> Self {
        TabContent::Document(DocumentState {
            title: title.to_string(),
            path: path.to_string(),
            page: 1,
            total_pages: 1,
            zoom: 1.0,
            document_type: detect_document_type(path),
        })
    }
    
    /// Create game content
    pub fn game(name: &str, game_type: GameType) -> Self {
        TabContent::Game(GameState {
            name: name.to_string(),
            game_type,
            world_id: None,
            is_paused: true,
            players: vec![],
            state_hash: None,
        })
    }
    
    /// Create widget content
    pub fn widget(name: &str, widget_type: WidgetType) -> Self {
        TabContent::Widget(WidgetState {
            name: name.to_string(),
            widget_type,
            data: HashMap::new(),
            refresh_interval_secs: 60,
            last_refresh: 0,
        })
    }
    
    /// Create custom app content
    pub fn custom(name: &str, app_id: &str) -> Self {
        TabContent::Custom(CustomAppState {
            name: name.to_string(),
            app_id: app_id.to_string(),
            state: HashMap::new(),
            permissions: vec![],
        })
    }
}

/// Browser tab state
#[derive(Debug, Clone)]
pub struct BrowserState {
    pub url: String,
    pub title: String,
    pub favicon: Option<Vec<u8>>,
    pub scroll_position: f32,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
    pub history: Vec<String>,
    pub history_index: usize,
}

/// Video player state
#[derive(Debug, Clone)]
pub struct VideoState {
    pub url: String,
    pub title: String,
    pub duration_secs: u64,
    pub position_secs: u64,
    pub is_playing: bool,
    pub volume: f32,
    pub playback_rate: f32,
    pub thumbnail: Option<Vec<u8>>,
}

/// Code editor state
#[derive(Debug, Clone)]
pub struct EditorState {
    pub file_path: String,
    pub file_name: String,
    pub content: String,
    pub cursor_line: u32,
    pub cursor_column: u32,
    pub selection: Option<Selection>,
    pub language: String,
    pub is_modified: bool,
}

/// Text selection in editor
#[derive(Debug, Clone)]
pub struct Selection {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// Document viewer state
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub title: String,
    pub path: String,
    pub page: u32,
    pub total_pages: u32,
    pub zoom: f32,
    pub document_type: DocumentType,
}

/// Document types
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    Pdf,
    Text,
    Markdown,
    RichText,
    Spreadsheet,
    Presentation,
    Other,
}

/// Game state
#[derive(Debug, Clone)]
pub struct GameState {
    pub name: String,
    pub game_type: GameType,
    pub world_id: Option<String>,
    pub is_paused: bool,
    pub players: Vec<String>,
    pub state_hash: Option<[u8; 32]>,
}

/// Game types
#[derive(Debug, Clone, PartialEq)]
pub enum GameType {
    VoxelBuilder,  // Minecraft-style
    BoardGame,     // Chess, checkers, etc.
    Racing,        // Racing games
    Strategy,      // Tower defense, RTS
    Puzzle,        // Physics puzzles
    Action,        // Action games
    Custom,        // Custom game type
}

/// Widget state
#[derive(Debug, Clone)]
pub struct WidgetState {
    pub name: String,
    pub widget_type: WidgetType,
    pub data: HashMap<String, String>,
    pub refresh_interval_secs: u32,
    pub last_refresh: u64,
}

/// Widget types
#[derive(Debug, Clone, PartialEq)]
pub enum WidgetType {
    Clock,
    Weather,
    Calendar,
    Notes,
    Timer,
    StockTicker,
    SocialFeed,
    Custom,
}

/// Custom app state
#[derive(Debug, Clone)]
pub struct CustomAppState {
    pub name: String,
    pub app_id: String,
    pub state: HashMap<String, String>,
    pub permissions: Vec<String>,
}

/// Tab size in real-world units (meters)
#[derive(Debug, Clone)]
pub struct TabSize {
    /// Width in meters
    pub width_m: f32,
    /// Height in meters
    pub height_m: f32,
    /// Depth for 3D content
    pub depth_m: f32,
    /// Aspect ratio (width/height)
    pub aspect_ratio: f32,
    /// What fraction of FOV it occupies
    pub fov_fraction: f32,
}

impl Default for TabSize {
    fn default() -> Self {
        Self {
            width_m: 0.8,       // 80cm wide (like a small TV)
            height_m: 0.5,     // 50cm tall
            depth_m: 0.0,       // 2D by default
            aspect_ratio: 16.0 / 10.0,
            fov_fraction: 0.4,  // 40% of horizontal FOV
        }
    }
}

impl TabSize {
    /// Create a small tab (like a post-it)
    pub fn small() -> Self {
        Self {
            width_m: 0.2,
            height_m: 0.15,
            depth_m: 0.0,
            aspect_ratio: 4.0 / 3.0,
            fov_fraction: 0.1,
        }
    }
    
    /// Create a medium tab (like a tablet)
    pub fn medium() -> Self {
        Self {
            width_m: 0.4,
            height_m: 0.3,
            depth_m: 0.0,
            aspect_ratio: 4.0 / 3.0,
            fov_fraction: 0.25,
        }
    }
    
    /// Create a large tab (like a monitor)
    pub fn large() -> Self {
        Self {
            width_m: 1.0,
            height_m: 0.6,
            depth_m: 0.0,
            aspect_ratio: 16.0 / 9.0,
            fov_fraction: 0.6,
        }
    }
    
    /// Create an immersive tab (full FOV)
    pub fn immersive() -> Self {
        Self {
            width_m: 2.0,
            height_m: 1.5,
            depth_m: 1.0,
            aspect_ratio: 4.0 / 3.0,
            fov_fraction: 1.0,
        }
    }
    
    /// Create a 3D content box
    pub fn box_3d(size: f32) -> Self {
        Self {
            width_m: size,
            height_m: size,
            depth_m: size,
            aspect_ratio: 1.0,
            fov_fraction: 0.3,
        }
    }
    
    /// Scale the size uniformly
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            width_m: self.width_m * factor,
            height_m: self.height_m * factor,
            depth_m: self.depth_m * factor,
            aspect_ratio: self.aspect_ratio,
            fov_fraction: (self.fov_fraction * factor).min(1.0),
        }
    }
}

/// Tab state
#[derive(Debug, Clone, PartialEq)]
pub enum TabState {
    /// Tab is focused and interactive
    Active,
    /// Tab is visible but not focused
    Background,
    /// Tab is minimized (icon only)
    Minimized,
    /// Tab is loading content
    Loading,
    /// Tab is suspended (memory saved)
    Suspended,
    /// Tab has an error
    Error(String),
}

/// Tab visual style
#[derive(Debug, Clone)]
pub struct TabStyle {
    /// Background color (RGBA)
    pub background: [f32; 4],
    /// Border color
    pub border_color: [f32; 4],
    /// Border width in meters
    pub border_width: f32,
    /// Corner radius in meters
    pub corner_radius: f32,
    /// Opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Shadow enabled
    pub shadow: bool,
    /// Blur background behind tab
    pub blur_behind: bool,
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            background: [0.1, 0.1, 0.12, 0.95],  // Dark semi-transparent
            border_color: [0.3, 0.3, 0.35, 1.0], // Subtle border
            border_width: 0.002,                  // 2mm
            corner_radius: 0.02,                  // 2cm
            opacity: 1.0,
            shadow: true,
            blur_behind: true,
        }
    }
}

impl TabStyle {
    /// Transparent style (for videos, games)
    pub fn transparent() -> Self {
        Self {
            background: [0.0, 0.0, 0.0, 0.0],
            border_color: [0.0, 0.0, 0.0, 0.0],
            border_width: 0.0,
            corner_radius: 0.0,
            opacity: 1.0,
            shadow: false,
            blur_behind: false,
        }
    }
    
    /// Focused style (highlighted border)
    pub fn focused() -> Self {
        Self {
            background: [0.1, 0.1, 0.12, 0.95],
            border_color: [0.2, 0.6, 1.0, 1.0],  // Blue highlight
            border_width: 0.003,
            corner_radius: 0.02,
            opacity: 1.0,
            shadow: true,
            blur_behind: true,
        }
    }
    
    /// Minimized style (compact)
    pub fn minimized() -> Self {
        Self {
            background: [0.15, 0.15, 0.18, 0.9],
            border_color: [0.3, 0.3, 0.35, 0.5],
            border_width: 0.001,
            corner_radius: 0.01,
            opacity: 0.8,
            shadow: true,
            blur_behind: false,
        }
    }
}

/// Interaction zone for gaze/gesture targeting
#[derive(Debug, Clone)]
pub struct InteractionZone {
    /// Center position in world coordinates
    pub center: WorldPosition,
    /// Bounding box half-extents (meters)
    pub half_extents: (f32, f32, f32),
    /// Whether zone is currently active
    pub active: bool,
}

impl InteractionZone {
    /// Create from size and center position
    pub fn from_size_and_position(size: &TabSize, center: &WorldPosition) -> Self {
        Self {
            center: center.clone(),
            half_extents: (
                size.width_m / 2.0,
                size.height_m / 2.0,
                size.depth_m.max(0.1) / 2.0,  // Min depth for hit testing
            ),
            active: true,
        }
    }
    
    /// Check if a point is within the interaction zone
    pub fn contains(&self, point: &WorldPosition) -> bool {
        if !self.active {
            return false;
        }
        
        let dx = (point.local.x - self.center.local.x).abs();
        let dy = (point.local.y - self.center.local.y).abs();
        let dz = (point.local.z - self.center.local.z).abs();
        
        dx <= self.half_extents.0 && 
        dy <= self.half_extents.1 && 
        dz <= self.half_extents.2
    }
    
    /// Convert world point to local coordinates within tab
    pub fn to_local(&self, point: &WorldPosition) -> (f32, f32) {
        let local_x = (point.local.x - self.center.local.x + self.half_extents.0) 
                      / (self.half_extents.0 * 2.0);
        let local_y = (point.local.y - self.center.local.y + self.half_extents.1) 
                      / (self.half_extents.1 * 2.0);
        (local_x.clamp(0.0, 1.0), local_y.clamp(0.0, 1.0))
    }
    
    /// Expand zone by margin (for easier targeting)
    pub fn expand(&mut self, margin: f32) {
        self.half_extents.0 += margin;
        self.half_extents.1 += margin;
        self.half_extents.2 += margin;
    }
}

/// Tab metadata
#[derive(Debug, Clone)]
pub struct TabMetadata {
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Total time spent (seconds)
    pub total_time_secs: u64,
    /// Access count
    pub access_count: u32,
    /// Location hint (human-readable)
    pub location_hint: String,
    /// Tags for organization
    pub tags: Vec<String>,
}

impl TabMetadata {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            created_at: now,
            last_accessed: now,
            total_time_secs: 0,
            access_count: 1,
            location_hint: String::new(),
            tags: vec![],
        }
    }
    
    pub fn with_location(mut self, hint: &str) -> Self {
        self.location_hint = hint.to_string();
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl Default for TabMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab permissions
#[derive(Debug, Clone)]
pub struct TabPermissions {
    /// Can access camera
    pub camera: bool,
    /// Can access microphone
    pub microphone: bool,
    /// Can access location
    pub location: bool,
    /// Can access local storage
    pub storage: bool,
    /// Can run in background
    pub background: bool,
    /// Can send notifications
    pub notifications: bool,
    /// Can use haptics
    pub haptics: bool,
}

impl Default for TabPermissions {
    fn default() -> Self {
        Self {
            camera: false,
            microphone: false,
            location: false,
            storage: true,
            background: true,
            notifications: true,
            haptics: true,
        }
    }
}

impl TabPermissions {
    /// Full permissions (for trusted apps)
    pub fn full() -> Self {
        Self {
            camera: true,
            microphone: true,
            location: true,
            storage: true,
            background: true,
            notifications: true,
            haptics: true,
        }
    }
    
    /// Minimal permissions (for untrusted content)
    pub fn minimal() -> Self {
        Self {
            camera: false,
            microphone: false,
            location: false,
            storage: false,
            background: false,
            notifications: false,
            haptics: true,
        }
    }
}

// Helper functions

fn detect_language(file_path: &str) -> String {
    let extension = file_path.split('.').last().unwrap_or("");
    match extension {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" => "typescriptreact",
        "jsx" => "javascriptreact",
        "html" => "html",
        "css" => "css",
        "json" => "json",
        "md" => "markdown",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "go" => "go",
        "java" => "java",
        "swift" => "swift",
        "kt" => "kotlin",
        _ => "plaintext",
    }.to_string()
}

fn detect_document_type(path: &str) -> DocumentType {
    let extension = path.split('.').last().unwrap_or("");
    match extension {
        "pdf" => DocumentType::Pdf,
        "txt" => DocumentType::Text,
        "md" | "markdown" => DocumentType::Markdown,
        "rtf" | "doc" | "docx" => DocumentType::RichText,
        "xls" | "xlsx" | "csv" => DocumentType::Spreadsheet,
        "ppt" | "pptx" | "key" => DocumentType::Presentation,
        _ => DocumentType::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{WorldPosition, SpatialAnchor, AnchorContent, AnchorState, Quaternion};

    fn create_test_anchor() -> SpatialAnchor {
        SpatialAnchor {
            id: 1,
            position: WorldPosition::from_local(1.0, 1.5, 2.0),
            orientation: Quaternion::identity(),
            visual_signature: [0u8; 32],
            content_hash: [0u8; 32],
            content: AnchorContent::Text { text: "test".to_string() },
            state: AnchorState::Active,
            confidence: 1.0,
            created_at: 0,
            updated_at: 0,
            owner_did: None,
            label: None,
        }
    }

    #[test]
    fn test_tab_creation() {
        let anchor = create_test_anchor();
        let content = TabContent::browser("https://example.com");
        let tab = ARTab::new(content, anchor);
        
        assert!(tab.is_visible());
        assert!(tab.is_interactive());
        assert_eq!(tab.state, TabState::Active);
        assert_eq!(tab.icon(), "🌐");
    }

    #[test]
    fn test_tab_content_types() {
        // Browser
        let browser = TabContent::browser("https://test.com");
        assert!(matches!(browser, TabContent::Browser(_)));
        
        // Video
        let video = TabContent::video("https://youtube.com/watch?v=123", "Test Video");
        assert!(matches!(video, TabContent::VideoPlayer(_)));
        
        // Code editor
        let editor = TabContent::code_editor("/src/main.rs", "fn main() {}");
        if let TabContent::CodeEditor(state) = editor {
            assert_eq!(state.language, "rust");
            assert_eq!(state.file_name, "main.rs");
        } else {
            panic!("Expected CodeEditor");
        }
        
        // Game
        let game = TabContent::game("Minecraft", GameType::VoxelBuilder);
        assert!(matches!(game, TabContent::Game(_)));
    }

    #[test]
    fn test_tab_sizes() {
        let small = TabSize::small();
        let medium = TabSize::medium();
        let large = TabSize::large();
        
        assert!(small.width_m < medium.width_m);
        assert!(medium.width_m < large.width_m);
        
        let scaled = small.scale(2.0);
        assert!((scaled.width_m - small.width_m * 2.0).abs() < 0.001);
    }

    #[test]
    fn test_tab_state_transitions() {
        let anchor = create_test_anchor();
        let mut tab = ARTab::new(TabContent::browser("https://test.com"), anchor);
        
        assert_eq!(tab.state, TabState::Active);
        assert!(tab.is_visible());
        assert!(tab.is_interactive());
        
        tab.set_state(TabState::Background);
        assert!(tab.is_visible());
        assert!(!tab.is_interactive());
        
        tab.set_state(TabState::Minimized);
        assert!(!tab.is_visible());
        assert!(!tab.is_interactive());
    }

    #[test]
    fn test_interaction_zone() {
        let size = TabSize::default();
        let position = WorldPosition::from_local(0.0, 1.5, 2.0);
        let zone = InteractionZone::from_size_and_position(&size, &position);
        
        // Point at center should be inside
        let center = WorldPosition::from_local(0.0, 1.5, 2.0);
        assert!(zone.contains(&center));
        
        // Point inside bounds
        let inside = WorldPosition::from_local(0.2, 1.6, 2.0);
        assert!(zone.contains(&inside));
        
        // Point outside bounds
        let outside = WorldPosition::from_local(2.0, 1.5, 2.0);
        assert!(!zone.contains(&outside));
    }

    #[test]
    fn test_zone_to_local() {
        let size = TabSize { 
            width_m: 1.0, 
            height_m: 1.0,
            depth_m: 0.0,
            aspect_ratio: 1.0,
            fov_fraction: 0.4,
        };
        let position = WorldPosition::from_local(0.0, 0.0, 0.0);
        let zone = InteractionZone::from_size_and_position(&size, &position);
        
        // Center should map to (0.5, 0.5)
        let center = WorldPosition::from_local(0.0, 0.0, 0.0);
        let (lx, ly) = zone.to_local(&center);
        assert!((lx - 0.5).abs() < 0.001);
        assert!((ly - 0.5).abs() < 0.001);
        
        // Bottom-left corner
        let bl = WorldPosition::from_local(-0.5, -0.5, 0.0);
        let (lx, ly) = zone.to_local(&bl);
        assert!((lx - 0.0).abs() < 0.001);
        assert!((ly - 0.0).abs() < 0.001);
        
        // Top-right corner
        let tr = WorldPosition::from_local(0.5, 0.5, 0.0);
        let (lx, ly) = zone.to_local(&tr);
        assert!((lx - 1.0).abs() < 0.001);
        assert!((ly - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tab_resize() {
        let anchor = create_test_anchor();
        let mut tab = ARTab::new(TabContent::browser("https://test.com"), anchor);
        let original_width = tab.size.width_m;
        
        tab.resize(2.0);
        assert!((tab.size.width_m - original_width * 2.0).abs() < 0.001);
    }

    #[test]
    fn test_tab_move() {
        let anchor = create_test_anchor();
        let mut tab = ARTab::new(TabContent::browser("https://test.com"), anchor);
        let original_x = tab.position().local.x;
        
        tab.move_by(1.0, 0.0, 0.0);
        assert!((tab.position().local.x - (original_x + 1.0)).abs() < 0.001);
    }

    #[test]
    fn test_tab_metadata() {
        let meta = TabMetadata::new()
            .with_location("Kitchen Counter")
            .with_tags(vec!["work".to_string(), "recipes".to_string()]);
        
        assert_eq!(meta.location_hint, "Kitchen Counter");
        assert_eq!(meta.tags.len(), 2);
        assert!(meta.created_at > 0);
    }

    #[test]
    fn test_tab_permissions() {
        let default = TabPermissions::default();
        assert!(!default.camera);
        assert!(default.storage);
        
        let full = TabPermissions::full();
        assert!(full.camera);
        assert!(full.microphone);
        
        let minimal = TabPermissions::minimal();
        assert!(!minimal.storage);
        assert!(minimal.haptics);
    }

    #[test]
    fn test_tab_styles() {
        let default = TabStyle::default();
        assert!(default.shadow);
        assert!(default.opacity > 0.9);
        
        let transparent = TabStyle::transparent();
        assert_eq!(transparent.background[3], 0.0);
        assert!(!transparent.shadow);
        
        let focused = TabStyle::focused();
        assert!(focused.border_color[2] > focused.border_color[0]); // Blue highlight
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), "rust");
        assert_eq!(detect_language("app.tsx"), "typescriptreact");
        assert_eq!(detect_language("style.css"), "css");
        assert_eq!(detect_language("config.toml"), "toml");
        assert_eq!(detect_language("unknown.xyz"), "plaintext");
    }

    #[test]
    fn test_detect_document_type() {
        assert_eq!(detect_document_type("doc.pdf"), DocumentType::Pdf);
        assert_eq!(detect_document_type("readme.md"), DocumentType::Markdown);
        assert_eq!(detect_document_type("data.xlsx"), DocumentType::Spreadsheet);
    }
}
