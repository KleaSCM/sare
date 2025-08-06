/**
 * ANSI escape sequence parser and terminal protocol implementation
 * 
 * This module provides comprehensive ANSI escape sequence parsing for VT100/VT220/VT320
 * terminal protocols, including color support, cursor control, screen buffer management,
 * mouse support, and all essential terminal emulation features.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: protocol.rs
 * Description: ANSI escape sequence parser and terminal protocol implementation
 */

use anyhow::Result;
use std::collections::HashMap;
use crate::terminal::renderer::Cell;

/**
 * ANSI escape sequence parser
 * 
 * Parses and handles ANSI escape sequences for terminal emulation,
 * including cursor control, color management, screen buffer operations,
 * and mouse support.
 */
#[derive(Debug, Clone)]
pub struct AnsiParser {
	/// Parser state
	pub state: ParserState,
	/// Mouse state
	pub mouse_state: MouseState,
	/// Bracketed paste mode
	pub bracketed_paste: bool,
	/// Current escape sequence
	pub escape_sequence: Vec<u8>,
	/// Current control sequence
	pub control_sequence: Vec<u8>,
	/// Current parameter values
	pub parameters: Vec<u32>,
	/// Current intermediate characters
	pub intermediates: Vec<u8>,
	/// Current final character
	pub final_char: Option<u8>,
	/// Terminal state
	pub terminal_state: TerminalState,
	/// Color palette (256 colors)
	pub color_palette: HashMap<u8, Color>,
	/// Screen buffers (primary and alternate)
	pub screen_buffers: ScreenBuffers,
	/// Current parameters (for backward compatibility)
	pub params: Vec<u32>,
	/// Current parameter being built
	pub current_param: String,
	/// Application cursor keys mode
	pub app_cursor_keys: bool,
}

/**
 * Parser state machine states
 */
#[derive(Debug, Clone, PartialEq)]
pub enum ParserState {
	/// Normal text mode
	Normal,
	/// Escape sequence started
	Escape,
	/// Control sequence started
	ControlSequence,
	/// Parameter collection
	Parameter,
	/// Intermediate character collection
	Intermediate,
	/// Final character collection
	Final,
	/// String terminator
	StringTerminator,
	/// Operating system command
	OperatingSystem,
	/// Device control string
	DeviceControl,
}

#[derive(Debug, Clone)]
pub struct TerminalState {
	/// Foreground color
	pub fg_color: Color,
	/// Background color
	pub bg_color: Color,
	/// Text attributes
	pub attributes: TextAttributes,
	/// Cursor position (column, row)
	pub cursor_pos: (u16, u16),
	/// Terminal size (columns, rows)
	pub size: (u16, u16),
	/// Insert mode
	pub insert_mode: bool,
	/// Origin mode
	pub origin_mode: bool,
	/// Auto wrap mode
	pub auto_wrap: bool,
	/// Cursor visible
	pub cursor_visible: bool,
	/// Scroll region (top, bottom)
	pub scroll_region: Option<(u16, u16)>,
	/// Application cursor keys mode
	pub app_cursor_keys: bool,
}

#[derive(Debug, Clone)]
pub struct ScreenBuffers {
	/// Primary screen buffer
	pub primary: ScreenBuffer,
	/// Alternate screen buffer
	pub alternate: ScreenBuffer,
	/// Current active buffer
	pub active: ScreenBufferType,
}

#[derive(Debug, Clone)]
pub enum ScreenBufferType {
	Primary,
	Alternate,
}

#[derive(Debug, Clone)]
pub enum ActiveBuffer {
	Primary,
	Alternate,
}

/**
 * Cursor shape variants
 */
#[derive(Debug, Clone, PartialEq)]
pub enum CursorShape {
	/// Block cursor
	Block,
	/// Underline cursor
	Underline,
	/// Vertical bar cursor
	Bar,
}

/**
 * Color representation
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
	/// Red component (0-255)
	pub r: u8,
	/// Green component (0-255)
	pub g: u8,
	/// Blue component (0-255)
	pub b: u8,
	/// Color type
	pub color_type: ColorType,
}

/**
 * Color type variants
 */
#[derive(Debug, Clone, PartialEq)]
pub enum ColorType {
	/// Named color (0-15)
	Named(u8),
	/// 256-color index (16-255)
	Index(u8),
	/// True color (24-bit)
	TrueColor,
	/// Default color
	Default,
}

/**
 * Text attributes
 */
#[derive(Debug, Clone)]
pub struct TextAttributes {
	/// Bold text
	pub bold: bool,
	/// Dim/faint text
	pub dim: bool,
	/// Italic text
	pub italic: bool,
	/// Underlined text
	pub underline: bool,
	/// Blinking text
	pub blink: bool,
	/// Reverse video
	pub reverse: bool,
	/// Hidden text
	pub hidden: bool,
	/// Strikethrough text
	pub strikethrough: bool,
}

#[derive(Debug, Clone)]
pub struct ScreenBuffer {
	/// Screen content
	pub content: Vec<Vec<Cell>>,
	/// Scrollback buffer
	pub scrollback: Vec<Vec<Cell>>,
	/// Screen size
	pub size: (u16, u16),
}

/**
 * Mouse state for tracking
 */
#[derive(Debug, Clone)]
pub struct MouseState {
	/// Mouse tracking enabled
	pub tracking_enabled: bool,
	/// Mouse reporting mode
	pub reporting_mode: MouseReportingMode,
	/// Last mouse position
	pub last_pos: Option<(u16, u16)>,
	/// Mouse button state
	pub button_state: u8,
}

/**
 * Mouse reporting mode variants
 */
#[derive(Debug, Clone, PartialEq)]
pub enum MouseReportingMode {
	/// No mouse reporting
	None,
	/// X10 mouse reporting
	X10,
	/// VT200 mouse reporting
	VT200,
	/// VT200 highlight mouse reporting
	VT200Highlight,
	/// Button event mouse reporting
	ButtonEvent,
	/// Any event mouse reporting
	AnyEvent,
}

impl Default for AnsiParser {
	fn default() -> Self {
		Self {
			state: ParserState::Normal,
			mouse_state: MouseState::default(),
			bracketed_paste: false,
			escape_sequence: Vec::new(),
			control_sequence: Vec::new(),
			parameters: Vec::new(),
			intermediates: Vec::new(),
			final_char: None,
			terminal_state: TerminalState::default(),
			color_palette: Self::default_color_palette(),
			screen_buffers: ScreenBuffers::default(),
			params: Vec::new(),
			current_param: String::new(),
			app_cursor_keys: false,
		}
	}
}

impl Default for TerminalState {
	fn default() -> Self {
		Self {
			fg_color: Color::default(),
			bg_color: Color::default(),
			attributes: TextAttributes::default(),
			cursor_pos: (0, 0),
			size: (80, 24),
			insert_mode: false,
			origin_mode: false,
			auto_wrap: true,
			cursor_visible: true,
			scroll_region: None,
			app_cursor_keys: false,
		}
	}
}

impl Default for TextAttributes {
	fn default() -> Self {
		Self {
			bold: false,
			dim: false,
			italic: false,
			underline: false,
			blink: false,
			reverse: false,
			hidden: false,
			strikethrough: false,
		}
	}
}

impl Default for Color {
	fn default() -> Self {
		Self {
			r: 255,
			g: 255,
			b: 255,
			color_type: ColorType::Default,
		}
	}
}

impl Default for ScreenBuffers {
	fn default() -> Self {
		Self {
			primary: ScreenBuffer::new(80, 24),
			alternate: ScreenBuffer::new(80, 24),
			active: ScreenBufferType::Primary,
		}
	}
}

impl Default for MouseState {
	fn default() -> Self {
		Self {
			tracking_enabled: false,
			reporting_mode: MouseReportingMode::None,
			last_pos: None,
			button_state: 0,
		}
	}
}

impl ScreenBuffer {
	/**
	 * Creates a new screen buffer
	 * 
	 * @param cols - Number of columns
	 * @param rows - Number of rows
	 * @return ScreenBuffer - New screen buffer
	 */
	pub fn new(cols: u16, rows: u16) -> Self {
		let mut content = Vec::new();
		for _ in 0..rows {
			let mut row = Vec::new();
			for _ in 0..cols {
				row.push(Cell::default());
			}
			content.push(row);
		}
		
		Self {
			content,
			scrollback: Vec::new(),
			size: (cols, rows),
		}
	}
	
	/**
	 * Resizes the screen buffer
	 * 
	 * @param cols - New number of columns
	 * @param rows - New number of rows
	 */
	pub fn resize(&mut self, cols: u16, rows: u16) {
		let mut new_content = Vec::new();
		for row in 0..rows {
			let mut new_row = Vec::new();
			for col in 0..cols {
				if row < self.content.len() as u16 && col < self.content[row as usize].len() as u16 {
					new_row.push(self.content[row as usize][col as usize].clone());
				} else {
					new_row.push(Cell::default());
				}
			}
			new_content.push(new_row);
		}
		self.content = new_content;
		
		// Adjust cursor position is handled by terminal state
	}
}

impl Default for Cell {
	fn default() -> Self {
		Self {
			char: ' ',
			fg_color: Color::default(),
			bg_color: Color::default(),
			attributes: TextAttributes::default(),
			dirty: false,
		}
	}
}

impl AnsiParser {
	/**
	 * Creates a new ANSI parser
	 * 
	 * @return AnsiParser - New ANSI parser instance
	 */
	pub fn new() -> Self {
		Self {
			state: ParserState::Normal,
			mouse_state: MouseState::default(),
			bracketed_paste: false,
			escape_sequence: Vec::new(),
			control_sequence: Vec::new(),
			parameters: Vec::new(),
			intermediates: Vec::new(),
			final_char: None,
			terminal_state: TerminalState {
				fg_color: Color::default(),
				bg_color: Color::default(),
				attributes: TextAttributes::default(),
				cursor_pos: (0, 0),
				size: (80, 24),
				insert_mode: false,
				origin_mode: false,
				auto_wrap: true,
				cursor_visible: true,
				scroll_region: None,
				app_cursor_keys: false,
			},
			color_palette: HashMap::new(),
			screen_buffers: ScreenBuffers {
				primary: ScreenBuffer {
					content: vec![vec![Cell::default(); 80]; 24],
					scrollback: Vec::new(),
					size: (80, 24),
				},
				alternate: ScreenBuffer {
					content: vec![vec![Cell::default(); 80]; 24],
					scrollback: Vec::new(),
					size: (80, 24),
				},
				active: ScreenBufferType::Primary,
			},
			params: Vec::new(),
			app_cursor_keys: false,
			current_param: String::new(),
		}
	}
	
	/**
	 * Processes input bytes and returns parsed commands
	 * 
	 * @param input - Input bytes to process
	 * @return Vec<AnsiCommand> - Parsed ANSI commands
	 */
	pub fn process_input(&mut self, input: &[u8]) -> Result<Vec<AnsiCommand>> {
		let mut commands = Vec::new();
		
		for &byte in input {
			match self.state {
				ParserState::Normal => {
					if byte == 0x1b {
						self.state = ParserState::Escape;
					} else {
						commands.push(AnsiCommand::Print(byte as char));
					}
				}
				ParserState::Escape => {
					match byte {
						b'[' => {
							self.state = ParserState::ControlSequence;
							self.params.clear();
							self.current_param.clear();
						}
						b'P' => {
							self.state = ParserState::DeviceControl;
						}
						b']' => {
							self.state = ParserState::OperatingSystem;
						}
						b'(' | b')' => {
							self.state = ParserState::Normal;
						}
						_ => {
							self.state = ParserState::Normal;
						}
					}
				}
				ParserState::ControlSequence => {
					match byte {
						b'0'..=b'9' => {
							self.current_param.push(byte as char);
							self.state = ParserState::Parameter;
						}
						b';' => {
							self.params.push(self.current_param.parse().unwrap_or(0));
							self.current_param.clear();
						}
						b'?' => {
						}
						b'<' => {
							self.state = ParserState::Normal;
							commands.push(AnsiCommand::MouseTracking(true));
						}
						_ => {
							if !self.current_param.is_empty() {
								self.params.push(self.current_param.parse().unwrap_or(0));
							}
							commands.extend(self.handle_control_sequence(byte)?);
							self.state = ParserState::Normal;
						}
					}
				}
				ParserState::Parameter => {
					match byte {
						b'0'..=b'9' => {
							self.current_param.push(byte as char);
						}
						b';' => {
							self.params.push(self.current_param.parse().unwrap_or(0));
							self.current_param.clear();
						}
						_ => {
							if !self.current_param.is_empty() {
								self.params.push(self.current_param.parse().unwrap_or(0));
							}
							commands.extend(self.handle_control_sequence(byte)?);
							self.state = ParserState::Normal;
						}
					}
				}
				ParserState::OperatingSystem => {
					if byte == 0x07 {
						self.state = ParserState::Normal;
					}
				}
				ParserState::DeviceControl => {
					if byte == 0x07 {
						self.state = ParserState::Normal;
					}
				}
				_ => {
					self.state = ParserState::Normal;
				}
			}
		}
		
		Ok(commands)
	}
	
	/**
	 * Handles control sequence final characters
	 * 
	 * @param final_char - Final character of the sequence
	 * @return Result<Vec<AnsiCommand>> - Generated commands
	 */
	fn handle_control_sequence(&mut self, final_char: u8) -> Result<Vec<AnsiCommand>> {
		let mut commands = Vec::new();
		
		match final_char {
			b'A' => {
				// Cursor Up
				let count = self.params.get(0).copied().unwrap_or(1);
				commands.push(AnsiCommand::CursorUp(count));
			}
			b'B' => {
				// Cursor Down
				let count = self.params.get(0).copied().unwrap_or(1);
				commands.push(AnsiCommand::CursorDown(count));
			}
			b'C' => {
				// Cursor Forward
				let count = self.params.get(0).copied().unwrap_or(1);
				commands.push(AnsiCommand::CursorForward(count));
			}
			b'D' => {
				// Cursor Backward
				let count = self.params.get(0).copied().unwrap_or(1);
				commands.push(AnsiCommand::CursorBackward(count));
			}
			b'E' => {
				// Cursor Next Line
				let count = self.params.get(0).copied().unwrap_or(1);
				commands.push(AnsiCommand::CursorNextLine(count));
			}
			b'F' => {
				// Cursor Previous Line
				let count = self.params.get(0).copied().unwrap_or(1);
				commands.push(AnsiCommand::CursorPreviousLine(count));
			}
			b'G' => {
				// Cursor Horizontal Absolute
				let col = self.params.get(0).copied().unwrap_or(1);
				commands.push(AnsiCommand::CursorHorizontalAbsolute(col));
			}
			b'H' => {
				// Cursor Position
				let row = self.params.get(0).copied().unwrap_or(1);
				let col = self.params.get(1).copied().unwrap_or(1);
				commands.push(AnsiCommand::CursorPosition(row, col));
			}
			b'J' => {
				// Erase in Display
				let mode = self.params.get(0).copied().unwrap_or(0);
				commands.push(AnsiCommand::EraseInDisplay(mode));
			}
			b'K' => {
				// Erase in Line
				let mode = self.params.get(0).copied().unwrap_or(0);
				commands.push(AnsiCommand::EraseInLine(mode));
			}
			b'm' => {
				// Set Graphics Mode
				commands.extend(self.handle_graphics_mode()?);
			}
			b's' => {
				// Save Cursor Position
				commands.push(AnsiCommand::SaveCursor);
			}
			b'u' => {
				// Restore Cursor Position
				commands.push(AnsiCommand::RestoreCursor);
			}
			b'h' => {
				// Set Mode
				commands.extend(self.handle_set_mode()?);
			}
			b'l' => {
				// Reset Mode
				commands.extend(self.handle_reset_mode()?);
			}
			b'c' => {
				// Device Attributes
				commands.push(AnsiCommand::DeviceAttributes);
			}
			b'n' => {
				// Device Status Report
				let mode = self.params.get(0).copied().unwrap_or(0);
				commands.push(AnsiCommand::DeviceStatusReport(mode));
			}
			b'r' => {
				// Set Top and Bottom Margins
				let top = self.params.get(0).copied().unwrap_or(1);
				let bottom = self.params.get(1).copied().unwrap_or(self.terminal_state.size.1 as u32);
				commands.push(AnsiCommand::SetScrollRegion(top, bottom));
			}
			_ => {
			}
		}
		
		Ok(commands)
	}
	
	/**
	 * Handles graphics mode (SGR) sequences
	 * 
	 * @return Result<Vec<AnsiCommand>> - Generated commands
	 */
	fn handle_graphics_mode(&mut self) -> Result<Vec<AnsiCommand>> {
		let mut commands = Vec::new();
		
		if self.params.is_empty() {
			commands.push(AnsiCommand::ResetAttributes);
			return Ok(commands);
		}
		
		for &param in &self.params {
			match param {
				0 => commands.push(AnsiCommand::ResetAttributes),
				1 => commands.push(AnsiCommand::SetBold),
				2 => commands.push(AnsiCommand::SetDim),
				3 => commands.push(AnsiCommand::SetItalic),
				4 => commands.push(AnsiCommand::SetUnderline),
				5 => commands.push(AnsiCommand::SetBlink),
				7 => commands.push(AnsiCommand::SetReverse),
				8 => commands.push(AnsiCommand::SetHidden),
				9 => commands.push(AnsiCommand::SetStrikethrough),
				30..=37 => {
					// Set foreground color (named)
					commands.push(AnsiCommand::SetForegroundColor(ColorType::Named((param - 30) as u8)));
				}
				38 => {
					// Set foreground color (extended)
					commands.extend(self.handle_extended_color(true)?.into_iter());
				}
				39 => {
					// Reset foreground color
					commands.push(AnsiCommand::ResetForegroundColor);
				}
				40..=47 => {
					// Set background color (named)
					commands.push(AnsiCommand::SetBackgroundColor(ColorType::Named((param - 40) as u8)));
				}
				48 => {
					// Set background color (extended)
					commands.extend(self.handle_extended_color(false)?.into_iter());
				}
				49 => {
					// Reset background color
					commands.push(AnsiCommand::ResetBackgroundColor);
				}
				90..=97 => {
					// Set foreground color (bright named)
					commands.push(AnsiCommand::SetForegroundColor(ColorType::Named((param - 90 + 8) as u8)));
				}
				100..=107 => {
					// Set background color (bright named)
					commands.push(AnsiCommand::SetBackgroundColor(ColorType::Named((param - 100 + 8) as u8)));
				}
				_ => {
				}
			}
		}
		
		Ok(commands)
	}
	
	/**
	 * Handles extended color sequences
	 * 
	 * @param is_foreground - Whether this is foreground color
	 * @return Result<Vec<AnsiCommand>> - Generated commands
	 */
	fn handle_extended_color(&mut self, is_foreground: bool) -> Result<Vec<AnsiCommand>> {
		let mut commands = Vec::new();
		
		// Parse extended color parameters
		if self.params.len() >= 2 {
			match self.params[0] {
				5 => {
					// 256-color mode: ESC[38;5;n or ESC[48;5;n
					if self.params.len() >= 3 && self.params[1] == 5 {
						let color_index = self.params[2];
						if color_index <= 255 {
							let color_type = ColorType::Index(color_index as u8);
							if is_foreground {
								commands.push(AnsiCommand::SetForegroundColor(color_type));
							} else {
								commands.push(AnsiCommand::SetBackgroundColor(color_type));
							}
						}
					}
				}
				2 => {
					// True color mode: ESC[38;2;r;g;b or ESC[48;2;r;g;b
					if self.params.len() >= 5 && self.params[1] == 2 {
						let r = self.params[2] as u8;
						let g = self.params[3] as u8;
						let b = self.params[4] as u8;
						
						// Create true color
						let color = Color {
							r, g, b,
							color_type: ColorType::TrueColor,
						};
						
						if is_foreground {
							commands.push(AnsiCommand::SetForegroundColor(ColorType::TrueColor));
						} else {
							commands.push(AnsiCommand::SetBackgroundColor(ColorType::TrueColor));
						}
						
						// Update terminal state
						if is_foreground {
							self.terminal_state.fg_color = color;
						} else {
							self.terminal_state.bg_color = color;
						}
					}
				}
				_ => {
					// Fallback to default color
					if is_foreground {
						commands.push(AnsiCommand::SetForegroundColor(ColorType::Default));
					} else {
						commands.push(AnsiCommand::SetBackgroundColor(ColorType::Default));
					}
				}
			}
		} else {
			// Fallback to default color
			if is_foreground {
				commands.push(AnsiCommand::SetForegroundColor(ColorType::Default));
			} else {
				commands.push(AnsiCommand::SetBackgroundColor(ColorType::Default));
			}
		}
		
		Ok(commands)
	}
	
	/**
	 * Handles set mode sequences
	 * 
	 * @return Result<Vec<AnsiCommand>> - Generated commands
	 */
	fn handle_set_mode(&mut self) -> Result<Vec<AnsiCommand>> {
		let mut commands = Vec::new();
		
		for &param in &self.params {
			match param {
				1 => commands.push(AnsiCommand::SetApplicationCursorKeys),
				4 => commands.push(AnsiCommand::SetInsertMode),
				5 => commands.push(AnsiCommand::SetReverseVideo),
				6 => commands.push(AnsiCommand::SetOriginMode),
				7 => commands.push(AnsiCommand::SetAutoWrap),
				12 => commands.push(AnsiCommand::SetBlinkingCursor),
				25 => commands.push(AnsiCommand::ShowCursor),
				1000 => commands.push(AnsiCommand::SetMouseTracking(MouseReportingMode::X10)),
				1001 => commands.push(AnsiCommand::SetMouseTracking(MouseReportingMode::VT200Highlight)),
				1002 => commands.push(AnsiCommand::SetMouseTracking(MouseReportingMode::VT200)),
				1003 => commands.push(AnsiCommand::SetMouseTracking(MouseReportingMode::AnyEvent)),
				2004 => commands.push(AnsiCommand::SetBracketedPaste(true)),
				_ => {
				}
			}
		}
		
		Ok(commands)
	}
	
	/**
	 * Handles reset mode sequences
	 * 
	 * @return Result<Vec<AnsiCommand>> - Generated commands
	 */
	fn handle_reset_mode(&mut self) -> Result<Vec<AnsiCommand>> {
		let mut commands = Vec::new();
		
		for &param in &self.params {
			match param {
				1 => commands.push(AnsiCommand::ResetApplicationCursorKeys),
				4 => commands.push(AnsiCommand::ResetInsertMode),
				5 => commands.push(AnsiCommand::ResetReverseVideo),
				6 => commands.push(AnsiCommand::ResetOriginMode),
				7 => commands.push(AnsiCommand::ResetAutoWrap),
				12 => commands.push(AnsiCommand::ResetBlinkingCursor),
				25 => commands.push(AnsiCommand::HideCursor),
				1000..=1003 => commands.push(AnsiCommand::SetMouseTracking(MouseReportingMode::None)),
				2004 => commands.push(AnsiCommand::SetBracketedPaste(false)),
				_ => {
				}
			}
		}
		
		Ok(commands)
	}
	
	/**
	 * Creates default color palette
	 * 
	 * @return HashMap<u8, Color> - Default color palette
	 */
	fn default_color_palette() -> HashMap<u8, Color> {
		let mut palette = HashMap::new();
		
		// Standard colors (0-7)
		palette.insert(0, Color { r: 0, g: 0, b: 0, color_type: ColorType::Named(0) });
		palette.insert(1, Color { r: 205, g: 0, b: 0, color_type: ColorType::Named(1) });
		palette.insert(2, Color { r: 0, g: 205, b: 0, color_type: ColorType::Named(2) });
		palette.insert(3, Color { r: 205, g: 205, b: 0, color_type: ColorType::Named(3) });
		palette.insert(4, Color { r: 0, g: 0, b: 238, color_type: ColorType::Named(4) });
		palette.insert(5, Color { r: 205, g: 0, b: 205, color_type: ColorType::Named(5) });
		palette.insert(6, Color { r: 0, g: 205, b: 205, color_type: ColorType::Named(6) });
		palette.insert(7, Color { r: 229, g: 229, b: 229, color_type: ColorType::Named(7) });
		
		// Bright colors (8-15)
		palette.insert(8, Color { r: 127, g: 127, b: 127, color_type: ColorType::Named(8) });
		palette.insert(9, Color { r: 255, g: 0, b: 0, color_type: ColorType::Named(9) });
		palette.insert(10, Color { r: 0, g: 255, b: 0, color_type: ColorType::Named(10) });
		palette.insert(11, Color { r: 255, g: 255, b: 0, color_type: ColorType::Named(11) });
		palette.insert(12, Color { r: 0, g: 0, b: 255, color_type: ColorType::Named(12) });
		palette.insert(13, Color { r: 255, g: 0, b: 255, color_type: ColorType::Named(13) });
		palette.insert(14, Color { r: 0, g: 255, b: 255, color_type: ColorType::Named(14) });
		palette.insert(15, Color { r: 255, g: 255, b: 255, color_type: ColorType::Named(15) });
		
		// 216-color cube (16-231)
		for i in 0..6 {
			for j in 0..6 {
				for k in 0..6 {
					let index = 16 + i * 36 + j * 6 + k;
					let r = if i == 0 { 0 } else { 55 + i * 40 };
					let g = if j == 0 { 0 } else { 55 + j * 40 };
					let b = if k == 0 { 0 } else { 55 + k * 40 };
					palette.insert(index as u8, Color { r, g, b, color_type: ColorType::Index(index as u8) });
				}
			}
		}
		
		// Grayscale ramp (232-255)
		for i in 0..24 {
			let index = 232 + i;
			let gray = 8 + i * 10;
			palette.insert(index as u8, Color { r: gray, g: gray, b: gray, color_type: ColorType::Index(index as u8) });
		}
		
		palette
	}
}

/**
 * ANSI command variants
 */
#[derive(Debug, Clone)]
pub enum AnsiCommand {
	/// Print a character
	Print(char),
	/// Move cursor up
	CursorUp(u32),
	/// Move cursor down
	CursorDown(u32),
	/// Move cursor forward
	CursorForward(u32),
	/// Move cursor backward
	CursorBackward(u32),
	/// Move cursor to next line
	CursorNextLine(u32),
	/// Move cursor to previous line
	CursorPreviousLine(u32),
	/// Move cursor to absolute column
	CursorHorizontalAbsolute(u32),
	/// Move cursor to absolute position
	CursorPosition(u32, u32),
	/// Save cursor position
	SaveCursor,
	/// Restore cursor position
	RestoreCursor,
	/// Erase in display
	EraseInDisplay(u32),
	/// Erase in line
	EraseInLine(u32),
	/// Set graphics mode
	SetGraphicsMode(Vec<u32>),
	/// Reset all attributes
	ResetAttributes,
	/// Set bold
	SetBold,
	/// Set dim
	SetDim,
	/// Set italic
	SetItalic,
	/// Set underline
	SetUnderline,
	/// Set blink
	SetBlink,
	/// Set reverse video
	SetReverse,
	/// Set hidden
	SetHidden,
	/// Set strikethrough
	SetStrikethrough,
	/// Set foreground color
	SetForegroundColor(ColorType),
	/// Set background color
	SetBackgroundColor(ColorType),
	/// Reset foreground color
	ResetForegroundColor,
	/// Reset background color
	ResetBackgroundColor,
	/// Set application cursor keys
	SetApplicationCursorKeys,
	/// Reset application cursor keys
	ResetApplicationCursorKeys,
	/// Set insert mode
	SetInsertMode,
	/// Reset insert mode
	ResetInsertMode,
	/// Set reverse video
	SetReverseVideo,
	/// Reset reverse video
	ResetReverseVideo,
	/// Set origin mode
	SetOriginMode,
	/// Reset origin mode
	ResetOriginMode,
	/// Set auto wrap
	SetAutoWrap,
	/// Reset auto wrap
	ResetAutoWrap,
	/// Set blinking cursor
	SetBlinkingCursor,
	/// Reset blinking cursor
	ResetBlinkingCursor,
	/// Show cursor
	ShowCursor,
	/// Hide cursor
	HideCursor,
	/// Set mouse tracking
	SetMouseTracking(MouseReportingMode),
	/// Set bracketed paste mode
	SetBracketedPaste(bool),
	/// Device attributes
	DeviceAttributes,
	/// Device status report
	DeviceStatusReport(u32),
	/// Set scroll region
	SetScrollRegion(u32, u32),
	/// Mouse tracking
	MouseTracking(bool),
} 