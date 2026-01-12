// Kāraṇa OS - Phase 55: Intent-Aware AI Scheduler
// Context-aware model scheduling with idle detection and user pattern learning

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// User interaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionState {
    /// Actively using device (looking at HUD, giving commands)
    Active,
    
    /// Device on but user not actively interacting
    Idle,
    
    /// User in passive monitoring mode (occasional glances)
    Passive,
    
    /// Device in background (user focused elsewhere)
    Background,
    
    /// Device in low-power mode
    Sleep,
}

impl InteractionState {
    /// Get recommended AI update frequency (Hz)
    pub fn update_frequency(&self) -> f32 {
        match self {
            Self::Active => 10.0,      // 100ms updates
            Self::Idle => 1.0,         // 1s updates
            Self::Passive => 0.5,      // 2s updates
            Self::Background => 0.2,   // 5s updates
            Self::Sleep => 0.0,        // No updates
        }
    }
    
    /// Get maximum model complexity allowed
    pub fn max_model_complexity(&self) -> ModelComplexity {
        match self {
            Self::Active => ModelComplexity::Full,
            Self::Idle => ModelComplexity::Standard,
            Self::Passive => ModelComplexity::Lite,
            Self::Background => ModelComplexity::Minimal,
            Self::Sleep => ModelComplexity::None,
        }
    }
}

/// Model complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModelComplexity {
    None,
    Minimal,
    Lite,
    Standard,
    Full,
}

/// AI task priority based on user intent
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IntentPriority {
    /// User explicitly requested this
    Explicit = 5,
    
    /// Likely needed based on context
    Anticipated = 4,
    
    /// Standard background processing
    Background = 3,
    
    /// Speculative pre-computation
    Speculative = 2,
    
    /// Can be deferred indefinitely
    Deferrable = 1,
}

/// User activity pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActivityPattern {
    hour_of_day: u8,
    day_of_week: u8,
    typical_state: InteractionState,
    confidence: f32,
}

/// AI task with intent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAwareTask {
    pub id: String,
    pub model: String,
    pub priority: IntentPriority,
    pub user_intent: Option<String>,
    pub context: TaskContext,
    pub can_defer: bool,
    pub max_latency_ms: Option<u64>,
    pub created_at: u64,
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub interaction_state: InteractionState,
    pub location: Option<String>,
    pub time_of_day: u8,
    pub battery_level: f32,
    pub thermal_state: String,
}

/// Scheduling decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingDecision {
    pub task_id: String,
    pub decision: SchedulingAction,
    pub reason: String,
    pub estimated_completion_s: u64,
}

/// Scheduling action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulingAction {
    /// Execute immediately
    ExecuteNow,
    
    /// Defer until later
    Defer { until_s: u64 },
    
    /// Pause until conditions improve
    Pause { reason: String },
    
    /// Cancel task
    Cancel { reason: String },
    
    /// Downgrade to simpler model
    Downgrade { from: String, to: String },
}

/// Intent-aware scheduler
pub struct IntentScheduler {
    current_state: Arc<RwLock<InteractionState>>,
    activity_history: Arc<RwLock<VecDeque<ActivityPattern>>>,
    pending_tasks: Arc<RwLock<Vec<IntentAwareTask>>>,
    task_history: Arc<RwLock<VecDeque<SchedulingDecision>>>,
    config: SchedulerConfig,
    stats: Arc<RwLock<SchedulerStats>>,
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// How long of inactivity to consider idle (seconds)
    pub idle_threshold_s: u64,
    
    /// Learn from user patterns
    pub pattern_learning_enabled: bool,
    
    /// Maximum deferred tasks
    pub max_deferred_tasks: usize,
    
    /// Aggressiveness of downsampling (0.0 = conservative, 1.0 = aggressive)
    pub downsample_aggressiveness: f32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            idle_threshold_s: 30,
            pattern_learning_enabled: true,
            max_deferred_tasks: 100,
            downsample_aggressiveness: 0.5,
        }
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub tasks_scheduled: usize,
    pub tasks_deferred: usize,
    pub tasks_downgraded: usize,
    pub tasks_cancelled: usize,
    pub avg_defer_time_s: f32,
    pub patterns_learned: usize,
}

impl IntentScheduler {
    /// Create new intent-aware scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            current_state: Arc::new(RwLock::new(InteractionState::Active)),
            activity_history: Arc::new(RwLock::new(VecDeque::new())),
            pending_tasks: Arc::new(RwLock::new(Vec::new())),
            task_history: Arc::new(RwLock::new(VecDeque::new())),
            config,
            stats: Arc::new(RwLock::new(SchedulerStats::default())),
        }
    }
    
    /// Update user interaction state
    pub async fn update_interaction_state(&self, state: InteractionState) {
        *self.current_state.write().await = state;
        
        // Record activity pattern if learning enabled
        if self.config.pattern_learning_enabled {
            self.record_activity_pattern(state).await;
        }
    }
    
    /// Schedule an AI task
    pub async fn schedule_task(&self, task: IntentAwareTask) -> Result<SchedulingDecision> {
        let current_state = *self.current_state.read().await;
        
        let decision = self.make_scheduling_decision(&task, current_state).await?;
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.tasks_scheduled += 1;
        match &decision.decision {
            SchedulingAction::Defer { .. } => stats.tasks_deferred += 1,
            SchedulingAction::Downgrade { .. } => stats.tasks_downgraded += 1,
            SchedulingAction::Cancel { .. } => stats.tasks_cancelled += 1,
            _ => {}
        }
        drop(stats);
        
        // Record decision
        let mut history = self.task_history.write().await;
        history.push_back(decision.clone());
        if history.len() > 1000 {
            history.pop_front();
        }
        drop(history);
        
        // Add to pending if deferred
        if matches!(decision.decision, SchedulingAction::Defer { .. }) {
            let mut pending = self.pending_tasks.write().await;
            pending.push(task);
            
            // Limit pending queue
            if pending.len() > self.config.max_deferred_tasks {
                pending.remove(0); // Remove oldest
            }
        }
        
        Ok(decision)
    }
    
    /// Make scheduling decision for a task
    async fn make_scheduling_decision(
        &self,
        task: &IntentAwareTask,
        state: InteractionState,
    ) -> Result<SchedulingDecision> {
        // Explicit user requests always execute immediately
        if task.priority == IntentPriority::Explicit {
            return Ok(SchedulingDecision {
                task_id: task.id.clone(),
                decision: SchedulingAction::ExecuteNow,
                reason: "Explicit user request".to_string(),
                estimated_completion_s: self.estimate_completion_time(task, state),
            });
        }
        
        // Check if task is latency-sensitive
        if let Some(max_latency) = task.max_latency_ms {
            if max_latency < 100 {
                return Ok(SchedulingDecision {
                    task_id: task.id.clone(),
                    decision: SchedulingAction::ExecuteNow,
                    reason: "Latency-critical task".to_string(),
                    estimated_completion_s: self.estimate_completion_time(task, state),
                });
            }
        }
        
        // Check interaction state
        match state {
            InteractionState::Active => {
                // Execute anticipated and background tasks
                if task.priority >= IntentPriority::Background {
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::ExecuteNow,
                        reason: "User actively engaged".to_string(),
                        estimated_completion_s: self.estimate_completion_time(task, state),
                    })
                } else {
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::Defer {
                            until_s: 60, // Defer speculative tasks
                        },
                        reason: "Low priority, defer to maintain responsiveness".to_string(),
                        estimated_completion_s: 60,
                    })
                }
            }
            
            InteractionState::Idle => {
                // Good time for background processing
                if task.can_defer {
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::ExecuteNow,
                        reason: "Idle state - good for background tasks".to_string(),
                        estimated_completion_s: self.estimate_completion_time(task, state),
                    })
                } else {
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::ExecuteNow,
                        reason: "Non-deferrable task".to_string(),
                        estimated_completion_s: self.estimate_completion_time(task, state),
                    })
                }
            }
            
            InteractionState::Passive | InteractionState::Background => {
                // Downsample or defer expensive tasks
                if task.priority <= IntentPriority::Speculative {
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::Defer {
                            until_s: 300, // 5 minutes
                        },
                        reason: "User not actively engaged".to_string(),
                        estimated_completion_s: 300,
                    })
                } else if self.should_downsample(task, state).await {
                    // Suggest simpler model
                    let lite_model = format!("{}_lite", task.model);
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::Downgrade {
                            from: task.model.clone(),
                            to: lite_model,
                        },
                        reason: "Use lighter model to conserve resources".to_string(),
                        estimated_completion_s: self.estimate_completion_time(task, state) / 2,
                    })
                } else {
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::ExecuteNow,
                        reason: "Background task execution".to_string(),
                        estimated_completion_s: self.estimate_completion_time(task, state),
                    })
                }
            }
            
            InteractionState::Sleep => {
                // Cancel or defer all non-essential tasks
                if task.priority < IntentPriority::Anticipated {
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::Cancel {
                            reason: "Device in sleep mode".to_string(),
                        },
                        reason: "Device in sleep mode".to_string(),
                        estimated_completion_s: 0,
                    })
                } else {
                    Ok(SchedulingDecision {
                        task_id: task.id.clone(),
                        decision: SchedulingAction::Pause {
                            reason: "Wait for wake".to_string(),
                        },
                        reason: "Paused until device wakes".to_string(),
                        estimated_completion_s: 0,
                    })
                }
            }
        }
    }
    
    /// Check if task should be downsampled
    async fn should_downsample(&self, task: &IntentAwareTask, state: InteractionState) -> bool {
        // More aggressive downsampling in low-interaction states
        let state_threshold = match state {
            InteractionState::Active => 0.9,
            InteractionState::Idle => 0.7,
            InteractionState::Passive => 0.5,
            InteractionState::Background => 0.3,
            InteractionState::Sleep => 0.0,
        };
        
        // Check thermal and battery constraints
        let thermal_ok = task.context.thermal_state == "Normal";
        let battery_ok = task.context.battery_level > 30.0;
        
        !thermal_ok || !battery_ok || self.config.downsample_aggressiveness > state_threshold
    }
    
    /// Estimate task completion time
    fn estimate_completion_time(&self, task: &IntentAwareTask, state: InteractionState) -> u64 {
        // Base time depends on model
        let base_time = match task.model.as_str() {
            m if m.contains("whisper") => 100,
            m if m.contains("blip") => 200,
            m if m.contains("llama") => 500,
            _ => 150,
        };
        
        // Adjust for interaction state (lower priority in busy states)
        let state_multiplier = match state {
            InteractionState::Active => 1.5,   // Contention with user tasks
            InteractionState::Idle => 1.0,     // Full resources
            InteractionState::Passive => 1.2,
            InteractionState::Background => 1.3,
            InteractionState::Sleep => 0.5,
        };
        
        (base_time as f32 * state_multiplier) as u64
    }
    
    /// Record activity pattern for learning
    async fn record_activity_pattern(&self, state: InteractionState) {
        use std::time::SystemTime;
        
        // Get current time (simplified - no chrono dependency)
        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Calculate hour (0-23) and day of week (0-6)
        let total_hours = now_secs / 3600;
        let hour = (total_hours % 24) as u8;
        let day = ((total_hours / 24) % 7) as u8;
        
        let pattern = ActivityPattern {
            hour_of_day: hour,
            day_of_week: day,
            typical_state: state,
            confidence: 0.5, // Initial confidence
        };
        
        let mut history = self.activity_history.write().await;
        history.push_back(pattern);
        
        // Keep last 1000 patterns
        if history.len() > 1000 {
            history.pop_front();
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.patterns_learned = history.len();
    }
    
    /// Predict likely interaction state based on learned patterns
    pub async fn predict_interaction_state(&self) -> Option<(InteractionState, f32)> {
        if !self.config.pattern_learning_enabled {
            return None;
        }
        
        let history = self.activity_history.read().await;
        if history.len() < 10 {
            return None; // Not enough data
        }
        
        use std::time::SystemTime;
        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let total_hours = now_secs / 3600;
        let current_hour = (total_hours % 24) as u8;
        let current_day = ((total_hours / 24) % 7) as u8;
        
        // Find similar time patterns
        let similar: Vec<&ActivityPattern> = history
            .iter()
            .filter(|p| {
                let hour_diff = (p.hour_of_day as i16 - current_hour as i16).abs();
                let day_match = p.day_of_week == current_day;
                hour_diff <= 1 && day_match
            })
            .collect();
        
        if similar.is_empty() {
            return None;
        }
        
        // Count state occurrences
        let mut state_counts: HashMap<InteractionState, usize> = HashMap::new();
        for pattern in &similar {
            *state_counts.entry(pattern.typical_state).or_insert(0) += 1;
        }
        
        // Find most common state
        let (state, count) = state_counts
            .iter()
            .max_by_key(|(_, count)| *count)?;
        
        let confidence = *count as f32 / similar.len() as f32;
        
        Some((*state, confidence))
    }
    
    /// Get statistics
    pub async fn stats(&self) -> SchedulerStats {
        self.stats.read().await.clone()
    }
    
    /// Get pending tasks
    pub async fn pending_tasks(&self) -> Vec<IntentAwareTask> {
        self.pending_tasks.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interaction_state_frequency() {
        assert_eq!(InteractionState::Active.update_frequency(), 10.0);
        assert_eq!(InteractionState::Idle.update_frequency(), 1.0);
        assert_eq!(InteractionState::Sleep.update_frequency(), 0.0);
    }
    
    #[tokio::test]
    async fn test_explicit_task_priority() {
        let scheduler = IntentScheduler::new(SchedulerConfig::default());
        scheduler.update_interaction_state(InteractionState::Sleep).await;
        
        let task = IntentAwareTask {
            id: "task1".to_string(),
            model: "whisper".to_string(),
            priority: IntentPriority::Explicit,
            user_intent: Some("transcribe audio".to_string()),
            context: TaskContext {
                interaction_state: InteractionState::Sleep,
                location: None,
                time_of_day: 14,
                battery_level: 50.0,
                thermal_state: "Normal".to_string(),
            },
            can_defer: true,
            max_latency_ms: None,
            created_at: 0,
        };
        
        let decision = scheduler.schedule_task(task).await.unwrap();
        assert!(matches!(decision.decision, SchedulingAction::ExecuteNow));
    }
    
    #[tokio::test]
    async fn test_defer_speculative_task() {
        let scheduler = IntentScheduler::new(SchedulerConfig::default());
        scheduler.update_interaction_state(InteractionState::Active).await;
        
        let task = IntentAwareTask {
            id: "task2".to_string(),
            model: "llama".to_string(),
            priority: IntentPriority::Speculative,
            user_intent: None,
            context: TaskContext {
                interaction_state: InteractionState::Active,
                location: None,
                time_of_day: 14,
                battery_level: 50.0,
                thermal_state: "Normal".to_string(),
            },
            can_defer: true,
            max_latency_ms: None,
            created_at: 0,
        };
        
        let decision = scheduler.schedule_task(task).await.unwrap();
        assert!(matches!(decision.decision, SchedulingAction::Defer { .. }));
    }
    
    #[tokio::test]
    async fn test_idle_background_processing() {
        let scheduler = IntentScheduler::new(SchedulerConfig::default());
        scheduler.update_interaction_state(InteractionState::Idle).await;
        
        let task = IntentAwareTask {
            id: "task3".to_string(),
            model: "blip".to_string(),
            priority: IntentPriority::Background,
            user_intent: None,
            context: TaskContext {
                interaction_state: InteractionState::Idle,
                location: None,
                time_of_day: 14,
                battery_level: 70.0,
                thermal_state: "Normal".to_string(),
            },
            can_defer: true,
            max_latency_ms: None,
            created_at: 0,
        };
        
        let decision = scheduler.schedule_task(task).await.unwrap();
        assert!(matches!(decision.decision, SchedulingAction::ExecuteNow));
    }
    
    #[tokio::test]
    async fn test_scheduler_stats() {
        let scheduler = IntentScheduler::new(SchedulerConfig::default());
        
        // Schedule various tasks
        for i in 0..5 {
            let task = IntentAwareTask {
                id: format!("task{}", i),
                model: "test".to_string(),
                priority: if i % 2 == 0 {
                    IntentPriority::Explicit
                } else {
                    IntentPriority::Speculative
                },
                user_intent: None,
                context: TaskContext {
                    interaction_state: InteractionState::Active,
                    location: None,
                    time_of_day: 14,
                    battery_level: 50.0,
                    thermal_state: "Normal".to_string(),
                },
                can_defer: true,
                max_latency_ms: None,
                created_at: 0,
            };
            let _ = scheduler.schedule_task(task).await;
        }
        
        let stats = scheduler.stats().await;
        assert_eq!(stats.tasks_scheduled, 5);
        assert!(stats.tasks_deferred > 0);
    }
}
