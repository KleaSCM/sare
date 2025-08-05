/**
 * ANSI protocol implementation tests
 * 
 * This module provides comprehensive tests for the ANSI escape sequence
 * parser and terminal renderer, verifying that VT100/VT220/VT320
 * terminal protocols are handled correctly.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_ansi_protocol.rs
 * Description: Tests for ANSI protocol implementation
 */

use sare_terminal::terminal::protocol::{AnsiParser, AnsiCommand, ColorType, MouseReportingMode};
use sare_terminal::terminal::renderer::{TerminalRenderer, RendererConfig};

/**
 * Test ANSI parser functionality
 */
#[test]
fn test_ansi_parser_creation() {
	let parser = AnsiParser::new();
	assert_eq!(parser.state, sare_terminal::terminal::protocol::ParserState::Normal);
}

/**
 * Test basic text printing
 */
#[test]
fn test_basic_text_printing() {
	let mut parser = AnsiParser::new();
	let input = b"Hello, World!";
	let commands = parser.process_input(input).unwrap();
	
	assert_eq!(commands.len(), 13);
	for (i, command) in commands.iter().enumerate() {
		match command {
			AnsiCommand::Print(ch) => {
				assert_eq!(*ch, input[i] as char);
			}
			_ => panic!("Expected Print command, got {:?}", command),
		}
	}
}

/**
 * Test cursor movement commands
 */
#[test]
fn test_cursor_movement() {
	let mut parser = AnsiParser::new();
	
	// Test cursor up
	let input = b"\x1b[5A";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::CursorUp(count) => assert_eq!(*count, 5),
		_ => panic!("Expected CursorUp command"),
	}
	
	// Test cursor down
	let input = b"\x1b[3B";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::CursorDown(count) => assert_eq!(*count, 3),
		_ => panic!("Expected CursorDown command"),
	}
	
	// Test cursor forward
	let input = b"\x1b[10C";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::CursorForward(count) => assert_eq!(*count, 10),
		_ => panic!("Expected CursorForward command"),
	}
	
	// Test cursor backward
	let input = b"\x1b[7D";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::CursorBackward(count) => assert_eq!(*count, 7),
		_ => panic!("Expected CursorBackward command"),
	}
}

/**
 * Test cursor positioning
 */
#[test]
fn test_cursor_positioning() {
	let mut parser = AnsiParser::new();
	
	// Test cursor position
	let input = b"\x1b[10;20H";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::CursorPosition(row, col) => {
			assert_eq!(*row, 10);
			assert_eq!(*col, 20);
		}
		_ => panic!("Expected CursorPosition command"),
	}
	
	// Test cursor horizontal absolute
	let input = b"\x1b[15G";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::CursorHorizontalAbsolute(col) => assert_eq!(*col, 15),
		_ => panic!("Expected CursorHorizontalAbsolute command"),
	}
}

/**
 * Test screen clearing commands
 */
#[test]
fn test_screen_clearing() {
	let mut parser = AnsiParser::new();
	
	// Test erase in display
	let input = b"\x1b[2J";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::EraseInDisplay(mode) => assert_eq!(*mode, 2),
		_ => panic!("Expected EraseInDisplay command"),
	}
	
	// Test erase in line
	let input = b"\x1b[1K";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::EraseInLine(mode) => assert_eq!(*mode, 1),
		_ => panic!("Expected EraseInLine command"),
	}
}

/**
 * Test color and attribute commands
 */
#[test]
fn test_color_and_attributes() {
	let mut parser = AnsiParser::new();
	
	// Test reset attributes
	let input = b"\x1b[0m";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::ResetAttributes => {},
		_ => panic!("Expected ResetAttributes command"),
	}
	
	// Test bold
	let input = b"\x1b[1m";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::SetBold => {},
		_ => panic!("Expected SetBold command"),
	}
	
	// Test foreground color
	let input = b"\x1b[31m";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::SetForegroundColor(ColorType::Named(index)) => assert_eq!(*index, 1),
		_ => panic!("Expected SetForegroundColor command"),
	}
	
	// Test background color
	let input = b"\x1b[42m";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::SetBackgroundColor(ColorType::Named(index)) => assert_eq!(*index, 2),
		_ => panic!("Expected SetBackgroundColor command"),
	}
}

/**
 * Test mode setting commands
 */
#[test]
fn test_mode_setting() {
	let mut parser = AnsiParser::new();
	
	// Test show cursor
	let input = b"\x1b[?25h";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::ShowCursor => {},
		_ => panic!("Expected ShowCursor command"),
	}
	
	// Test hide cursor
	let input = b"\x1b[?25l";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::HideCursor => {},
		_ => panic!("Expected HideCursor command"),
	}
	
	// Test mouse tracking
	let input = b"\x1b[?1000h";
	let commands = parser.process_input(input).unwrap();
	assert_eq!(commands.len(), 1);
	match &commands[0] {
		AnsiCommand::SetMouseTracking(MouseReportingMode::X10) => {},
		_ => panic!("Expected SetMouseTracking command"),
	}
}

/**
 * Test terminal renderer creation
 */
#[test]
fn test_renderer_creation() {
	let config = RendererConfig::default();
	let renderer = TerminalRenderer::new(config);
	
	assert_eq!(renderer.state().cursor_pos, (0, 0));
	assert_eq!(renderer.state().size, (80, 24));
}

/**
 * Test renderer text processing
 */
#[test]
fn test_renderer_text_processing() {
	let config = RendererConfig::default();
	let mut renderer = TerminalRenderer::new(config);
	
	// Process simple text
	let input = b"Hello, World!";
	renderer.process_input(input).unwrap();
	
	// Check that text was rendered
	let content = renderer.screen_content();
	assert!(!content.is_empty());
	assert!(!content[0].is_empty());
	
	// Check first character
	let first_cell = &content[0][0];
	assert_eq!(first_cell.char, 'H');
}

/**
 * Test renderer cursor movement
 */
#[test]
fn test_renderer_cursor_movement() {
	let config = RendererConfig::default();
	let mut renderer = TerminalRenderer::new(config);
	
	// Move cursor to position (5, 5)
	let input = b"\x1b[6;6H";
	renderer.process_input(input).unwrap();
	
	// Check cursor position
	let state = renderer.state();
	assert_eq!(state.cursor_pos, (5, 5));
}

/**
 * Test renderer color support
 */
#[test]
fn test_renderer_color_support() {
	let config = RendererConfig::default();
	let mut renderer = TerminalRenderer::new(config);
	
	// Set red foreground color
	let input = b"\x1b[31mX";
	renderer.process_input(input).unwrap();
	
	// Check that character was rendered with color
	let content = renderer.screen_content();
	let cell = &content[0][0];
	assert_eq!(cell.char, 'X');
	assert_eq!(cell.fg_color.color_type, ColorType::Named(1));
}

/**
 * Test renderer screen clearing
 */
#[test]
fn test_renderer_screen_clearing() {
	let config = RendererConfig::default();
	let mut renderer = TerminalRenderer::new(config);
	
	// Print some text
	let input = b"Hello, World!";
	renderer.process_input(input).unwrap();
	
	// Clear screen
	let clear_input = b"\x1b[2J";
	renderer.process_input(clear_input).unwrap();
	
	// Check that screen was cleared
	let content = renderer.screen_content();
	for row in content {
		for cell in row {
			assert_eq!(cell.char, ' ');
		}
	}
}

/**
 * Test renderer resize functionality
 */
#[test]
fn test_renderer_resize() {
	let config = RendererConfig::default();
	let mut renderer = TerminalRenderer::new(config);
	
	// Resize to 100x30
	renderer.resize(100, 30);
	
	// Check new size
	let state = renderer.state();
	assert_eq!(state.size, (100, 30));
	
	// Check content dimensions
	let content = renderer.screen_content();
	assert_eq!(content.len(), 30);
	assert_eq!(content[0].len(), 100);
}

/**
 * Test complex ANSI sequences
 */
#[test]
fn test_complex_ansi_sequences() {
	let mut parser = AnsiParser::new();
	
	// Test multiple commands in sequence
	let input = b"\x1b[1;31mHello\x1b[0m";
	let commands = parser.process_input(input).unwrap();
	
	assert_eq!(commands.len(), 4); // SetBold, SetForegroundColor, Print chars, ResetAttributes
	
	// Check commands
	match &commands[0] {
		AnsiCommand::SetBold => {},
		_ => panic!("Expected SetBold command"),
	}
	
	match &commands[1] {
		AnsiCommand::SetForegroundColor(ColorType::Named(index)) => assert_eq!(*index, 1),
		_ => panic!("Expected SetForegroundColor command"),
	}
	
	match &commands[2] {
		AnsiCommand::Print(ch) => assert_eq!(*ch, 'H'),
		_ => panic!("Expected Print command"),
	}
	
	match &commands[3] {
		AnsiCommand::ResetAttributes => {},
		_ => panic!("Expected ResetAttributes command"),
	}
}

/**
 * Test renderer with complex sequences
 */
#[test]
fn test_renderer_complex_sequences() {
	let config = RendererConfig::default();
	let mut renderer = TerminalRenderer::new(config);
	
	// Process complex sequence: bold red text
	let input = b"\x1b[1;31mHello\x1b[0m";
	renderer.process_input(input).unwrap();
	
	// Check that text was rendered with attributes
	let content = renderer.screen_content();
	let cell = &content[0][0];
	assert_eq!(cell.char, 'H');
	assert!(cell.attributes.bold);
	assert_eq!(cell.fg_color.color_type, ColorType::Named(1));
}

/**
 * Test renderer scrollback functionality
 */
#[test]
fn test_renderer_scrollback() {
	let config = RendererConfig {
		size: (80, 5), // Small terminal for testing
		max_scrollback: 10,
		..Default::default()
	};
	let mut renderer = TerminalRenderer::new(config);
	
	// Fill the screen and trigger scrolling
	for i in 0..10 {
		let input = format!("Line {}\n", i).into_bytes();
		renderer.process_input(&input).unwrap();
	}
	
	// Check scrollback content
	let scrollback = renderer.scrollback_content();
	assert!(!scrollback.is_empty());
}

/**
 * Test renderer dirty region tracking
 */
#[test]
fn test_renderer_dirty_regions() {
	let config = RendererConfig::default();
	let mut renderer = TerminalRenderer::new(config);
	
	// Process some input
	let input = b"Hello";
	renderer.process_input(input).unwrap();
	
	// Check that dirty regions were tracked
	let dirty_regions = renderer.dirty_regions();
	assert!(!dirty_regions.is_empty());
	
	// Clear dirty regions
	renderer.clear_dirty_regions();
	assert!(renderer.dirty_regions().is_empty());
}

/**
 * Test ANSI parser state machine
 */
#[test]
fn test_parser_state_machine() {
	let mut parser = AnsiParser::new();
	
	// Start in normal state
	assert_eq!(parser.state, sare_terminal::terminal::protocol::ParserState::Normal);
	
	// Enter escape state
	let input = b"\x1b";
	parser.process_input(input).unwrap();
	assert_eq!(parser.state, sare_terminal::terminal::protocol::ParserState::Normal);
	
	// Enter control sequence state
	let input = b"\x1b[";
	parser.process_input(input).unwrap();
	assert_eq!(parser.state, sare_terminal::terminal::protocol::ParserState::Normal);
}

/**
 * Test renderer with mixed content
 */
#[test]
fn test_renderer_mixed_content() {
	let config = RendererConfig::default();
	let mut renderer = TerminalRenderer::new(config);
	
	// Process mixed text and control sequences
	let input = b"Hello\x1b[31mRed\x1b[0mWorld";
	renderer.process_input(input).unwrap();
	
	// Check that content was rendered correctly
	let content = renderer.screen_content();
	
	// First part: "Hello"
	assert_eq!(content[0][0].char, 'H');
	assert_eq!(content[0][1].char, 'e');
	assert_eq!(content[0][2].char, 'l');
	assert_eq!(content[0][3].char, 'l');
	assert_eq!(content[0][4].char, 'o');
	
	// Red part: "Red"
	assert_eq!(content[0][5].char, 'R');
	assert_eq!(content[0][5].fg_color.color_type, ColorType::Named(1));
	assert_eq!(content[0][6].char, 'e');
	assert_eq!(content[0][6].fg_color.color_type, ColorType::Named(1));
	assert_eq!(content[0][7].char, 'd');
	assert_eq!(content[0][7].fg_color.color_type, ColorType::Named(1));
	
	// Reset part: "World"
	assert_eq!(content[0][8].char, 'W');
	assert_eq!(content[0][8].fg_color.color_type, ColorType::Default);
}

/**
 * Run all ANSI protocol tests
 */
pub fn run_ansi_protocol_tests() -> Vec<(&'static str, bool)> {
	let mut results = Vec::new();
	
	let tests = vec![
		("test_ansi_parser_creation", test_ansi_parser_creation),
		("test_basic_text_printing", test_basic_text_printing),
		("test_cursor_movement", test_cursor_movement),
		("test_cursor_positioning", test_cursor_positioning),
		("test_screen_clearing", test_screen_clearing),
		("test_color_and_attributes", test_color_and_attributes),
		("test_mode_setting", test_mode_setting),
		("test_renderer_creation", test_renderer_creation),
		("test_renderer_text_processing", test_renderer_text_processing),
		("test_renderer_cursor_movement", test_renderer_cursor_movement),
		("test_renderer_color_support", test_renderer_color_support),
		("test_renderer_screen_clearing", test_renderer_screen_clearing),
		("test_renderer_resize", test_renderer_resize),
		("test_complex_ansi_sequences", test_complex_ansi_sequences),
		("test_renderer_complex_sequences", test_renderer_complex_sequences),
		("test_renderer_scrollback", test_renderer_scrollback),
		("test_renderer_dirty_regions", test_renderer_dirty_regions),
		("test_parser_state_machine", test_parser_state_machine),
		("test_renderer_mixed_content", test_renderer_mixed_content),
	];
	
	for (name, test_fn) in tests {
		let result = std::panic::catch_unwind(test_fn).is_ok();
		results.push((name, result));
	}
	
	results
} 