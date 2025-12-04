// Task Scheduler for Kāraṇa OS
// Manages background tasks and work prioritization for AR glasses

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskPriority {
    /// Critical system tasks
    Critical = 0,
    /// High priority (user-facing)
    High = 1,
    /// Normal priority
    Normal = 2,
    /// Low priority (background)
    Low = 3,
    /// Idle - run only when system is idle
    Idle = 4,
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskState {
    Pending,
    Ready,
    Running,
    Blocked,
    Completed,
    Cancelled,
    Failed,
}

/// Task type categorization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskType {
    /// One-shot task
    OneShot,
    /// Periodic task
    Periodic { interval_ms: u64 },
    /// Deferred task
    Deferred { delay_ms: u64 },
    /// Batched task (combine with similar)
    Batched,
}

/// A scheduled task
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub task_id: String,
    pub name: String,
    pub priority: TaskPriority,
    pub task_type: TaskType,
    pub state: TaskState,
    pub deadline_ms: Option<u64>,
    pub estimated_duration_ms: u64,
    pub cpu_affinity: Option<u32>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}

impl ScheduledTask {
    pub fn new(task_id: &str, name: &str, priority: TaskPriority) -> Self {
        Self {
            task_id: task_id.to_string(),
            name: name.to_string(),
            priority,
            task_type: TaskType::OneShot,
            state: TaskState::Pending,
            deadline_ms: None,
            estimated_duration_ms: 10,
            cpu_affinity: None,
            created_at: 0,
            started_at: None,
            completed_at: None,
        }
    }

    pub fn with_deadline(mut self, deadline_ms: u64) -> Self {
        self.deadline_ms = Some(deadline_ms);
        self
    }

    pub fn with_type(mut self, task_type: TaskType) -> Self {
        self.task_type = task_type;
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.estimated_duration_ms = duration_ms;
        self
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl Eq for ScheduledTask {}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower priority value = higher priority (reversed for BinaryHeap max-heap)
        other.priority.cmp(&self.priority)
            .then_with(|| {
                // Earlier deadline is higher priority
                match (&self.deadline_ms, &other.deadline_ms) {
                    (Some(a), Some(b)) => b.cmp(a),
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    (None, None) => Ordering::Equal,
                }
            })
    }
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Work stealing queue for load balancing
#[derive(Debug)]
pub struct WorkQueue {
    pub queue_id: u32,
    tasks: Vec<ScheduledTask>,
    capacity: usize,
}

impl WorkQueue {
    pub fn new(queue_id: u32, capacity: usize) -> Self {
        Self {
            queue_id,
            tasks: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, task: ScheduledTask) -> bool {
        if self.tasks.len() < self.capacity {
            self.tasks.push(task);
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<ScheduledTask> {
        self.tasks.pop()
    }

    pub fn steal(&mut self) -> Option<ScheduledTask> {
        if self.tasks.len() > 1 {
            Some(self.tasks.remove(0))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

/// Task scheduler managing work distribution
pub struct TaskScheduler {
    ready_queue: BinaryHeap<ScheduledTask>,
    tasks: HashMap<String, ScheduledTask>,
    work_queues: Vec<WorkQueue>,
    running_tasks: Vec<String>,
    max_concurrent: usize,
    current_time_ms: u64,
    tasks_completed: u64,
    tasks_cancelled: u64,
    total_wait_time_ms: u64,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self::with_queues(4, 32)
    }

    pub fn with_queues(queue_count: usize, queue_capacity: usize) -> Self {
        let work_queues = (0..queue_count)
            .map(|i| WorkQueue::new(i as u32, queue_capacity))
            .collect();

        Self {
            ready_queue: BinaryHeap::new(),
            tasks: HashMap::new(),
            work_queues,
            running_tasks: Vec::new(),
            max_concurrent: queue_count,
            current_time_ms: 0,
            tasks_completed: 0,
            tasks_cancelled: 0,
            total_wait_time_ms: 0,
        }
    }

    pub fn schedule(&mut self, mut task: ScheduledTask) {
        task.created_at = self.current_time_ms;
        task.state = TaskState::Ready;
        let id = task.task_id.clone();
        self.tasks.insert(id, task.clone());
        self.ready_queue.push(task);
    }

    pub fn cancel(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            if task.state != TaskState::Running {
                task.state = TaskState::Cancelled;
                self.tasks_cancelled += 1;
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self, delta_ms: u64) {
        self.current_time_ms += delta_ms;
        self.process_ready_queue();
    }

    fn process_ready_queue(&mut self) {
        while self.running_tasks.len() < self.max_concurrent {
            if let Some(mut task) = self.ready_queue.pop() {
                if let Some(stored) = self.tasks.get(&task.task_id) {
                    if stored.state == TaskState::Cancelled {
                        continue;
                    }
                }
                
                task.state = TaskState::Running;
                task.started_at = Some(self.current_time_ms);
                self.total_wait_time_ms += self.current_time_ms - task.created_at;
                self.running_tasks.push(task.task_id.clone());
                self.tasks.insert(task.task_id.clone(), task);
            } else {
                break;
            }
        }
    }

    pub fn complete_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.state = TaskState::Completed;
            task.completed_at = Some(self.current_time_ms);
            self.running_tasks.retain(|id| id != task_id);
            self.tasks_completed += 1;

            // Handle periodic tasks
            if let TaskType::Periodic { interval_ms } = task.task_type {
                let mut new_task = task.clone();
                new_task.state = TaskState::Ready;
                new_task.created_at = self.current_time_ms;
                new_task.started_at = None;
                new_task.completed_at = None;
                if let Some(deadline) = new_task.deadline_ms {
                    new_task.deadline_ms = Some(deadline + interval_ms);
                }
                self.ready_queue.push(new_task);
            }
            
            return true;
        }
        false
    }

    pub fn fail_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.state = TaskState::Failed;
            self.running_tasks.retain(|id| id != task_id);
            return true;
        }
        false
    }

    pub fn get_task(&self, task_id: &str) -> Option<&ScheduledTask> {
        self.tasks.get(task_id)
    }

    pub fn get_running_tasks(&self) -> &[String] {
        &self.running_tasks
    }

    pub fn get_pending_count(&self) -> usize {
        self.ready_queue.len()
    }

    pub fn get_running_count(&self) -> usize {
        self.running_tasks.len()
    }

    pub fn get_completed_count(&self) -> u64 {
        self.tasks_completed
    }

    pub fn get_cancelled_count(&self) -> u64 {
        self.tasks_cancelled
    }

    pub fn get_average_wait_time(&self) -> f64 {
        if self.tasks_completed == 0 {
            return 0.0;
        }
        self.total_wait_time_ms as f64 / self.tasks_completed as f64
    }

    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max;
    }

    pub fn get_max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    pub fn balance_work_queues(&mut self) {
        // Find queues with most and least work
        let mut loads: Vec<_> = self.work_queues.iter()
            .enumerate()
            .map(|(i, q)| (i, q.len()))
            .collect();
        loads.sort_by_key(|(_, len)| *len);

        if loads.len() < 2 {
            return;
        }

        let (min_idx, min_len) = loads[0];
        let (max_idx, max_len) = loads[loads.len() - 1];

        // Steal work if imbalance is significant
        if max_len > min_len + 2 {
            if let Some(task) = self.work_queues[max_idx].steal() {
                self.work_queues[min_idx].push(task);
            }
        }
    }

    pub fn get_queue(&self, queue_id: u32) -> Option<&WorkQueue> {
        self.work_queues.iter().find(|q| q.queue_id == queue_id)
    }

    pub fn clear_completed(&mut self) {
        self.tasks.retain(|_, task| {
            task.state != TaskState::Completed && task.state != TaskState::Cancelled
        });
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = TaskScheduler::new();
        assert_eq!(scheduler.get_pending_count(), 0);
        assert_eq!(scheduler.get_running_count(), 0);
    }

    #[test]
    fn test_schedule_task() {
        let mut scheduler = TaskScheduler::new();
        let task = ScheduledTask::new("task1", "Test Task", TaskPriority::Normal);
        scheduler.schedule(task);
        
        assert_eq!(scheduler.get_pending_count(), 1);
    }

    #[test]
    fn test_task_execution() {
        let mut scheduler = TaskScheduler::new();
        let task = ScheduledTask::new("task1", "Test Task", TaskPriority::Normal);
        scheduler.schedule(task);
        
        scheduler.tick(10);
        
        assert_eq!(scheduler.get_running_count(), 1);
        assert_eq!(scheduler.get_pending_count(), 0);
    }

    #[test]
    fn test_task_completion() {
        let mut scheduler = TaskScheduler::new();
        let task = ScheduledTask::new("task1", "Test Task", TaskPriority::Normal);
        scheduler.schedule(task);
        scheduler.tick(10);
        
        assert!(scheduler.complete_task("task1"));
        assert_eq!(scheduler.get_completed_count(), 1);
        assert_eq!(scheduler.get_running_count(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let mut scheduler = TaskScheduler::with_queues(1, 10);
        
        scheduler.schedule(ScheduledTask::new("low", "Low", TaskPriority::Low));
        scheduler.schedule(ScheduledTask::new("high", "High", TaskPriority::High));
        scheduler.schedule(ScheduledTask::new("critical", "Critical", TaskPriority::Critical));
        
        scheduler.tick(10);
        
        let running = scheduler.get_running_tasks();
        assert_eq!(running[0], "critical");
    }

    #[test]
    fn test_cancel_task() {
        let mut scheduler = TaskScheduler::new();
        let task = ScheduledTask::new("task1", "Test Task", TaskPriority::Normal);
        scheduler.schedule(task);
        
        assert!(scheduler.cancel("task1"));
        assert_eq!(scheduler.get_cancelled_count(), 1);
    }

    #[test]
    fn test_periodic_task() {
        let mut scheduler = TaskScheduler::with_queues(1, 10);
        let task = ScheduledTask::new("periodic", "Periodic Task", TaskPriority::Normal)
            .with_type(TaskType::Periodic { interval_ms: 100 });
        scheduler.schedule(task);
        
        scheduler.tick(10);
        scheduler.complete_task("periodic");
        
        // Should reschedule
        assert_eq!(scheduler.get_pending_count(), 1);
    }

    #[test]
    fn test_max_concurrent() {
        let mut scheduler = TaskScheduler::with_queues(2, 10);
        scheduler.set_max_concurrent(2);
        
        for i in 0..5 {
            let task = ScheduledTask::new(&format!("task{}", i), "Task", TaskPriority::Normal);
            scheduler.schedule(task);
        }
        
        scheduler.tick(10);
        
        assert_eq!(scheduler.get_running_count(), 2);
        assert_eq!(scheduler.get_pending_count(), 3);
    }

    #[test]
    fn test_deadline_priority() {
        let mut scheduler = TaskScheduler::with_queues(1, 10);
        
        scheduler.schedule(
            ScheduledTask::new("late", "Late", TaskPriority::Normal)
                .with_deadline(1000)
        );
        scheduler.schedule(
            ScheduledTask::new("early", "Early", TaskPriority::Normal)
                .with_deadline(100)
        );
        
        scheduler.tick(10);
        
        let running = scheduler.get_running_tasks();
        assert_eq!(running[0], "early");
    }

    #[test]
    fn test_work_queue() {
        let mut queue = WorkQueue::new(0, 10);
        
        let task = ScheduledTask::new("task1", "Task", TaskPriority::Normal);
        assert!(queue.push(task));
        assert_eq!(queue.len(), 1);
        
        let popped = queue.pop();
        assert!(popped.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_work_stealing() {
        let mut queue = WorkQueue::new(0, 10);
        
        queue.push(ScheduledTask::new("task1", "Task 1", TaskPriority::Normal));
        queue.push(ScheduledTask::new("task2", "Task 2", TaskPriority::Normal));
        
        let stolen = queue.steal();
        assert!(stolen.is_some());
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_average_wait_time() {
        let mut scheduler = TaskScheduler::with_queues(4, 10);
        
        for i in 0..5 {
            let task = ScheduledTask::new(&format!("task{}", i), "Task", TaskPriority::Normal);
            scheduler.schedule(task);
        }
        
        scheduler.tick(100);
        
        for i in 0..5 {
            scheduler.complete_task(&format!("task{}", i));
        }
        
        let avg_wait = scheduler.get_average_wait_time();
        assert!(avg_wait >= 0.0);
    }

    #[test]
    fn test_fail_task() {
        let mut scheduler = TaskScheduler::new();
        let task = ScheduledTask::new("task1", "Test Task", TaskPriority::Normal);
        scheduler.schedule(task);
        scheduler.tick(10);
        
        assert!(scheduler.fail_task("task1"));
        let task = scheduler.get_task("task1");
        assert_eq!(task.unwrap().state, TaskState::Failed);
    }
}
