
/**
 * Terminal renderer module for Sare terminal
 * 
 * This module provides terminal rendering functionality including
 * pane layout, output display, and cursor management.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: renderer.rs
 * Description: Terminal rendering and display management
 */

use eframe::egui;

use super::terminal::SareTerminal;

pub struct TerminalRenderer;

impl TerminalRenderer {
	pub fn render_terminal(terminal: &mut SareTerminal, ctx: &egui::Context) {
		/**
		 * ターミナルをレンダリングする関数です
		 * 
		 * マルチペインターミナルインターフェースを描画し、
		 * 各ペインの出力バッファとカーソルを表示します。
		 * 
		 * ペイン管理、カーソルアニメーション、プロンプト表示を
		 * 含む完全なターミナルレンダリングを実行します
		 */
		
		egui::CentralPanel::default().show(ctx, |ui| {
			let background_color = egui::Color32::from_rgb(30, 30, 30);
			ui.painter().rect_filled(
				ui.available_rect_before_wrap(),
				0.0,
				background_color,
			);
			
			for (pane_index, pane) in terminal.panes.iter().enumerate() {
				let (x, y, width, height) = pane.layout;
				
				let available_size = ui.available_size();
				let pane_x = x * available_size.x;
				let pane_y = y * available_size.y;
				let pane_width = width * available_size.x;
				let pane_height = height * available_size.y;
				
				let pane_rect = egui::Rect::from_min_size(
					egui::pos2(pane_x, pane_y),
					egui::vec2(pane_width, pane_height),
				);
				
				if terminal.panes.len() > 1 {
					let border_color = if pane.active {
						egui::Color32::from_rgb(100, 100, 100)
					} else {
						egui::Color32::from_rgb(60, 60, 60)
					};
					
					ui.painter().rect_stroke(
						pane_rect,
						0.0,
						(1.0, border_color),
					);
				}
				
				ui.set_clip_rect(pane_rect);
				
				ui.allocate_ui_at_rect(pane_rect, |ui| {
					ui.push_id(format!("pane_{}", pane_index), |ui| {
						for line in &pane.output_buffer {
							ui.label(egui::RichText::new(&line.content)
								.color(line.color)
								.text_style(egui::TextStyle::Monospace));
						}
						
						let input_text = &pane.current_input;
						ui.label(egui::RichText::new(input_text)
							.color(egui::Color32::from_rgb(255, 255, 255))
							.text_style(egui::TextStyle::Monospace));
						
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