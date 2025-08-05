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

mod terminal;
mod tui;
mod gui;

use gui::SareTerminal;

/**
 * Main entry point for the Sare terminal emulator
 * 
 * Initializes the terminal emulator with GPU acceleration
 * and starts the GUI application with proper error handling.
 */
fn main() -> Result<()> {
	// Set up native options for the GUI
	let options = NativeOptions::default();
	
	// Run the terminal emulator
	eframe::run_native(
		"Sare Terminal",
		options,
		Box::new(|_cc| Box::new(SareTerminal::new().unwrap())),
	)
	.map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;
	
	Ok(())
} 