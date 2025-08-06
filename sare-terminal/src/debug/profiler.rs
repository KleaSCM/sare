/**
 * Performance profiler for Sare terminal
 * 
 * This module provides performance profiling and profiling tools
 * for monitoring terminal performance and identifying bottlenecks.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: profiler.rs
 * Description: Performance profiling and profiling tools
 */

use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;

/**
 * Profiler configuration
 * 
 * プロファイラー設定です。
 * プロファイリング機能の設定を
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
	/// Enable profiling
	pub enable_profiling: bool,
	/// Profiling interval in milliseconds
	pub profiling_interval: u64,
	/// Enable CPU profiling
	pub enable_cpu_profiling: bool,
	/// Enable memory profiling
	pub enable_memory_profiling: bool,
	/// Enable I/O profiling
	pub enable_io_profiling: bool,
	/// Enable render profiling
	pub enable_render_profiling: bool,
	/// Profiling output format
	pub output_format: ProfilerOutputFormat,
}

impl Default for ProfilerConfig {
	fn default() -> Self {
		Self {
			enable_profiling: false,
			profiling_interval: 1000, // 1 second
			enable_cpu_profiling: true,
			enable_memory_profiling: true,
			enable_io_profiling: true,
			enable_render_profiling: true,
			output_format: ProfilerOutputFormat::Text,
		}
	}
}

/**
 * Profiler output format
 * 
 * プロファイラー出力形式です。
 * プロファイリング結果の
 * 出力形式を定義します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProfilerOutputFormat {
	/// Text output
	Text,
	/// JSON output
	Json,
	/// CSV output
	Csv,
	/// HTML output
	Html,
}

/**
 * Profiler sample
 * 
 * プロファイラーサンプルです。
 * 個別のプロファイリング
 * サンプルを管理します。
 */
#[derive(Debug, Clone)]
pub struct ProfilerSample {
	/// Sample timestamp
	pub timestamp: Instant,
	/// Sample duration
	pub duration: Duration,
	/// Sample function name
	pub function_name: String,
	/// Sample module name
	pub module_name: String,
	/// Sample CPU usage
	pub cpu_usage: f64,
	/// Sample memory usage
	pub memory_usage: u64,
	/// Sample I/O operations
	pub io_operations: u64,
	/// Sample render operations
	pub render_operations: u64,
}

/**
 * Profiler metrics
 * 
 * プロファイラーメトリクスです。
 * プロファイリング結果の
 * メトリクスを管理します。
 */
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfilerMetrics {
	/// Total samples
	pub total_samples: u64,
	/// Average CPU usage
	pub avg_cpu_usage: f64,
	/// Peak CPU usage
	pub peak_cpu_usage: f64,
	/// Average memory usage
	pub avg_memory_usage: u64,
	/// Peak memory usage
	pub peak_memory_usage: u64,
	/// Total I/O operations
	pub total_io_operations: u64,
	/// Total render operations
	pub total_render_operations: u64,
	/// Function call counts
	pub function_calls: HashMap<String, u64>,
	/// Module call counts
	pub module_calls: HashMap<String, u64>,
}

/**
 * Performance profiler
 * 
 * パフォーマンスプロファイラーです。
 * ターミナルのパフォーマンスを
 * 監視し、ボトルネックを特定します。
 */
pub struct Profiler {
	/// Profiler configuration
	config: ProfilerConfig,
	/// Profiler samples
	samples: Arc<RwLock<Vec<ProfilerSample>>>,
	/// Profiler metrics
	metrics: Arc<RwLock<ProfilerMetrics>>,
	/// Profiler start time
	start_time: Instant,
	/// Last profiling time
	last_profiling: Instant,
	/// Active profiling sessions
	active_sessions: Arc<RwLock<HashMap<String, Instant>>>,
}

impl Profiler {
	/**
	 * Creates a new profiler
	 * 
	 * @param config - Profiler configuration
	 * @return Profiler - New profiler
	 */
	pub fn new(config: ProfilerConfig) -> Self {
		Self {
			config,
			samples: Arc::new(RwLock::new(Vec::new())),
			metrics: Arc::new(RwLock::new(ProfilerMetrics {
				total_samples: 0,
				avg_cpu_usage: 0.0,
				peak_cpu_usage: 0.0,
				avg_memory_usage: 0,
				peak_memory_usage: 0,
				total_io_operations: 0,
				total_render_operations: 0,
				function_calls: HashMap::new(),
				module_calls: HashMap::new(),
			})),
			start_time: Instant::now(),
			last_profiling: Instant::now(),
			active_sessions: Arc::new(RwLock::new(HashMap::new())),
		}
	}
	
	/**
	 * Initializes the profiler
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		if self.config.enable_profiling {
			println!("📊 Performance profiler enabled");
			println!("⏱️ Profiling interval: {}ms", self.config.profiling_interval);
			println!("🖥️ CPU profiling: {}", self.config.enable_cpu_profiling);
			println!("💾 Memory profiling: {}", self.config.enable_memory_profiling);
			println!("📡 I/O profiling: {}", self.config.enable_io_profiling);
			println!("🎨 Render profiling: {}", self.config.enable_render_profiling);
		}
		
		Ok(())
	}
	
	/**
	 * Starts a profiling session
	 * 
	 * @param session_name - Session name
	 * @return Result<()> - Success or error status
	 */
	pub async fn start_session(&self, session_name: String) -> Result<()> {
		let mut sessions = self.active_sessions.write().await;
		sessions.insert(session_name.clone(), Instant::now());
		
		println!("📊 Started profiling session: {}", session_name);
		
		Ok(())
	}
	
	/**
	 * Ends a profiling session
	 * 
	 * @param session_name - Session name
	 * @return Result<Duration> - Session duration
	 */
	pub async fn end_session(&self, session_name: &str) -> Result<Duration> {
		let mut sessions = self.active_sessions.write().await;
		
		if let Some(start_time) = sessions.remove(session_name) {
			let duration = start_time.elapsed();
			println!("📊 Ended profiling session: {} (Duration: {:?})", session_name, duration);
			Ok(duration)
		} else {
			Err(anyhow::anyhow!("Session not found: {}", session_name))
		}
	}
	
	/**
	 * Records a profiling sample
	 * 
	 * @param function_name - Function name
	 * @param module_name - Module name
	 * @param duration - Function duration
	 * @return Result<()> - Success or error status
	 */
	pub async fn record_sample(&self, function_name: String, module_name: String, duration: Duration) -> Result<()> {
		if !self.config.enable_profiling {
			return Ok(());
		}
		
		let cpu_usage = self.get_cpu_usage().await?;
		let memory_usage = self.get_memory_usage().await?;
		let io_operations = self.get_io_operations().await?;
		let render_operations = self.get_render_operations().await?;
		
		let sample = ProfilerSample {
			timestamp: Instant::now(),
			duration,
			function_name,
			module_name,
			cpu_usage,
			memory_usage,
			io_operations,
			render_operations,
		};
		
		// Add sample
		let mut samples = self.samples.write().await;
		samples.push(sample.clone());
		
		// Update metrics
		let mut metrics = self.metrics.write().await;
		metrics.total_samples += 1;
		
		// Update function call counts
		*metrics.function_calls.entry(sample.function_name.clone()).or_insert(0) += 1;
		*metrics.module_calls.entry(sample.module_name.clone()).or_insert(0) += 1;
		
		// Update averages
		let total_samples = metrics.total_samples as f64;
		metrics.avg_cpu_usage = (metrics.avg_cpu_usage * (total_samples - 1.0) + cpu_usage) / total_samples;
		metrics.avg_memory_usage = (metrics.avg_memory_usage * (metrics.total_samples - 1) + memory_usage) / metrics.total_samples;
		
		// Update peaks
		if cpu_usage > metrics.peak_cpu_usage {
			metrics.peak_cpu_usage = cpu_usage;
		}
		if memory_usage > metrics.peak_memory_usage {
			metrics.peak_memory_usage = memory_usage;
		}
		
		// Update totals
		metrics.total_io_operations += io_operations;
		metrics.total_render_operations += render_operations;
		
		Ok(())
	}
	
	/**
	 * Gets CPU usage
	 * 
	 * @return Result<f64> - CPU usage percentage
	 */
	async fn get_cpu_usage(&self) -> Result<f64> {
		if !self.config.enable_cpu_profiling {
			return Ok(0.0);
		}
		
		// Read CPU stats from /proc/stat
		let stat_content = std::fs::read_to_string("/proc/stat")?;
		let first_line = stat_content.lines().next().ok_or_else(|| anyhow::anyhow!("No CPU stats found"))?;
		
		// Parse CPU time values
		let parts: Vec<u64> = first_line
			.split_whitespace()
			.skip(1) // Skip "cpu" identifier
			.filter_map(|s| s.parse::<u64>().ok())
			.collect();
		
		if parts.len() < 4 {
			return Ok(0.0);
		}
		
		// Calculate CPU usage
		let total = parts.iter().sum::<u64>();
		let idle = parts[3];
		let usage = if total > 0 {
			((total - idle) * 100) / total
		} else {
			0
		};
		
		Ok(usage as f64)
	}
	
	/**
	 * Gets memory usage
	 * 
	 * @return Result<u64> - Memory usage in bytes
	 */
	async fn get_memory_usage(&self) -> Result<u64> {
		if !self.config.enable_memory_profiling {
			return Ok(0);
		}
		
		// Read memory info from /proc/meminfo
		let meminfo_content = std::fs::read_to_string("/proc/meminfo")?;
		let mut total_mem = 0u64;
		let mut available_mem = 0u64;
		
		for line in meminfo_content.lines() {
			if line.starts_with("MemTotal:") {
				total_mem = line
					.split_whitespace()
					.nth(1)
					.and_then(|s| s.parse::<u64>().ok())
					.unwrap_or(0) * 1024; // Convert KB to bytes
			} else if line.starts_with("MemAvailable:") {
				available_mem = line
					.split_whitespace()
					.nth(1)
					.and_then(|s| s.parse::<u64>().ok())
					.unwrap_or(0) * 1024; // Convert KB to bytes
			}
		}
		
		Ok(total_mem - available_mem)
	}
	
	/**
	 * Gets I/O operations count
	 * 
	 * @return Result<u64> - I/O operations count
	 */
	async fn get_io_operations(&self) -> Result<u64> {
		if !self.config.enable_io_profiling {
			return Ok(0);
		}
		
		// Read I/O stats from /proc/diskstats
		let diskstats_content = std::fs::read_to_string("/proc/diskstats")?;
		let mut total_io = 0u64;
		
		for line in diskstats_content.lines() {
			let parts: Vec<&str> = line.split_whitespace().collect();
			if parts.len() >= 14 {
				if let (Ok(reads), Ok(writes)) = (parts[3].parse::<u64>(), parts[7].parse::<u64>()) {
					total_io += reads + writes;
				}
			}
		}
		
		Ok(total_io)
	}
	
	/**
	 * Gets render operations count
	 * 
	 * @return Result<u64> - Render operations count
	 */
	async fn get_render_operations(&self) -> Result<u64> {
		if !self.config.enable_render_profiling {
			return Ok(0);
		}
		
		// For now, return a placeholder value
		// In a real implementation, this would track actual render operations
		Ok(0)
	}
	
	/**
	 * Generates profiling report
	 * 
	 * @return Result<String> - Profiling report
	 */
	pub async fn generate_report(&self) -> Result<String> {
		let metrics = self.metrics.read().await;
		let samples = self.samples.read().await;
		
		let mut report = String::new();
		
		match self.config.output_format {
			ProfilerOutputFormat::Text => {
				report.push_str("📊 Performance Profiling Report\n");
				report.push_str("================================\n\n");
				
				report.push_str(&format!("Total Samples: {}\n", metrics.total_samples));
				report.push_str(&format!("Average CPU Usage: {:.2}%\n", metrics.avg_cpu_usage));
				report.push_str(&format!("Peak CPU Usage: {:.2}%\n", metrics.peak_cpu_usage));
				report.push_str(&format!("Average Memory Usage: {:.2} MB\n", metrics.avg_memory_usage as f64 / 1024.0 / 1024.0));
				report.push_str(&format!("Peak Memory Usage: {:.2} MB\n", metrics.peak_memory_usage as f64 / 1024.0 / 1024.0));
				report.push_str(&format!("Total I/O Operations: {}\n", metrics.total_io_operations));
				report.push_str(&format!("Total Render Operations: {}\n", metrics.total_render_operations));
				
				report.push_str("\nFunction Call Counts:\n");
				for (function, count) in &metrics.function_calls {
					report.push_str(&format!("  {}: {}\n", function, count));
				}
				
				report.push_str("\nModule Call Counts:\n");
				for (module, count) in &metrics.module_calls {
					report.push_str(&format!("  {}: {}\n", module, count));
				}
			}
			ProfilerOutputFormat::Json => {
				report.push_str(&serde_json::to_string_pretty(&*metrics)?);
			}
			ProfilerOutputFormat::Csv => {
				report.push_str("Function,Module,CPU,Memory,IO,Render\n");
				for sample in samples.iter() {
					report.push_str(&format!("{},{},{:.2},{},{},{}\n",
						sample.function_name,
						sample.module_name,
						sample.cpu_usage,
						sample.memory_usage,
						sample.io_operations,
						sample.render_operations
					));
				}
			}
			ProfilerOutputFormat::Html => {
				report.push_str("<html><head><title>Performance Report</title></head><body>");
				report.push_str("<h1>Performance Profiling Report</h1>");
				report.push_str(&format!("<p>Total Samples: {}</p>", metrics.total_samples));
				report.push_str(&format!("<p>Average CPU Usage: {:.2}%</p>", metrics.avg_cpu_usage));
				report.push_str("</body></html>");
			}
		}
		
		Ok(report)
	}
	
	/**
	 * Clears profiling data
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn clear_data(&self) -> Result<()> {
		let mut samples = self.samples.write().await;
		let mut metrics = self.metrics.write().await;
		
		samples.clear();
		*metrics = ProfilerMetrics {
			total_samples: 0,
			avg_cpu_usage: 0.0,
			peak_cpu_usage: 0.0,
			avg_memory_usage: 0,
			peak_memory_usage: 0,
			total_io_operations: 0,
			total_render_operations: 0,
			function_calls: HashMap::new(),
			module_calls: HashMap::new(),
		};
		
		println!("📊 Profiling data cleared");
		
		Ok(())
	}
	
	/**
	 * Gets profiler configuration
	 * 
	 * @return &ProfilerConfig - Profiler configuration
	 */
	pub fn config(&self) -> &ProfilerConfig {
		&self.config
	}
	
	/**
	 * Updates profiler configuration
	 * 
	 * @param config - New profiler configuration
	 */
	pub fn update_config(&mut self, config: ProfilerConfig) {
		self.config = config;
	}
} 