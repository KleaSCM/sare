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
			// Fill entire background with dark color
			let background_color = egui::Color32::from_rgb(30, 30, 30);
			ui.painter().rect_filled(
				ui.available_rect_before_wrap(),
				0.0,
				background_color,
			);
			
			// Render each pane
			for (pane_index, pane) in terminal.panes.iter().enumerate() {
				let (x, y, width, height) = pane.layout;
				
				// Calculate absolute pixel coordinates
				let available_size = ui.available_size();
				let pane_x = x * available_size.x;
				let pane_y = y * available_size.y;
				let pane_width = width * available_size.x;
				let pane_height = height * available_size.y;
				
				// Create pane rectangle
				let pane_rect = egui::Rect::from_min_size(
					egui::pos2(pane_x, pane_y),
					egui::vec2(pane_width, pane_height),
				);
				
				// Only draw borders when there are multiple panes
				if terminal.panes.len() > 1 {
					// Draw thin border around each pane
					let border_color = if pane.active {
						egui::Color32::from_rgb(100, 100, 100) // Brighter border for active pane
					} else {
						egui::Color32::from_rgb(60, 60, 60) // Subtle border for inactive panes
					};
					
					ui.painter().rect_stroke(
						pane_rect,
						0.0,
						(1.0, border_color), // 1 pixel thin border
					);
				}
				
				// Set clip rect to ensure content stays within pane bounds
				ui.set_clip_rect(pane_rect);
				
				// Create child UI for this pane
				ui.allocate_ui_at_rect(pane_rect, |ui| {
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
						
						// Render blinking cursor (only for active pane)
						if pane.active {
							let cursor_char = if (ctx.input(|i| i.time) * 2.0) as i32 % 2 == 0 {
								"█"
							} else {
								" "
							};
							
							ui.label(egui::RichText::new(cursor_char)
								.color(egui::Color32::from_rgb(255, 255, 255))
								.text_style(egui::TextStyle::Monospace));
						}
						
						// Render prompt for this pane
						let prompt_text = if terminal.multiline_state.is_multiline() && pane_index == terminal.focused_pane {
							format!("sare@user:{} {} ", pane.working_directory, terminal.multiline_state.multiline_prompt)
						} else {
							format!("sare@user:{} $ ", pane.working_directory)
						};
						ui.label(egui::RichText::new(prompt_text)
							.color(egui::Color32::from_rgb(0, 255, 0))
							.text_style(egui::TextStyle::Monospace));
					});
				});
			}
		});
	}
} 