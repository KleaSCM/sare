/**
 * Sare Terminal Emulator
 * 
 * A modern, developer-focused terminal emulator with GPU acceleration,
 * multi-pane support, and advanced features for software development.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: main.rs
 * Description: Main entry point for Sare terminal emulator
 */

use anyhow::Result;
use eframe::egui;

// Core modules
mod shell;
mod tui;
mod terminal;
mod config;
mod history;

// GUI module
mod gui;

use gui::SareTerminal;

impl eframe::App for SareTerminal {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Handle key input
		self.handle_key_input(ctx);
		
		// Render the terminal
		gui::renderer::TerminalRenderer::render_terminal(self, ctx);
		
		// Request continuous updates for smooth cursor blinking
		ctx.request_repaint();
	}
	
	fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
		// Dark background color
		[0.1, 0.1, 0.1, 1.0]
	}
}

fn main() -> anyhow::Result<()> {
	/**
	 * Sareメイン関数です
	 * 
	 * ターミナルエミュレーターアプリケーションを初期化し、
	 * GUIフレームワークを設定してアプリケーションを実行します。
	 * 
	 * eframeを使用してネイティブウィンドウを作成し、
	 * ターミナルインスタンスを初期化してアプリケーションを開始します
	 */
	
	// Create native options
	let options = eframe::NativeOptions::default();
	
	// Create terminal instance
	let terminal = SareTerminal::new()?;
	
	// Run the application
	eframe::run_native(
		"Sare Terminal",
		options,
		Box::new(|_cc| Box::new(terminal)),
	)
	.map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))
} 