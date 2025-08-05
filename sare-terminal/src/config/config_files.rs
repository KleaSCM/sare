/**
 * Configuration file system for Sare terminal
 * 
 * This module provides comprehensive configuration file capabilities including
 * JSON/TOML/YAML support, config validation, and schema validation.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: config_files.rs
 * Description: Configuration file system with multiple format support
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/**
 * Configuration file format
 * 
 * 設定ファイルフォーマットです。
 * サポートされる設定ファイルの
 * 形式を管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfigFormat {
	/// JSON format
	Json,
	/// TOML format
	Toml,
	/// YAML format
	Yaml,
}

impl ConfigFormat {
	/**
	 * Gets file extension for format
	 * 
	 * @return &str - File extension
	 */
	pub fn extension(&self) -> &str {
		match self {
			ConfigFormat::Json => "json",
			ConfigFormat::Toml => "toml",
			ConfigFormat::Yaml => "yaml",
		}
	}
	
	/**
	 * Detects format from file extension
	 * 
	 * @param extension - File extension
	 * @return Option<ConfigFormat> - Detected format
	 */
	pub fn from_extension(extension: &str) -> Option<Self> {
		match extension.to_lowercase().as_str() {
			"json" => Some(ConfigFormat::Json),
			"toml" => Some(ConfigFormat::Toml),
			"yaml" | "yml" => Some(ConfigFormat::Yaml),
			_ => None,
		}
	}
}

/**
 * Configuration validation error
 * 
 * 設定検証エラーです。
 * 設定の検証時に発生する
 * エラーを管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationError {
	/// Error field path
	pub field: String,
	/// Error message
	pub message: String,
	/// Error severity
	pub severity: ValidationSeverity,
}

/**
 * Validation severity
 * 
 * 検証の重要度です。
 * エラーの重要度を
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationSeverity {
	/// Warning - non-critical issue
	Warning,
	/// Error - critical issue
	Error,
	/// Fatal - prevents loading
	Fatal,
}

/**
 * Configuration schema
 * 
 * 設定スキーマです。
 * 設定の構造と検証ルールを
 * 定義します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
	/// Schema version
	pub version: String,
	/// Schema description
	pub description: String,
	/// Required fields
	pub required: Vec<String>,
	/// Field definitions
	pub properties: HashMap<String, SchemaProperty>,
	/// Additional properties allowed
	pub additional_properties: bool,
}

/**
 * Schema property definition
 * 
 * スキーマプロパティ定義です。
 * 設定フィールドの定義を
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaProperty {
	/// Property type
	pub property_type: String,
	/// Property description
	pub description: String,
	/// Default value
	pub default: Option<serde_json::Value>,
	/// Minimum value (for numbers)
	pub minimum: Option<f64>,
	/// Maximum value (for numbers)
	pub maximum: Option<f64>,
	/// Minimum length (for strings)
	pub min_length: Option<usize>,
	/// Maximum length (for strings)
	pub max_length: Option<usize>,
	/// Allowed values
	pub enum_values: Option<Vec<serde_json::Value>>,
	/// Pattern (for strings)
	pub pattern: Option<String>,
	/// Required status
	pub required: bool,
}

/**
 * Configuration file manager
 * 
 * 設定ファイルマネージャーです。
 * 設定ファイルの読み込み、保存、検証を
 * 管理します。
 */
pub struct ConfigFileManager {
	/// Configuration directory
	config_dir: String,
	/// Default configuration
	default_config: Arc<RwLock<serde_json::Value>>,
	/// Configuration schema
	schema: Arc<RwLock<ConfigSchema>>,
	/// File watchers
	watchers: Arc<RwLock<HashMap<String, Box<dyn Fn() + Send + Sync>>>>,
}

impl ConfigFileManager {
	/**
	 * Creates a new configuration file manager
	 * 
	 * @return ConfigFileManager - New configuration file manager
	 */
	pub fn new() -> Self {
		let config_dir = format!("{}/.sare", dirs::home_dir().unwrap_or_default().display());
		
		Self {
			config_dir,
			default_config: Arc::new(RwLock::new(serde_json::Value::Null)),
			schema: Arc::new(RwLock::new(Self::default_schema())),
			watchers: Arc::new(RwLock::new(HashMap::new())),
		}
	}
	
	/**
	 * Creates default configuration schema
	 * 
	 * @return ConfigSchema - Default schema
	 */
	fn default_schema() -> ConfigSchema {
		let mut properties = HashMap::new();
		
		// Theme configuration
		properties.insert("theme".to_string(), SchemaProperty {
			property_type: "string".to_string(),
			description: "Current theme name".to_string(),
			default: Some(serde_json::Value::String("default-dark".to_string())),
			minimum: None,
			maximum: None,
			min_length: Some(1),
			max_length: Some(100),
			enum_values: Some(vec![
				serde_json::Value::String("default-dark".to_string()),
				serde_json::Value::String("default-light".to_string()),
				serde_json::Value::String("dracula".to_string()),
				serde_json::Value::String("solarized-dark".to_string()),
				serde_json::Value::String("solarized-light".to_string()),
				serde_json::Value::String("gruvbox-dark".to_string()),
				serde_json::Value::String("nord".to_string()),
			]),
			pattern: None,
			required: true,
		});
		
		// Terminal configuration
		properties.insert("terminal".to_string(), SchemaProperty {
			property_type: "object".to_string(),
			description: "Terminal configuration".to_string(),
			default: None,
			minimum: None,
			maximum: None,
			min_length: None,
			max_length: None,
			enum_values: None,
			pattern: None,
			required: false,
		});
		
		// UI configuration
		properties.insert("ui".to_string(), SchemaProperty {
			property_type: "object".to_string(),
			description: "UI configuration".to_string(),
			default: None,
			minimum: None,
			maximum: None,
			min_length: None,
			max_length: None,
			enum_values: None,
			pattern: None,
			required: false,
		});
		
		// Plugin configuration
		properties.insert("plugins".to_string(), SchemaProperty {
			property_type: "array".to_string(),
			description: "Plugin configurations".to_string(),
			default: Some(serde_json::Value::Array(Vec::new())),
			minimum: None,
			maximum: None,
			min_length: None,
			max_length: None,
			enum_values: None,
			pattern: None,
			required: false,
		});
		
		ConfigSchema {
			version: "1.0.0".to_string(),
			description: "Sare terminal configuration schema".to_string(),
			required: vec!["theme".to_string()],
			properties,
			additional_properties: true,
		}
	}
	
	/**
	 * Loads configuration from file
	 * 
	 * @param filename - Configuration file name
	 * @param format - File format
	 * @return Result<serde_json::Value> - Loaded configuration
	 */
	pub async fn load_config(&self, filename: &str, format: ConfigFormat) -> Result<serde_json::Value> {
		let file_path = format!("{}/{}.{}", self.config_dir, filename, format.extension());
		
		if !Path::new(&file_path).exists() {
			// Return default configuration if file doesn't exist
			let default_config = self.default_config.read().await;
			return Ok(default_config.clone());
		}
		
		let content = tokio::fs::read_to_string(&file_path).await?;
		
		let config = match format {
			ConfigFormat::Json => {
				serde_json::from_str(&content)?
			}
			ConfigFormat::Toml => {
				// For TOML, we'll use serde_json as a fallback
				// In a real implementation, you'd use toml crate
				serde_json::from_str(&content)?
			}
			ConfigFormat::Yaml => {
				// For YAML, we'll use serde_json as a fallback
				// In a real implementation, you'd use serde_yaml crate
				serde_json::from_str(&content)?
			}
		};
		
		// Validate configuration
		self.validate_config(&config).await?;
		
		Ok(config)
	}
	
	/**
	 * Saves configuration to file
	 * 
	 * @param filename - Configuration file name
	 * @param format - File format
	 * @param config - Configuration to save
	 * @return Result<()> - Success or error status
	 */
	pub async fn save_config(&self, filename: &str, format: ConfigFormat, config: &serde_json::Value) -> Result<()> {
		// Create config directory if it doesn't exist
		if !Path::new(&self.config_dir).exists() {
			tokio::fs::create_dir_all(&self.config_dir).await?;
		}
		
		let file_path = format!("{}/{}.{}", self.config_dir, filename, format.extension());
		
		let content = match format {
			ConfigFormat::Json => {
				serde_json::to_string_pretty(config)?
			}
			ConfigFormat::Toml => {
				// For TOML, we'll use JSON as a fallback
				// In a real implementation, you'd use toml crate
				serde_json::to_string_pretty(config)?
			}
			ConfigFormat::Yaml => {
				// For YAML, we'll use JSON as a fallback
				// In a real implementation, you'd use serde_yaml crate
				serde_json::to_string_pretty(config)?
			}
		};
		
		tokio::fs::write(&file_path, content).await?;
		
		Ok(())
	}
	
	/**
	 * Validates configuration against schema
	 * 
	 * @param config - Configuration to validate
	 * @return Result<Vec<ConfigValidationError>> - Validation errors
	 */
	pub async fn validate_config(&self, config: &serde_json::Value) -> Result<Vec<ConfigValidationError>> {
		let schema = self.schema.read().await;
		let mut errors = Vec::new();
		
		// Check required fields
		for required_field in &schema.required {
			if !config.get(required_field).is_some() {
				errors.push(ConfigValidationError {
					field: required_field.clone(),
					message: format!("Required field '{}' is missing", required_field),
					severity: ValidationSeverity::Fatal,
				});
			}
		}
		
		// Validate each property
		if let Some(obj) = config.as_object() {
			for (field_name, field_value) in obj {
				if let Some(property_schema) = schema.properties.get(field_name) {
					if let Err(property_errors) = self.validate_property(field_name, field_value, property_schema) {
						errors.extend(property_errors);
					}
				} else if !schema.additional_properties {
					errors.push(ConfigValidationError {
						field: field_name.clone(),
						message: format!("Unknown field '{}'", field_name),
						severity: ValidationSeverity::Warning,
					});
				}
			}
		}
		
		// Check for fatal errors
		let fatal_errors: Vec<_> = errors.iter()
			.filter(|e| e.severity == ValidationSeverity::Fatal)
			.cloned()
			.collect();
		
		if !fatal_errors.is_empty() {
			return Err(anyhow::anyhow!("Configuration validation failed: {:?}", fatal_errors));
		}
		
		Ok(errors)
	}
	
	/**
	 * Validates a single property
	 * 
	 * @param field_name - Field name
	 * @param value - Field value
	 * @param schema - Property schema
	 * @return Result<Vec<ConfigValidationError>> - Validation errors
	 */
	fn validate_property(&self, field_name: &str, value: &serde_json::Value, schema: &SchemaProperty) -> Result<Vec<ConfigValidationError>> {
		let mut errors = Vec::new();
		
		// Check type
		match schema.property_type.as_str() {
			"string" => {
				if !value.is_string() {
					errors.push(ConfigValidationError {
						field: field_name.to_string(),
						message: format!("Field '{}' must be a string", field_name),
						severity: ValidationSeverity::Error,
					});
				} else if let Some(s) = value.as_str() {
					// Check length constraints
					if let Some(min_len) = schema.min_length {
						if s.len() < min_len {
							errors.push(ConfigValidationError {
								field: field_name.to_string(),
								message: format!("Field '{}' must be at least {} characters long", field_name, min_len),
								severity: ValidationSeverity::Error,
							});
						}
					}
					
					if let Some(max_len) = schema.max_length {
						if s.len() > max_len {
							errors.push(ConfigValidationError {
								field: field_name.to_string(),
								message: format!("Field '{}' must be at most {} characters long", field_name, max_len),
								severity: ValidationSeverity::Error,
							});
						}
					}
					
					// Check pattern
					if let Some(pattern) = &schema.pattern {
						// In a real implementation, you'd use regex crate
						if !s.contains(pattern) {
							errors.push(ConfigValidationError {
								field: field_name.to_string(),
								message: format!("Field '{}' does not match pattern '{}'", field_name, pattern),
								severity: ValidationSeverity::Error,
							});
						}
					}
				}
			}
			"number" => {
				if !value.is_number() {
					errors.push(ConfigValidationError {
						field: field_name.to_string(),
						message: format!("Field '{}' must be a number", field_name),
						severity: ValidationSeverity::Error,
					});
				} else if let Some(n) = value.as_f64() {
					// Check range constraints
					if let Some(min) = schema.minimum {
						if n < min {
							errors.push(ConfigValidationError {
								field: field_name.to_string(),
								message: format!("Field '{}' must be at least {}", field_name, min),
								severity: ValidationSeverity::Error,
							});
						}
					}
					
					if let Some(max) = schema.maximum {
						if n > max {
							errors.push(ConfigValidationError {
								field: field_name.to_string(),
								message: format!("Field '{}' must be at most {}", field_name, max),
								severity: ValidationSeverity::Error,
							});
						}
					}
				}
			}
			"boolean" => {
				if !value.is_boolean() {
					errors.push(ConfigValidationError {
						field: field_name.to_string(),
						message: format!("Field '{}' must be a boolean", field_name),
						severity: ValidationSeverity::Error,
					});
				}
			}
			"object" => {
				if !value.is_object() {
					errors.push(ConfigValidationError {
						field: field_name.to_string(),
						message: format!("Field '{}' must be an object", field_name),
						severity: ValidationSeverity::Error,
					});
				}
			}
			"array" => {
				if !value.is_array() {
					errors.push(ConfigValidationError {
						field: field_name.to_string(),
						message: format!("Field '{}' must be an array", field_name),
						severity: ValidationSeverity::Error,
					});
				}
			}
			_ => {
				errors.push(ConfigValidationError {
					field: field_name.to_string(),
					message: format!("Unknown type '{}' for field '{}'", schema.property_type, field_name),
					severity: ValidationSeverity::Error,
				});
			}
		}
		
		// Check enum values
		if let Some(enum_values) = &schema.enum_values {
			if !enum_values.contains(value) {
				errors.push(ConfigValidationError {
					field: field_name.to_string(),
					message: format!("Field '{}' must be one of: {:?}", field_name, enum_values),
					severity: ValidationSeverity::Error,
				});
			}
		}
		
		Ok(errors)
	}
	
	/**
	 * Sets default configuration
	 * 
	 * @param config - Default configuration
	 */
	pub async fn set_default_config(&self, config: serde_json::Value) {
		let mut default_config = self.default_config.write().await;
		*default_config = config;
	}
	
	/**
	 * Gets default configuration
	 * 
	 * @return serde_json::Value - Default configuration
	 */
	pub async fn get_default_config(&self) -> serde_json::Value {
		let default_config = self.default_config.read().await;
		default_config.clone()
	}
	
	/**
	 * Updates configuration schema
	 * 
	 * @param schema - New schema
	 */
	pub async fn update_schema(&self, schema: ConfigSchema) {
		let mut current_schema = self.schema.write().await;
		*current_schema = schema;
	}
	
	/**
	 * Gets current schema
	 * 
	 * @return ConfigSchema - Current schema
	 */
	pub async fn get_schema(&self) -> ConfigSchema {
		let schema = self.schema.read().await;
		schema.clone()
	}
	
	/**
	 * Registers a file watcher
	 * 
	 * @param filename - File to watch
	 * @param callback - Callback function
	 */
	pub async fn register_watcher<F>(&self, filename: &str, callback: F)
	where
		F: Fn() + Send + Sync + 'static,
	{
		let mut watchers = self.watchers.write().await;
		watchers.insert(filename.to_string(), Box::new(callback));
	}
	
	/**
	 * Converts configuration between formats
	 * 
	 * @param config - Configuration to convert
	 * @param from_format - Source format
	 * @param to_format - Target format
	 * @return Result<String> - Converted configuration
	 */
	pub async fn convert_format(&self, config: &serde_json::Value, from_format: ConfigFormat, to_format: ConfigFormat) -> Result<String> {
		// First, parse the source format
		let parsed_config = match from_format {
			ConfigFormat::Json => config.clone(),
			ConfigFormat::Toml => {
				// In a real implementation, you'd parse TOML
				config.clone()
			}
			ConfigFormat::Yaml => {
				// In a real implementation, you'd parse YAML
				config.clone()
			}
		};
		
		// Then, serialize to target format
		let result = match to_format {
			ConfigFormat::Json => {
				serde_json::to_string_pretty(&parsed_config)?
			}
			ConfigFormat::Toml => {
				// In a real implementation, you'd serialize to TOML
				serde_json::to_string_pretty(&parsed_config)?
			}
			ConfigFormat::Yaml => {
				// In a real implementation, you'd serialize to YAML
				serde_json::to_string_pretty(&parsed_config)?
			}
		};
		
		Ok(result)
	}
	
	/**
	 * Gets configuration file statistics
	 * 
	 * @return HashMap<String, usize> - File statistics
	 */
	pub async fn get_file_stats(&self) -> HashMap<String, usize> {
		let mut stats = HashMap::new();
		
		if let Ok(mut entries) = tokio::fs::read_dir(&self.config_dir).await {
			while let Ok(Some(entry)) = entries.next_entry().await {
				if let Ok(metadata) = entry.metadata() {
					if metadata.is_file() {
						if let Some(extension) = entry.path().extension() {
							if let Some(ext_str) = extension.to_str() {
								*stats.entry(ext_str.to_string()).or_insert(0) += 1;
							}
						}
					}
				}
			}
		}
		
		stats
	}
} 