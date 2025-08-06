use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

// Module declarations
pub mod gui;
pub mod tui;
pub mod terminal;
pub mod session;
pub mod config;
pub mod features;
pub mod ui;
pub mod debug;
pub mod unicode;
pub mod history;

pub struct SareTerminal {
	terminal_state: Arc<RwLock<TerminalState>>,
	running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct TerminalState {
	pub size: (u16, u16),
	pub cursor_pos: (u16, u16),
	pub scroll_pos: u32,
	pub mode: TerminalMode,
	pub last_update: Instant,
}

#[derive(Debug, Clone)]
pub struct TerminalMode {
	pub insert_mode: bool,
	pub app_cursor_keys: bool,
	pub app_keypad: bool,
	pub mouse_tracking: bool,
	pub bracketed_paste: bool,
	pub debug_mode: bool,
}

impl Default for TerminalMode {
	fn default() -> Self {
		Self {
			insert_mode: false,
			app_cursor_keys: false,
			app_keypad: false,
			mouse_tracking: false,
			bracketed_paste: false,
			debug_mode: false,
		}
	}
}

impl SareTerminal {
	pub async fn new() -> Result<Self> {
		let terminal_state = Arc::new(RwLock::new(TerminalState {
			size: (80, 24),
			cursor_pos: (0, 0),
			scroll_pos: 0,
			mode: TerminalMode::default(),
			last_update: Instant::now(),
		}));
		let running = Arc::new(RwLock::new(true));
		
		Ok(Self {
			terminal_state,
			running,
		})
	}
	
	pub async fn initialize(&mut self) -> Result<()> {
		println!("🚀 Initializing Sare Terminal Emulator...");
		println!("✅ Sare Terminal Emulator initialized successfully!");
		Ok(())
	}
	
	pub async fn run(&mut self) -> Result<()> {
		println!("🎯 Starting Sare Terminal Emulator main loop...");
		
		let start_time = Instant::now();
		
		while *self.running.read().await {
			let loop_start = Instant::now();
			
			self.update_terminal_state().await?;
			
			let loop_duration = loop_start.elapsed();
			if loop_duration < Duration::from_millis(16) {
				tokio::time::sleep(Duration::from_millis(16) - loop_duration).await;
			}
		}
		
		let total_runtime = start_time.elapsed();
		println!("🛑 Sare Terminal Emulator stopped after {:?}", total_runtime);
		
		Ok(())
	}
	
	async fn update_terminal_state(&self) -> Result<()> {
		let mut state = self.terminal_state.write().await;
		state.last_update = Instant::now();
		Ok(())
	}
	
	pub async fn stop(&self) -> Result<()> {
		println!("🛑 Stopping Sare Terminal Emulator...");
		
		{
			let mut running = self.running.write().await;
			*running = false;
		}
		
		Ok(())
	}
} 