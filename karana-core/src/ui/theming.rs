//! UI Theme System
//! 
//! Comprehensive theming with support for:
//! - Light/dark modes
//! - Custom color palettes
//! - Typography scales
//! - Component-specific styling

use super::*;
use std::collections::HashMap;

/// Theme mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

/// Complete theme definition
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Theme mode
    pub mode: ThemeMode,
    /// Color palette
    pub colors: ColorPalette,
    /// Typography settings
    pub typography: Typography,
    /// Spacing scale
    pub spacing: SpacingScale,
    /// Border radius scale
    pub radii: RadiusScale,
    /// Shadow presets
    pub shadows: ShadowPresets,
    /// Component styles
    pub components: ComponentStyles,
    /// Custom values
    pub custom: HashMap<String, String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            name: "KaranaOS Dark".into(),
            mode: ThemeMode::Dark,
            colors: ColorPalette::dark(),
            typography: Typography::default(),
            spacing: SpacingScale::default(),
            radii: RadiusScale::default(),
            shadows: ShadowPresets::dark(),
            components: ComponentStyles::dark(),
            custom: HashMap::new(),
        }
    }

    /// Create light theme
    pub fn light() -> Self {
        Self {
            name: "KaranaOS Light".into(),
            mode: ThemeMode::Light,
            colors: ColorPalette::light(),
            typography: Typography::default(),
            spacing: SpacingScale::default(),
            radii: RadiusScale::default(),
            shadows: ShadowPresets::light(),
            components: ComponentStyles::light(),
            custom: HashMap::new(),
        }
    }

    /// Create AR-optimized transparent theme
    pub fn ar_glass() -> Self {
        Self {
            name: "KaranaOS Glass".into(),
            mode: ThemeMode::Dark,
            colors: ColorPalette::glass(),
            typography: Typography::ar_optimized(),
            spacing: SpacingScale::ar_optimized(),
            radii: RadiusScale::default(),
            shadows: ShadowPresets::glass(),
            components: ComponentStyles::glass(),
            custom: HashMap::new(),
        }
    }

    /// Get color by name
    pub fn color(&self, name: &str) -> Color {
        match name {
            "primary" => self.colors.primary,
            "secondary" => self.colors.secondary,
            "accent" => self.colors.accent,
            "background" => self.colors.background,
            "surface" => self.colors.surface,
            "error" => self.colors.error,
            "warning" => self.colors.warning,
            "success" => self.colors.success,
            "info" => self.colors.info,
            "text" => self.colors.text,
            "text_secondary" => self.colors.text_secondary,
            "disabled" => self.colors.disabled,
            _ => Color::WHITE,
        }
    }
}

/// Color palette
#[derive(Debug, Clone)]
pub struct ColorPalette {
    /// Primary brand color
    pub primary: Color,
    /// Primary variant (lighter/darker)
    pub primary_variant: Color,
    /// Secondary color
    pub secondary: Color,
    /// Secondary variant
    pub secondary_variant: Color,
    /// Accent/highlight color
    pub accent: Color,
    /// Background color
    pub background: Color,
    /// Surface color (cards, dialogs)
    pub surface: Color,
    /// Elevated surface
    pub surface_elevated: Color,
    /// Error color
    pub error: Color,
    /// Warning color
    pub warning: Color,
    /// Success color
    pub success: Color,
    /// Info color
    pub info: Color,
    /// Primary text color
    pub text: Color,
    /// Secondary text color
    pub text_secondary: Color,
    /// Disabled state color
    pub disabled: Color,
    /// Divider color
    pub divider: Color,
    /// Overlay color
    pub overlay: Color,
}

impl ColorPalette {
    pub fn dark() -> Self {
        Self {
            primary: Color::from_hex(0x00BCD4),       // Cyan
            primary_variant: Color::from_hex(0x0097A7),
            secondary: Color::from_hex(0x7C4DFF),     // Purple
            secondary_variant: Color::from_hex(0x651FFF),
            accent: Color::from_hex(0xFF4081),        // Pink
            background: Color::from_hex(0x121212),
            surface: Color::from_hex(0x1E1E1E),
            surface_elevated: Color::from_hex(0x2D2D2D),
            error: Color::from_hex(0xCF6679),
            warning: Color::from_hex(0xFFB74D),
            success: Color::from_hex(0x81C784),
            info: Color::from_hex(0x64B5F6),
            text: Color::from_hex(0xFFFFFF),
            text_secondary: Color::from_hex(0xB3B3B3),
            disabled: Color::from_hex(0x666666),
            divider: Color::from_hex(0x333333),
            overlay: Color::rgba(0, 0, 0, 128),
        }
    }

    pub fn light() -> Self {
        Self {
            primary: Color::from_hex(0x0097A7),
            primary_variant: Color::from_hex(0x006978),
            secondary: Color::from_hex(0x7C4DFF),
            secondary_variant: Color::from_hex(0x3D1A78),
            accent: Color::from_hex(0xF50057),
            background: Color::from_hex(0xFAFAFA),
            surface: Color::from_hex(0xFFFFFF),
            surface_elevated: Color::from_hex(0xFFFFFF),
            error: Color::from_hex(0xB00020),
            warning: Color::from_hex(0xFF9800),
            success: Color::from_hex(0x4CAF50),
            info: Color::from_hex(0x2196F3),
            text: Color::from_hex(0x212121),
            text_secondary: Color::from_hex(0x757575),
            disabled: Color::from_hex(0xBDBDBD),
            divider: Color::from_hex(0xE0E0E0),
            overlay: Color::rgba(0, 0, 0, 80),
        }
    }

    /// Glass/transparent palette for AR
    pub fn glass() -> Self {
        Self {
            primary: Color::from_hex(0x00E5FF),       // Bright cyan
            primary_variant: Color::from_hex(0x00B8D4),
            secondary: Color::from_hex(0xE040FB),     // Bright purple
            secondary_variant: Color::from_hex(0xAA00FF),
            accent: Color::from_hex(0xFF4081),
            background: Color::rgba(0, 0, 0, 0),      // Transparent
            surface: Color::rgba(30, 30, 30, 180),    // Semi-transparent
            surface_elevated: Color::rgba(45, 45, 45, 200),
            error: Color::from_hex(0xFF5252),
            warning: Color::from_hex(0xFFD740),
            success: Color::from_hex(0x69F0AE),
            info: Color::from_hex(0x40C4FF),
            text: Color::from_hex(0xFFFFFF),
            text_secondary: Color::rgba(255, 255, 255, 180),
            disabled: Color::rgba(255, 255, 255, 100),
            divider: Color::rgba(255, 255, 255, 50),
            overlay: Color::rgba(0, 0, 0, 100),
        }
    }
}

/// Typography settings
#[derive(Debug, Clone)]
pub struct Typography {
    /// Display large (hero text)
    pub display_large: TextStyle,
    /// Display medium
    pub display_medium: TextStyle,
    /// Display small
    pub display_small: TextStyle,
    /// Headline large
    pub headline_large: TextStyle,
    /// Headline medium
    pub headline_medium: TextStyle,
    /// Headline small
    pub headline_small: TextStyle,
    /// Title large
    pub title_large: TextStyle,
    /// Title medium
    pub title_medium: TextStyle,
    /// Title small
    pub title_small: TextStyle,
    /// Body large
    pub body_large: TextStyle,
    /// Body medium
    pub body_medium: TextStyle,
    /// Body small
    pub body_small: TextStyle,
    /// Label large
    pub label_large: TextStyle,
    /// Label medium
    pub label_medium: TextStyle,
    /// Label small
    pub label_small: TextStyle,
    /// Caption
    pub caption: TextStyle,
    /// Overline
    pub overline: TextStyle,
}

impl Default for Typography {
    fn default() -> Self {
        let base_family = "Inter".to_string();
        Self {
            display_large: TextStyle { family: base_family.clone(), size: 57.0, weight: FontWeight::Regular, line_height: 1.12, letter_spacing: -0.25 },
            display_medium: TextStyle { family: base_family.clone(), size: 45.0, weight: FontWeight::Regular, line_height: 1.16, letter_spacing: 0.0 },
            display_small: TextStyle { family: base_family.clone(), size: 36.0, weight: FontWeight::Regular, line_height: 1.22, letter_spacing: 0.0 },
            headline_large: TextStyle { family: base_family.clone(), size: 32.0, weight: FontWeight::Regular, line_height: 1.25, letter_spacing: 0.0 },
            headline_medium: TextStyle { family: base_family.clone(), size: 28.0, weight: FontWeight::Regular, line_height: 1.29, letter_spacing: 0.0 },
            headline_small: TextStyle { family: base_family.clone(), size: 24.0, weight: FontWeight::Regular, line_height: 1.33, letter_spacing: 0.0 },
            title_large: TextStyle { family: base_family.clone(), size: 22.0, weight: FontWeight::Medium, line_height: 1.27, letter_spacing: 0.0 },
            title_medium: TextStyle { family: base_family.clone(), size: 16.0, weight: FontWeight::Medium, line_height: 1.5, letter_spacing: 0.15 },
            title_small: TextStyle { family: base_family.clone(), size: 14.0, weight: FontWeight::Medium, line_height: 1.43, letter_spacing: 0.1 },
            body_large: TextStyle { family: base_family.clone(), size: 16.0, weight: FontWeight::Regular, line_height: 1.5, letter_spacing: 0.5 },
            body_medium: TextStyle { family: base_family.clone(), size: 14.0, weight: FontWeight::Regular, line_height: 1.43, letter_spacing: 0.25 },
            body_small: TextStyle { family: base_family.clone(), size: 12.0, weight: FontWeight::Regular, line_height: 1.33, letter_spacing: 0.4 },
            label_large: TextStyle { family: base_family.clone(), size: 14.0, weight: FontWeight::Medium, line_height: 1.43, letter_spacing: 0.1 },
            label_medium: TextStyle { family: base_family.clone(), size: 12.0, weight: FontWeight::Medium, line_height: 1.33, letter_spacing: 0.5 },
            label_small: TextStyle { family: base_family.clone(), size: 11.0, weight: FontWeight::Medium, line_height: 1.45, letter_spacing: 0.5 },
            caption: TextStyle { family: base_family.clone(), size: 12.0, weight: FontWeight::Regular, line_height: 1.33, letter_spacing: 0.4 },
            overline: TextStyle { family: base_family, size: 10.0, weight: FontWeight::Medium, line_height: 1.6, letter_spacing: 1.5 },
        }
    }
}

impl Typography {
    /// AR-optimized typography (larger, more legible)
    pub fn ar_optimized() -> Self {
        let base_family = "Inter".to_string();
        Self {
            display_large: TextStyle { family: base_family.clone(), size: 64.0, weight: FontWeight::Bold, line_height: 1.1, letter_spacing: -0.5 },
            display_medium: TextStyle { family: base_family.clone(), size: 52.0, weight: FontWeight::Bold, line_height: 1.15, letter_spacing: -0.25 },
            display_small: TextStyle { family: base_family.clone(), size: 40.0, weight: FontWeight::SemiBold, line_height: 1.2, letter_spacing: 0.0 },
            headline_large: TextStyle { family: base_family.clone(), size: 36.0, weight: FontWeight::SemiBold, line_height: 1.22, letter_spacing: 0.0 },
            headline_medium: TextStyle { family: base_family.clone(), size: 32.0, weight: FontWeight::Medium, line_height: 1.25, letter_spacing: 0.0 },
            headline_small: TextStyle { family: base_family.clone(), size: 28.0, weight: FontWeight::Medium, line_height: 1.28, letter_spacing: 0.0 },
            title_large: TextStyle { family: base_family.clone(), size: 26.0, weight: FontWeight::Medium, line_height: 1.23, letter_spacing: 0.0 },
            title_medium: TextStyle { family: base_family.clone(), size: 20.0, weight: FontWeight::Medium, line_height: 1.4, letter_spacing: 0.1 },
            title_small: TextStyle { family: base_family.clone(), size: 18.0, weight: FontWeight::Medium, line_height: 1.33, letter_spacing: 0.1 },
            body_large: TextStyle { family: base_family.clone(), size: 20.0, weight: FontWeight::Regular, line_height: 1.5, letter_spacing: 0.25 },
            body_medium: TextStyle { family: base_family.clone(), size: 18.0, weight: FontWeight::Regular, line_height: 1.44, letter_spacing: 0.25 },
            body_small: TextStyle { family: base_family.clone(), size: 16.0, weight: FontWeight::Regular, line_height: 1.37, letter_spacing: 0.3 },
            label_large: TextStyle { family: base_family.clone(), size: 18.0, weight: FontWeight::Medium, line_height: 1.33, letter_spacing: 0.1 },
            label_medium: TextStyle { family: base_family.clone(), size: 16.0, weight: FontWeight::Medium, line_height: 1.25, letter_spacing: 0.4 },
            label_small: TextStyle { family: base_family.clone(), size: 14.0, weight: FontWeight::Medium, line_height: 1.28, letter_spacing: 0.4 },
            caption: TextStyle { family: base_family.clone(), size: 14.0, weight: FontWeight::Regular, line_height: 1.28, letter_spacing: 0.3 },
            overline: TextStyle { family: base_family, size: 12.0, weight: FontWeight::SemiBold, line_height: 1.5, letter_spacing: 1.0 },
        }
    }
}

/// Text style definition
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl TextStyle {
    pub fn to_font_config(&self) -> FontConfig {
        FontConfig {
            family: self.family.clone(),
            size: self.size,
            weight: self.weight,
            style: FontStyle::Normal,
            line_height: self.line_height,
            letter_spacing: self.letter_spacing,
        }
    }
}

/// Spacing scale (for consistent spacing)
#[derive(Debug, Clone)]
pub struct SpacingScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub xxxl: f32,
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
            xxxl: 64.0,
        }
    }
}

impl SpacingScale {
    /// AR-optimized spacing (more generous)
    pub fn ar_optimized() -> Self {
        Self {
            xs: 6.0,
            sm: 12.0,
            md: 20.0,
            lg: 32.0,
            xl: 44.0,
            xxl: 64.0,
            xxxl: 88.0,
        }
    }
}

/// Border radius scale
#[derive(Debug, Clone)]
pub struct RadiusScale {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub full: f32,
}

impl Default for RadiusScale {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            full: 9999.0,
        }
    }
}

/// Shadow presets
#[derive(Debug, Clone)]
pub struct ShadowPresets {
    pub none: Shadow,
    pub sm: Shadow,
    pub md: Shadow,
    pub lg: Shadow,
    pub xl: Shadow,
    pub glow: Shadow,
}

impl ShadowPresets {
    pub fn dark() -> Self {
        Self {
            none: Shadow::default(),
            sm: Shadow::new(0.0, 1.0, 2.0, Color::rgba(0, 0, 0, 100)),
            md: Shadow::new(0.0, 2.0, 4.0, Color::rgba(0, 0, 0, 120)),
            lg: Shadow::new(0.0, 4.0, 8.0, Color::rgba(0, 0, 0, 140)),
            xl: Shadow::new(0.0, 8.0, 16.0, Color::rgba(0, 0, 0, 160)),
            glow: Shadow::new(0.0, 0.0, 20.0, Color::rgba(0, 188, 212, 100)),
        }
    }

    pub fn light() -> Self {
        Self {
            none: Shadow::default(),
            sm: Shadow::new(0.0, 1.0, 3.0, Color::rgba(0, 0, 0, 30)),
            md: Shadow::new(0.0, 2.0, 6.0, Color::rgba(0, 0, 0, 40)),
            lg: Shadow::new(0.0, 4.0, 12.0, Color::rgba(0, 0, 0, 50)),
            xl: Shadow::new(0.0, 8.0, 24.0, Color::rgba(0, 0, 0, 60)),
            glow: Shadow::new(0.0, 0.0, 20.0, Color::rgba(0, 151, 167, 60)),
        }
    }

    pub fn glass() -> Self {
        Self {
            none: Shadow::default(),
            sm: Shadow::new(0.0, 1.0, 4.0, Color::rgba(0, 229, 255, 30)),
            md: Shadow::new(0.0, 2.0, 8.0, Color::rgba(0, 229, 255, 50)),
            lg: Shadow::new(0.0, 4.0, 16.0, Color::rgba(0, 229, 255, 70)),
            xl: Shadow::new(0.0, 8.0, 32.0, Color::rgba(0, 229, 255, 90)),
            glow: Shadow::new(0.0, 0.0, 24.0, Color::rgba(0, 229, 255, 120)),
        }
    }
}

/// Component-specific styles
#[derive(Debug, Clone)]
pub struct ComponentStyles {
    pub button: ButtonTheme,
    pub text_field: TextFieldTheme,
    pub card: CardTheme,
    pub dialog: DialogTheme,
    pub toast: ToastTheme,
    pub app_bar: AppBarTheme,
    pub navigation: NavigationTheme,
}

impl ComponentStyles {
    pub fn dark() -> Self {
        Self {
            button: ButtonTheme::dark(),
            text_field: TextFieldTheme::dark(),
            card: CardTheme::dark(),
            dialog: DialogTheme::dark(),
            toast: ToastTheme::dark(),
            app_bar: AppBarTheme::dark(),
            navigation: NavigationTheme::dark(),
        }
    }

    pub fn light() -> Self {
        Self {
            button: ButtonTheme::light(),
            text_field: TextFieldTheme::light(),
            card: CardTheme::light(),
            dialog: DialogTheme::light(),
            toast: ToastTheme::light(),
            app_bar: AppBarTheme::light(),
            navigation: NavigationTheme::light(),
        }
    }

    pub fn glass() -> Self {
        Self {
            button: ButtonTheme::glass(),
            text_field: TextFieldTheme::glass(),
            card: CardTheme::glass(),
            dialog: DialogTheme::glass(),
            toast: ToastTheme::glass(),
            app_bar: AppBarTheme::glass(),
            navigation: NavigationTheme::glass(),
        }
    }
}

/// Button theme
#[derive(Debug, Clone)]
pub struct ButtonTheme {
    pub primary_background: Color,
    pub primary_foreground: Color,
    pub secondary_background: Color,
    pub secondary_foreground: Color,
    pub outline_border: Color,
    pub disabled_background: Color,
    pub disabled_foreground: Color,
    pub hover_overlay: Color,
    pub press_overlay: Color,
    pub border_radius: f32,
    pub padding: EdgeInsets,
    pub min_height: f32,
}

impl ButtonTheme {
    pub fn dark() -> Self {
        Self {
            primary_background: Color::from_hex(0x00BCD4),
            primary_foreground: Color::BLACK,
            secondary_background: Color::from_hex(0x333333),
            secondary_foreground: Color::WHITE,
            outline_border: Color::from_hex(0x00BCD4),
            disabled_background: Color::from_hex(0x444444),
            disabled_foreground: Color::from_hex(0x888888),
            hover_overlay: Color::rgba(255, 255, 255, 20),
            press_overlay: Color::rgba(255, 255, 255, 40),
            border_radius: 8.0,
            padding: EdgeInsets::symmetric(20.0, 12.0),
            min_height: 44.0,
        }
    }

    pub fn light() -> Self {
        Self {
            primary_background: Color::from_hex(0x0097A7),
            primary_foreground: Color::WHITE,
            secondary_background: Color::from_hex(0xEEEEEE),
            secondary_foreground: Color::from_hex(0x333333),
            outline_border: Color::from_hex(0x0097A7),
            disabled_background: Color::from_hex(0xE0E0E0),
            disabled_foreground: Color::from_hex(0x9E9E9E),
            hover_overlay: Color::rgba(0, 0, 0, 10),
            press_overlay: Color::rgba(0, 0, 0, 20),
            border_radius: 8.0,
            padding: EdgeInsets::symmetric(20.0, 12.0),
            min_height: 44.0,
        }
    }

    pub fn glass() -> Self {
        Self {
            primary_background: Color::rgba(0, 229, 255, 180),
            primary_foreground: Color::BLACK,
            secondary_background: Color::rgba(255, 255, 255, 30),
            secondary_foreground: Color::WHITE,
            outline_border: Color::rgba(0, 229, 255, 150),
            disabled_background: Color::rgba(100, 100, 100, 80),
            disabled_foreground: Color::rgba(200, 200, 200, 100),
            hover_overlay: Color::rgba(255, 255, 255, 30),
            press_overlay: Color::rgba(255, 255, 255, 50),
            border_radius: 12.0,
            padding: EdgeInsets::symmetric(24.0, 14.0),
            min_height: 48.0,
        }
    }
}

/// TextField theme
#[derive(Debug, Clone)]
pub struct TextFieldTheme {
    pub background: Color,
    pub background_focused: Color,
    pub text: Color,
    pub placeholder: Color,
    pub border: Color,
    pub border_focused: Color,
    pub error: Color,
    pub border_radius: f32,
    pub padding: EdgeInsets,
}

impl TextFieldTheme {
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex(0x1E1E1E),
            background_focused: Color::from_hex(0x2A2A2A),
            text: Color::WHITE,
            placeholder: Color::from_hex(0x888888),
            border: Color::from_hex(0x444444),
            border_focused: Color::from_hex(0x00BCD4),
            error: Color::from_hex(0xCF6679),
            border_radius: 8.0,
            padding: EdgeInsets::symmetric(16.0, 12.0),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::WHITE,
            background_focused: Color::WHITE,
            text: Color::from_hex(0x212121),
            placeholder: Color::from_hex(0x9E9E9E),
            border: Color::from_hex(0xE0E0E0),
            border_focused: Color::from_hex(0x0097A7),
            error: Color::from_hex(0xB00020),
            border_radius: 8.0,
            padding: EdgeInsets::symmetric(16.0, 12.0),
        }
    }

    pub fn glass() -> Self {
        Self {
            background: Color::rgba(30, 30, 30, 150),
            background_focused: Color::rgba(40, 40, 40, 180),
            text: Color::WHITE,
            placeholder: Color::rgba(200, 200, 200, 150),
            border: Color::rgba(100, 100, 100, 100),
            border_focused: Color::rgba(0, 229, 255, 180),
            error: Color::from_hex(0xFF5252),
            border_radius: 10.0,
            padding: EdgeInsets::symmetric(18.0, 14.0),
        }
    }
}

/// Card theme
#[derive(Debug, Clone)]
pub struct CardTheme {
    pub background: Color,
    pub border: Color,
    pub shadow: Shadow,
    pub border_radius: f32,
    pub padding: EdgeInsets,
}

impl CardTheme {
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex(0x1E1E1E),
            border: Color::from_hex(0x333333),
            shadow: Shadow::new(0.0, 2.0, 8.0, Color::rgba(0, 0, 0, 80)),
            border_radius: 12.0,
            padding: EdgeInsets::all(16.0),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::WHITE,
            border: Color::from_hex(0xE0E0E0),
            shadow: Shadow::new(0.0, 2.0, 8.0, Color::rgba(0, 0, 0, 30)),
            border_radius: 12.0,
            padding: EdgeInsets::all(16.0),
        }
    }

    pub fn glass() -> Self {
        Self {
            background: Color::rgba(30, 30, 30, 160),
            border: Color::rgba(100, 100, 100, 80),
            shadow: Shadow::new(0.0, 4.0, 16.0, Color::rgba(0, 229, 255, 40)),
            border_radius: 16.0,
            padding: EdgeInsets::all(20.0),
        }
    }
}

/// Dialog theme
#[derive(Debug, Clone)]
pub struct DialogTheme {
    pub background: Color,
    pub title_color: Color,
    pub content_color: Color,
    pub border_radius: f32,
    pub shadow: Shadow,
    pub overlay: Color,
}

impl DialogTheme {
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex(0x2D2D2D),
            title_color: Color::WHITE,
            content_color: Color::from_hex(0xCCCCCC),
            border_radius: 16.0,
            shadow: Shadow::new(0.0, 8.0, 32.0, Color::rgba(0, 0, 0, 150)),
            overlay: Color::rgba(0, 0, 0, 180),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::WHITE,
            title_color: Color::from_hex(0x212121),
            content_color: Color::from_hex(0x616161),
            border_radius: 16.0,
            shadow: Shadow::new(0.0, 8.0, 32.0, Color::rgba(0, 0, 0, 80)),
            overlay: Color::rgba(0, 0, 0, 100),
        }
    }

    pub fn glass() -> Self {
        Self {
            background: Color::rgba(40, 40, 40, 200),
            title_color: Color::WHITE,
            content_color: Color::rgba(230, 230, 230, 230),
            border_radius: 20.0,
            shadow: Shadow::new(0.0, 8.0, 40.0, Color::rgba(0, 229, 255, 60)),
            overlay: Color::rgba(0, 0, 0, 150),
        }
    }
}

/// Toast/snackbar theme
#[derive(Debug, Clone)]
pub struct ToastTheme {
    pub background: Color,
    pub text_color: Color,
    pub action_color: Color,
    pub border_radius: f32,
}

impl ToastTheme {
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex(0x333333),
            text_color: Color::WHITE,
            action_color: Color::from_hex(0x00BCD4),
            border_radius: 8.0,
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::from_hex(0x323232),
            text_color: Color::WHITE,
            action_color: Color::from_hex(0x00BCD4),
            border_radius: 8.0,
        }
    }

    pub fn glass() -> Self {
        Self {
            background: Color::rgba(50, 50, 50, 200),
            text_color: Color::WHITE,
            action_color: Color::from_hex(0x00E5FF),
            border_radius: 12.0,
        }
    }
}

/// App bar theme
#[derive(Debug, Clone)]
pub struct AppBarTheme {
    pub background: Color,
    pub title_color: Color,
    pub icon_color: Color,
    pub height: f32,
    pub shadow: Shadow,
}

impl AppBarTheme {
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex(0x1E1E1E),
            title_color: Color::WHITE,
            icon_color: Color::WHITE,
            height: 56.0,
            shadow: Shadow::new(0.0, 2.0, 4.0, Color::rgba(0, 0, 0, 60)),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::WHITE,
            title_color: Color::from_hex(0x212121),
            icon_color: Color::from_hex(0x616161),
            height: 56.0,
            shadow: Shadow::new(0.0, 2.0, 4.0, Color::rgba(0, 0, 0, 30)),
        }
    }

    pub fn glass() -> Self {
        Self {
            background: Color::rgba(20, 20, 20, 180),
            title_color: Color::WHITE,
            icon_color: Color::rgba(255, 255, 255, 220),
            height: 64.0,
            shadow: Shadow::new(0.0, 2.0, 8.0, Color::rgba(0, 229, 255, 30)),
        }
    }
}

/// Navigation theme
#[derive(Debug, Clone)]
pub struct NavigationTheme {
    pub background: Color,
    pub selected_item: Color,
    pub unselected_item: Color,
    pub indicator: Color,
    pub height: f32,
}

impl NavigationTheme {
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex(0x1E1E1E),
            selected_item: Color::from_hex(0x00BCD4),
            unselected_item: Color::from_hex(0x888888),
            indicator: Color::rgba(0, 188, 212, 30),
            height: 64.0,
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::WHITE,
            selected_item: Color::from_hex(0x0097A7),
            unselected_item: Color::from_hex(0x757575),
            indicator: Color::rgba(0, 151, 167, 20),
            height: 64.0,
        }
    }

    pub fn glass() -> Self {
        Self {
            background: Color::rgba(20, 20, 20, 180),
            selected_item: Color::from_hex(0x00E5FF),
            unselected_item: Color::rgba(200, 200, 200, 150),
            indicator: Color::rgba(0, 229, 255, 40),
            height: 72.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert_eq!(theme.name, "KaranaOS Dark");
    }

    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert_eq!(theme.mode, ThemeMode::Light);
    }

    #[test]
    fn test_glass_theme() {
        let theme = Theme::ar_glass();
        assert_eq!(theme.name, "KaranaOS Glass");
    }

    #[test]
    fn test_color_by_name() {
        let theme = Theme::dark();
        let primary = theme.color("primary");
        assert_eq!(primary.r, 0);
        assert_eq!(primary.g, 188);
        assert_eq!(primary.b, 212);
    }

    #[test]
    fn test_spacing_scale() {
        let spacing = SpacingScale::default();
        assert!(spacing.xs < spacing.sm);
        assert!(spacing.sm < spacing.md);
        assert!(spacing.md < spacing.lg);
    }

    #[test]
    fn test_typography() {
        let typography = Typography::default();
        assert!(typography.display_large.size > typography.body_medium.size);
    }

    #[test]
    fn test_ar_typography() {
        let ar = Typography::ar_optimized();
        let normal = Typography::default();
        assert!(ar.body_medium.size > normal.body_medium.size);
    }
}
