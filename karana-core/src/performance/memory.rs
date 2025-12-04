// Memory Management for Kāraṇa OS
// Tracks and optimizes memory usage for AR smart glasses

use std::collections::HashMap;

/// Memory region types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryRegion {
    /// System core memory
    System,
    /// Application memory
    Application,
    /// Texture and graphics memory
    Graphics,
    /// Audio buffers
    Audio,
    /// AR rendering buffers
    ArRendering,
    /// Cache memory
    Cache,
    /// Shared memory
    Shared,
}

/// Memory allocation tracking
#[derive(Debug, Clone)]
pub struct MemoryAllocation {
    pub allocation_id: String,
    pub region: MemoryRegion,
    pub size_bytes: usize,
    pub owner: String,
    pub timestamp: u64,
    pub is_pinned: bool,
}

/// Memory pressure level
#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum MemoryPressure {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Memory pool for efficient allocation
#[derive(Debug, Clone)]
pub struct MemoryPool {
    pub name: String,
    pub total_size_bytes: usize,
    pub used_bytes: usize,
    pub block_size: usize,
}

impl MemoryPool {
    pub fn new(name: &str, total_size: usize, block_size: usize) -> Self {
        Self {
            name: name.to_string(),
            total_size_bytes: total_size,
            used_bytes: 0,
            block_size,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        let aligned_size = ((size + self.block_size - 1) / self.block_size) * self.block_size;
        if self.used_bytes + aligned_size <= self.total_size_bytes {
            let offset = self.used_bytes;
            self.used_bytes += aligned_size;
            Some(offset)
        } else {
            None
        }
    }

    pub fn available(&self) -> usize {
        self.total_size_bytes - self.used_bytes
    }

    pub fn usage_percent(&self) -> f32 {
        (self.used_bytes as f32 / self.total_size_bytes as f32) * 100.0
    }
}

/// Memory manager for tracking and optimizing memory
pub struct MemoryManager {
    total_memory_mb: f32,
    used_memory_mb: f32,
    allocations: HashMap<String, MemoryAllocation>,
    region_limits: HashMap<MemoryRegion, usize>,
    region_usage: HashMap<MemoryRegion, usize>,
    pools: HashMap<String, MemoryPool>,
    gc_threshold: f32,
    pressure: MemoryPressure,
    low_memory_callbacks: Vec<String>,
}

impl MemoryManager {
    pub fn new() -> Self {
        // Default 512MB for smart glasses
        Self::with_capacity(512.0)
    }

    pub fn with_capacity(total_mb: f32) -> Self {
        let mut region_limits = HashMap::new();
        let total_bytes = (total_mb * 1024.0 * 1024.0) as usize;
        
        // Default region allocations
        region_limits.insert(MemoryRegion::System, total_bytes / 8);
        region_limits.insert(MemoryRegion::Application, total_bytes / 4);
        region_limits.insert(MemoryRegion::Graphics, total_bytes / 4);
        region_limits.insert(MemoryRegion::Audio, total_bytes / 16);
        region_limits.insert(MemoryRegion::ArRendering, total_bytes / 4);
        region_limits.insert(MemoryRegion::Cache, total_bytes / 16);
        region_limits.insert(MemoryRegion::Shared, total_bytes / 16);

        Self {
            total_memory_mb: total_mb,
            used_memory_mb: 0.0,
            allocations: HashMap::new(),
            region_limits,
            region_usage: HashMap::new(),
            pools: HashMap::new(),
            gc_threshold: 0.85,
            pressure: MemoryPressure::None,
            low_memory_callbacks: Vec::new(),
        }
    }

    pub fn allocate(&mut self, allocation: MemoryAllocation) -> Result<(), String> {
        let region_limit = self.region_limits.get(&allocation.region)
            .copied()
            .unwrap_or(usize::MAX);
        
        let current_usage = self.region_usage.get(&allocation.region)
            .copied()
            .unwrap_or(0);
        
        if current_usage + allocation.size_bytes > region_limit {
            return Err(format!("Region {:?} limit exceeded", allocation.region));
        }

        let total_after = self.used_memory_mb + (allocation.size_bytes as f32 / (1024.0 * 1024.0));
        if total_after > self.total_memory_mb {
            return Err("Total memory limit exceeded".to_string());
        }

        *self.region_usage.entry(allocation.region).or_insert(0) += allocation.size_bytes;
        self.used_memory_mb = total_after;
        self.allocations.insert(allocation.allocation_id.clone(), allocation);
        
        self.update_pressure();
        Ok(())
    }

    pub fn deallocate(&mut self, allocation_id: &str) -> bool {
        if let Some(allocation) = self.allocations.remove(allocation_id) {
            if let Some(usage) = self.region_usage.get_mut(&allocation.region) {
                *usage = usage.saturating_sub(allocation.size_bytes);
            }
            self.used_memory_mb -= allocation.size_bytes as f32 / (1024.0 * 1024.0);
            self.update_pressure();
            true
        } else {
            false
        }
    }

    fn update_pressure(&mut self) {
        let usage_ratio = self.used_memory_mb / self.total_memory_mb;
        self.pressure = if usage_ratio > 0.95 {
            MemoryPressure::Critical
        } else if usage_ratio > 0.85 {
            MemoryPressure::High
        } else if usage_ratio > 0.70 {
            MemoryPressure::Medium
        } else if usage_ratio > 0.50 {
            MemoryPressure::Low
        } else {
            MemoryPressure::None
        };
    }

    pub fn get_pressure(&self) -> MemoryPressure {
        self.pressure
    }

    pub fn trigger_gc(&mut self) {
        // Clear non-pinned cache allocations first
        let cache_allocations: Vec<_> = self.allocations.iter()
            .filter(|(_, a)| a.region == MemoryRegion::Cache && !a.is_pinned)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in cache_allocations {
            self.deallocate(&id);
        }
    }

    pub fn get_usage_mb(&self) -> f32 {
        self.used_memory_mb
    }

    pub fn get_available_mb(&self) -> f32 {
        self.total_memory_mb - self.used_memory_mb
    }

    pub fn get_region_usage(&self, region: MemoryRegion) -> usize {
        self.region_usage.get(&region).copied().unwrap_or(0)
    }

    pub fn get_region_limit(&self, region: MemoryRegion) -> usize {
        self.region_limits.get(&region).copied().unwrap_or(0)
    }

    pub fn set_region_limit(&mut self, region: MemoryRegion, limit_bytes: usize) {
        self.region_limits.insert(region, limit_bytes);
    }

    pub fn create_pool(&mut self, name: &str, size: usize, block_size: usize) {
        let pool = MemoryPool::new(name, size, block_size);
        self.pools.insert(name.to_string(), pool);
    }

    pub fn get_pool(&mut self, name: &str) -> Option<&mut MemoryPool> {
        self.pools.get_mut(name)
    }

    pub fn register_low_memory_callback(&mut self, callback_id: String) {
        self.low_memory_callbacks.push(callback_id);
    }

    pub fn get_allocation(&self, id: &str) -> Option<&MemoryAllocation> {
        self.allocations.get(id)
    }

    pub fn get_allocations_by_owner(&self, owner: &str) -> Vec<&MemoryAllocation> {
        self.allocations.values()
            .filter(|a| a.owner == owner)
            .collect()
    }

    pub fn get_total_by_region(&self, region: MemoryRegion) -> usize {
        self.allocations.values()
            .filter(|a| a.region == region)
            .map(|a| a.size_bytes)
            .sum()
    }

    pub fn compact(&mut self) {
        // Simulate memory compaction (in real implementation would defragment)
        // For now, just trigger GC
        self.trigger_gc();
    }

    pub fn get_fragmentation_percent(&self) -> f32 {
        // Simplified fragmentation estimate
        let allocation_count = self.allocations.len() as f32;
        if allocation_count == 0.0 {
            return 0.0;
        }
        // More allocations = higher fragmentation estimate
        (allocation_count / 100.0 * 5.0).min(50.0)
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_creation() {
        let manager = MemoryManager::new();
        assert_eq!(manager.total_memory_mb, 512.0);
        assert_eq!(manager.get_usage_mb(), 0.0);
    }

    #[test]
    fn test_allocation() {
        let mut manager = MemoryManager::new();
        let allocation = MemoryAllocation {
            allocation_id: "test1".to_string(),
            region: MemoryRegion::Application,
            size_bytes: 1024 * 1024, // 1 MB
            owner: "test_app".to_string(),
            timestamp: 0,
            is_pinned: false,
        };
        
        assert!(manager.allocate(allocation).is_ok());
        assert!(manager.get_usage_mb() > 0.0);
    }

    #[test]
    fn test_deallocation() {
        let mut manager = MemoryManager::new();
        let allocation = MemoryAllocation {
            allocation_id: "test1".to_string(),
            region: MemoryRegion::Application,
            size_bytes: 1024 * 1024,
            owner: "test_app".to_string(),
            timestamp: 0,
            is_pinned: false,
        };
        
        manager.allocate(allocation).unwrap();
        let usage_before = manager.get_usage_mb();
        
        assert!(manager.deallocate("test1"));
        assert!(manager.get_usage_mb() < usage_before);
    }

    #[test]
    fn test_region_limits() {
        let mut manager = MemoryManager::with_capacity(100.0);
        manager.set_region_limit(MemoryRegion::Audio, 1024 * 1024); // 1 MB limit
        
        let allocation = MemoryAllocation {
            allocation_id: "audio1".to_string(),
            region: MemoryRegion::Audio,
            size_bytes: 2 * 1024 * 1024, // 2 MB - exceeds limit
            owner: "audio_system".to_string(),
            timestamp: 0,
            is_pinned: false,
        };
        
        assert!(manager.allocate(allocation).is_err());
    }

    #[test]
    fn test_memory_pressure() {
        let mut manager = MemoryManager::with_capacity(10.0);
        // Set a larger region limit to allow our test allocation
        manager.set_region_limit(MemoryRegion::Application, 10 * 1024 * 1024);
        
        // Fill up memory to trigger high pressure
        let allocation = MemoryAllocation {
            allocation_id: "big".to_string(),
            region: MemoryRegion::Application,
            size_bytes: 9 * 1024 * 1024, // 9 MB of 10 MB
            owner: "test".to_string(),
            timestamp: 0,
            is_pinned: false,
        };
        
        manager.allocate(allocation).unwrap();
        assert!(manager.get_pressure() >= MemoryPressure::High);
    }

    #[test]
    fn test_gc_clears_cache() {
        let mut manager = MemoryManager::new();
        
        let cache_alloc = MemoryAllocation {
            allocation_id: "cache1".to_string(),
            region: MemoryRegion::Cache,
            size_bytes: 1024 * 1024,
            owner: "cache".to_string(),
            timestamp: 0,
            is_pinned: false,
        };
        
        manager.allocate(cache_alloc).unwrap();
        assert!(manager.get_allocation("cache1").is_some());
        
        manager.trigger_gc();
        assert!(manager.get_allocation("cache1").is_none());
    }

    #[test]
    fn test_pinned_survives_gc() {
        let mut manager = MemoryManager::new();
        
        let pinned_alloc = MemoryAllocation {
            allocation_id: "pinned_cache".to_string(),
            region: MemoryRegion::Cache,
            size_bytes: 1024 * 1024,
            owner: "important".to_string(),
            timestamp: 0,
            is_pinned: true,
        };
        
        manager.allocate(pinned_alloc).unwrap();
        manager.trigger_gc();
        assert!(manager.get_allocation("pinned_cache").is_some());
    }

    #[test]
    fn test_memory_pool() {
        let mut manager = MemoryManager::new();
        manager.create_pool("texture_pool", 1024 * 1024, 4096);
        
        let pool = manager.get_pool("texture_pool").unwrap();
        assert_eq!(pool.total_size_bytes, 1024 * 1024);
        
        let offset = pool.allocate(8192);
        assert!(offset.is_some());
        assert_eq!(offset.unwrap(), 0);
        
        let offset2 = pool.allocate(4096);
        assert!(offset2.is_some());
        assert_eq!(offset2.unwrap(), 8192);
    }

    #[test]
    fn test_allocations_by_owner() {
        let mut manager = MemoryManager::new();
        
        for i in 0..3 {
            let alloc = MemoryAllocation {
                allocation_id: format!("app_{}", i),
                region: MemoryRegion::Application,
                size_bytes: 1024,
                owner: "my_app".to_string(),
                timestamp: 0,
                is_pinned: false,
            };
            manager.allocate(alloc).unwrap();
        }
        
        let app_allocs = manager.get_allocations_by_owner("my_app");
        assert_eq!(app_allocs.len(), 3);
    }
}
