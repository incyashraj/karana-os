//! Tab Command Parser - Parse natural language to AR Tab commands
//!
//! This module parses voice commands like:
//! - "Pin a browser here showing YouTube"
//! - "Open Google on the wall"
//! - "Close the kitchen tab"
//! - "Show me my tabs"
//! - "Make this bigger"
//! - "Scroll down"

use crate::oracle::command::{
    OracleCommand, ScrollAmount, ScrollDirection, TabCycleDirection, TabLayoutHint,
    TabNavAction, TabSizeHint, WidgetType,
};
use std::collections::HashMap;

// ============================================================================
// TAB COMMAND PARSER
// ============================================================================

/// Parser for natural language tab commands
pub struct TabCommandParser {
    /// Common URL shortcuts
    url_shortcuts: HashMap<String, String>,
    /// Location keyword mappings
    location_keywords: HashMap<String, String>,
}

impl Default for TabCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TabCommandParser {
    pub fn new() -> Self {
        let mut url_shortcuts = HashMap::new();
        url_shortcuts.insert("youtube".to_string(), "https://youtube.com".to_string());
        url_shortcuts.insert("google".to_string(), "https://google.com".to_string());
        url_shortcuts.insert("gmail".to_string(), "https://mail.google.com".to_string());
        url_shortcuts.insert("twitter".to_string(), "https://twitter.com".to_string());
        url_shortcuts.insert("x".to_string(), "https://x.com".to_string());
        url_shortcuts.insert("reddit".to_string(), "https://reddit.com".to_string());
        url_shortcuts.insert("github".to_string(), "https://github.com".to_string());
        url_shortcuts.insert("netflix".to_string(), "https://netflix.com".to_string());
        url_shortcuts.insert("spotify".to_string(), "https://open.spotify.com".to_string());
        url_shortcuts.insert("amazon".to_string(), "https://amazon.com".to_string());
        url_shortcuts.insert("maps".to_string(), "https://maps.google.com".to_string());
        url_shortcuts.insert("news".to_string(), "https://news.google.com".to_string());
        url_shortcuts.insert("weather".to_string(), "https://weather.com".to_string());
        url_shortcuts.insert("wikipedia".to_string(), "https://wikipedia.org".to_string());

        let mut location_keywords = HashMap::new();
        location_keywords.insert("desk".to_string(), "desk".to_string());
        location_keywords.insert("table".to_string(), "desk".to_string());
        location_keywords.insert("wall".to_string(), "wall".to_string());
        location_keywords.insert("kitchen".to_string(), "kitchen".to_string());
        location_keywords.insert("bedroom".to_string(), "bedroom".to_string());
        location_keywords.insert("living room".to_string(), "living_room".to_string());
        location_keywords.insert("livingroom".to_string(), "living_room".to_string());
        location_keywords.insert("couch".to_string(), "couch".to_string());
        location_keywords.insert("sofa".to_string(), "couch".to_string());
        location_keywords.insert("bed".to_string(), "bed".to_string());
        location_keywords.insert("office".to_string(), "office".to_string());
        location_keywords.insert("bathroom".to_string(), "bathroom".to_string());
        location_keywords.insert("here".to_string(), "current".to_string());

        Self {
            url_shortcuts,
            location_keywords,
        }
    }

    /// Try to parse a command string into an AR Tab command
    pub fn parse(&self, input: &str) -> Option<OracleCommand> {
        let input = input.to_lowercase().trim().to_string();
        
        // Try each parser in order of specificity
        // Note: list_command before pin_command because "show my tabs" vs "show [url]"
        self.parse_list_command(&input)
            .or_else(|| self.parse_pin_command(&input))
            .or_else(|| self.parse_close_command(&input))
            .or_else(|| self.parse_focus_command(&input))
            .or_else(|| self.parse_minimize_command(&input))
            .or_else(|| self.parse_move_command(&input))
            .or_else(|| self.parse_resize_command(&input))
            .or_else(|| self.parse_layout_command(&input))
            .or_else(|| self.parse_navigate_command(&input))
            .or_else(|| self.parse_cycle_command(&input))
            .or_else(|| self.parse_widget_command(&input))
    }

    /// Parse pin/open commands
    /// "pin a browser here", "open youtube on the wall", "put google on my desk"
    fn parse_pin_command(&self, input: &str) -> Option<OracleCommand> {
        // Check for pin/open/put/show keywords
        let is_pin_command = input.starts_with("pin ")
            || input.starts_with("open ")
            || input.starts_with("put ")
            || input.starts_with("show ")
            || input.contains("pin here")
            || input.contains("pin this");

        if !is_pin_command {
            return None;
        }

        // Extract URL or shortcut
        let url = self.extract_url(input)?;
        
        // Extract location hint
        let location_hint = self.extract_location(input);
        
        // Extract size hint
        let size = self.extract_size(input);

        // Determine content type
        if self.is_video_url(&url) {
            Some(OracleCommand::TabPinVideo {
                url,
                size,
                location_hint,
            })
        } else {
            Some(OracleCommand::TabPinBrowser {
                url,
                size,
                location_hint,
            })
        }
    }

    /// Parse close commands
    /// "close the tab", "close kitchen browser", "clear the wall"
    fn parse_close_command(&self, input: &str) -> Option<OracleCommand> {
        if input.starts_with("close all") || input.starts_with("clear ") {
            // Close all tabs in a location
            let location = self.extract_location(input)?;
            return Some(OracleCommand::TabCloseLocation { location });
        }

        if input.starts_with("close") || input.contains("close tab") || input.contains("close this") {
            let query = self.extract_tab_query(input);
            return Some(OracleCommand::TabClose { query });
        }

        None
    }

    /// Parse focus commands
    /// "focus youtube", "show me the kitchen tab", "switch to browser"
    fn parse_focus_command(&self, input: &str) -> Option<OracleCommand> {
        let is_focus = input.starts_with("focus ")
            || input.starts_with("switch to ")
            || input.starts_with("go to tab")
            || (input.starts_with("show") && input.contains("tab"));

        if !is_focus {
            return None;
        }

        let query = self.extract_tab_query(input).unwrap_or_else(|| input.to_string());
        Some(OracleCommand::TabFocus { query })
    }

    /// Parse minimize commands
    /// "minimize", "minimize this", "hide the browser"
    fn parse_minimize_command(&self, input: &str) -> Option<OracleCommand> {
        if input.starts_with("minimize") || input.starts_with("hide") {
            let query = self.extract_tab_query(input);
            return Some(OracleCommand::TabMinimize { query });
        }
        None
    }

    /// Parse list commands
    /// "show my tabs", "list tabs", "what tabs do i have"
    fn parse_list_command(&self, input: &str) -> Option<OracleCommand> {
        let is_list = input.contains("list tab")
            || input.contains("show tab")
            || input.contains("my tab")
            || input.contains("what tab")
            || input.contains("show my")
            || input == "tabs";

        if !is_list {
            return None;
        }

        let location_filter = self.extract_location(input);
        Some(OracleCommand::TabList { location_filter })
    }

    /// Parse move commands
    /// "move this to the wall", "put youtube on the desk"
    fn parse_move_command(&self, input: &str) -> Option<OracleCommand> {
        if !input.starts_with("move") && !input.contains("move to") {
            return None;
        }

        let query = self.extract_tab_query(input);
        let target_location = self.extract_location(input)?;

        Some(OracleCommand::TabMove {
            query,
            target_location,
        })
    }

    /// Parse resize commands
    /// "make this bigger", "shrink the video", "full size"
    fn parse_resize_command(&self, input: &str) -> Option<OracleCommand> {
        let size = if input.contains("bigger") || input.contains("larger") || input.contains("expand") {
            Some(TabSizeHint::Large)
        } else if input.contains("smaller") || input.contains("shrink") || input.contains("compact") {
            Some(TabSizeHint::Small)
        } else if input.contains("full") || input.contains("maximize") {
            Some(TabSizeHint::Full)
        } else if input.contains("medium") || input.contains("normal") {
            Some(TabSizeHint::Medium)
        } else {
            None
        };

        size.map(|size| {
            let query = self.extract_tab_query(input);
            OracleCommand::TabResize { query, size }
        })
    }

    /// Parse layout commands
    /// "arrange in a grid", "stack the tabs", "carousel layout"
    fn parse_layout_command(&self, input: &str) -> Option<OracleCommand> {
        let layout = if input.contains("grid") {
            Some(TabLayoutHint::Grid)
        } else if input.contains("stack") {
            Some(TabLayoutHint::Stack)
        } else if input.contains("carousel") {
            Some(TabLayoutHint::Carousel)
        } else if input.contains("dock") {
            Some(TabLayoutHint::Dock)
        } else if input.contains("free") || input.contains("scatter") {
            Some(TabLayoutHint::Free)
        } else {
            None
        };

        layout.map(|layout| {
            let location = self.extract_location(input);
            OracleCommand::TabSetLayout { location, layout }
        })
    }

    /// Parse navigation commands
    /// "scroll down", "go back", "reload", "play", "pause"
    fn parse_navigate_command(&self, input: &str) -> Option<OracleCommand> {
        let action = if input == "back" || input == "go back" {
            Some(TabNavAction::Back)
        } else if input == "forward" || input == "go forward" {
            Some(TabNavAction::Forward)
        } else if input == "reload" || input == "refresh" {
            Some(TabNavAction::Reload)
        } else if input == "play" || input == "pause" || input == "play pause" {
            Some(TabNavAction::PlayPause)
        } else if input.starts_with("scroll") {
            self.parse_scroll_action(input)
        } else if input.starts_with("go to ") || input.starts_with("navigate to ") {
            let url = input
                .strip_prefix("go to ")
                .or_else(|| input.strip_prefix("navigate to "))
                .map(|s| self.resolve_url(s.trim()))?;
            Some(TabNavAction::GoTo { url })
        } else if input.starts_with("search ") || input.starts_with("search for ") {
            let query = input
                .strip_prefix("search for ")
                .or_else(|| input.strip_prefix("search "))
                .map(|s| s.to_string())?;
            Some(TabNavAction::Search { query })
        } else if input.starts_with("volume") {
            self.parse_volume_action(input)
        } else if input.starts_with("zoom") {
            self.parse_zoom_action(input)
        } else {
            None
        };

        action.map(|action| OracleCommand::TabNavigate { action })
    }

    /// Parse tab cycle commands
    /// "next tab", "previous tab", "recent tab"
    fn parse_cycle_command(&self, input: &str) -> Option<OracleCommand> {
        let direction = if input == "next tab" || input == "next" {
            Some(TabCycleDirection::Next)
        } else if input == "previous tab" || input == "previous" || input == "prev tab" {
            Some(TabCycleDirection::Previous)
        } else if input == "recent tab" || input == "last tab" || input == "recent" {
            Some(TabCycleDirection::Recent)
        } else {
            None
        };

        direction.map(|direction| OracleCommand::TabCycle { direction })
    }

    /// Parse widget commands
    /// "show me a clock", "put weather here", "timer widget"
    fn parse_widget_command(&self, input: &str) -> Option<OracleCommand> {
        let widget_type = if input.contains("clock") || input.contains("time") {
            Some(WidgetType::Clock)
        } else if input.contains("weather") {
            Some(WidgetType::Weather)
        } else if input.contains("calendar") || input.contains("schedule") {
            Some(WidgetType::Calendar)
        } else if input.contains("stock") {
            Some(WidgetType::Stocks)
        } else if input.contains("music") || input.contains("player") {
            Some(WidgetType::Music)
        } else if input.contains("timer") || input.contains("countdown") {
            Some(WidgetType::Timer)
        } else if input.contains("todo") || input.contains("task") {
            Some(WidgetType::Todo)
        } else if input.contains("notification") {
            Some(WidgetType::Notifications)
        } else if input.contains("note") || input.contains("sticky") {
            Some(WidgetType::StickyNote)
        } else if input.contains("status") || input.contains("battery") {
            Some(WidgetType::SystemStatus)
        } else {
            None
        };

        widget_type.map(|widget_type| {
            let size = self.extract_size(input);
            let location_hint = self.extract_location(input);
            OracleCommand::TabPinWidget {
                widget_type,
                size,
                location_hint,
            }
        })
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    /// Extract URL from input (or resolve shortcut)
    fn extract_url(&self, input: &str) -> Option<String> {
        // Check for direct URLs
        if let Some(url) = self.find_url_in_text(input) {
            return Some(url);
        }

        // Check for shortcuts
        for (shortcut, url) in &self.url_shortcuts {
            if input.contains(shortcut.as_str()) {
                return Some(url.clone());
            }
        }

        // Default browser
        if input.contains("browser") {
            return Some("https://google.com".to_string());
        }

        None
    }

    /// Find a URL pattern in text
    fn find_url_in_text(&self, input: &str) -> Option<String> {
        // Simple URL detection
        for word in input.split_whitespace() {
            if word.starts_with("http://") || word.starts_with("https://") {
                return Some(word.to_string());
            }
            if word.contains(".com")
                || word.contains(".org")
                || word.contains(".net")
                || word.contains(".io")
            {
                return Some(format!("https://{}", word));
            }
        }
        None
    }

    /// Resolve a potential URL or shortcut
    fn resolve_url(&self, input: &str) -> String {
        if input.starts_with("http://") || input.starts_with("https://") {
            return input.to_string();
        }

        if let Some(url) = self.url_shortcuts.get(input) {
            return url.clone();
        }

        if input.contains('.') {
            return format!("https://{}", input);
        }

        // Default to search
        // Simple URL encoding - replace spaces with +
        let encoded = input.replace(' ', "+");
        format!("https://google.com/search?q={}", encoded)
    }

    /// Extract location hint from input
    fn extract_location(&self, input: &str) -> Option<String> {
        for (keyword, location) in &self.location_keywords {
            if input.contains(keyword.as_str()) {
                return Some(location.clone());
            }
        }

        // Check for "on the X" or "in the X" patterns
        if let Some(idx) = input.find("on the ") {
            let rest = &input[idx + 7..];
            if let Some(word) = rest.split_whitespace().next() {
                return Some(word.to_string());
            }
        }

        if let Some(idx) = input.find("in the ") {
            let rest = &input[idx + 7..];
            if let Some(word) = rest.split_whitespace().next() {
                return Some(word.to_string());
            }
        }

        None
    }

    /// Extract size hint from input
    fn extract_size(&self, input: &str) -> TabSizeHint {
        if input.contains("small") || input.contains("tiny") {
            TabSizeHint::Small
        } else if input.contains("large") || input.contains("big") {
            TabSizeHint::Large
        } else if input.contains("full") || input.contains("huge") {
            TabSizeHint::Full
        } else {
            TabSizeHint::Medium
        }
    }

    /// Extract tab query (which tab to operate on)
    fn extract_tab_query(&self, input: &str) -> Option<String> {
        // Check for URL shortcuts as tab identifiers
        for shortcut in self.url_shortcuts.keys() {
            if input.contains(shortcut.as_str()) {
                return Some(shortcut.clone());
            }
        }

        // Check for location as identifier
        for (keyword, location) in &self.location_keywords {
            if input.contains(keyword.as_str()) && keyword != "here" {
                return Some(format!("{} tab", location));
            }
        }

        // "this tab" means currently focused
        if input.contains("this") || input.contains("current") {
            return None; // None = currently focused
        }

        None
    }

    /// Check if URL is a video service
    fn is_video_url(&self, url: &str) -> bool {
        url.contains("youtube.com")
            || url.contains("youtu.be")
            || url.contains("netflix.com")
            || url.contains("vimeo.com")
            || url.contains("twitch.tv")
            || url.contains("dailymotion.com")
            || url.contains("hulu.com")
            || url.contains("disneyplus.com")
            || url.contains("primevideo")
    }

    /// Parse scroll action
    fn parse_scroll_action(&self, input: &str) -> Option<TabNavAction> {
        let direction = if input.contains("down") {
            ScrollDirection::Down
        } else if input.contains("up") {
            ScrollDirection::Up
        } else if input.contains("left") {
            ScrollDirection::Left
        } else if input.contains("right") {
            ScrollDirection::Right
        } else {
            ScrollDirection::Down // default
        };

        let amount = if input.contains("page") {
            ScrollAmount::Page
        } else if input.contains("half") {
            ScrollAmount::HalfPage
        } else if input.contains("bottom") || input.contains("top") || input.contains("end") {
            ScrollAmount::End
        } else {
            ScrollAmount::Line
        };

        Some(TabNavAction::Scroll { direction, amount })
    }

    /// Parse volume action
    fn parse_volume_action(&self, input: &str) -> Option<TabNavAction> {
        let level = if input.contains("up") || input.contains("louder") {
            0.8
        } else if input.contains("down") || input.contains("quieter") {
            0.3
        } else if input.contains("mute") || input.contains("off") {
            0.0
        } else if input.contains("max") || input.contains("full") {
            1.0
        } else {
            0.5
        };

        Some(TabNavAction::Volume { level })
    }

    /// Parse zoom action
    fn parse_zoom_action(&self, input: &str) -> Option<TabNavAction> {
        let factor = if input.contains("in") {
            1.25
        } else if input.contains("out") {
            0.8
        } else if input.contains("reset") || input.contains("normal") {
            1.0
        } else {
            1.0
        };

        Some(TabNavAction::Zoom { factor })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> TabCommandParser {
        TabCommandParser::new()
    }

    #[test]
    fn test_pin_browser_youtube() {
        let cmd = parser().parse("open youtube here").unwrap();
        match cmd {
            OracleCommand::TabPinVideo { url, .. } => {
                assert!(url.contains("youtube"));
            }
            _ => panic!("Expected TabPinVideo"),
        }
    }

    #[test]
    fn test_pin_browser_google() {
        let cmd = parser().parse("pin google on the wall").unwrap();
        match cmd {
            OracleCommand::TabPinBrowser { url, location_hint, .. } => {
                assert!(url.contains("google"));
                assert_eq!(location_hint, Some("wall".to_string()));
            }
            _ => panic!("Expected TabPinBrowser"),
        }
    }

    #[test]
    fn test_pin_with_size() {
        let cmd = parser().parse("open a large browser here").unwrap();
        match cmd {
            OracleCommand::TabPinBrowser { size, .. } => {
                assert_eq!(size, TabSizeHint::Large);
            }
            _ => panic!("Expected TabPinBrowser"),
        }
    }

    #[test]
    fn test_close_tab() {
        let cmd = parser().parse("close the youtube tab").unwrap();
        match cmd {
            OracleCommand::TabClose { query } => {
                assert_eq!(query, Some("youtube".to_string()));
            }
            _ => panic!("Expected TabClose"),
        }
    }

    #[test]
    fn test_close_all_location() {
        let cmd = parser().parse("close all kitchen tabs").unwrap();
        match cmd {
            OracleCommand::TabCloseLocation { location } => {
                assert_eq!(location, "kitchen".to_string());
            }
            _ => panic!("Expected TabCloseLocation"),
        }
    }

    #[test]
    fn test_list_tabs() {
        let cmd = parser().parse("show my tabs").unwrap();
        match cmd {
            OracleCommand::TabList { .. } => {}
            _ => panic!("Expected TabList"),
        }
    }

    #[test]
    fn test_scroll_down() {
        let cmd = parser().parse("scroll down").unwrap();
        match cmd {
            OracleCommand::TabNavigate { action } => match action {
                TabNavAction::Scroll { direction, .. } => {
                    assert_eq!(direction, ScrollDirection::Down);
                }
                _ => panic!("Expected Scroll action"),
            },
            _ => panic!("Expected TabNavigate"),
        }
    }

    #[test]
    fn test_next_tab() {
        let cmd = parser().parse("next tab").unwrap();
        match cmd {
            OracleCommand::TabCycle { direction } => {
                assert_eq!(direction, TabCycleDirection::Next);
            }
            _ => panic!("Expected TabCycle"),
        }
    }

    #[test]
    fn test_widget_clock() {
        let cmd = parser().parse("show me a clock on the wall").unwrap();
        match cmd {
            OracleCommand::TabPinWidget {
                widget_type,
                location_hint,
                ..
            } => {
                assert_eq!(widget_type, WidgetType::Clock);
                assert_eq!(location_hint, Some("wall".to_string()));
            }
            _ => panic!("Expected TabPinWidget"),
        }
    }

    #[test]
    fn test_resize_bigger() {
        let cmd = parser().parse("make this bigger").unwrap();
        match cmd {
            OracleCommand::TabResize { size, .. } => {
                assert_eq!(size, TabSizeHint::Large);
            }
            _ => panic!("Expected TabResize"),
        }
    }

    #[test]
    fn test_go_back() {
        let cmd = parser().parse("go back").unwrap();
        match cmd {
            OracleCommand::TabNavigate { action } => match action {
                TabNavAction::Back => {}
                _ => panic!("Expected Back action"),
            },
            _ => panic!("Expected TabNavigate"),
        }
    }

    #[test]
    fn test_focus_tab() {
        let cmd = parser().parse("focus the youtube tab").unwrap();
        match cmd {
            OracleCommand::TabFocus { query } => {
                assert!(query.contains("youtube"));
            }
            _ => panic!("Expected TabFocus"),
        }
    }

    #[test]
    fn test_minimize() {
        let cmd = parser().parse("minimize this tab").unwrap();
        match cmd {
            OracleCommand::TabMinimize { query } => {
                assert!(query.is_none()); // "this" = currently focused
            }
            _ => panic!("Expected TabMinimize"),
        }
    }

    #[test]
    fn test_grid_layout() {
        let cmd = parser().parse("arrange in a grid").unwrap();
        match cmd {
            OracleCommand::TabSetLayout { layout, .. } => {
                assert_eq!(layout, TabLayoutHint::Grid);
            }
            _ => panic!("Expected TabSetLayout"),
        }
    }

    #[test]
    fn test_play_pause() {
        let cmd = parser().parse("pause").unwrap();
        match cmd {
            OracleCommand::TabNavigate { action } => match action {
                TabNavAction::PlayPause => {}
                _ => panic!("Expected PlayPause action"),
            },
            _ => panic!("Expected TabNavigate"),
        }
    }
}
