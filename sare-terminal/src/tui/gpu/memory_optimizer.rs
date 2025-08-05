/**
 * Memory optimization system for Sare terminal
 * 
 * This module provides efficient memory management, garbage collection,
 * and memory pooling to optimize memory usage throughout the terminal.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: memory_optimizer.rs
 * Description: Memory optimization and garbage collection system
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/**
 * Memory pool for efficient allocation
 * 
 * メモリプールを管理する構造体です。
 * 頻繁に使用されるデータ構造の再利用を
 * 提供し、メモリ割り当てのオーバーヘッドを
 * 削減します。
 */
pub struct MemoryPool<T> {
	/// Available items in the pool
	available: Vec<T>,
	/// Maximum pool size
	max_size: usize,
	/// Current pool size
	current_size: usize,
	/// Last cleanup time
	last_cleanup: Instant,
	/// Cleanup interval
	cleanup_interval: Duration,
}

impl<T> MemoryPool<T> {
	/**
	 * Creates a new memory pool
	 * 
	 * @param max_size - Maximum pool size
	 * @return MemoryPool<T> - New memory pool
	 */
	pub fn new(max_size: usize) -> Self {
		Self {
			available: Vec::with_capacity(max_size),
			max_size,
			current_size: 0,
			last_cleanup: Instant::now(),
			cleanup_interval: Duration::from_secs(300), // 5 minutes
		}
	}
	
	/**
	 * Acquires an item from the pool
	 * 
	 * @param create_fn - Function to create new item if pool is empty
	 * @return T - Item from pool or newly created
	 */
	pub fn acquire<F>(&mut self, create_fn: F) -> T
	where
		F: FnOnce() -> T,
	{
		if let Some(item) = self.available.pop() {
			item
		} else {
			self.current_size += 1;
			create_fn()
		}
	}
	
	/**
	 * Returns an item to the pool
	 * 
	 * @param item - Item to return to pool
	 */
	pub fn release(&mut self, item: T) {
		if self.available.len() < self.max_size {
			self.available.push(item);
		}
		// Otherwise, item is dropped
	}
	
	/**
	 * Performs garbage collection on the pool
	 * 
	 * @param should_cleanup - Function to determine if item should be cleaned up
	 */
	pub fn garbage_collect<F>(&mut self, should_cleanup: F)
	where
		F: Fn(&T) -> bool,
	{
		if self.last_cleanup.elapsed() >= self.cleanup_interval {
			self.available.retain(|item| !should_cleanup(item));
			self.last_cleanup = Instant::now();
		}
	}
	
	/**
	 * Gets current pool statistics
	 * 
	 * @return PoolStats - Current pool statistics
	 */
	pub fn stats(&self) -> PoolStats {
		PoolStats {
			available: self.available.len(),
			current_size: self.current_size,
			max_size: self.max_size,
		}
	}
}

/**
 * Pool statistics
 */
#[derive(Debug, Clone)]
pub struct PoolStats {
	/// Number of available items
	pub available: usize,
	/// Current pool size
	pub current_size: usize,
	/// Maximum pool size
	pub max_size: usize,
}

/**
 * Memory optimizer for the terminal
 * 
 * ターミナルのメモリ最適化を管理する構造体です。
 * メモリプール、ガベージコレクション、メモリ使用量の
 * 監視を提供します。
 */
pub struct MemoryOptimizer {
	/// Vector pools for different sizes
	vector_pools: Arc<RwLock<HashMap<usize, MemoryPool<Vec<u8>>>>>,
	/// String pools
	string_pools: Arc<RwLock<MemoryPool<String>>>,
	/// Memory usage tracking
	memory_usage: Arc<RwLock<MemoryUsage>>,
	/// Garbage collection interval
	gc_interval: Duration,
	/// Last garbage collection
	last_gc: Instant,
}

impl MemoryOptimizer {
	/**
	 * Creates a new memory optimizer
	 * 
	 * @return MemoryOptimizer - New memory optimizer
	 */
	pub fn new() -> Self {
		Self {
			vector_pools: Arc::new(RwLock::new(HashMap::new())),
			string_pools: Arc::new(RwLock::new(MemoryPool::new(1000))),
			memory_usage: Arc::new(RwLock::new(MemoryUsage::default())),
			gc_interval: Duration::from_secs(60), // 1 minute
			last_gc: Instant::now(),
		}
	}
	
	/**
	 * Acquires a vector from the pool
	 * 
	 * @param size - Vector size
	 * @return Vec<u8> - Vector from pool or newly created
	 */
	pub async fn acquire_vector(&self, size: usize) -> Vec<u8> {
		let mut pools = self.vector_pools.write().await;
		let pool = pools.entry(size).or_insert_with(|| MemoryPool::new(100));
		pool.acquire(|| Vec::with_capacity(size))
	}
	
	/**
	 * Returns a vector to the pool
	 * 
	 * @param vector - Vector to return to pool
	 */
	pub async fn release_vector(&self, vector: Vec<u8>) {
		let mut pools = self.vector_pools.write().await;
		if let Some(pool) = pools.get_mut(&vector.capacity()) {
			pool.release(vector);
		}
	}
	
	/**
	 * Acquires a string from the pool
	 * 
	 * @return String - String from pool or newly created
	 */
	pub async fn acquire_string(&self) -> String {
		let mut pool = self.string_pools.write().await;
		pool.acquire(String::new)
	}
	
	/**
	 * Returns a string to the pool
	 * 
	 * @param string - String to return to pool
	 */
	pub async fn release_string(&self, string: String) {
		let mut pool = self.string_pools.write().await;
		pool.release(string);
	}
	
	/**
	 * Updates memory usage statistics
	 * 
	 * @param allocated - Newly allocated bytes
	 * @param freed - Newly freed bytes
	 */
	pub async fn update_memory_usage(&self, allocated: usize, freed: usize) {
		let mut usage = self.memory_usage.write().await;
		usage.total_allocated += allocated;
		usage.total_freed += freed;
		usage.current_usage = usage.current_usage.saturating_add(allocated).saturating_sub(freed);
		usage.peak_usage = usage.peak_usage.max(usage.current_usage);
	}
	
	/**
	 * Performs garbage collection
	 */
	pub async fn garbage_collect(&mut self) {
		if self.last_gc.elapsed() >= self.gc_interval {
			// Garbage collect vector pools
			{
				let mut pools = self.vector_pools.write().await;
				for pool in pools.values_mut() {
					pool.garbage_collect(|vec| vec.capacity() > 1024 * 1024); // Clean up large vectors
				}
			}
			
			// Garbage collect string pools
			{
				let mut pool = self.string_pools.write().await;
				pool.garbage_collect(|s| s.capacity() > 1024); // Clean up large strings
			}
			
			self.last_gc = Instant::now();
		}
	}
	
	/**
	 * Gets memory usage statistics
	 * 
	 * @return MemoryUsage - Current memory usage
	 */
	pub async fn memory_usage(&self) -> MemoryUsage {
		self.memory_usage.read().await.clone()
	}
	
	/**
	 * Gets pool statistics
	 * 
	 * @return Vec<PoolStats> - Statistics for all pools
	 */
	pub async fn pool_stats(&self) -> Vec<PoolStats> {
		let mut stats = Vec::new();
		
		// Vector pool stats
		{
			let pools = self.vector_pools.read().await;
			for (size, pool) in pools.iter() {
				let mut stat = pool.stats();
				stat.max_size = *size;
				stats.push(stat);
			}
		}
		
		// String pool stats
		{
			let pool = self.string_pools.read().await;
			stats.push(pool.stats());
		}
		
		stats
	}
}

/**
 * Memory usage statistics
 */
#[derive(Debug, Clone)]
pub struct MemoryUsage {
	/// Current memory usage in bytes
	pub current_usage: usize,
	/// Peak memory usage in bytes
	pub peak_usage: usize,
	/// Total allocated bytes
	pub total_allocated: usize,
	/// Total freed bytes
	pub total_freed: usize,
	/// Number of allocations
	pub allocation_count: usize,
	/// Number of deallocations
	pub deallocation_count: usize,
}

impl Default for MemoryUsage {
	fn default() -> Self {
		Self {
			current_usage: 0,
			peak_usage: 0,
			total_allocated: 0,
			total_freed: 0,
			allocation_count: 0,
			deallocation_count: 0,
		}
	}
}

/**
 * Optimized vector that uses memory pooling
 */
pub struct OptimizedVector {
	/// Inner vector
	inner: Vec<u8>,
	/// Memory optimizer reference
	optimizer: Arc<MemoryOptimizer>,
}

impl OptimizedVector {
	/**
	 * Creates a new optimized vector
	 * 
	 * @param optimizer - Memory optimizer reference
	 * @param size - Initial size
	 * @return OptimizedVector - New optimized vector
	 */
	pub async fn new(optimizer: Arc<MemoryOptimizer>, size: usize) -> Self {
		let inner = optimizer.acquire_vector(size).await;
		Self { inner, optimizer }
	}
	
	/**
	 * Pushes data to the vector
	 * 
	 * @param data - Data to push
	 */
	pub fn push(&mut self, data: u8) {
		self.inner.push(data);
	}
	
	/**
	 * Extends the vector with data
	 * 
	 * @param data - Data to extend with
	 */
	pub fn extend(&mut self, data: &[u8]) {
		self.inner.extend_from_slice(data);
	}
	
	/**
	 * Gets the vector as a slice
	 * 
	 * @return &[u8] - Vector as slice
	 */
	pub fn as_slice(&self) -> &[u8] {
		&self.inner
	}
	
	/**
	 * Gets the vector as a mutable slice
	 * 
	 * @return &mut [u8] - Vector as mutable slice
	 */
	pub fn as_mut_slice(&mut self) -> &mut [u8] {
		&mut self.inner
	}
	
	/**
	 * Gets the vector length
	 * 
	 * @return usize - Vector length
	 */
	pub fn len(&self) -> usize {
		self.inner.len()
	}
	
	/**
	 * Checks if the vector is empty
	 * 
	 * @return bool - True if empty
	 */
	pub fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}
}

impl Drop for OptimizedVector {
	fn drop(&mut self) {
		// Return to pool when dropped
		let optimizer = self.optimizer.clone();
		let inner = std::mem::replace(&mut self.inner, Vec::new());
		tokio::spawn(async move {
			optimizer.release_vector(inner).await;
		});
	}
}

/**
 * Optimized string that uses memory pooling
 */
pub struct OptimizedString {
	/// Inner string
	inner: String,
	/// Memory optimizer reference
	optimizer: Arc<MemoryOptimizer>,
}

impl OptimizedString {
	/**
	 * Creates a new optimized string
	 * 
	 * @param optimizer - Memory optimizer reference
	 * @return OptimizedString - New optimized string
	 */
	pub async fn new(optimizer: Arc<MemoryOptimizer>) -> Self {
		let inner = optimizer.acquire_string().await;
		Self { inner, optimizer }
	}
	
	/**
	 * Pushes a character to the string
	 * 
	 * @param ch - Character to push
	 */
	pub fn push(&mut self, ch: char) {
		self.inner.push(ch);
	}
	
	/**
	 * Pushes a string to the string
	 * 
	 * @param s - String to push
	 */
	pub fn push_str(&mut self, s: &str) {
		self.inner.push_str(s);
	}
	
	/**
	 * Gets the string as a slice
	 * 
	 * @return &str - String as slice
	 */
	pub fn as_str(&self) -> &str {
		&self.inner
	}
	
	/**
	 * Gets the string length
	 * 
	 * @return usize - String length
	 */
	pub fn len(&self) -> usize {
		self.inner.len()
	}
	
	/**
	 * Checks if the string is empty
	 * 
	 * @return bool - True if empty
	 */
	pub fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}
}

impl Drop for OptimizedString {
	fn drop(&mut self) {
		// Return to pool when dropped
		let optimizer = self.optimizer.clone();
		let inner = std::mem::replace(&mut self.inner, String::new());
		tokio::spawn(async move {
			optimizer.release_string(inner).await;
		});
	}
} 