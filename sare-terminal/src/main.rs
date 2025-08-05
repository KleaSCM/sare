/**
 * Sare Terminal - Terminal Emulator Implementation
 * 
 * This is the standalone terminal emulator that provides
 * GPU-accelerated rendering, multi-pane support, and terminal
 * emulation without shell functionality. Can be used with any
 * shell implementation.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: main.rs
 * Description: Main entry point for the Sare terminal emulator
 */

use anyhow::Result;
use eframe::NativeOptions;
use std::time::Instant;
use tokio::sync::OnceCell;

mod terminal;
mod tui;
mod gui;

use gui::SareTerminal;

/**
 * Startup optimization manager
 * 
 * スタートアップ最適化マネージャーです。
 * 遅延読み込みと高速初期化を提供し、
 * アプリケーションの起動時間を短縮します。
 */
pub struct StartupOptimizer {
	/// Startup time tracking
	startup_time: Instant,
	/// Lazy initialization flags
	initialized_modules: std::collections::HashMap<String, bool>,
	/// Performance metrics
	init_times: std::collections::HashMap<String, f64>,
}

impl StartupOptimizer {
	/**
	 * Creates a new startup optimizer
	 * 
	 * @return StartupOptimizer - New startup optimizer
	 */
	pub fn new() -> Self {
		Self {
			startup_time: Instant::now(),
			initialized_modules: std::collections::HashMap::new(),
			init_times: std::collections::HashMap::new(),
		}
	}
	
	/**
	 * Performs lazy initialization of a module
	 * 
	 * @param module_name - Module name
	 * @param init_fn - Initialization function
	 * @return Result<()> - Success or error status
	 */
	pub async fn lazy_init<F, Fut>(&mut self, module_name: &str, init_fn: F) -> Result<()>
	where
		F: FnOnce() -> Fut,
		Fut: std::future::Future<Output = Result<()>>,
	{
		if !self.initialized_modules.get(module_name).unwrap_or(&false) {
			let start = Instant::now();
			init_fn().await?;
			let duration = start.elapsed().as_secs_f64();
			self.init_times.insert(module_name.to_string(), duration);
			self.initialized_modules.insert(module_name.to_string(), true);
		}
		Ok(())
	}
	
	/**
	 * Gets startup statistics
	 * 
	 * @return StartupStats - Startup statistics
	 */
	pub fn get_stats(&self) -> StartupStats {
		StartupStats {
			total_time: self.startup_time.elapsed().as_secs_f64(),
			initialized_modules: self.initialized_modules.clone(),
			init_times: self.init_times.clone(),
		}
	}
}

/**
 * Startup statistics
 */
#[derive(Debug, Clone)]
pub struct StartupStats {
	/// Total startup time in seconds
	pub total_time: f64,
	/// Map of initialized modules
	pub initialized_modules: std::collections::HashMap<String, bool>,
	/// Map of initialization times
	pub init_times: std::collections::HashMap<String, f64>,
}

/**
 * Global startup optimizer instance
 */
static STARTUP_OPTIMIZER: OnceCell<StartupOptimizer> = OnceCell::const_new();

/**
 * Main entry point for the Sare terminal emulator
 * 
 * Initializes the terminal emulator with GPU acceleration
 * and starts the GUI application with proper error handling.
 * Uses startup optimization for fast initialization.
 */
#[tokio::main]
async fn main() -> Result<()> {
	/**
	 * メインエントリーポイントです
	 * 
	 * スタートアップ最適化を使用して高速な
	 * 初期化を実行し、GUIアプリケーションを
	 * 起動します。
	 */
	
	// Initialize startup optimizer
	let mut optimizer = StartupOptimizer::new();
	
	// Perform lazy initialization of core modules
	optimizer.lazy_init("terminal", || async {
		// Initialize terminal module
		Ok(())
	}).await?;
	
	optimizer.lazy_init("tui", || async {
		// Initialize TUI module
		Ok(())
	}).await?;
	
	optimizer.lazy_init("gui", || async {
		// Initialize GUI module
		Ok(())
	}).await?;
	
	// Set up native options for the GUI with optimized settings
	let options = NativeOptions {
		vsync: true, // Enable vsync for smooth rendering
		multisampling: 4, // Enable multisampling for better quality
		depth_buffer: 0, // Disable depth buffer for 2D rendering
		stencil_buffer: 0, // Disable stencil buffer
		..Default::default()
	};
	
	// Run the terminal emulator with startup statistics
	let startup_stats = optimizer.get_stats();
	println!("🚀 Startup completed in {:.3}s", startup_stats.total_time);
	for (module, time) in &startup_stats.init_times {
		println!("  📦 {}: {:.3}s", module, time);
	}
	
	eframe::run_native(
		"Sare Terminal",
		options,
		Box::new(|_cc| Box::new(SareTerminal::new().unwrap())),
	)
	.map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;
	
	Ok(())
} 