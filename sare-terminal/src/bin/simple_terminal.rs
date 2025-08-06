/**
 * Simple Sare Terminal - Standalone Binary
 * 
 * This is a standalone binary for the Sare terminal emulator
 * that doesn't depend on the broken lib.rs modules.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: simple_terminal.rs
 * Description: Standalone GUI terminal binary
 */

use anyhow::Result;
use eframe;
use egui;
use std::process::Command;

/**
 * Simple GUI Terminal App
 * 
 * A basic working GUI terminal that actually compiles and runs.
 */
struct SimpleTerminalApp {
	/// Current input text
	input_text: String,
	/// Output history
	output_history: Vec<String>,
	/// Current working directory
	current_dir: String,
}

impl Default for SimpleTerminalApp {
	fn default() -> Self {
		Self {
			input_text: String::new(),
			output_history: vec![
				"🚀 Welcome to Sare Terminal Emulator!".to_string(),
				"💕 Built with love and passion by Yuriko and KleaSCM".to_string(),
				"".to_string(),
				"Type 'help' for available commands".to_string(),
				"".to_string(),
			],
			current_dir: std::env::current_dir()
				.unwrap_or_default()
				.to_string_lossy()
				.to_string(),
		}
	}
}

impl SimpleTerminalApp {
	/**
	 * Executes a command and returns output
	 */
	fn execute_command(&mut self, command: &str) -> String {
		match command.trim() {
			"help" => {
				"Available commands:\n\
				• help - Show this help\n\
				• pwd - Show current directory\n\
				• ls - List files\n\
				• clear - Clear screen\n\
				• exit - Exit terminal".to_string()
			}
			"pwd" => {
				self.current_dir.clone()
			}
			"ls" => {
				match std::fs::read_dir(&self.current_dir) {
					Ok(entries) => {
						let mut files = Vec::new();
						for entry in entries {
							if let Ok(entry) = entry {
								if let Ok(name) = entry.file_name().into_string() {
									files.push(name);
								}
							}
						}
						files.join("  ")
					}
					Err(e) => format!("Error: {}", e)
				}
			}
			"clear" => {
				self.output_history.clear();
				"Screen cleared".to_string()
			}
			"exit" => {
				std::process::exit(0);
			}
			"" => {
				"".to_string()
			}
			_ => {
				// Try to execute as a system command
				match Command::new("sh")
					.arg("-c")
					.arg(command)
					.current_dir(&self.current_dir)
					.output() {
					Ok(output) => {
						let stdout = String::from_utf8_lossy(&output.stdout);
						let stderr = String::from_utf8_lossy(&output.stderr);
						
						let mut result = String::new();
						if !stdout.is_empty() {
							result.push_str(&stdout);
						}
						if !stderr.is_empty() {
							if !result.is_empty() {
								result.push('\n');
							}
							result.push_str(&stderr);
						}
						
						if result.is_empty() {
							format!("Command executed successfully")
						} else {
							result
						}
					}
					Err(e) => {
						format!("Error executing command: {}", e)
					}
				}
			}
		}
	}
}

impl eframe::App for SimpleTerminalApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			// Title
			ui.heading("🖥️ Sare Terminal Emulator");
			ui.separator();
			
			// Output area
			ui.group(|ui| {
				ui.label("Output:");
				egui::ScrollArea::vertical()
					.max_height(400.0)
					.show(ui, |ui| {
						for line in &self.output_history {
							ui.label(line);
						}
					});
			});
			
			// Current directory
			ui.label(format!("📁 Current directory: {}", self.current_dir));
			
			// Input area
			ui.group(|ui| {
				ui.label("💻 Enter command:");
				let response = ui.text_edit_singleline(&mut self.input_text);
				
				if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
					if !self.input_text.trim().is_empty() {
						// Add command to history
						self.output_history.push(format!("$ {}", self.input_text));
						
						// Execute command
						let output = self.execute_command(&self.input_text);
						if !output.is_empty() {
							self.output_history.push(output);
						}
						
						// Clear input
						self.input_text.clear();
					}
				}
			});
			
			// Status bar
			ui.separator();
			ui.horizontal(|ui| {
				ui.label("💕 Built with love by Yuriko and KleaSCM");
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.label("Press Enter to execute commands");
				});
			});
		});
	}
}

/**
 * Main entry point for Sare terminal emulator
 * 
 * @return Result<()> - Success or error status
 */
fn main() -> Result<()> {
	println!("🚀 Starting Sare Terminal Emulator...");
	println!("💕 Built with love and passion by Yuriko and KleaSCM");
	
	// Create the GUI terminal app
	let app = SimpleTerminalApp::default();
	
	// Run the GUI terminal with egui
	let native_options = eframe::NativeOptions {
		viewport: eframe::ViewportBuilder::default()
			.with_inner_size([1200.0, 800.0])
			.with_min_inner_size([400.0, 300.0]),
		..Default::default()
	};
	
	println!("🖼️  Starting GUI terminal window...");
	let run_result = eframe::run_native(
		"Sare Terminal Emulator",
		native_options,
		Box::new(|_cc| Box::new(app)),
	);
	
	// Handle run result
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