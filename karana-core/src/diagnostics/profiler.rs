// Kāraṇa OS - System Profiler
// Performance profiling and tracing

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// System profiler for performance analysis
pub struct SystemProfiler {
    /// Active sessions
    sessions: HashMap<String, ProfilingSession>,
    /// Completed sessions
    completed: Vec<ProfilingResult>,
    /// Max completed to keep
    max_completed: usize,
    /// Global profiler state
    enabled: bool,
    /// Span stack for nested profiling
    span_stack: Vec<ProfilingSpan>,
}

/// Profiling session
#[derive(Debug, Clone)]
pub struct ProfilingSession {
    /// Session name
    pub name: String,
    /// Session ID
    pub id: u64,
    /// Start time
    pub started_at: Instant,
    /// Spans within this session
    pub spans: Vec<ProfilingSpan>,
    /// Custom markers
    pub markers: Vec<ProfilingMarker>,
    /// Counters
    pub counters: HashMap<String, i64>,
}

/// Profiling span (time slice)
#[derive(Debug, Clone)]
pub struct ProfilingSpan {
    /// Span name
    pub name: String,
    /// Start time
    pub start: Instant,
    /// End time (None if still running)
    pub end: Option<Instant>,
    /// Parent span index (for nesting)
    pub parent: Option<usize>,
    /// Category
    pub category: SpanCategory,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Span categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanCategory {
    /// CPU-bound work
    Compute,
    /// Rendering/graphics
    Render,
    /// I/O operations
    IO,
    /// Network operations
    Network,
    /// AI/ML inference
    AI,
    /// User interaction handling
    Input,
    /// System/OS operations
    System,
    /// Custom category
    Custom,
}

/// Profiling marker (point event)
#[derive(Debug, Clone)]
pub struct ProfilingMarker {
    /// Marker name
    pub name: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Category
    pub category: SpanCategory,
    /// Message
    pub message: Option<String>,
}

/// Completed profiling result
#[derive(Debug, Clone)]
pub struct ProfilingResult {
    /// Session name
    pub name: String,
    /// Total duration
    pub duration: Duration,
    /// Span summaries
    pub span_summaries: Vec<SpanSummary>,
    /// Marker count
    pub marker_count: usize,
    /// Counter values
    pub counters: HashMap<String, i64>,
    /// Timestamp
    pub completed_at: Instant,
}

/// Summary of spans by name
#[derive(Debug, Clone)]
pub struct SpanSummary {
    /// Span name
    pub name: String,
    /// Category
    pub category: SpanCategory,
    /// Total time across all occurrences
    pub total_time: Duration,
    /// Number of occurrences
    pub count: u32,
    /// Average time
    pub avg_time: Duration,
    /// Min time
    pub min_time: Duration,
    /// Max time
    pub max_time: Duration,
    /// Percentage of total session time
    pub percent_of_total: f32,
}

impl SystemProfiler {
    /// Create new profiler
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            completed: Vec::new(),
            max_completed: 100,
            enabled: true,
            span_stack: Vec::new(),
        }
    }

    /// Enable/disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Is profiling enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start a new profiling session
    pub fn start_session(&mut self, name: &str) -> ProfilingSession {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        
        let session = ProfilingSession {
            name: name.to_string(),
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            started_at: Instant::now(),
            spans: Vec::new(),
            markers: Vec::new(),
            counters: HashMap::new(),
        };

        self.sessions.insert(name.to_string(), session.clone());
        session
    }

    /// End a profiling session and get results
    pub fn end_session(&mut self, name: &str) -> Option<ProfilingResult> {
        let session = self.sessions.remove(name)?;
        let result = self.compute_result(session);
        
        self.completed.push(result.clone());
        if self.completed.len() > self.max_completed {
            self.completed.remove(0);
        }

        Some(result)
    }

    /// Begin a span
    pub fn begin_span(&mut self, session_name: &str, span_name: &str, category: SpanCategory) {
        if !self.enabled {
            return;
        }

        if let Some(session) = self.sessions.get_mut(session_name) {
            let parent = if self.span_stack.is_empty() {
                None
            } else {
                Some(session.spans.len().saturating_sub(1))
            };

            let span = ProfilingSpan {
                name: span_name.to_string(),
                start: Instant::now(),
                end: None,
                parent,
                category,
                metadata: HashMap::new(),
            };

            self.span_stack.push(span.clone());
            session.spans.push(span);
        }
    }

    /// End the current span
    pub fn end_span(&mut self, session_name: &str) {
        if !self.enabled {
            return;
        }

        if let Some(session) = self.sessions.get_mut(session_name) {
            if let Some(mut span) = self.span_stack.pop() {
                span.end = Some(Instant::now());
                
                // Update the span in the session
                if let Some(session_span) = session.spans.last_mut() {
                    session_span.end = span.end;
                }
            }
        }
    }

    /// Add a marker
    pub fn add_marker(&mut self, session_name: &str, marker_name: &str, category: SpanCategory) {
        if !self.enabled {
            return;
        }

        if let Some(session) = self.sessions.get_mut(session_name) {
            session.markers.push(ProfilingMarker {
                name: marker_name.to_string(),
                timestamp: Instant::now(),
                category,
                message: None,
            });
        }
    }

    /// Increment a counter
    pub fn increment_counter(&mut self, session_name: &str, counter_name: &str, delta: i64) {
        if let Some(session) = self.sessions.get_mut(session_name) {
            *session.counters.entry(counter_name.to_string()).or_insert(0) += delta;
        }
    }

    fn compute_result(&self, session: ProfilingSession) -> ProfilingResult {
        let duration = session.started_at.elapsed();
        
        // Group spans by name
        let mut span_groups: HashMap<String, Vec<&ProfilingSpan>> = HashMap::new();
        for span in &session.spans {
            span_groups.entry(span.name.clone()).or_default().push(span);
        }

        // Compute summaries
        let span_summaries: Vec<SpanSummary> = span_groups.into_iter().map(|(name, spans)| {
            let times: Vec<Duration> = spans.iter()
                .filter_map(|s| s.end.map(|e| e.duration_since(s.start)))
                .collect();

            let total_time: Duration = times.iter().sum();
            let count = times.len() as u32;

            SpanSummary {
                name,
                category: spans.first().map(|s| s.category).unwrap_or(SpanCategory::Custom),
                total_time,
                count,
                avg_time: if count > 0 { total_time / count } else { Duration::ZERO },
                min_time: times.iter().min().copied().unwrap_or(Duration::ZERO),
                max_time: times.iter().max().copied().unwrap_or(Duration::ZERO),
                percent_of_total: if !duration.is_zero() {
                    (total_time.as_secs_f32() / duration.as_secs_f32()) * 100.0
                } else {
                    0.0
                },
            }
        }).collect();

        ProfilingResult {
            name: session.name,
            duration,
            span_summaries,
            marker_count: session.markers.len(),
            counters: session.counters,
            completed_at: Instant::now(),
        }
    }

    /// Get completed results
    pub fn completed_results(&self) -> &[ProfilingResult] {
        &self.completed
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.sessions.clear();
        self.completed.clear();
        self.span_stack.clear();
    }
}

impl Default for SystemProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped span guard
pub struct SpanGuard<'a> {
    profiler: &'a mut SystemProfiler,
    session_name: String,
}

impl<'a> SpanGuard<'a> {
    pub fn new(profiler: &'a mut SystemProfiler, session_name: &str, span_name: &str, category: SpanCategory) -> Self {
        profiler.begin_span(session_name, span_name, category);
        Self {
            profiler,
            session_name: session_name.to_string(),
        }
    }
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        self.profiler.end_span(&self.session_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler_creation() {
        let profiler = SystemProfiler::new();
        assert!(profiler.is_enabled());
    }

    #[test]
    fn test_start_session() {
        let mut profiler = SystemProfiler::new();
        let session = profiler.start_session("test");
        assert_eq!(session.name, "test");
    }

    #[test]
    fn test_end_session() {
        let mut profiler = SystemProfiler::new();
        profiler.start_session("test");
        
        let result = profiler.end_session("test");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test");
    }

    #[test]
    fn test_spans() {
        let mut profiler = SystemProfiler::new();
        profiler.start_session("test");
        
        profiler.begin_span("test", "work", SpanCategory::Compute);
        thread::sleep(Duration::from_millis(10));
        profiler.end_span("test");
        
        let result = profiler.end_session("test").unwrap();
        assert!(!result.span_summaries.is_empty());
    }

    #[test]
    fn test_markers() {
        let mut profiler = SystemProfiler::new();
        profiler.start_session("test");
        
        profiler.add_marker("test", "checkpoint", SpanCategory::System);
        profiler.add_marker("test", "checkpoint2", SpanCategory::System);
        
        let result = profiler.end_session("test").unwrap();
        assert_eq!(result.marker_count, 2);
    }

    #[test]
    fn test_counters() {
        let mut profiler = SystemProfiler::new();
        profiler.start_session("test");
        
        profiler.increment_counter("test", "iterations", 1);
        profiler.increment_counter("test", "iterations", 1);
        profiler.increment_counter("test", "iterations", 1);
        
        let result = profiler.end_session("test").unwrap();
        assert_eq!(*result.counters.get("iterations").unwrap(), 3);
    }

    #[test]
    fn test_disabled_profiler() {
        let mut profiler = SystemProfiler::new();
        profiler.set_enabled(false);
        
        profiler.start_session("test");
        profiler.begin_span("test", "work", SpanCategory::Compute);
        profiler.end_span("test");
        
        // Should still be able to end session but spans are empty
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_span_categories() {
        assert_ne!(SpanCategory::Compute, SpanCategory::Render);
        assert_eq!(SpanCategory::AI, SpanCategory::AI);
    }

    #[test]
    fn test_clear() {
        let mut profiler = SystemProfiler::new();
        profiler.start_session("test");
        profiler.end_session("test");
        
        assert!(!profiler.completed_results().is_empty());
        
        profiler.clear();
        assert!(profiler.completed_results().is_empty());
    }
}
