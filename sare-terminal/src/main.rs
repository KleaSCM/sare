/**
 * Sare Terminal Emulator - Main Entry Point
 * 
 * This is the main entry point for the Sare terminal emulator,
 * providing a modern, feature-rich terminal experience with
 * GPU acceleration, advanced rendering, and developer tools.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: main.rs
 * Description: Main entry point for Sare terminal emulator
 */

use anyhow::Result;
use std::time::Instant;
use tokio::sync::OnceCell;
use std::sync::Arc;

/**
 * Startup optimizer for performance tracking
 * 
 * スタートアップ最適化器です。
 * 起動時間の測定と
 * 最適化を行います。
 */
#[derive(Debug)]
pub struct StartupOptimizer {
	/// Startup time
	pub startup_time: Instant,
	/// Initialized modules
	pub initialized_modules: Vec<String>,
	/// Initialization times
	pub init_times: Vec<(String, std::time::Duration)>,
}

/**
 * Startup statistics
 * 
 * スタートアップ統計です。
 * 起動統計情報を
 * 管理します。
 */
#[derive(Debug)]
pub struct StartupStats {
	/// Total startup time
	pub total_time: std::time::Duration,
	/// Module initialization times
	pub module_times: Vec<(String, std::time::Duration)>,
	/// Number of modules initialized
	pub module_count: usize,
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
			initialized_modules: Vec::new(),
			init_times: Vec::new(),
		}
	}
	
	/**
	 * Lazy initializes a module
	 * 
	 * @param module_name - Module name
	 * @param init_fn - Initialization function
	 * @return Result<()> - Success or error status
	 */
	pub async fn lazy_init<F>(&mut self, module_name: &str, init_fn: F) -> Result<()>
	where
		F: FnOnce() -> Result<()>,
	{
		let start = Instant::now();
		init_fn()?;
		let duration = start.elapsed();
		
		self.initialized_modules.push(module_name.to_string());
		self.init_times.push((module_name.to_string(), duration));
		
		println!("⚡ Initialized {} in {:?}", module_name, duration);
		
		Ok(())
	}
	
	/**
	 * Gets startup statistics
	 * 
	 * @return StartupStats - Startup statistics
	 */
	pub fn get_stats(&self) -> StartupStats {
		let total_time = self.startup_time.elapsed();
		
		StartupStats {
			total_time,
			module_times: self.init_times.clone(),
			module_count: self.initialized_modules.len(),
		}
	}
}

// Global startup optimizer
static STARTUP_OPTIMIZER: OnceCell<StartupOptimizer> = OnceCell::const_new();

/**
 * Main entry point for Sare terminal emulator
 * 
 * @return Result<()> - Success or error status
 */
#[tokio::main]
async fn main() -> Result<()> {
	println!("🚀 Starting Sare Terminal Emulator...");
	println!("💕 Built with love and passion by Yuriko and KleaSCM");
	
	// Initialize startup optimizer
	let optimizer = STARTUP_OPTIMIZER.get_or_init(StartupOptimizer::new);
	
	// Initialize core modules
	optimizer.lazy_init("terminal", || {
		println!("🔧 Initializing terminal core...");
		Ok(())
	}).await?;
	
	optimizer.lazy_init("tui", || {
		println!("🎨 Initializing TUI components...");
		Ok(())
	}).await?;
	
	optimizer.lazy_init("gui", || {
		println!("🖼️ Initializing GUI components...");
		Ok(())
	}).await?;
	
	// Create terminal configuration
	let config = sare_terminal::config::Config::default();
	
	// Create and initialize terminal emulator
	let mut terminal = sare_terminal::SareTerminal::new(config).await?;
	terminal.initialize().await?;
	
	// Set up signal handling for graceful shutdown
	let terminal_ref = Arc::new(tokio::sync::RwLock::new(terminal));
	let terminal_clone = Arc::clone(&terminal_ref);
	
	// Handle Ctrl+C for graceful shutdown
	tokio::spawn(async move {
		if let Ok(()) = tokio::signal::ctrl_c().await {
			println!("\n🛑 Received shutdown signal...");
			if let Ok(mut terminal) = terminal_clone.write().await {
				if let Err(e) = terminal.stop().await {
					eprintln!("Error stopping terminal: {}", e);
				}
			}
		}
	});
	
	// Run the terminal emulator
	let run_result = {
		let mut terminal = terminal_ref.write().await;
		terminal.run().await
	};
	
	// Print startup statistics
	let stats = optimizer.get_stats();
	println!("📊 Startup Statistics:");
	println!("   Total time: {:?}", stats.total_time);
	println!("   Modules initialized: {}", stats.module_count);
	for (module, duration) in stats.module_times {
		println!("   {}: {:?}", module, duration);
	}
	
	// Handle run result
	match run_result {
		Ok(()) => {
			println!("✅ Sare Terminal Emulator completed successfully!");
		}
		Err(e) => {
			eprintln!("❌ Sare Terminal Emulator failed: {}", e);
			std::process::exit(1);
		}
	}
	
	Ok(())
} 