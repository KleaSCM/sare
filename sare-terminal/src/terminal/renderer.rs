/**
 * Terminal renderer for ANSI command processing
 * 
 * This module provides terminal rendering capabilities that process ANSI commands
 * and render terminal content with proper color support, cursor positioning,
 * screen buffer management, and all essential terminal emulation features.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: renderer.rs
 * Description: Terminal renderer for ANSI command processing and display
 */

use anyhow::Result;
use crate::terminal::protocol::{
	AnsiParser, AnsiCommand, TerminalState, ScreenBuffers, ActiveBuffer,
	Color, ColorType, TextAttributes, CursorShape, MouseReportingMode
};

/**
 * Terminal renderer
 * 
 * Processes ANSI commands and renders terminal content with
 * proper color support, cursor positioning, and screen buffer management.
 */
#[derive(Debug)]
pub struct TerminalRenderer {
	/// ANSI parser for processing escape sequences
	parser: AnsiParser,
	/// Current terminal state
	state: TerminalState,
	/// Screen buffers (primary and alternate)
	screen_buffers: ScreenBuffers,
	/// Saved cursor position
	saved_cursor: Option<(u16, u16)>,
	/// Terminal size (columns, rows)
	size: (u16, u16),
	/// Scrollback buffer
	scrollback: Vec<Vec<Cell>>,
	/// Maximum scrollback lines
	max_scrollback: usize,
	/// Current scroll position
	scroll_pos: usize,
	/// Dirty regions for efficient redraw
	dirty_regions: Vec<(u16, u16, u16, u16)>,
}

/**
 * Cell content for rendering
 */
#[derive(Debug, Clone)]
pub struct Cell {
	/// Character content
	pub char: char,
	/// Foreground color
	pub fg_color: Color,
	/// Background color
	pub bg_color: Color,
	/// Text attributes
	pub attributes: TextAttributes,
	/// Whether cell is dirty (needs redraw)
	pub dirty: bool,
}

/**
 * Terminal renderer configuration
 */
#[derive(Debug, Clone)]
pub struct RendererConfig {
	/// Terminal size (columns, rows)
	pub size: (u16, u16),
	/// Maximum scrollback lines
	pub max_scrollback: usize,
	/// Enable color support
	pub color_support: bool,
	/// Enable mouse support
	pub mouse_support: bool,
	/// Default foreground color
	pub default_fg_color: Color,
	/// Default background color
	pub default_bg_color: Color,
}

impl Default for RendererConfig {
	fn default() -> Self {
		Self {
			size: (80, 24),
			max_scrollback: 1000,
			color_support: true,
			mouse_support: true,
			default_fg_color: Color::default(),
			default_bg_color: Color { r: 0, g: 0, b: 0, color_type: ColorType::Default },
		}
	}
}

impl Default for Cell {
	fn default() -> Self {
		Self {
			char: ' ',
			fg_color: Color::default(),
			bg_color: Color { r: 0, g: 0, b: 0, color_type: ColorType::Default },
			attributes: TextAttributes::default(),
			dirty: false,
		}
	}
}

impl TerminalRenderer {
	/**
	 * Creates a new terminal renderer
	 * 
	 * @param config - Renderer configuration
	 * @return TerminalRenderer - New terminal renderer instance
	 */
	pub fn new(config: RendererConfig) -> Self {
		let mut renderer = Self {
			parser: AnsiParser::new(),
			state: TerminalState::default(),
			screen_buffers: ScreenBuffers::default(),
			saved_cursor: None,
			size: config.size,
			scrollback: Vec::new(),
			max_scrollback: config.max_scrollback,
			scroll_pos: 0,
			dirty_regions: Vec::new(),
		};
		
		// Initialize screen buffers
		renderer.screen_buffers.primary.resize(config.size.0, config.size.1);
		renderer.screen_buffers.alternate.resize(config.size.0, config.size.1);
		
		// Set default colors
		renderer.state.fg_color = config.default_fg_color;
		renderer.state.bg_color = config.default_bg_color;
		
		renderer
	}
	
	/**
	 * Processes input bytes and renders terminal content
	 * 
	 * @param input - Input bytes to process
	 * @return Result<()> - Success or error
	 */
	pub fn process_input(&mut self, input: &[u8]) -> Result<()> {
		let commands = self.parser.process_input(input)?;
		
		for command in commands {
			self.handle_command(command)?;
		}
		
		Ok(())
	}
	
	/**
	 * Handles individual ANSI commands
	 * 
	 * @param command - ANSI command to handle
	 * @return Result<()> - Success or error
	 */
	fn handle_command(&mut self, command: AnsiCommand) -> Result<()> {
		match command {
			AnsiCommand::Print(ch) => {
				self.print_character(ch);
			}
			AnsiCommand::CursorUp(count) => {
				self.move_cursor_up(count);
			}
			AnsiCommand::CursorDown(count) => {
				self.move_cursor_down(count);
			}
			AnsiCommand::CursorForward(count) => {
				self.move_cursor_forward(count);
			}
			AnsiCommand::CursorBackward(count) => {
				self.move_cursor_backward(count);
			}
			AnsiCommand::CursorNextLine(count) => {
				self.move_cursor_next_line(count);
			}
			AnsiCommand::CursorPreviousLine(count) => {
				self.move_cursor_previous_line(count);
			}
			AnsiCommand::CursorHorizontalAbsolute(col) => {
				self.move_cursor_horizontal_absolute(col);
			}
			AnsiCommand::CursorPosition(row, col) => {
				self.move_cursor_position(row, col);
			}
			AnsiCommand::SaveCursor => {
				self.save_cursor();
			}
			AnsiCommand::RestoreCursor => {
				self.restore_cursor();
			}
			AnsiCommand::EraseInDisplay(mode) => {
				self.erase_in_display(mode);
			}
			AnsiCommand::EraseInLine(mode) => {
				self.erase_in_line(mode);
			}
			AnsiCommand::ResetAttributes => {
				self.reset_attributes();
			}
			AnsiCommand::SetBold => {
				self.set_bold();
			}
			AnsiCommand::SetDim => {
				self.set_dim();
			}
			AnsiCommand::SetItalic => {
				self.set_italic();
			}
			AnsiCommand::SetUnderline => {
				self.set_underline();
			}
			AnsiCommand::SetBlink => {
				self.set_blink();
			}
			AnsiCommand::SetReverse => {
				self.set_reverse();
			}
			AnsiCommand::SetHidden => {
				self.set_hidden();
			}
			AnsiCommand::SetStrikethrough => {
				self.set_strikethrough();
			}
			AnsiCommand::SetForegroundColor(color_type) => {
				self.set_foreground_color(color_type);
			}
			AnsiCommand::SetBackgroundColor(color_type) => {
				self.set_background_color(color_type);
			}
			AnsiCommand::ResetForegroundColor => {
				self.reset_foreground_color();
			}
			AnsiCommand::ResetBackgroundColor => {
				self.reset_background_color();
			}
			AnsiCommand::SetApplicationCursorKeys => {
				self.set_application_cursor_keys();
			}
			AnsiCommand::ResetApplicationCursorKeys => {
				self.reset_application_cursor_keys();
			}
			AnsiCommand::SetInsertMode => {
				self.set_insert_mode();
			}
			AnsiCommand::ResetInsertMode => {
				self.reset_insert_mode();
			}
			AnsiCommand::SetReverseVideo => {
				self.set_reverse_video();
			}
			AnsiCommand::ResetReverseVideo => {
				self.reset_reverse_video();
			}
			AnsiCommand::SetOriginMode => {
				self.set_origin_mode();
			}
			AnsiCommand::ResetOriginMode => {
				self.reset_origin_mode();
			}
			AnsiCommand::SetAutoWrap => {
				self.set_auto_wrap();
			}
			AnsiCommand::ResetAutoWrap => {
				self.reset_auto_wrap();
			}
			AnsiCommand::SetBlinkingCursor => {
				self.set_blinking_cursor();
			}
			AnsiCommand::ResetBlinkingCursor => {
				self.reset_blinking_cursor();
			}
			AnsiCommand::ShowCursor => {
				self.show_cursor();
			}
			AnsiCommand::HideCursor => {
				self.hide_cursor();
			}
			AnsiCommand::SetMouseTracking(mode) => {
				self.set_mouse_tracking(mode);
			}
			AnsiCommand::SetBracketedPaste(enabled) => {
				self.set_bracketed_paste(enabled);
			}
			AnsiCommand::DeviceAttributes => {
				self.device_attributes();
			}
			AnsiCommand::DeviceStatusReport(mode) => {
				self.device_status_report(mode);
			}
			AnsiCommand::SetScrollRegion(top, bottom) => {
				self.set_scroll_region(top, bottom);
			}
			AnsiCommand::MouseTracking(enabled) => {
				self.mouse_tracking(enabled);
			}
			_ => {
				// Unknown command
			}
		}
		
		Ok(())
	}
	
	/**
	 * Prints a character at the current cursor position
	 * 
	 * @param ch - Character to print
	 */
	fn print_character(&mut self, ch: char) {
		let buffer = self.get_active_buffer();
		let (col, row) = self.state.cursor_pos;
		
		if row < buffer.content.len() as u16 && col < buffer.content[row as usize].len() as u16 {
			let cell = &mut buffer.content[row as usize][col as usize];
			cell.char = ch;
			cell.fg_color = self.state.fg_color.clone();
			cell.bg_color = self.state.bg_color.clone();
			cell.attributes = self.state.attributes.clone();
			cell.dirty = true;
			
			// Mark region as dirty
			self.mark_dirty_region(col, row, col + 1, row + 1);
			
			// Move cursor forward
			if self.state.auto_wrap && col + 1 >= self.size.0 {
				// Auto-wrap to next line
				if row + 1 < self.size.1 {
					self.state.cursor_pos = (0, row + 1);
				} else {
					// Scroll up
					self.scroll_up();
					self.state.cursor_pos = (0, self.size.1 - 1);
				}
			} else {
				self.state.cursor_pos.0 += 1;
			}
		}
	}
	
	/**
	 * Moves cursor up
	 * 
	 * @param count - Number of lines to move up
	 */
	fn move_cursor_up(&mut self, count: u32) {
		let count = count as u16;
		if self.state.cursor_pos.1 >= count {
			self.state.cursor_pos.1 -= count;
		} else {
			self.state.cursor_pos.1 = 0;
		}
	}
	
	/**
	 * Moves cursor down
	 * 
	 * @param count - Number of lines to move down
	 */
	fn move_cursor_down(&mut self, count: u32) {
		let count = count as u16;
		self.state.cursor_pos.1 = (self.state.cursor_pos.1 + count).min(self.size.1 - 1);
	}
	
	/**
	 * Moves cursor forward
	 * 
	 * @param count - Number of columns to move forward
	 */
	fn move_cursor_forward(&mut self, count: u32) {
		let count = count as u16;
		self.state.cursor_pos.0 = (self.state.cursor_pos.0 + count).min(self.size.0 - 1);
	}
	
	/**
	 * Moves cursor backward
	 * 
	 * @param count - Number of columns to move backward
	 */
	fn move_cursor_backward(&mut self, count: u32) {
		let count = count as u16;
		if self.state.cursor_pos.0 >= count {
			self.state.cursor_pos.0 -= count;
		} else {
			self.state.cursor_pos.0 = 0;
		}
	}
	
	/**
	 * Moves cursor to next line
	 * 
	 * @param count - Number of lines to move down
	 */
	fn move_cursor_next_line(&mut self, count: u32) {
		self.move_cursor_down(count);
		self.state.cursor_pos.0 = 0;
	}
	
	/**
	 * Moves cursor to previous line
	 * 
	 * @param count - Number of lines to move up
	 */
	fn move_cursor_previous_line(&mut self, count: u32) {
		self.move_cursor_up(count);
		self.state.cursor_pos.0 = 0;
	}
	
	/**
	 * Moves cursor to absolute column
	 * 
	 * @param col - Column position (1-based)
	 */
	fn move_cursor_horizontal_absolute(&mut self, col: u32) {
		let col = (col as u16).saturating_sub(1);
		self.state.cursor_pos.0 = col.min(self.size.0 - 1);
	}
	
	/**
	 * Moves cursor to absolute position
	 * 
	 * @param row - Row position (1-based)
	 * @param col - Column position (1-based)
	 */
	fn move_cursor_position(&mut self, row: u32, col: u32) {
		let row = (row as u16).saturating_sub(1);
		let col = (col as u16).saturating_sub(1);
		
		if self.state.origin_mode {
			// Relative to scroll region
			if let Some((top, bottom)) = self.state.scroll_region {
				self.state.cursor_pos.1 = (top + row).min(bottom);
			} else {
				self.state.cursor_pos.1 = row.min(self.size.1 - 1);
			}
		} else {
			// Absolute positioning
			self.state.cursor_pos.1 = row.min(self.size.1 - 1);
		}
		
		self.state.cursor_pos.0 = col.min(self.size.0 - 1);
	}
	
	/**
	 * Saves cursor position
	 */
	fn save_cursor(&mut self) {
		self.saved_cursor = Some(self.state.cursor_pos);
	}
	
	/**
	 * Restores cursor position
	 */
	fn restore_cursor(&mut self) {
		if let Some(pos) = self.saved_cursor {
			self.state.cursor_pos = pos;
		}
	}
	
	/**
	 * Erases display content
	 * 
	 * @param mode - Erase mode (0=from cursor to end, 1=from beginning to cursor, 2=all)
	 */
	fn erase_in_display(&mut self, mode: u32) {
		let buffer = self.get_active_buffer();
		let (col, row) = self.state.cursor_pos;
		
		match mode {
			0 => {
				// From cursor to end of display
				for r in row..self.size.1 {
					let start_col = if r == row { col } else { 0 };
					for c in start_col..self.size.0 {
						if r < buffer.content.len() as u16 && c < buffer.content[r as usize].len() as u16 {
							let cell = &mut buffer.content[r as usize][c as usize];
							*cell = Cell::default();
							cell.dirty = true;
						}
					}
				}
			}
			1 => {
				// From beginning to cursor
				for r in 0..=row {
					let end_col = if r == row { col + 1 } else { self.size.0 };
					for c in 0..end_col {
						if r < buffer.content.len() as u16 && c < buffer.content[r as usize].len() as u16 {
							let cell = &mut buffer.content[r as usize][c as usize];
							*cell = Cell::default();
							cell.dirty = true;
						}
					}
				}
			}
			2 => {
				// Entire display
				for r in 0..self.size.1 {
					for c in 0..self.size.0 {
						if r < buffer.content.len() as u16 && c < buffer.content[r as usize].len() as u16 {
							let cell = &mut buffer.content[r as usize][c as usize];
							*cell = Cell::default();
							cell.dirty = true;
						}
					}
				}
			}
			_ => {}
		}
		
		// Mark entire screen as dirty
		self.mark_dirty_region(0, 0, self.size.0, self.size.1);
	}
	
	/**
	 * Erases line content
	 * 
	 * @param mode - Erase mode (0=from cursor to end, 1=from beginning to cursor, 2=entire line)
	 */
	fn erase_in_line(&mut self, mode: u32) {
		let buffer = self.get_active_buffer();
		let (col, row) = self.state.cursor_pos;
		
		if row < buffer.content.len() as u16 {
			match mode {
				0 => {
					// From cursor to end of line
					for c in col..self.size.0 {
						if c < buffer.content[row as usize].len() as u16 {
							let cell = &mut buffer.content[row as usize][c as usize];
							*cell = Cell::default();
							cell.dirty = true;
						}
					}
				}
				1 => {
					// From beginning to cursor
					for c in 0..=col {
						if c < buffer.content[row as usize].len() as u16 {
							let cell = &mut buffer.content[row as usize][c as usize];
							*cell = Cell::default();
							cell.dirty = true;
						}
					}
				}
				2 => {
					// Entire line
					for c in 0..self.size.0 {
						if c < buffer.content[row as usize].len() as u16 {
							let cell = &mut buffer.content[row as usize][c as usize];
							*cell = Cell::default();
							cell.dirty = true;
						}
					}
				}
				_ => {}
			}
			
			// Mark line as dirty
			self.mark_dirty_region(0, row, self.size.0, row + 1);
		}
	}
	
	/**
	 * Resets all text attributes
	 */
	fn reset_attributes(&mut self) {
		self.state.attributes = TextAttributes::default();
		self.state.fg_color = Color::default();
		self.state.bg_color = Color { r: 0, g: 0, b: 0, color_type: ColorType::Default };
	}
	
	/**
	 * Sets bold attribute
	 */
	fn set_bold(&mut self) {
		self.state.attributes.bold = true;
	}
	
	/**
	 * Sets dim attribute
	 */
	fn set_dim(&mut self) {
		self.state.attributes.dim = true;
	}
	
	/**
	 * Sets italic attribute
	 */
	fn set_italic(&mut self) {
		self.state.attributes.italic = true;
	}
	
	/**
	 * Sets underline attribute
	 */
	fn set_underline(&mut self) {
		self.state.attributes.underline = true;
	}
	
	/**
	 * Sets blink attribute
	 */
	fn set_blink(&mut self) {
		self.state.attributes.blink = true;
	}
	
	/**
	 * Sets reverse video attribute
	 */
	fn set_reverse(&mut self) {
		self.state.attributes.reverse = true;
	}
	
	/**
	 * Sets hidden attribute
	 */
	fn set_hidden(&mut self) {
		self.state.attributes.hidden = true;
	}
	
	/**
	 * Sets strikethrough attribute
	 */
	fn set_strikethrough(&mut self) {
		self.state.attributes.strikethrough = true;
	}
	
	/**
	 * Sets foreground color
	 * 
	 * @param color_type - Color type to set
	 */
	fn set_foreground_color(&mut self, color_type: ColorType) {
		match color_type {
			ColorType::Named(index) => {
				if let Some(color) = self.parser.color_palette.get(&index) {
					self.state.fg_color = color.clone();
				}
			}
			ColorType::Index(index) => {
				if let Some(color) = self.parser.color_palette.get(&index) {
					self.state.fg_color = color.clone();
				}
			}
			ColorType::TrueColor => {
				// True color support would be implemented here
			}
			ColorType::Default => {
				self.state.fg_color = Color::default();
			}
		}
	}
	
	/**
	 * Sets background color
	 * 
	 * @param color_type - Color type to set
	 */
	fn set_background_color(&mut self, color_type: ColorType) {
		match color_type {
			ColorType::Named(index) => {
				if let Some(color) = self.parser.color_palette.get(&index) {
					self.state.bg_color = color.clone();
				}
			}
			ColorType::Index(index) => {
				if let Some(color) = self.parser.color_palette.get(&index) {
					self.state.bg_color = color.clone();
				}
			}
			ColorType::TrueColor => {
				// True color support would be implemented here
			}
			ColorType::Default => {
				self.state.bg_color = Color { r: 0, g: 0, b: 0, color_type: ColorType::Default };
			}
		}
	}
	
	/**
	 * Resets foreground color
	 */
	fn reset_foreground_color(&mut self) {
		self.state.fg_color = Color::default();
	}
	
	/**
	 * Resets background color
	 */
	fn reset_background_color(&mut self) {
		self.state.bg_color = Color { r: 0, g: 0, b: 0, color_type: ColorType::Default };
	}
	
	/**
	 * Sets application cursor keys mode
	 */
	fn set_application_cursor_keys(&mut self) {
		self.state.app_cursor_keys = true;
	}
	
	/**
	 * Resets application cursor keys mode
	 */
	fn reset_application_cursor_keys(&mut self) {
		self.state.app_cursor_keys = false;
	}
	
	/**
	 * Sets insert mode
	 */
	fn set_insert_mode(&mut self) {
		self.state.insert_mode = true;
	}
	
	/**
	 * Resets insert mode
	 */
	fn reset_insert_mode(&mut self) {
		self.state.insert_mode = false;
	}
	
	/**
	 * Sets reverse video mode
	 */
	fn set_reverse_video(&mut self) {
		self.state.attributes.reverse = true;
	}
	
	/**
	 * Resets reverse video mode
	 */
	fn reset_reverse_video(&mut self) {
		self.state.attributes.reverse = false;
	}
	
	/**
	 * Sets origin mode
	 */
	fn set_origin_mode(&mut self) {
		self.state.origin_mode = true;
	}
	
	/**
	 * Resets origin mode
	 */
	fn reset_origin_mode(&mut self) {
		self.state.origin_mode = false;
	}
	
	/**
	 * Sets auto wrap mode
	 */
	fn set_auto_wrap(&mut self) {
		self.state.auto_wrap = true;
	}
	
	/**
	 * Resets auto wrap mode
	 */
	fn reset_auto_wrap(&mut self) {
		self.state.auto_wrap = false;
	}
	
	/**
	 * Sets blinking cursor
	 */
	fn set_blinking_cursor(&mut self) {
		// Cursor blinking would be implemented here
	}
	
	/**
	 * Resets blinking cursor
	 */
	fn reset_blinking_cursor(&mut self) {
		// Cursor blinking would be implemented here
	}
	
	/**
	 * Shows cursor
	 */
	fn show_cursor(&mut self) {
		self.state.cursor_visible = true;
	}
	
	/**
	 * Hides cursor
	 */
	fn hide_cursor(&mut self) {
		self.state.cursor_visible = false;
	}
	
	/**
	 * Sets mouse tracking mode
	 * 
	 * @param mode - Mouse reporting mode
	 */
	fn set_mouse_tracking(&mut self, mode: MouseReportingMode) {
		self.parser.mouse_state.reporting_mode = mode;
		self.parser.mouse_state.tracking_enabled = mode != MouseReportingMode::None;
	}
	
	/**
	 * Sets bracketed paste mode
	 * 
	 * @param enabled - Whether to enable bracketed paste
	 */
	fn set_bracketed_paste(&mut self, enabled: bool) {
		self.parser.bracketed_paste = enabled;
	}
	
	/**
	 * Handles device attributes request
	 */
	fn device_attributes(&mut self) {
		// Device attributes response would be implemented here
	}
	
	/**
	 * Handles device status report
	 * 
	 * @param mode - Status report mode
	 */
	fn device_status_report(&mut self, mode: u32) {
		// Device status report response would be implemented here
	}
	
	/**
	 * Sets scroll region
	 * 
	 * @param top - Top margin (1-based)
	 * @param bottom - Bottom margin (1-based)
	 */
	fn set_scroll_region(&mut self, top: u32, bottom: u32) {
		let top = (top as u16).saturating_sub(1);
		let bottom = (bottom as u16).saturating_sub(1);
		
		if top < bottom && bottom < self.size.1 {
			self.state.scroll_region = Some((top, bottom));
		}
	}
	
	/**
	 * Handles mouse tracking
	 * 
	 * @param enabled - Whether mouse tracking is enabled
	 */
	fn mouse_tracking(&mut self, enabled: bool) {
		self.parser.mouse_state.tracking_enabled = enabled;
	}
	
	/**
	 * Gets the currently active screen buffer
	 * 
	 * @return &mut ScreenBuffer - Active screen buffer
	 */
	fn get_active_buffer(&mut self) -> &mut crate::terminal::protocol::ScreenBuffer {
		match self.screen_buffers.active {
			ActiveBuffer::Primary => &mut self.screen_buffers.primary,
			ActiveBuffer::Alternate => &mut self.screen_buffers.alternate,
		}
	}
	
	/**
	 * Scrolls the screen up
	 */
	fn scroll_up(&mut self) {
		let buffer = self.get_active_buffer();
		
		// Save top line to scrollback
		if !buffer.content.is_empty() {
			let top_line = buffer.content[0].clone();
			self.scrollback.push(top_line);
			
			// Limit scrollback size
			if self.scrollback.len() > self.max_scrollback {
				self.scrollback.remove(0);
			}
		}
		
		// Move all lines up
		for i in 0..buffer.content.len() - 1 {
			buffer.content[i] = buffer.content[i + 1].clone();
		}
		
		// Clear bottom line
		if !buffer.content.is_empty() {
			let bottom_line = &mut buffer.content[buffer.content.len() - 1];
			for cell in bottom_line.iter_mut() {
				*cell = Cell::default();
				cell.dirty = true;
			}
		}
	}
	
	/**
	 * Marks a region as dirty for redraw
	 * 
	 * @param x1 - Left column
	 * @param y1 - Top row
	 * @param x2 - Right column
	 * @param y2 - Bottom row
	 */
	fn mark_dirty_region(&mut self, x1: u16, y1: u16, x2: u16, y2: u16) {
		self.dirty_regions.push((x1, y1, x2, y2));
	}
	
	/**
	 * Gets the current terminal state
	 * 
	 * @return &TerminalState - Current terminal state
	 */
	pub fn state(&self) -> &TerminalState {
		&self.state
	}
	
	/**
	 * Gets the current screen content
	 * 
	 * @return &Vec<Vec<Cell>> - Screen content
	 */
	pub fn screen_content(&self) -> &Vec<Vec<Cell>> {
		match self.screen_buffers.active {
			ActiveBuffer::Primary => &self.screen_buffers.primary.content,
			ActiveBuffer::Alternate => &self.screen_buffers.alternate.content,
		}
	}
	
	/**
	 * Gets the scrollback content
	 * 
	 * @return &Vec<Vec<Cell>> - Scrollback content
	 */
	pub fn scrollback_content(&self) -> &Vec<Vec<Cell>> {
		&self.scrollback
	}
	
	/**
	 * Gets dirty regions for efficient redraw
	 * 
	 * @return &Vec<(u16, u16, u16, u16)> - Dirty regions
	 */
	pub fn dirty_regions(&self) -> &Vec<(u16, u16, u16, u16)> {
		&self.dirty_regions
	}
	
	/**
	 * Clears dirty regions
	 */
	pub fn clear_dirty_regions(&mut self) {
		self.dirty_regions.clear();
	}
	
	/**
	 * Resizes the terminal
	 * 
	 * @param cols - New number of columns
	 * @param rows - New number of rows
	 */
	pub fn resize(&mut self, cols: u16, rows: u16) {
		self.size = (cols, rows);
		self.state.size = (cols, rows);
		
		// Resize screen buffers
		self.screen_buffers.primary.resize(cols, rows);
		self.screen_buffers.alternate.resize(cols, rows);
		
		// Adjust cursor position
		if self.state.cursor_pos.0 >= cols {
			self.state.cursor_pos.0 = cols - 1;
		}
		if self.state.cursor_pos.1 >= rows {
			self.state.cursor_pos.1 = rows - 1;
		}
		
		// Mark entire screen as dirty
		self.mark_dirty_region(0, 0, cols, rows);
	}
} 