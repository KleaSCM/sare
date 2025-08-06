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
use eframe;
use egui;
use std::process::Command;

// Import the REAL GuiTerminal directly to bypass lib.rs issues
mod gui {
	pub mod terminal;
	pub mod pane;
	pub mod multiline;
	pub mod heredoc;
	pub mod substitution;
	pub mod expansion;
	pub mod renderer;
}

// History module will be handled by GuiTerminal

use gui::terminal::GuiTerminal;

fn main() -> Result<()> {
	println!("🚀 Starting Sare Terminal Emulator...");
	println!("💕 Built with love and passion by Yuriko and KleaSCM");
	
	// Use the REAL GuiTerminal with ALL features!
	let app = GuiTerminal::new()?;
	
	let native_options = eframe::NativeOptions {
		initial_window_size: Some(egui::vec2(1200.0, 800.0)),
		min_window_size: Some(egui::vec2(400.0, 300.0)),
		..Default::default()
	};
	
	println!("🖼️  Starting FULL Sare Terminal with ALL features...");
	let run_result = eframe::run_native(
		"Sare Terminal Emulator - Full Feature Set",
		native_options,
		Box::new(|_cc| Box::new(app)),
	);
	
	match run_result {
		Ok(_) => {
			println!("✅ Sare Terminal Emulator completed successfully!");
		}
		Err(e) => {
			eprintln!("❌ Sare Terminal Emulator failed: {}", e);
			std::process::exit(1);
		}
	}
	
	Ok(())
} 