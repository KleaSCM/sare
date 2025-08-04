/**
 * Terminal renderer for Sare GUI
 * 
 * This module contains the rendering logic for the terminal
 * interface including pane rendering and UI components.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: renderer.rs
 * Description: Terminal rendering implementation
 */

use eframe::egui;

use super::terminal::SareTerminal;

/**
 * Terminal renderer
 * 
 * Handles the rendering of the terminal interface
 * including panes, output, and input areas.
 */
pub struct TerminalRenderer;

impl TerminalRenderer {
	/**
	 * Renders the terminal interface
	 */
	pub fn render_terminal(terminal: &mut SareTerminal, ctx: &egui::Context) {
		egui::CentralPanel::default().show(ctx, |ui| {
			// Render each pane
			for (pane_index, pane) in terminal.panes.iter().enumerate() {
				// Use unique ID for each pane to prevent ID clashes
				ui.push_id(format!("pane_{}", pane_index), |ui| {
					// Render output buffer
					for line in &pane.output_buffer {
						ui.label(egui::RichText::new(&line.content)
							.color(line.color)
							.text_style(egui::TextStyle::Monospace));
					}
					
					// Render current input with cursor
					let input_text = &pane.current_input;
					ui.label(egui::RichText::new(input_text)
						.color(egui::Color32::from_rgb(255, 255, 255))
						.text_style(egui::TextStyle::Monospace));
					
					// Render blinking cursor
					let cursor_char = if (ctx.input(|i| i.time) * 2.0) as i32 % 2 == 0 {
						"█"
					} else {
						" "
					};
					
					ui.label(egui::RichText::new(cursor_char)
						.color(egui::Color32::from_rgb(255, 255, 255))
						.text_style(egui::TextStyle::Monospace));
				});
			}
			
			// Render prompt at the bottom
			ui.vertical(|ui| {
				let prompt_text = format!("sare@user:{} $ ", terminal.current_dir);
				ui.label(egui::RichText::new(prompt_text)
					.color(egui::Color32::from_rgb(0, 255, 0))
					.text_style(egui::TextStyle::Monospace));
			});
		});
	}
} 