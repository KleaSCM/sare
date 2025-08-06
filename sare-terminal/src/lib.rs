/**
 * Sare Terminal Emulator Library
 * 
 * This library provides a modern, feature-rich terminal emulator
 * with GPU acceleration, advanced rendering, and developer tools.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: lib.rs
 * Description: Main library module for Sare terminal emulator
 */

pub mod terminal;
pub mod tui;
pub mod gui;
pub mod history;
pub mod features;
pub mod session;
pub mod config;
pub mod unicode;
pub mod ui;

use anyhow::Result;

/**
 * Main terminal emulator struct
 * 
 * メインターミナルエミュレーター構造体です。
 * すべてのターミナル機能を
 * 統合管理します。
 */
pub struct SareTerminal {
	/// Terminal configuration
	config: crate::config::Config,
	/// UI manager
	ui_manager: crate::ui::UiManager,
	/// Terminal sessions
	sessions: Vec<crate::session::SessionManager>,
	/// History manager
	history: crate::history::HistoryManager,
}

impl SareTerminal {
	/**
	 * Creates a new terminal emulator instance
	 * 
	 * @param config - Terminal configuration
	 * @return Result<SareTerminal> - New terminal instance or error
	 */
	pub async fn new(config: crate::config::Config) -> Result<Self> {
		let ui_config = crate::ui::UiConfig::default();
		let ui_manager = crate::ui::UiManager::new(ui_config);
		
		Ok(Self {
			config,
			ui_manager,
			sessions: Vec::new(),
			history: crate::history::HistoryManager::new()?,
		})
	}
	
	/**
	 * Initializes the terminal emulator
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		// Initialize UI manager
		self.ui_manager.initialize().await?;
		
		// Initialize history manager
		self.history.initialize().await?;
		
		Ok(())
	}
	
	/**
	 * Runs the terminal emulator
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn run(&mut self) -> Result<()> {
		// Main terminal loop
		loop {
			// Update UI
			if self.ui_manager.update().await? {
				// Render UI
				let ui_content = self.ui_manager.render().await?;
				print!("{}", ui_content);
			}
			
			// Process input
			// TODO: Implement input processing
			
			// Small delay to prevent high CPU usage
			tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
		}
	}
	
	/**
	 * Gets the UI manager
	 * 
	 * @return &crate::ui::UiManager - UI manager reference
	 */
	pub fn ui_manager(&self) -> &crate::ui::UiManager {
		&self.ui_manager
	}
	
	/**
	 * Gets a mutable UI manager
	 * 
	 * @return &mut crate::ui::UiManager - Mutable UI manager reference
	 */
	pub fn ui_manager_mut(&mut self) -> &mut crate::ui::UiManager {
		&mut self.ui_manager
	}
	
	/**
	 * Gets the history manager
	 * 
	 * @return &crate::history::HistoryManager - History manager reference
	 */
	pub fn history_manager(&self) -> &crate::history::HistoryManager {
		&self.history
	}
	
	/**
	 * Gets a mutable history manager
	 * 
	 * @return &mut crate::history::HistoryManager - Mutable history manager reference
	 */
	pub fn history_manager_mut(&mut self) -> &mut crate::history::HistoryManager {
		&mut self.history
	}
} 