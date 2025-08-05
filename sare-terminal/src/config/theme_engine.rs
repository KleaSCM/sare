/**
 * Theme engine for Sare terminal
 * 
 * This module provides comprehensive theming capabilities including
 * color schemes, fonts, styling, and dynamic theme switching.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: theme_engine.rs
 * Description: Theme engine with color schemes, fonts, and styling
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/**
 * Color scheme definition
 * 
 * カラースキームの定義です。
 * ターミナルの色設定を管理し、
 * 美しいテーマを提供します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
	/// Background color (RGBA)
	pub background: u32,
	/// Foreground color (RGBA)
	pub foreground: u32,
	/// Cursor color (RGBA)
	pub cursor: u32,
	/// Selection background (RGBA)
	pub selection_background: u32,
	/// Selection foreground (RGBA)
	pub selection_foreground: u32,
	/// ANSI color palette (16 colors)
	pub palette: [u32; 16],
	/// Bright ANSI colors (8 colors)
	pub bright_palette: [u32; 8],
	/// Dim ANSI colors (8 colors)
	pub dim_palette: [u32; 8],
}

impl Default for ColorScheme {
	fn default() -> Self {
		Self {
			background: 0xFF1E1E1E, // Dark gray
			foreground: 0xFFD4D4D4, // Light gray
			cursor: 0xFFFFFFFF,     // White
			selection_background: 0xFF264F78, // Blue
			selection_foreground: 0xFFFFFFFF, // White
			palette: [
				0xFF000000, // Black
				0xFFCD3131, // Red
				0xFF0DBC79, // Green
				0xFFE5E510, // Yellow
				0xFF2472C8, // Blue
				0xFFBC3FBC, // Magenta
				0xFF11A8CD, // Cyan
				0xFFE5E5E5, // White
				0xFF666666, // Bright Black
				0xFFF14C4C, // Bright Red
				0xFF23D18B, // Bright Green
				0xFFF5F543, // Bright Yellow
				0xFF3B8EEA, // Bright Blue
				0xFFD670D6, // Bright Magenta
				0xFF29B8DB, // Bright Cyan
				0xFFFFFFFF, // Bright White
			],
			bright_palette: [
				0xFF666666, // Bright Black
				0xFFF14C4C, // Bright Red
				0xFF23D18B, // Bright Green
				0xFFF5F543, // Bright Yellow
				0xFF3B8EEA, // Bright Blue
				0xFFD670D6, // Bright Magenta
				0xFF29B8DB, // Bright Cyan
				0xFFFFFFFF, // Bright White
			],
			dim_palette: [
				0xFF000000, // Dim Black
				0xFF8B0000, // Dim Red
				0xFF006400, // Dim Green
				0xFF8B8B00, // Dim Yellow
				0xFF00008B, // Dim Blue
				0xFF8B008B, // Dim Magenta
				0xFF008B8B, // Dim Cyan
				0xFF8B8B8B, // Dim White
			],
		}
	}
}

/**
 * Font configuration
 * 
 * フォント設定です。
 * フォントファミリー、サイズ、スタイルを
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
	/// Font family name
	pub family: String,
	/// Font size in points
	pub size: f32,
	/// Font weight (100-900)
	pub weight: u32,
	/// Font style (normal, italic)
	pub style: String,
	/// Enable ligatures
	pub ligatures: bool,
	/// Enable subpixel antialiasing
	pub subpixel_antialiasing: bool,
}

impl Default for FontConfig {
	fn default() -> Self {
		Self {
			family: "Monaco".to_string(),
			size: 14.0,
			weight: 400,
			style: "normal".to_string(),
			ligatures: true,
			subpixel_antialiasing: true,
		}
	}
}

/**
 * Theme definition
 * 
 * テーマの定義です。
 * カラースキーム、フォント、スタイルを
 * 統合してテーマを管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
	/// Theme name
	pub name: String,
	/// Theme description
	pub description: String,
	/// Color scheme
	pub colors: ColorScheme,
	/// Font configuration
	pub font: FontConfig,
	/// UI styling
	pub ui: UiStyle,
}

impl Default for Theme {
	fn default() -> Self {
		Self {
			name: "Default Dark".to_string(),
			description: "Default dark theme for Sare terminal".to_string(),
			colors: ColorScheme::default(),
			font: FontConfig::default(),
			ui: UiStyle::default(),
		}
	}
}

/**
 * UI styling configuration
 * 
 * UIスタイリング設定です。
 * パネル、ボーダー、スクロールバーの
 * スタイルを管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiStyle {
	/// Panel background color
	pub panel_background: u32,
	/// Panel border color
	pub panel_border: u32,
	/// Panel border width
	pub panel_border_width: f32,
	/// Scrollbar color
	pub scrollbar_color: u32,
	/// Scrollbar width
	pub scrollbar_width: f32,
	/// Status bar background
	pub status_background: u32,
	/// Status bar foreground
	pub status_foreground: u32,
	/// Tab background
	pub tab_background: u32,
	/// Tab foreground
	pub tab_foreground: u32,
	/// Active tab background
	pub active_tab_background: u32,
	/// Active tab foreground
	pub active_tab_foreground: u32,
}

impl Default for UiStyle {
	fn default() -> Self {
		Self {
			panel_background: 0xFF2D2D2D,
			panel_border: 0xFF404040,
			panel_border_width: 1.0,
			scrollbar_color: 0xFF404040,
			scrollbar_width: 8.0,
			status_background: 0xFF1E1E1E,
			status_foreground: 0xFFD4D4D4,
			tab_background: 0xFF2D2D2D,
			tab_foreground: 0xFFD4D4D4,
			active_tab_background: 0xFF404040,
			active_tab_foreground: 0xFFFFFFFF,
		}
	}
}

/**
 * Theme engine
 * 
 * テーマエンジンです。
 * テーマの管理、切り替え、動的更新を
 * 提供します。
 */
pub struct ThemeEngine {
	/// Available themes
	themes: Arc<RwLock<HashMap<String, Theme>>>,
	/// Current active theme
	current_theme: Arc<RwLock<String>>,
	/// Theme change callbacks
	callbacks: Arc<RwLock<Vec<Box<dyn Fn(&Theme) + Send + Sync>>>>,
}

impl ThemeEngine {
	/**
	 * Creates a new theme engine
	 * 
	 * @return ThemeEngine - New theme engine instance
	 */
	pub fn new() -> Self {
		let mut themes = HashMap::new();
		
		// Add default themes
		themes.insert("default-dark".to_string(), Theme::default());
		themes.insert("default-light".to_string(), Self::create_light_theme());
		themes.insert("dracula".to_string(), Self::create_dracula_theme());
		themes.insert("solarized-dark".to_string(), Self::create_solarized_dark_theme());
		themes.insert("solarized-light".to_string(), Self::create_solarized_light_theme());
		themes.insert("gruvbox-dark".to_string(), Self::create_gruvbox_dark_theme());
		themes.insert("nord".to_string(), Self::create_nord_theme());
		
		Self {
			themes: Arc::new(RwLock::new(themes)),
			current_theme: Arc::new(RwLock::new("default-dark".to_string())),
			callbacks: Arc::new(RwLock::new(Vec::new())),
		}
	}
	
	/**
	 * Gets the current theme
	 * 
	 * @return Result<Theme> - Current theme or error
	 */
	pub async fn get_current_theme(&self) -> Result<Theme> {
		let theme_name = self.current_theme.read().await;
		let themes = self.themes.read().await;
		
		themes.get(&*theme_name)
			.cloned()
			.ok_or_else(|| anyhow::anyhow!("Theme not found: {}", *theme_name))
	}
	
	/**
	 * Sets the current theme
	 * 
	 * @param theme_name - Theme name to switch to
	 * @return Result<()> - Success or error status
	 */
	pub async fn set_theme(&self, theme_name: &str) -> Result<()> {
		let themes = self.themes.read().await;
		if !themes.contains_key(theme_name) {
			return Err(anyhow::anyhow!("Theme not found: {}", theme_name));
		}
		
		// Update current theme
		{
			let mut current = self.current_theme.write().await;
			*current = theme_name.to_string();
		}
		
		// Notify callbacks
		if let Some(theme) = themes.get(theme_name) {
			let callbacks = self.callbacks.read().await;
			for callback in callbacks.iter() {
				callback(theme);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Adds a theme
	 * 
	 * @param theme - Theme to add
	 * @return Result<()> - Success or error status
	 */
	pub async fn add_theme(&self, theme: Theme) -> Result<()> {
		let mut themes = self.themes.write().await;
		themes.insert(theme.name.clone(), theme);
		Ok(())
	}
	
	/**
	 * Removes a theme
	 * 
	 * @param theme_name - Theme name to remove
	 * @return Result<()> - Success or error status
	 */
	pub async fn remove_theme(&self, theme_name: &str) -> Result<()> {
		let mut themes = self.themes.write().await;
		if theme_name == "default-dark" {
			return Err(anyhow::anyhow!("Cannot remove default theme"));
		}
		
		themes.remove(theme_name);
		Ok(())
	}
	
	/**
	 * Gets all available themes
	 * 
	 * @return Vec<String> - List of theme names
	 */
	pub async fn get_theme_names(&self) -> Vec<String> {
		let themes = self.themes.read().await;
		themes.keys().cloned().collect()
	}
	
	/**
	 * Registers a theme change callback
	 * 
	 * @param callback - Callback function
	 */
	pub async fn register_callback<F>(&self, callback: F)
	where
		F: Fn(&Theme) + Send + Sync + 'static,
	{
		let mut callbacks = self.callbacks.write().await;
		callbacks.push(Box::new(callback));
	}
	
	/**
	 * Creates a light theme
	 * 
	 * @return Theme - Light theme
	 */
	fn create_light_theme() -> Theme {
		Theme {
			name: "Default Light".to_string(),
			description: "Default light theme for Sare terminal".to_string(),
			colors: ColorScheme {
				background: 0xFFFFFFFF, // White
				foreground: 0xFF000000, // Black
				cursor: 0xFF000000,     // Black
				selection_background: 0xFFB3D4FC, // Light blue
				selection_foreground: 0xFF000000, // Black
				palette: [
					0xFF000000, // Black
					0xFFCD3131, // Red
					0xFF00BC00, // Green
					0xFF949800, // Yellow
					0xFF0451A5, // Blue
					0xFFBC05BC, // Magenta
					0xFF0598BC, // Cyan
					0xFF555555, // White
					0xFF666666, // Bright Black
					0xFFCD3131, // Bright Red
					0xFF14CE14, // Bright Green
					0xFFB5BA00, // Bright Yellow
					0xFF0451A5, // Bright Blue
					0xFFBC05BC, // Bright Magenta
					0xFF0598BC, // Bright Cyan
					0xFFA5A5A5, // Bright White
				],
				bright_palette: [
					0xFF666666, // Bright Black
					0xFFCD3131, // Bright Red
					0xFF14CE14, // Bright Green
					0xFFB5BA00, // Bright Yellow
					0xFF0451A5, // Bright Blue
					0xFFBC05BC, // Bright Magenta
					0xFF0598BC, // Bright Cyan
					0xFFA5A5A5, // Bright White
				],
				dim_palette: [
					0xFF000000, // Dim Black
					0xFF8B0000, // Dim Red
					0xFF006400, // Dim Green
					0xFF8B8B00, // Dim Yellow
					0xFF00008B, // Dim Blue
					0xFF8B008B, // Dim Magenta
					0xFF008B8B, // Dim Cyan
					0xFF8B8B8B, // Dim White
				],
			},
			font: FontConfig::default(),
			ui: UiStyle {
				panel_background: 0xFFF5F5F5,
				panel_border: 0xFFE0E0E0,
				panel_border_width: 1.0,
				scrollbar_color: 0xFFE0E0E0,
				scrollbar_width: 8.0,
				status_background: 0xFFF5F5F5,
				status_foreground: 0xFF000000,
				tab_background: 0xFFE0E0E0,
				tab_foreground: 0xFF000000,
				active_tab_background: 0xFFF5F5F5,
				active_tab_foreground: 0xFF000000,
			},
		}
	}
	
	/**
	 * Creates Dracula theme
	 * 
	 * @return Theme - Dracula theme
	 */
	fn create_dracula_theme() -> Theme {
		Theme {
			name: "Dracula".to_string(),
			description: "Dracula color scheme".to_string(),
			colors: ColorScheme {
				background: 0xFF282A36, // Dracula background
				foreground: 0xFFF8F8F2, // Dracula foreground
				cursor: 0xFFF8F8F2,     // Dracula cursor
				selection_background: 0xFF44475A, // Dracula selection
				selection_foreground: 0xFFF8F8F2, // Dracula selection fg
				palette: [
					0xFF000000, // Black
					0xFFFF5555, // Dracula red
					0xFF50FA7B, // Dracula green
					0xFFFFB86C, // Dracula yellow
					0xFFBD93F9, // Dracula purple
					0xFFFF79C6, // Dracula pink
					0xFF8BE9FD, // Dracula cyan
					0xFFBFBFBF, // Dracula white
					0xFF4D4D4D, // Bright black
					0xFFFF6E6E, // Bright red
					0xFF69FF94, // Bright green
					0xFFFFCB8B, // Bright yellow
					0xFFD6ACFF, // Bright purple
					0xFFFF92DF, // Bright pink
					0xFFA4FFFF, // Bright cyan
					0xFFFFFFFF, // Bright white
				],
				bright_palette: [
					0xFF4D4D4D, // Bright black
					0xFFFF6E6E, // Bright red
					0xFF69FF94, // Bright green
					0xFFFFCB8B, // Bright yellow
					0xFFD6ACFF, // Bright purple
					0xFFFF92DF, // Bright pink
					0xFFA4FFFF, // Bright cyan
					0xFFFFFFFF, // Bright white
				],
				dim_palette: [
					0xFF000000, // Dim black
					0xFF8B0000, // Dim red
					0xFF006400, // Dim green
					0xFF8B8B00, // Dim yellow
					0xFF00008B, // Dim blue
					0xFF8B008B, // Dim magenta
					0xFF008B8B, // Dim cyan
					0xFF8B8B8B, // Dim white
				],
			},
			font: FontConfig {
				family: "JetBrains Mono".to_string(),
				size: 14.0,
				weight: 400,
				style: "normal".to_string(),
				ligatures: true,
				subpixel_antialiasing: true,
			},
			ui: UiStyle {
				panel_background: 0xFF282A36,
				panel_border: 0xFF44475A,
				panel_border_width: 1.0,
				scrollbar_color: 0xFF44475A,
				scrollbar_width: 8.0,
				status_background: 0xFF282A36,
				status_foreground: 0xFFF8F8F2,
				tab_background: 0xFF44475A,
				tab_foreground: 0xFFF8F8F2,
				active_tab_background: 0xFF6272A4,
				active_tab_foreground: 0xFFF8F8F2,
			},
		}
	}
	
	/**
	 * Creates Solarized Dark theme
	 * 
	 * @return Theme - Solarized Dark theme
	 */
	fn create_solarized_dark_theme() -> Theme {
		Theme {
			name: "Solarized Dark".to_string(),
			description: "Solarized dark color scheme".to_string(),
			colors: ColorScheme {
				background: 0xFF002B36, // Solarized base03
				foreground: 0xFF839496, // Solarized base0
				cursor: 0xFF93A1A1,     // Solarized base1
				selection_background: 0xFF073642, // Solarized base02
				selection_foreground: 0xFF93A1A1, // Solarized base1
				palette: [
					0xFF073642, // Solarized base02
					0xFFDC322F, // Solarized red
					0xFF859900, // Solarized green
					0xFFB58900, // Solarized yellow
					0xFF268BD2, // Solarized blue
					0xFFD33682, // Solarized magenta
					0xFF2AA198, // Solarized cyan
					0xFFEEE8D5, // Solarized base2
					0xFF002B36, // Solarized base03
					0xFFCB4B16, // Solarized orange
					0xFF586E75, // Solarized base01
					0xFF657B83, // Solarized base00
					0xFF839496, // Solarized base0
					0xFF6C71C4, // Solarized violet
					0xFF93A1A1, // Solarized base1
					0xFFFDF6E3, // Solarized base3
				],
				bright_palette: [
					0xFF002B36, // Solarized base03
					0xFFCB4B16, // Solarized orange
					0xFF586E75, // Solarized base01
					0xFF657B83, // Solarized base00
					0xFF839496, // Solarized base0
					0xFF6C71C4, // Solarized violet
					0xFF93A1A1, // Solarized base1
					0xFFFDF6E3, // Solarized base3
				],
				dim_palette: [
					0xFF073642, // Solarized base02
					0xFF8B0000, // Dim red
					0xFF006400, // Dim green
					0xFF8B8B00, // Dim yellow
					0xFF00008B, // Dim blue
					0xFF8B008B, // Dim magenta
					0xFF008B8B, // Dim cyan
					0xFF8B8B8B, // Dim white
				],
			},
			font: FontConfig::default(),
			ui: UiStyle {
				panel_background: 0xFF002B36,
				panel_border: 0xFF073642,
				panel_border_width: 1.0,
				scrollbar_color: 0xFF073642,
				scrollbar_width: 8.0,
				status_background: 0xFF002B36,
				status_foreground: 0xFF839496,
				tab_background: 0xFF073642,
				tab_foreground: 0xFF839496,
				active_tab_background: 0xFF073642,
				active_tab_foreground: 0xFF93A1A1,
			},
		}
	}
	
	/**
	 * Creates Solarized Light theme
	 * 
	 * @return Theme - Solarized Light theme
	 */
	fn create_solarized_light_theme() -> Theme {
		Theme {
			name: "Solarized Light".to_string(),
			description: "Solarized light color scheme".to_string(),
			colors: ColorScheme {
				background: 0xFFFDF6E3, // Solarized base3
				foreground: 0xFF657B83, // Solarized base00
				cursor: 0xFF586E75,     // Solarized base01
				selection_background: 0xFFEEE8D5, // Solarized base2
				selection_foreground: 0xFF586E75, // Solarized base01
				palette: [
					0xFFEEE8D5, // Solarized base2
					0xFFDC322F, // Solarized red
					0xFF859900, // Solarized green
					0xFFB58900, // Solarized yellow
					0xFF268BD2, // Solarized blue
					0xFFD33682, // Solarized magenta
					0xFF2AA198, // Solarized cyan
					0xFF073642, // Solarized base02
					0xFFFDF6E3, // Solarized base3
					0xFFCB4B16, // Solarized orange
					0xFF93A1A1, // Solarized base1
					0xFF839496, // Solarized base0
					0xFF657B83, // Solarized base00
					0xFF6C71C4, // Solarized violet
					0xFF586E75, // Solarized base01
					0xFF002B36, // Solarized base03
				],
				bright_palette: [
					0xFFFDF6E3, // Solarized base3
					0xFFCB4B16, // Solarized orange
					0xFF93A1A1, // Solarized base1
					0xFF839496, // Solarized base0
					0xFF657B83, // Solarized base00
					0xFF6C71C4, // Solarized violet
					0xFF586E75, // Solarized base01
					0xFF002B36, // Solarized base03
				],
				dim_palette: [
					0xFFEEE8D5, // Solarized base2
					0xFF8B0000, // Dim red
					0xFF006400, // Dim green
					0xFF8B8B00, // Dim yellow
					0xFF00008B, // Dim blue
					0xFF8B008B, // Dim magenta
					0xFF008B8B, // Dim cyan
					0xFF8B8B8B, // Dim white
				],
			},
			font: FontConfig::default(),
			ui: UiStyle {
				panel_background: 0xFFFDF6E3,
				panel_border: 0xFFEEE8D5,
				panel_border_width: 1.0,
				scrollbar_color: 0xFFEEE8D5,
				scrollbar_width: 8.0,
				status_background: 0xFFFDF6E3,
				status_foreground: 0xFF657B83,
				tab_background: 0xFFEEE8D5,
				tab_foreground: 0xFF657B83,
				active_tab_background: 0xFFEEE8D5,
				active_tab_foreground: 0xFF586E75,
			},
		}
	}
	
	/**
	 * Creates Gruvbox Dark theme
	 * 
	 * @return Theme - Gruvbox Dark theme
	 */
	fn create_gruvbox_dark_theme() -> Theme {
		Theme {
			name: "Gruvbox Dark".to_string(),
			description: "Gruvbox dark color scheme".to_string(),
			colors: ColorScheme {
				background: 0xFF282828, // Gruvbox bg0
				foreground: 0xFFEBDBB2, // Gruvbox fg
				cursor: 0xFFEBDBB2,     // Gruvbox fg
				selection_background: 0xFF3C3836, // Gruvbox bg1
				selection_foreground: 0xFFEBDBB2, // Gruvbox fg
				palette: [
					0xFF282828, // Gruvbox bg0
					0xFFCC241D, // Gruvbox red
					0xFF98971A, // Gruvbox green
					0xFFD79921, // Gruvbox yellow
					0xFF458588, // Gruvbox blue
					0xFFB16286, // Gruvbox purple
					0xFF689D6A, // Gruvbox aqua
					0xFFA89984, // Gruvbox gray
					0xFF928374, // Gruvbox fg4
					0xFFFB4934, // Gruvbox red
					0xFFB8BB26, // Gruvbox green
					0xFFFABD2F, // Gruvbox yellow
					0xFF83A598, // Gruvbox blue
					0xFFD3869B, // Gruvbox purple
					0xFF8EC07C, // Gruvbox aqua
					0xFFEBDBB2, // Gruvbox fg
				],
				bright_palette: [
					0xFF928374, // Gruvbox fg4
					0xFFFB4934, // Gruvbox red
					0xFFB8BB26, // Gruvbox green
					0xFFFABD2F, // Gruvbox yellow
					0xFF83A598, // Gruvbox blue
					0xFFD3869B, // Gruvbox purple
					0xFF8EC07C, // Gruvbox aqua
					0xFFEBDBB2, // Gruvbox fg
				],
				dim_palette: [
					0xFF282828, // Gruvbox bg0
					0xFF8B0000, // Dim red
					0xFF006400, // Dim green
					0xFF8B8B00, // Dim yellow
					0xFF00008B, // Dim blue
					0xFF8B008B, // Dim magenta
					0xFF008B8B, // Dim cyan
					0xFF8B8B8B, // Dim white
				],
			},
			font: FontConfig::default(),
			ui: UiStyle {
				panel_background: 0xFF282828,
				panel_border: 0xFF3C3836,
				panel_border_width: 1.0,
				scrollbar_color: 0xFF3C3836,
				scrollbar_width: 8.0,
				status_background: 0xFF282828,
				status_foreground: 0xFFEBDBB2,
				tab_background: 0xFF3C3836,
				tab_foreground: 0xFFEBDBB2,
				active_tab_background: 0xFF504945,
				active_tab_foreground: 0xFFEBDBB2,
			},
		}
	}
	
	/**
	 * Creates Nord theme
	 * 
	 * @return Theme - Nord theme
	 */
	fn create_nord_theme() -> Theme {
		Theme {
			name: "Nord".to_string(),
			description: "Nord color scheme".to_string(),
			colors: ColorScheme {
				background: 0xFF2E3440, // Nord polar night
				foreground: 0xFFECEFF4, // Nord snow storm
				cursor: 0xFFECEFF4,     // Nord snow storm
				selection_background: 0xFF3B4252, // Nord polar night
				selection_foreground: 0xFFECEFF4, // Nord snow storm
				palette: [
					0xFF2E3440, // Nord polar night
					0xFFBF616A, // Nord aurora red
					0xFFA3BE8C, // Nord aurora green
					0xFFEBCB8B, // Nord aurora yellow
					0xFF81A1C1, // Nord aurora blue
					0xFFB48EAD, // Nord aurora purple
					0xFF88C0D0, // Nord aurora cyan
					0xFFECEFF4, // Nord snow storm
					0xFF4C566A, // Nord polar night
					0xFFBF616A, // Nord aurora red
					0xFFA3BE8C, // Nord aurora green
					0xFFEBCB8B, // Nord aurora yellow
					0xFF81A1C1, // Nord aurora blue
					0xFFB48EAD, // Nord aurora purple
					0xFF88C0D0, // Nord aurora cyan
					0xFFECEFF4, // Nord snow storm
				],
				bright_palette: [
					0xFF4C566A, // Nord polar night
					0xFFBF616A, // Nord aurora red
					0xFFA3BE8C, // Nord aurora green
					0xFFEBCB8B, // Nord aurora yellow
					0xFF81A1C1, // Nord aurora blue
					0xFFB48EAD, // Nord aurora purple
					0xFF88C0D0, // Nord aurora cyan
					0xFFECEFF4, // Nord snow storm
				],
				dim_palette: [
					0xFF2E3440, // Nord polar night
					0xFF8B0000, // Dim red
					0xFF006400, // Dim green
					0xFF8B8B00, // Dim yellow
					0xFF00008B, // Dim blue
					0xFF8B008B, // Dim magenta
					0xFF008B8B, // Dim cyan
					0xFF8B8B8B, // Dim white
				],
			},
			font: FontConfig::default(),
			ui: UiStyle {
				panel_background: 0xFF2E3440,
				panel_border: 0xFF3B4252,
				panel_border_width: 1.0,
				scrollbar_color: 0xFF3B4252,
				scrollbar_width: 8.0,
				status_background: 0xFF2E3440,
				status_foreground: 0xFFECEFF4,
				tab_background: 0xFF3B4252,
				tab_foreground: 0xFFECEFF4,
				active_tab_background: 0xFF434C5E,
				active_tab_foreground: 0xFFECEFF4,
			},
		}
	}
} 