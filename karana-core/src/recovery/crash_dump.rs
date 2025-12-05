// Kāraṇa OS - Crash Dump Manager
// Create and manage crash dumps for debugging

use std::collections::HashMap;
use std::time::Instant;

use super::error_log::SystemError;

/// Crash dump manager
pub struct CrashDumpManager {
    /// Dump directory
    dump_dir: String,
    /// Max dumps to keep
    max_dumps: usize,
    /// Dump metadata
    dumps: Vec<CrashDumpInfo>,
    /// ID counter
    next_id: u64,
}

/// Crash dump information
#[derive(Debug, Clone)]
pub struct CrashDumpInfo {
    /// Dump ID
    pub id: String,
    /// Creation time
    pub created_at: Instant,
    /// Error that caused the dump
    pub error_id: u64,
    /// Component that crashed
    pub component: String,
    /// Dump size in bytes
    pub size_bytes: u64,
    /// File path (if saved to disk)
    pub file_path: Option<String>,
    /// Dump type
    pub dump_type: DumpType,
    /// Is compressed
    pub compressed: bool,
}

/// Type of dump
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DumpType {
    /// Mini dump - basic info only
    Mini,
    /// Standard dump - includes stack traces
    Standard,
    /// Full dump - complete memory state
    Full,
}

/// Crash dump contents
#[derive(Debug, Clone)]
pub struct CrashDump {
    /// Dump info
    pub info: CrashDumpInfo,
    /// Error details
    pub error: SystemError,
    /// Thread states
    pub threads: Vec<ThreadState>,
    /// Memory regions (for full dumps)
    pub memory_regions: Vec<MemoryRegion>,
    /// Loaded modules
    pub modules: Vec<LoadedModule>,
    /// System info
    pub system_info: SystemDumpInfo,
    /// Custom data
    pub custom_data: HashMap<String, String>,
}

/// Thread state in dump
#[derive(Debug, Clone)]
pub struct ThreadState {
    /// Thread ID
    pub id: u64,
    /// Thread name
    pub name: String,
    /// Is main thread
    pub is_main: bool,
    /// Thread state
    pub state: String,
    /// Stack trace
    pub stack_trace: Vec<StackFrame>,
    /// CPU registers (if available)
    pub registers: HashMap<String, u64>,
}

/// Stack frame
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Frame index
    pub index: usize,
    /// Address
    pub address: u64,
    /// Module name
    pub module: String,
    /// Function name
    pub function: Option<String>,
    /// Source file
    pub file: Option<String>,
    /// Line number
    pub line: Option<u32>,
}

/// Memory region
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Start address
    pub start: u64,
    /// Size
    pub size: u64,
    /// Protection flags
    pub protection: String,
    /// Region type
    pub region_type: String,
}

/// Loaded module info
#[derive(Debug, Clone)]
pub struct LoadedModule {
    /// Module name
    pub name: String,
    /// Base address
    pub base_address: u64,
    /// Size
    pub size: u64,
    /// Version
    pub version: Option<String>,
    /// Path
    pub path: String,
}

/// System info for dump
#[derive(Debug, Clone)]
pub struct SystemDumpInfo {
    /// OS version
    pub os_version: String,
    /// Device model
    pub device_model: String,
    /// CPU architecture
    pub cpu_arch: String,
    /// Total memory
    pub total_memory: u64,
    /// Available memory at crash
    pub available_memory: u64,
    /// Uptime at crash
    pub uptime_secs: u64,
}

impl CrashDumpManager {
    /// Create new dump manager
    pub fn new(dump_dir: &str, max_dumps: usize) -> Self {
        Self {
            dump_dir: dump_dir.to_string(),
            max_dumps,
            dumps: Vec::new(),
            next_id: 1,
        }
    }

    /// Create a crash dump
    pub fn create_dump(&mut self, error: &SystemError) -> Option<CrashDump> {
        let dump_id = format!("crash_{:08x}_{}", self.next_id, timestamp_str());
        self.next_id += 1;

        let info = CrashDumpInfo {
            id: dump_id.clone(),
            created_at: Instant::now(),
            error_id: error.id,
            component: error.component.clone(),
            size_bytes: 0, // Will be calculated
            file_path: Some(format!("{}/{}.dump", self.dump_dir, dump_id)),
            dump_type: DumpType::Standard,
            compressed: true,
        };

        // Collect dump data
        let dump = CrashDump {
            info: info.clone(),
            error: error.clone(),
            threads: self.collect_threads(),
            memory_regions: Vec::new(), // Skip for standard dump
            modules: self.collect_modules(),
            system_info: self.collect_system_info(),
            custom_data: HashMap::new(),
        };

        // Store info
        self.dumps.push(info);

        // Cleanup old dumps
        self.cleanup();

        Some(dump)
    }

    /// Create mini dump (smaller, faster)
    pub fn create_mini_dump(&mut self, error: &SystemError) -> Option<CrashDumpInfo> {
        let dump_id = format!("mini_{:08x}_{}", self.next_id, timestamp_str());
        self.next_id += 1;

        let info = CrashDumpInfo {
            id: dump_id.clone(),
            created_at: Instant::now(),
            error_id: error.id,
            component: error.component.clone(),
            size_bytes: 1024, // Approximate
            file_path: Some(format!("{}/{}.mdmp", self.dump_dir, dump_id)),
            dump_type: DumpType::Mini,
            compressed: false,
        };

        self.dumps.push(info.clone());
        self.cleanup();

        Some(info)
    }

    fn collect_threads(&self) -> Vec<ThreadState> {
        // In real implementation, would enumerate threads
        // For simulation, return mock data
        vec![
            ThreadState {
                id: 1,
                name: "main".to_string(),
                is_main: true,
                state: "running".to_string(),
                stack_trace: vec![
                    StackFrame {
                        index: 0,
                        address: 0x7fff12345678,
                        module: "karana-core".to_string(),
                        function: Some("panic_handler".to_string()),
                        file: Some("src/panic.rs".to_string()),
                        line: Some(42),
                    },
                    StackFrame {
                        index: 1,
                        address: 0x7fff12345000,
                        module: "karana-core".to_string(),
                        function: Some("main".to_string()),
                        file: Some("src/main.rs".to_string()),
                        line: Some(100),
                    },
                ],
                registers: HashMap::new(),
            },
        ]
    }

    fn collect_modules(&self) -> Vec<LoadedModule> {
        vec![
            LoadedModule {
                name: "karana-core".to_string(),
                base_address: 0x7fff12340000,
                size: 0x100000,
                version: Some("0.1.0".to_string()),
                path: "/opt/karana/lib/libkarana-core.so".to_string(),
            },
        ]
    }

    fn collect_system_info(&self) -> SystemDumpInfo {
        SystemDumpInfo {
            os_version: "Karana OS 0.1.0".to_string(),
            device_model: "Smart Glasses Gen1".to_string(),
            cpu_arch: "aarch64".to_string(),
            total_memory: 2_000_000_000,
            available_memory: 500_000_000,
            uptime_secs: 3600,
        }
    }

    /// List all dumps
    pub fn list_dumps(&self) -> Vec<CrashDumpInfo> {
        self.dumps.clone()
    }

    /// Get dump by ID
    pub fn get_dump(&self, id: &str) -> Option<&CrashDumpInfo> {
        self.dumps.iter().find(|d| d.id == id)
    }

    /// Delete a dump
    pub fn delete_dump(&mut self, id: &str) -> bool {
        if let Some(pos) = self.dumps.iter().position(|d| d.id == id) {
            self.dumps.remove(pos);
            true
        } else {
            false
        }
    }

    /// Cleanup old dumps
    pub fn cleanup(&mut self) {
        while self.dumps.len() > self.max_dumps {
            // Remove oldest
            self.dumps.remove(0);
        }
    }

    /// Get total dump size
    pub fn total_size(&self) -> u64 {
        self.dumps.iter().map(|d| d.size_bytes).sum()
    }
}

fn timestamp_str() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{:x}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_error() -> SystemError {
        SystemError {
            id: 1,
            timestamp: Instant::now(),
            severity: super::super::error_log::ErrorSeverity::Fatal,
            component: "test".to_string(),
            message: "Test crash".to_string(),
            backtrace: None,
            context: HashMap::new(),
        }
    }

    #[test]
    fn test_dump_manager_creation() {
        let manager = CrashDumpManager::new("/tmp/dumps", 10);
        assert!(manager.list_dumps().is_empty());
    }

    #[test]
    fn test_create_dump() {
        let mut manager = CrashDumpManager::new("/tmp/dumps", 10);
        let error = mock_error();
        
        let dump = manager.create_dump(&error);
        assert!(dump.is_some());
        assert_eq!(manager.list_dumps().len(), 1);
    }

    #[test]
    fn test_create_mini_dump() {
        let mut manager = CrashDumpManager::new("/tmp/dumps", 10);
        let error = mock_error();
        
        let info = manager.create_mini_dump(&error);
        assert!(info.is_some());
        assert_eq!(info.unwrap().dump_type, DumpType::Mini);
    }

    #[test]
    fn test_cleanup() {
        let mut manager = CrashDumpManager::new("/tmp/dumps", 3);
        let error = mock_error();
        
        for _ in 0..5 {
            manager.create_mini_dump(&error);
        }
        
        assert!(manager.list_dumps().len() <= 3);
    }

    #[test]
    fn test_delete_dump() {
        let mut manager = CrashDumpManager::new("/tmp/dumps", 10);
        let error = mock_error();
        
        let info = manager.create_mini_dump(&error).unwrap();
        assert_eq!(manager.list_dumps().len(), 1);
        
        assert!(manager.delete_dump(&info.id));
        assert!(manager.list_dumps().is_empty());
    }

    #[test]
    fn test_get_dump() {
        let mut manager = CrashDumpManager::new("/tmp/dumps", 10);
        let error = mock_error();
        
        let info = manager.create_mini_dump(&error).unwrap();
        
        let found = manager.get_dump(&info.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, info.id);
    }

    #[test]
    fn test_dump_types() {
        assert_ne!(DumpType::Mini, DumpType::Standard);
        assert_ne!(DumpType::Standard, DumpType::Full);
    }
}
