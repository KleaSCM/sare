
use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SubstitutionState {
	/// Whether in substitution mode
	pub substitution_mode: bool,
	/// Current substitution depth
	pub substitution_depth: usize,
	/// Substitution buffer for nested commands
	pub substitution_buffer: String,
}

impl Default for SubstitutionState {
	fn default() -> Self {
		Self {
			substitution_mode: false,
			substitution_depth: 0,
			substitution_buffer: String::new(),
		}
	}
}

pub struct SubstitutionProcessor;

impl SubstitutionProcessor {
	pub fn detect_command_substitutions(input: &str) -> Vec<(usize, usize, String)> {
		
		let mut substitutions = Vec::new();
		let mut i = 0;
		
		while i < input.len() {
			// Check for $(command) syntax
			if input[i..].starts_with("$(") {
				let start = i;
				let mut depth = 1;
				let mut j = i + 2;
				
				while j < input.len() && depth > 0 {
					match input.chars().nth(j) {
						Some('(') => depth += 1,
						Some(')') => depth -= 1,
						_ => {}
					}
					j += 1;
				}
				
				if depth == 0 {
					let command = input[i + 2..j - 1].to_string();
					substitutions.push((start, j, command));
				}
				
				i = j;
			}
			// Check for `command` syntax
			else if input[i..].starts_with('`') {
				let start = i;
				let mut j = i + 1;
				
				while j < input.len() {
					if input.chars().nth(j) == Some('`') {
						break;
					}
					j += 1;
				}
				
				if j < input.len() {
					let command = input[i + 1..j].to_string();
					substitutions.push((start, j + 1, command));
				}
				
				i = j + 1;
			} else {
				i += 1;
			}
		}
		
		substitutions
	}
	
	pub fn execute_substitution_command(command: &str) -> Result<String> {
		
		// Split command into parts
		let parts: Vec<&str> = command.split_whitespace().collect();
		if parts.is_empty() {
			return Ok(String::new());
		}
		
		// Execute the command
		let output = Command::new(parts[0])
			.args(&parts[1..])
			.output()?;
		
		// Convert output to string
		let stdout = String::from_utf8(output.stdout)?;
		let stderr = String::from_utf8(output.stderr)?;
		
		// Combine stdout and stderr, trim whitespace
		let mut result = stdout;
		if !stderr.is_empty() {
			result.push_str(&stderr);
		}
		
		Ok(result.trim().to_string())
	}
	
	pub fn process_command_substitutions(input: &str) -> Result<String> {
		
		let mut result = input.to_string();
		let substitutions = Self::detect_command_substitutions(&result);
		
		// Process substitutions in reverse order to maintain indices
		for (start, end, command) in substitutions.iter().rev() {
			match Self::execute_substitution_command(command) {
				Ok(output) => {
					// Replace the substitution with the output
					result.replace_range(*start..*end, &output);
				}
				Err(_) => {
					// On error, replace with empty string
					result.replace_range(*start..*end, "");
				}
			}
		}
		
		Ok(result)
	}
}

impl SubstitutionState {
	pub fn is_substitution_mode(&self) -> bool {
		self.substitution_mode
	}
	
	pub fn set_substitution_mode(&mut self, mode: bool) {
		self.substitution_mode = mode;
	}
	
	pub fn get_depth(&self) -> usize {
		self.substitution_depth
	}
	
	pub fn set_depth(&mut self, depth: usize) {
		self.substitution_depth = depth;
	}
	
	pub fn get_buffer(&self) -> String {
		self.substitution_buffer.clone()
	}
	
	pub fn set_buffer(&mut self, buffer: String) {
		self.substitution_buffer = buffer;
	}
	
	pub fn process_substitutions(&self, input: &str) -> Result<String> {
		SubstitutionProcessor::process_command_substitutions(input)
	}
} 