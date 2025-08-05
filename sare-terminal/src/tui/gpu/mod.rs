/**
 * @file mod.rs
 * @brief GPU acceleration module for Sare terminal
 * 
 * This module provides GPU-accelerated rendering capabilities for the Sare terminal,
 * enabling hardware-accelerated text rendering, smooth scrolling, and high-performance
 * graphics operations. Supports multiple GPU backends with automatic fallback.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file mod.rs
 * @description GPU acceleration module that provides hardware-accelerated rendering
 * for the Sare terminal with support for multiple backends and automatic fallback.
 */

pub mod skia_backend;
pub mod wgpu_backend;
pub mod renderer;
pub mod text;
pub mod fonts;
pub mod advanced_renderer;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/**
 * GPU rendering backend types
 * 
 * Defines the available GPU rendering backends with their capabilities
 * and performance characteristics.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum GpuBackend {
	/// Skia GPU backend (like Kitty)
	Skia,
	/// WGPU backend for cross-platform GPU rendering
	Wgpu,
	/// CPU fallback rendering
	Cpu,
}

/**
 * GPU capability information
 * 
 * Contains information about the available GPU capabilities,
 * performance metrics, and supported features.
 */
#[derive(Debug, Clone)]
pub struct GpuCapabilities {
	/// Available GPU backends
	pub available_backends: Vec<GpuBackend>,
	/// Maximum texture size supported
	pub max_texture_size: u32,
	/// Whether hardware acceleration is available
	pub hardware_acceleration: bool,
	/// GPU memory in bytes
	pub gpu_memory: u64,
	/// Supported rendering APIs
	pub supported_apis: Vec<String>,
}

/**
 * GPU renderer configuration
 * 
 * Configuration options for GPU rendering including performance
 * settings, quality options, and backend preferences.
 */
#[derive(Debug, Clone)]
pub struct GpuConfig {
	/// Preferred GPU backend
	pub preferred_backend: GpuBackend,
	/// Target frame rate (default: 60fps)
	pub target_fps: u32,
	/// Enable hardware acceleration
	pub hardware_acceleration: bool,
	/// GPU memory pool size in bytes
	pub memory_pool_size: u64,
	/// Enable texture compression
	pub texture_compression: bool,
	/// Enable subpixel antialiasing
	pub subpixel_antialiasing: bool,
}

impl Default for GpuConfig {
	fn default() -> Self {
		Self {
			preferred_backend: GpuBackend::Skia,
			target_fps: 60,
			hardware_acceleration: true,
			memory_pool_size: 256 * 1024 * 1024, // 256MB
			texture_compression: true,
			subpixel_antialiasing: true,
		}
	}
}

/**
 * GPU renderer manager
 * 
 * Manages GPU rendering operations, backend selection, and performance
 * optimization for the Sare terminal.
 */
pub struct GpuRenderer {
	/// Current GPU configuration
	config: GpuConfig,
	/// Available GPU capabilities
	capabilities: GpuCapabilities,
	/// Current active backend
	active_backend: Option<GpuBackend>,
	/// Performance metrics
	performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

/**
 * GPU performance metrics
 * 
 * Tracks rendering performance including frame times, memory usage,
 * and efficiency metrics for optimization.
 */
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
	/// Average frame time in milliseconds
	pub average_frame_time: f64,
	/// Current frame rate
	pub current_fps: f64,
	/// GPU memory usage in bytes
	pub gpu_memory_usage: u64,
	/// CPU memory usage in bytes
	pub cpu_memory_usage: u64,
	/// Number of rendered frames
	pub frames_rendered: u64,
	/// Number of dropped frames
	pub frames_dropped: u64,
}

impl Default for PerformanceMetrics {
	fn default() -> Self {
		Self {
			average_frame_time: 0.0,
			current_fps: 0.0,
			gpu_memory_usage: 0,
			cpu_memory_usage: 0,
			frames_rendered: 0,
			frames_dropped: 0,
		}
	}
}

impl GpuRenderer {
	/**
	 * Creates a new GPU renderer instance
	 * 
	 * Initializes the GPU renderer with the specified configuration,
	 * detects available hardware capabilities, and selects the optimal
	 * rendering backend.
	 * 
	 * @param config - GPU configuration options
	 * @return Result<GpuRenderer> - New GPU renderer instance or error
	 */
	pub fn new(config: GpuConfig) -> Result<Self> {
		/**
		 * GPUレンダラーを初期化する関数です
		 * 
		 * システムのGPU能力を検出し、最適なバックエンドを選択して
		 * GPU加速レンダリングを初期化します。
		 * 
		 * detect_gpu_capabilities()で利用可能なバックエンドを検出し、
		 * select_optimal_backend()で設定に基づいて最適なバックエンドを
		 * 選択します。パフォーマンスメトリクスも初期化されます。
		 */
		let capabilities = Self::detect_gpu_capabilities()?;
		let active_backend = Self::select_optimal_backend(&config, &capabilities)?;
		
		Ok(Self {
			config,
			capabilities,
			active_backend,
			performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
		})
	}
	
	/**
	 * Detects available GPU capabilities
	 * 
	 * Scans the system for available GPU hardware, supported APIs,
	 * and performance characteristics to determine optimal rendering options.
	 * 
	 * @return Result<GpuCapabilities> - Detected GPU capabilities or error
	 */
	fn detect_gpu_capabilities() -> Result<GpuCapabilities> {
		/**
		 * GPU能力を検出する関数です
		 * 
		 * システムで利用可能なGPUバックエンド（Skia、WGPU）を検出し、
		 * サポートされているレンダリングAPIとハードウェア能力を
		 * 判定します。
		 * 
		 * is_skia_available()とis_wgpu_available()で各バックエンドの
		 * 可用性をチェックし、CPUフォールバックを含む利用可能な
		 * レンダリングオプションを返します。
		 */
		let mut available_backends = Vec::new();
		let mut supported_apis = Vec::new();
		
		// Detect Skia backend
		if Self::is_skia_available() {
			available_backends.push(GpuBackend::Skia);
			supported_apis.push("Skia".to_string());
		}
		
		// Detect WGPU backend
		if Self::is_wgpu_available() {
			available_backends.push(GpuBackend::Wgpu);
			supported_apis.push("WGPU".to_string());
		}
		
		// Always fallback to CPU
		available_backends.push(GpuBackend::Cpu);
		supported_apis.push("CPU".to_string());
		
		let hardware_acceleration = !available_backends.is_empty();
		
		Ok(GpuCapabilities {
			available_backends,
			max_texture_size: Self::detect_max_texture_size(),
			hardware_acceleration,
			gpu_memory: Self::detect_gpu_memory(),
			supported_apis,
		})
	}
	
	/**
	 * Selects the optimal GPU backend
	 * 
	 * Chooses the best available GPU backend based on configuration
	 * preferences and system capabilities.
	 * 
	 * @param config - GPU configuration
	 * @param capabilities - Available GPU capabilities
	 * @return Result<Option<GpuBackend>> - Selected backend or error
	 */
	fn select_optimal_backend(config: &GpuConfig, capabilities: &GpuCapabilities) -> Result<Option<GpuBackend>> {
		/**
		 * 最適なGPUバックエンドを選択する関数です
		 * 
		 * 設定で指定された優先バックエンドが利用可能かチェックし、
		 * 利用できない場合は利用可能なバックエンドから最適なものを
		 * 選択します。
		 * 
		 * 優先バックエンドが利用可能な場合はそれを選択し、そうでない
		 * 場合はSkia、WGPU、CPUの順でフォールバックします。
		 * 最終的にCPUフォールバックを保証します。
		 */
		
		// Check if preferred backend is available
		if capabilities.available_backends.contains(&config.preferred_backend) {
			return Ok(Some(config.preferred_backend.clone()));
		}
		
		// Fallback to best available backend
		for backend in &capabilities.available_backends {
			match backend {
				GpuBackend::Skia => return Ok(Some(GpuBackend::Skia)),
				GpuBackend::Wgpu => return Ok(Some(GpuBackend::Wgpu)),
				GpuBackend::Cpu => return Ok(Some(GpuBackend::Cpu)),
			}
		}
		
		// Final fallback to CPU
		Ok(Some(GpuBackend::Cpu))
	}
	
	/**
	 * Checks if Skia backend is available
	 * 
	 * @return bool - True if Skia is available
	 */
	fn is_skia_available() -> bool {
		// Implement Skia availability detection with actual library checks
		unsafe {
			// Check if Skia libraries are available
			use std::ffi::CString;
			
			// Try to load Skia libraries
			let lib_names = [
				"libskia.so",
				"libskia.so.1",
				"libskia.so.2",
				"libskia.dll",
				"libskia.dylib"
			];
			
			for lib_name in &lib_names {
				let lib_cstr = match CString::new(*lib_name) {
					Ok(s) => s,
					Err(_) => continue,
				};
				
				// Try to open the library
				let handle = libc::dlopen(lib_cstr.as_ptr(), libc::RTLD_NOW);
				if !handle.is_null() {
					libc::dlclose(handle);
					return true;
				}
			}
			
			// Also check if Skia is available via pkg-config
			if std::process::Command::new("pkg-config")
				.args(&["--exists", "skia"])
				.output()
				.is_ok() {
				return true;
			}
		}
		
		false
	}
	
	/**
	 * Checks if WGPU backend is available
	 * 
	 * @return bool - True if WGPU is available
	 */
	fn is_wgpu_available() -> bool {
		// Implement WGPU availability detection with actual GPU checks
		unsafe {
			// Check for Vulkan support (WGPU can use Vulkan)
			let vulkan_libs = [
				"libvulkan.so",
				"libvulkan.so.1",
				"vulkan-1.dll",
				"libvulkan.dylib"
			];
			
			for lib_name in &vulkan_libs {
				let lib_cstr = match std::ffi::CString::new(*lib_name) {
					Ok(s) => s,
					Err(_) => continue,
				};
				
				let handle = libc::dlopen(lib_cstr.as_ptr(), libc::RTLD_NOW);
				if !handle.is_null() {
					libc::dlclose(handle);
					return true;
				}
			}
			
			// Check for OpenGL support (WGPU can use OpenGL)
			let opengl_libs = [
				"libGL.so",
				"libGL.so.1",
				"opengl32.dll",
				"libGL.dylib"
			];
			
			for lib_name in &opengl_libs {
				let lib_cstr = match std::ffi::CString::new(*lib_name) {
					Ok(s) => s,
					Err(_) => continue,
				};
				
				let handle = libc::dlopen(lib_cstr.as_ptr(), libc::RTLD_NOW);
				if !handle.is_null() {
					libc::dlclose(handle);
					return true;
				}
			}
			
			// Check for Metal support (macOS)
			#[cfg(target_os = "macos")]
			{
				let metal_lib = std::ffi::CString::new("libMetal.dylib").unwrap();
				let handle = libc::dlopen(metal_lib.as_ptr(), libc::RTLD_NOW);
				if !handle.is_null() {
					libc::dlclose(handle);
					return true;
				}
			}
		}
		
		false
	}
	
	/**
	 * Detects maximum texture size supported by GPU
	 * 
	 * @return u32 - Maximum texture size in pixels
	 */
	fn detect_max_texture_size() -> u32 {
		// Try to get actual texture size from OpenGL using glxinfo
		if let Ok(output) = std::process::Command::new("glxinfo")
			.args(&["-B"])
			.output() {
			if let Ok(output_str) = String::from_utf8(output.stdout) {
				for line in output_str.lines() {
					if line.starts_with("Maximum texture size:") {
						if let Some(size_str) = line.split(':').nth(1) {
							if let Ok(size) = size_str.trim().parse::<u32>() {
								return size;
							}
						}
					}
				}
			}
		}
		
		// Try to get from GPU info files on Linux with actual parsing
		if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
			for entry in entries {
				if let Ok(entry) = entry {
					if let Ok(device_name) = entry.file_name().into_string() {
						if device_name.starts_with("card") {
							// Try to read actual GPU capabilities from sysfs
							if let Ok(content) = std::fs::read_to_string(format!("/sys/class/drm/{}/device/gpu_bus_info", device_name)) {
								// Parse GPU info to determine actual texture size
								if content.contains("PCI") {
									// Parse PCI info to determine GPU type
									if let Some(pci_info) = content.lines().next() {
										if pci_info.contains("NVIDIA") {
											return 16384; // High-end NVIDIA GPU
										} else if pci_info.contains("AMD") {
											return 16384; // High-end AMD GPU
										} else if pci_info.contains("Intel") {
											return 8192; // Intel GPU
										}
									}
								}
							}
							
							// Try to read GPU driver info
							if let Ok(driver_content) = std::fs::read_to_string(format!("/sys/class/drm/{}/device/driver", device_name)) {
								if driver_content.contains("nvidia") {
									return 16384; // NVIDIA driver
								} else if driver_content.contains("amdgpu") {
									return 16384; // AMD driver
								} else if driver_content.contains("i915") || driver_content.contains("i965") {
									return 8192; // Intel driver
								}
							}
							
							// Try to read GPU memory info for texture size estimation
							if let Ok(memory_content) = std::fs::read_to_string(format!("/sys/class/drm/{}/device/mem_info_vram_total", device_name)) {
								if let Ok(memory_bytes) = memory_content.trim().parse::<u64>() {
									// Estimate texture size based on available GPU memory
									if memory_bytes >= 8 * 1024 * 1024 * 1024 { // 8GB+
										return 16384; // High-end GPU
									} else if memory_bytes >= 4 * 1024 * 1024 * 1024 { // 4GB+
										return 8192; // Mid-range GPU
									} else if memory_bytes >= 2 * 1024 * 1024 * 1024 { // 2GB+
										return 4096; // Low-end GPU
									}
								}
							}
						}
					}
				}
			}
		}
		
		// Try to get from GPU memory to estimate texture size
		let gpu_memory = detect_gpu_memory();
		if gpu_memory >= 8 * 1024 * 1024 * 1024 { // 8GB+
			return 16384; // High-end GPU
		} else if gpu_memory >= 4 * 1024 * 1024 * 1024 { // 4GB+
			return 8192; // Mid-range GPU
		} else if gpu_memory >= 2 * 1024 * 1024 * 1024 { // 2GB+
			return 4096; // Low-end GPU
		}
		
		// Fallback based on common GPU capabilities
		4096
	}
	
	/**
	 * Detects available GPU memory
	 * 
	 * @return u64 - Available GPU memory in bytes
	 */
	fn detect_gpu_memory() -> u64 {
		// Implement GPU memory detection with actual system queries
		unsafe {
			// Try to get GPU memory from NVIDIA tools with enhanced parsing
			if let Ok(output) = std::process::Command::new("nvidia-smi")
				.args(&["--query-gpu=memory.total,memory.used,memory.free", "--format=csv,noheader,nounits"])
				.output() {
				if let Ok(memory_str) = String::from_utf8(output.stdout) {
					if let Some(line) = memory_str.lines().next() {
						let parts: Vec<&str> = line.split(',').collect();
						if parts.len() >= 3 {
							if let Ok(total_mb) = parts[0].trim().parse::<u64>() {
								return total_mb * 1024 * 1024; // Convert MB to bytes
							}
						}
					}
				}
			}
			
			// Try to get from AMD tools with enhanced parsing
			if let Ok(output) = std::process::Command::new("rocm-smi")
				.args(&["--showproductname", "--showmeminfo", "vram", "--showmeminfo", "gtt"])
				.output() {
				if let Ok(output_str) = String::from_utf8(output.stdout) {
					// Parse AMD GPU memory info with multiple memory types
					for line in output_str.lines() {
						if line.contains("Total Memory") || line.contains("VRAM Total") {
							if let Some(memory_str) = line.split(':').nth(1) {
								if let Ok(memory_mb) = memory_str.trim().replace("MB", "").replace("GB", "").parse::<u64>() {
									return memory_mb * 1024 * 1024;
								}
							}
						}
					}
				}
			}
			
			// Try to get from Intel GPU tools
			if let Ok(output) = std::process::Command::new("intel_gpu_top")
				.args(&["-J"])
				.output() {
				if let Ok(output_str) = String::from_utf8(output.stdout) {
					// Parse Intel GPU memory info from JSON
					if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
						if let Some(engines) = json.get("engines") {
							if let Some(render) = engines.get("Render/3D/0") {
								if let Some(memory) = render.get("busy") {
									if let Some(memory_mb) = memory.as_u64() {
										return memory_mb * 1024 * 1024; // Estimate from busy time
									}
								}
							}
						}
					}
				}
			}
			
			// Try to get from Linux GPU info files with enhanced parsing
			if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
				for entry in entries {
					if let Ok(entry) = entry {
						if let Ok(device_name) = entry.file_name().into_string() {
							if device_name.starts_with("card") {
								// Try multiple memory info files
								let memory_files = vec![
									format!("/sys/class/drm/{}/device/mem_info_vram_total", device_name),
									format!("/sys/class/drm/{}/device/mem_info_vram_used", device_name),
									format!("/sys/class/drm/{}/device/mem_info_gtt_total", device_name),
									format!("/sys/class/drm/{}/device/mem_info_gtt_used", device_name),
								];
								
								for memory_file in memory_files {
									if let Ok(content) = std::fs::read_to_string(&memory_file) {
										if let Ok(memory_bytes) = content.trim().parse::<u64>() {
											// If it's a used memory file, get actual total from total file
											if memory_file.contains("used") {
												// Try to get total memory from corresponding total file
												let total_file = memory_file.replace("used", "total");
												if let Ok(total_content) = std::fs::read_to_string(&total_file) {
													if let Ok(total_bytes) = total_content.trim().parse::<u64>() {
														return total_bytes;
													}
												}
												// Fallback to estimation if total file not available
												return memory_bytes * 2;
											} else {
												return memory_bytes;
											}
										}
									}
								}
								
								// Try to read from GPU driver info with actual memory detection
								if let Ok(driver_content) = std::fs::read_to_string(format!("/sys/class/drm/{}/device/driver", device_name)) {
									if driver_content.contains("nvidia") {
										// Try to get actual NVIDIA memory using nvidia-smi
										if let Ok(output) = std::process::Command::new("nvidia-smi")
											.args(&["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
											.output() {
											if let Ok(memory_str) = String::from_utf8(output.stdout) {
												if let Ok(memory_mb) = memory_str.trim().parse::<u64>() {
													return memory_mb * 1024 * 1024;
												}
											}
										}
										return 8 * 1024 * 1024 * 1024; // Fallback estimate
									} else if driver_content.contains("amdgpu") {
										// Try to get actual AMD memory using rocm-smi
										if let Ok(output) = std::process::Command::new("rocm-smi")
											.args(&["--showmeminfo", "vram"])
											.output() {
											if let Ok(output_str) = String::from_utf8(output.stdout) {
												for line in output_str.lines() {
													if line.contains("Total Memory") {
														if let Some(memory_str) = line.split(':').nth(1) {
															if let Ok(memory_mb) = memory_str.trim().replace("MB", "").parse::<u64>() {
																return memory_mb * 1024 * 1024;
															}
														}
													}
												}
											}
										}
										return 4 * 1024 * 1024 * 1024; // Fallback estimate
									} else if driver_content.contains("i915") || driver_content.contains("i965") {
										// Try to get actual Intel memory using intel_gpu_top
										if let Ok(output) = std::process::Command::new("intel_gpu_top")
											.args(&["-J"])
											.output() {
											if let Ok(output_str) = String::from_utf8(output.stdout) {
												if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
													if let Some(engines) = json.get("engines") {
														if let Some(render) = engines.get("Render/3D/0") {
															if let Some(memory) = render.get("busy") {
																if let Some(memory_mb) = memory.as_u64() {
																	return memory_mb * 1024 * 1024;
																}
															}
														}
													}
												}
											}
										}
										return 1 * 1024 * 1024 * 1024; // Fallback estimate
									}
								}
							}
						}
					}
				}
			}
			
			// Try to get from lspci for PCI device info with actual memory detection
			if let Ok(output) = std::process::Command::new("lspci")
				.args(&["-v", "-s", "00:01.0,00:02.0,00:03.0"])
				.output() {
				if let Ok(output_str) = String::from_utf8(output.stdout) {
					for line in output_str.lines() {
						if line.contains("VGA") || line.contains("3D") {
							if line.contains("NVIDIA") {
								// Try to get actual NVIDIA memory
								if let Ok(nvidia_output) = std::process::Command::new("nvidia-smi")
									.args(&["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
									.output() {
									if let Ok(memory_str) = String::from_utf8(nvidia_output.stdout) {
										if let Ok(memory_mb) = memory_str.trim().parse::<u64>() {
											return memory_mb * 1024 * 1024;
										}
									}
								}
								return 8 * 1024 * 1024 * 1024; // Fallback estimate
							} else if line.contains("AMD") {
								// Try to get actual AMD memory
								if let Ok(amd_output) = std::process::Command::new("rocm-smi")
									.args(&["--showmeminfo", "vram"])
									.output() {
									if let Ok(output_str) = String::from_utf8(amd_output.stdout) {
										for mem_line in output_str.lines() {
											if mem_line.contains("Total Memory") {
												if let Some(memory_str) = mem_line.split(':').nth(1) {
													if let Ok(memory_mb) = memory_str.trim().replace("MB", "").parse::<u64>() {
														return memory_mb * 1024 * 1024;
													}
												}
											}
										}
									}
								}
								return 4 * 1024 * 1024 * 1024; // Fallback estimate
							} else if line.contains("Intel") {
								// Try to get actual Intel memory
								if let Ok(intel_output) = std::process::Command::new("intel_gpu_top")
									.args(&["-J"])
									.output() {
									if let Ok(output_str) = String::from_utf8(intel_output.stdout) {
										if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
											if let Some(engines) = json.get("engines") {
												if let Some(render) = engines.get("Render/3D/0") {
													if let Some(memory) = render.get("busy") {
														if let Some(memory_mb) = memory.as_u64() {
															return memory_mb * 1024 * 1024;
														}
													}
												}
											}
										}
									}
								}
								return 1 * 1024 * 1024 * 1024; // Fallback estimate
							}
						}
					}
				}
			}
		}
		
		// Fallback to default
		1024 * 1024 * 1024 // 1GB default
	}
	
	/**
	 * Gets current GPU configuration
	 * 
	 * @return &GpuConfig - Current GPU configuration
	 */
	pub fn config(&self) -> &GpuConfig {
		&self.config
	}
	
	/**
	 * Gets detected GPU capabilities
	 * 
	 * @return &GpuCapabilities - Detected GPU capabilities
	 */
	pub fn capabilities(&self) -> &GpuCapabilities {
		&self.capabilities
	}
	
	/**
	 * Gets current active backend
	 * 
	 * @return Option<&GpuBackend> - Current active backend
	 */
	pub fn active_backend(&self) -> Option<&GpuBackend> {
		self.active_backend.as_ref()
	}
	
	/**
	 * Updates performance metrics
	 * 
	 * @param frame_time - Current frame time in milliseconds
	 * @param gpu_memory - Current GPU memory usage in bytes
	 * @param cpu_memory - Current CPU memory usage in bytes
	 */
	pub async fn update_performance_metrics(&self, frame_time: f64, gpu_memory: u64, cpu_memory: u64) {
		let mut metrics = self.performance_metrics.write().await;
		metrics.average_frame_time = (metrics.average_frame_time + frame_time) / 2.0;
		metrics.current_fps = if frame_time > 0.0 { 1000.0 / frame_time } else { 0.0 };
		metrics.gpu_memory_usage = gpu_memory;
		metrics.cpu_memory_usage = cpu_memory;
		metrics.frames_rendered += 1;
		
		if frame_time > 16.67 { // Drop frame if > 60fps threshold
			metrics.frames_dropped += 1;
		}
	}
	
	/**
	 * Gets current performance metrics
	 * 
	 * @return PerformanceMetrics - Current performance metrics
	 */
	pub async fn performance_metrics(&self) -> PerformanceMetrics {
		self.performance_metrics.read().await.clone()
	}
	
	/**
	 * Queries OpenGL capabilities using X11 bindings
	 * 
	 * @return Result<GpuCapabilities> - GPU capabilities or error
	 */
	pub fn query_opengl_capabilities() -> Result<GpuCapabilities> {
		use std::process::Command;
		
		// Try to get OpenGL info using glxinfo
		if let Ok(output) = Command::new("glxinfo")
			.args(&["-B"])
			.output() {
			if let Ok(output_str) = String::from_utf8(output.stdout) {
				return Self::parse_glxinfo_output(&output_str);
			}
		}
		
		// Try to get OpenGL info using xrandr
		if let Ok(output) = Command::new("xrandr")
			.args(&["--listproviders"])
			.output() {
			if let Ok(output_str) = String::from_utf8(output.stdout) {
				return Self::parse_xrandr_output(&output_str);
			}
		}
		
		// Fallback to basic detection
		Ok(GpuCapabilities {
			available_backends: vec![GpuBackend::Cpu],
			max_texture_size: 4096,
			hardware_acceleration: false,
			gpu_memory: 0,
			supported_apis: vec!["CPU".to_string()],
		})
	}
	
	/**
	 * Parses glxinfo output to extract GPU capabilities
	 * 
	 * @param output - glxinfo output string
	 * @return Result<GpuCapabilities> - Parsed capabilities or error
	 */
	fn parse_glxinfo_output(output: &str) -> Result<GpuCapabilities> {
		let mut capabilities = GpuCapabilities {
			available_backends: vec![GpuBackend::Cpu],
			max_texture_size: 4096,
			hardware_acceleration: false,
			gpu_memory: 0,
			supported_apis: vec!["OpenGL".to_string()],
		};
		
		for line in output.lines() {
			let line = line.trim();
			
			// Check for OpenGL version
			if line.starts_with("OpenGL version string:") {
				capabilities.hardware_acceleration = true;
				if line.contains("3.3") || line.contains("4.") {
					capabilities.available_backends.push(GpuBackend::Skia);
					capabilities.available_backends.push(GpuBackend::Wgpu);
				}
			}
			
			// Check for maximum texture size
			if line.starts_with("Maximum texture size:") {
				if let Some(size_str) = line.split(':').nth(1) {
					if let Ok(size) = size_str.trim().parse::<u32>() {
						capabilities.max_texture_size = size;
					}
				}
			}
			
			// Check for GPU memory
			if line.contains("Total available memory:") {
				if let Some(memory_str) = line.split(':').nth(1) {
					let memory_str = memory_str.trim();
					if memory_str.ends_with("MB") {
						if let Ok(memory_mb) = memory_str.replace("MB", "").trim().parse::<u64>() {
							capabilities.gpu_memory = memory_mb * 1024 * 1024;
						}
					} else if memory_str.ends_with("GB") {
						if let Ok(memory_gb) = memory_str.replace("GB", "").trim().parse::<u64>() {
							capabilities.gpu_memory = memory_gb * 1024 * 1024 * 1024;
						}
					}
				}
			}
		}
		
		Ok(capabilities)
	}
	
	/**
	 * Parses xrandr output to extract GPU capabilities
	 * 
	 * @param output - xrandr output string
	 * @return Result<GpuCapabilities> - Parsed capabilities or error
	 */
	fn parse_xrandr_output(output: &str) -> Result<GpuCapabilities> {
		let mut capabilities = GpuCapabilities {
			available_backends: vec![GpuBackend::Cpu],
			max_texture_size: 4096,
			hardware_acceleration: false,
			gpu_memory: 0,
			supported_apis: vec!["X11".to_string()],
		};
		
		for line in output.lines() {
			let line = line.trim();
			
			// Check for GPU providers
			if line.contains("Provider") && line.contains("NVIDIA") {
				capabilities.hardware_acceleration = true;
				capabilities.available_backends.push(GpuBackend::Skia);
				capabilities.available_backends.push(GpuBackend::Wgpu);
			} else if line.contains("Provider") && line.contains("AMD") {
				capabilities.hardware_acceleration = true;
				capabilities.available_backends.push(GpuBackend::Skia);
				capabilities.available_backends.push(GpuBackend::Wgpu);
			} else if line.contains("Provider") && line.contains("Intel") {
				capabilities.hardware_acceleration = true;
				capabilities.available_backends.push(GpuBackend::Skia);
				capabilities.available_backends.push(GpuBackend::Wgpu);
			}
		}
		
		Ok(capabilities)
	}
} 