// Phase 54: Task Scheduler
//
// Priority-based task scheduling with deadline enforcement:
// 1. AR render tasks get highest priority (16ms deadline)
// 2. Blockchain tasks can be deferred
// 3. Static analysis for state access patterns

use anyhow::Result;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Priority levels for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// AR rendering - must complete within 16ms
    Render = 5,
    /// User interaction response
    Interactive = 4,
    /// Time-sensitive operations
    Urgent = 3,
    /// Normal operations
    Normal = 2,
    /// Background tasks
    Background = 1,
    /// Deferrable maintenance
    Maintenance = 0,
}

/// Deadline specification for a task
#[derive(Debug, Clone)]
pub enum TaskDeadline {
    /// Must complete within specified duration
    Hard(Duration),
    /// Should complete within duration, but not critical
    Soft(Duration),
    /// No specific deadline
    None,
}

/// A scheduled task
pub struct ScheduledTask {
    /// Unique task ID
    pub id: String,
    
    /// Task name for debugging
    pub name: String,
    
    /// Priority level
    pub priority: TaskPriority,
    
    /// Deadline
    pub deadline: TaskDeadline,
    
    /// When task was created
    pub created_at: Instant,
    
    /// Estimated execution time
    pub estimated_duration: Option<Duration>,
    
    /// The actual task function
    pub task: Box<dyn FnOnce() -> Result<()> + Send + 'static>,
    
    /// Retry count if task fails
    pub retry_count: u8,
    
    /// Maximum retries
    pub max_retries: u8,
}

impl ScheduledTask {
    pub fn new<F>(name: String, priority: TaskPriority, task: F) -> Self
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            priority,
            deadline: TaskDeadline::None,
            created_at: Instant::now(),
            estimated_duration: None,
            task: Box::new(task),
            retry_count: 0,
            max_retries: 3,
        }
    }
    
    pub fn with_deadline(mut self, deadline: TaskDeadline) -> Self {
        self.deadline = deadline;
        self
    }
    
    pub fn with_estimated_duration(mut self, duration: Duration) -> Self {
        self.estimated_duration = Some(duration);
        self
    }
    
    pub fn with_max_retries(mut self, max_retries: u8) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    /// Check if deadline has been exceeded
    pub fn is_overdue(&self) -> bool {
        match &self.deadline {
            TaskDeadline::Hard(duration) | TaskDeadline::Soft(duration) => {
                self.created_at.elapsed() > *duration
            }
            TaskDeadline::None => false,
        }
    }
    
    /// Get time remaining until deadline
    pub fn time_remaining(&self) -> Option<Duration> {
        match &self.deadline {
            TaskDeadline::Hard(duration) | TaskDeadline::Soft(duration) => {
                duration.checked_sub(self.created_at.elapsed())
            }
            TaskDeadline::None => None,
        }
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (higher priority first)
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // If same priority, check deadlines
                let self_overdue = self.is_overdue();
                let other_overdue = other.is_overdue();
                
                match (self_overdue, other_overdue) {
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    _ => {
                        // Both overdue or both not overdue - use creation time
                        other.created_at.cmp(&self.created_at)
                    }
                }
            }
            ordering => ordering,
        }
    }
}

/// Task execution result
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub duration: Duration,
    pub overdue: bool,
    pub error: Option<String>,
}

/// Task scheduler with priority queue
pub struct TaskScheduler {
    /// Priority queue of pending tasks
    queue: Arc<RwLock<BinaryHeap<ScheduledTask>>>,
    
    /// Execution statistics
    stats: Arc<RwLock<SchedulerStats>>,
    
    /// Maximum concurrent tasks
    max_concurrent: usize,
    
    /// Current executing tasks count
    executing: Arc<RwLock<usize>>,
}

#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    pub tasks_scheduled: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_overdue: u64,
    pub tasks_retried: u64,
    pub total_execution_time_ms: u64,
    pub avg_wait_time_ms: u64,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            stats: Arc::new(RwLock::new(SchedulerStats::default())),
            max_concurrent,
            executing: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Schedule a task for execution
    pub async fn schedule(&self, task: ScheduledTask) -> Result<()> {
        let mut queue = self.queue.write().await;
        
        log::debug!("[SCHEDULER] Scheduling task: {} (priority: {:?})", task.name, task.priority);
        
        queue.push(task);
        
        let mut stats = self.stats.write().await;
        stats.tasks_scheduled += 1;
        
        Ok(())
    }
    
    /// Run the scheduler loop
    pub async fn run(&self) -> Result<()> {
        log::info!("[SCHEDULER] Starting scheduler loop");
        
        loop {
            // Check if we can execute more tasks
            let executing_count = *self.executing.read().await;
            if executing_count >= self.max_concurrent {
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
            
            // Get next task
            let task = {
                let mut queue = self.queue.write().await;
                queue.pop()
            };
            
            if let Some(task) = task {
                // Increment executing count
                {
                    let mut executing = self.executing.write().await;
                    *executing += 1;
                }
                
                // Execute task in separate tokio task
                let executing = self.executing.clone();
                let stats = self.stats.clone();
                
                tokio::spawn(async move {
                    let start = Instant::now();
                    let wait_time = task.created_at.elapsed();
                    let overdue = task.is_overdue();
                    let task_id = task.id.clone();
                    let task_name = task.name.clone();
                    
                    // Execute task
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (task.task)()
                    }));
                    
                    let duration = start.elapsed();
                    let success = result.is_ok() && result.unwrap().is_ok();
                    
                    // Update stats
                    let mut stats_write = stats.write().await;
                    if success {
                        stats_write.tasks_completed += 1;
                    } else {
                        stats_write.tasks_failed += 1;
                    }
                    if overdue {
                        stats_write.tasks_overdue += 1;
                    }
                    stats_write.total_execution_time_ms += duration.as_millis() as u64;
                    stats_write.avg_wait_time_ms = 
                        (stats_write.avg_wait_time_ms + wait_time.as_millis() as u64) / 2;
                    
                    log::debug!("[SCHEDULER] Task {} ({}) completed in {:?} (overdue: {})", 
                        task_id, task_name, duration, overdue);
                    
                    // Decrement executing count
                    let mut executing_write = executing.write().await;
                    *executing_write -= 1;
                });
            } else {
                // No tasks, wait a bit
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
    
    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        self.stats.read().await.clone()
    }
    
    /// Get number of pending tasks
    pub async fn pending_count(&self) -> usize {
        self.queue.read().await.len()
    }
    
    /// Get number of executing tasks
    pub async fn executing_count(&self) -> usize {
        *self.executing.read().await
    }
    
    /// Clear all pending tasks
    pub async fn clear(&self) {
        let mut queue = self.queue.write().await;
        queue.clear();
    }
}

/// Helper function to create a render task (highest priority, 16ms deadline)
pub fn create_render_task<F>(name: String, task: F) -> ScheduledTask
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    ScheduledTask::new(name, TaskPriority::Render, task)
        .with_deadline(TaskDeadline::Hard(Duration::from_millis(16)))
        .with_estimated_duration(Duration::from_millis(10))
}

/// Helper function to create an interactive task (high priority, 100ms soft deadline)
pub fn create_interactive_task<F>(name: String, task: F) -> ScheduledTask
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    ScheduledTask::new(name, TaskPriority::Interactive, task)
        .with_deadline(TaskDeadline::Soft(Duration::from_millis(100)))
}

/// Helper function to create a background task (low priority, no deadline)
pub fn create_background_task<F>(name: String, task: F) -> ScheduledTask
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    ScheduledTask::new(name, TaskPriority::Background, task)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_priority_ordering() {
        let task1 = ScheduledTask::new("task1".to_string(), TaskPriority::Render, || Ok(()));
        let task2 = ScheduledTask::new("task2".to_string(), TaskPriority::Background, || Ok(()));
        
        assert!(task1 > task2);
    }
    
    #[test]
    fn test_task_deadline() {
        let task = ScheduledTask::new("task".to_string(), TaskPriority::Render, || Ok(()))
            .with_deadline(TaskDeadline::Hard(Duration::from_millis(100)));
        
        assert!(!task.is_overdue());
    }
    
    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = TaskScheduler::new(4);
        assert_eq!(scheduler.pending_count().await, 0);
        assert_eq!(scheduler.executing_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_schedule_task() {
        let scheduler = TaskScheduler::new(4);
        
        let task = ScheduledTask::new(
            "test_task".to_string(),
            TaskPriority::Normal,
            || Ok(())
        );
        
        scheduler.schedule(task).await.unwrap();
        
        assert_eq!(scheduler.pending_count().await, 1);
    }
    
    #[test]
    fn test_render_task_creation() {
        let task = create_render_task("render".to_string(), || Ok(()));
        
        assert_eq!(task.priority, TaskPriority::Render);
        assert!(matches!(task.deadline, TaskDeadline::Hard(_)));
    }
}
