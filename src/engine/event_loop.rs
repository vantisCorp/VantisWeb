//! Event Loop
//! 
//! Event loop implementation:
//! - Event queue management
//! - Microtask queue
//! - Macrotask queue
//! - Event handling logic
//! - Timer management

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Mutex};

use crate::core::kernel::VantisKernel;

/// Task type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    /// Microtask (higher priority)
    Microtask,
    /// Macrotask (normal priority)
    Macrotask,
    /// Timer task
    Timer,
    /// Render task
    Render,
    /// Network task
    Network,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// Waiting to be executed
    Pending,
    /// Currently executing
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed(String),
}

/// Event loop task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier
    pub id: String,
    /// Task type
    pub task_type: TaskType,
    /// Task function name (for debugging)
    pub name: String,
    /// Task status
    pub status: TaskStatus,
    /// Creation timestamp
    pub created_at: i64,
    /// Execution timestamp
    pub executed_at: Option<i64>,
    /// Completion timestamp
    pub completed_at: Option<i64>,
    /// Priority (0 = highest, larger numbers = lower priority)
    pub priority: u32,
}

impl Task {
    /// Create a new task
    pub fn new(task_type: TaskType, name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_type,
            name,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now().timestamp_millis(),
            executed_at: None,
            completed_at: None,
            priority: 0,
        }
    }

    /// Set task priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

/// Timer entry
#[derive(Debug, Clone)]
pub struct TimerEntry {
    /// Task to execute
    pub task: Task,
    /// When to execute the timer
    pub execute_at: Instant,
    /// Interval for repeating timers
    pub interval: Option<Duration>,
    /// Whether timer is canceled
    pub canceled: bool,
}

/// Event loop statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLoopStats {
    /// Total microtasks processed
    pub microtasks_processed: u64,
    /// Total macrotasks processed
    pub macrotasks_processed: u64,
    /// Total timers processed
    pub timers_processed: u64,
    /// Total tasks processed
    pub total_tasks_processed: u64,
    /// Current queue sizes
    pub microtask_queue_size: usize,
    pub macrotask_queue_size: usize,
    pub timer_queue_size: usize,
    /// Average processing time (milliseconds)
    pub avg_processing_time_ms: f64,
}

/// Event Loop
pub struct EventLoop {
    kernel: Arc<VantisKernel>,
    /// Microtask queue (higher priority)
    microtask_queue: Arc<Mutex<VecDeque<Task>>>,
    /// Macrotask queue (normal priority)
    macrotask_queue: Arc<Mutex<VecDeque<Task>>>,
    /// Timer queue
    timer_queue: Arc<Mutex<Vec<TimerEntry>>>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Statistics
    stats: Arc<RwLock<EventLoopStats>>,
    /// Task completion channel
    task_tx: mpsc::UnboundedSender<Task>,
    task_rx: Arc<Mutex<mpsc::UnboundedReceiver<Task>>>,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new(kernel: Arc<VantisKernel>) -> Self {
        info!("Initializing Event Loop...");

        let (task_tx, task_rx) = mpsc::unbounded_channel();

        Self {
            kernel,
            microtask_queue: Arc::new(Mutex::new(VecDeque::new())),
            macrotask_queue: Arc::new(Mutex::new(VecDeque::new())),
            timer_queue: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(EventLoopStats {
                microtasks_processed: 0,
                macrotasks_processed: 0,
                timers_processed: 0,
                total_tasks_processed: 0,
                microtask_queue_size: 0,
                macrotask_queue_size: 0,
                timer_queue_size: 0,
                avg_processing_time_ms: 0.0,
            })),
            task_tx,
            task_rx: Arc::new(Mutex::new(task_rx)),
        }
    }

    /// Start the event loop
    pub async fn start(&self) -> Result<()> {
        info!("Starting Event Loop...");

        *self.running.write().await = true;

        // Start the event loop runner
        let kernel = self.kernel.clone();
        let microtask_queue = self.microtask_queue.clone();
        let macrotask_queue = self.macrotask_queue.clone();
        let timer_queue = self.timer_queue.clone();
        let running = self.running.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            Self::run_event_loop(
                kernel,
                microtask_queue,
                macrotask_queue,
                timer_queue,
                running,
                stats,
            )
            .await;
        });

        info!("Event Loop started");

        Ok(())
    }

    /// Stop the event loop
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Event Loop...");

        *self.running.write().await = false;

        info!("Event Loop stopped");

        Ok(())
    }

    /// Queue a microtask
    pub async fn queue_microtask(&self, name: String) -> Result<String> {
        debug!("Queueing microtask: {}", name);

        let mut task = Task::new(TaskType::Microtask, name);
        task.id = uuid::Uuid::new_v4().to_string();
        
        let task_id = task.id.clone();

        self.microtask_queue
            .lock()
            .await
            .push_back(task);

        Ok(task_id)
    }

    /// Queue a macrotask
    pub async fn queue_macrotask(&self, name: String, priority: u32) -> Result<String> {
        debug!("Queueing macrotask: {} (priority: {})", name, priority);

        let mut task = Task::new(TaskType::Macrotask, name).with_priority(priority);
        task.id = uuid::Uuid::new_v4().to_string();
        
        let task_id = task.id.clone();

        // Insert based on priority
        let mut queue = self.macrotask_queue.lock().await;
        let mut insert_pos = queue.len();
        
        for (i, existing_task) in queue.iter().enumerate() {
            if task.priority < existing_task.priority {
                insert_pos = i;
                break;
            }
        }
        
        queue.insert(insert_pos, task);

        Ok(task_id)
    }

    /// Set a timeout (one-shot timer)
    pub async fn set_timeout(&self, name: String, duration: Duration) -> Result<String> {
        debug!("Setting timeout: {} (duration: {:?})", name, duration);

        let task = Task::new(TaskType::Timer, name);
        let task_id = task.id.clone();

        let entry = TimerEntry {
            task,
            execute_at: Instant::now() + duration,
            interval: None,
            canceled: false,
        };

        self.timer_queue.lock().await.push(entry);

        Ok(task_id)
    }

    /// Set an interval (repeating timer)
    pub async fn set_interval(&self, name: String, interval: Duration) -> Result<String> {
        debug!("Setting interval: {} (interval: {:?})", name, interval);

        let task = Task::new(TaskType::Timer, name);
        let task_id = task.id.clone();

        let entry = TimerEntry {
            task,
            execute_at: Instant::now() + interval,
            interval: Some(interval),
            canceled: false,
        };

        self.timer_queue.lock().await.push(entry);

        Ok(task_id)
    }

    /// Cancel a timer
    pub async fn cancel_timer(&self, task_id: String) -> Result<bool> {
        debug!("Canceling timer: {}", task_id);

        let mut queue = self.timer_queue.lock().await;
        
        for entry in queue.iter_mut() {
            if entry.task.id == task_id {
                entry.canceled = true;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get event loop statistics
    pub async fn get_stats(&self) -> EventLoopStats {
        let stats = self.stats.read().await.clone();

        // Update queue sizes
        let mut updated_stats = stats;
        updated_stats.microtask_queue_size = self.microtask_queue.lock().await.len();
        updated_stats.macrotask_queue_size = self.macrotask_queue.lock().await.len();
        updated_stats.timer_queue_size = self.timer_queue.lock().await.len();

        updated_stats
    }

    /// Run the event loop (internal)
    async fn run_event_loop(
        _kernel: Arc<VantisKernel>,
        microtask_queue: Arc<Mutex<VecDeque<Task>>>,
        macrotask_queue: Arc<Mutex<VecDeque<Task>>>,
        timer_queue: Arc<Mutex<VecDeque<TimerEntry>>>,
        running: Arc<RwLock<bool>>,
        stats: Arc<RwLock<EventLoopStats>>,
    ) {
        let mut tick_count = 0u64;

        while *running.read().await {
            tick_count += 1;
            
            // Process microtasks first (highest priority)
            Self::process_microtasks(&microtask_queue, &stats).await;

            // Process timers
            Self::process_timers(&timer_queue, &stats).await;

            // Process macrotasks
            Self::process_macrotasks(&macrotask_queue, &stats).await;

            // Small sleep to prevent busy-waiting
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        info!("Event loop exited after {} ticks", tick_count);
    }

    /// Process microtasks
    async fn process_microtasks(
        queue: &Arc<Mutex<VecDeque<Task>>>,
        stats: &Arc<RwLock<EventLoopStats>>,
    ) {
        loop {
            let task = queue.lock().await.pop_front();

            match task {
                Some(mut task) => {
                    // In production: Execute the task
                    // For MVP: Simulate execution
                    task.status = TaskStatus::Completed;
                    task.completed_at = Some(chrono::Utc::now().timestamp_millis());

                    // Update stats
                    let mut s = stats.write().await;
                    s.microtasks_processed += 1;
                    s.total_tasks_processed += 1;
                }
                None => break,
            }
        }
    }

    /// Process timers
    async fn process_timers(
        queue: &Arc<Mutex<VecDeque<TimerEntry>>>,
        stats: &Arc<RwLock<EventLoopStats>>,
    ) {
        let now = Instant::now();
        let mut new_timers = VecDeque::new();

        while let Some(mut entry) = queue.lock().await.pop_front() {
            if entry.canceled {
                continue;
            }

            if now >= entry.execute_at {
                // Execute timer
                entry.task.status = TaskStatus::Completed;
                entry.task.completed_at = Some(chrono::Utc::now().timestamp_millis());

                // Update stats
                let mut s = stats.write().await;
                s.timers_processed += 1;
                s.total_tasks_processed += 1;

                // If interval, reschedule
                if let Some(interval) = entry.interval {
                    entry.execute_at = now + interval;
                    entry.task.status = TaskStatus::Pending;
                    entry.task.created_at = chrono::Utc::now().timestamp_millis();
                    entry.task.executed_at = None;
                    entry.task.completed_at = None;
                    new_timers.push_back(entry);
                }
            } else {
                // Not yet ready, put back
                new_timers.push_back(entry);
            }
        }

        // Put unexecuted timers back
        queue.lock().await.extend(new_timers);
    }

    /// Process macrotasks
    async fn process_macrotasks(
        queue: &Arc<Mutex<VecDeque<Task>>>,
        stats: &Arc<RwLock<EventLoopStats>>,
    ) {
        let task = queue.lock().await.pop_front();

        if let Some(mut task) = task {
            // In production: Execute the task
            // For MVP: Simulate execution
            task.status = TaskStatus::Completed;
            task.completed_at = Some(chrono::Utc::now().timestamp_millis());

            // Update stats
            let mut s = stats.write().await;
            s.macrotasks_processed += 1;
            s.total_tasks_processed += 1;
        }
    }

    /// Check if event loop is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_event_loop() -> EventLoop {
        let kernel = Arc::new(VantisKernel::new().await.unwrap());
        EventLoop::new(kernel)
    }

    #[tokio::test]
    async fn test_event_loop_creation() {
        let event_loop = create_event_loop().await;
        assert!(!event_loop.is_running().await);
    }

    #[tokio::test]
    async fn test_queue_microtask() {
        let event_loop = create_event_loop().await;

        let task_id = event_loop
            .queue_microtask("test_microtask".to_string())
            .await
            .unwrap();

        assert!(!task_id.is_empty());

        let stats = event_loop.get_stats().await;
        assert_eq!(stats.microtask_queue_size, 1);
    }

    #[tokio::test]
    async fn test_queue_macrotask() {
        let event_loop = create_event_loop().await;

        let task_id = event_loop
            .queue_macrotask("test_macrotask".to_string(), 5)
            .await
            .unwrap();

        assert!(!task_id.is_empty());

        let stats = event_loop.get_stats().await;
        assert_eq!(stats.macrotask_queue_size, 1);
    }

    #[tokio::test]
    async fn test_set_timeout() {
        let event_loop = create_event_loop().await;

        let task_id = event_loop
            .set_timeout("test_timeout".to_string(), Duration::from_millis(100))
            .await
            .unwrap();

        assert!(!task_id.is_empty());

        let stats = event_loop.get_stats().await;
        assert_eq!(stats.timer_queue_size, 1);
    }

    #[tokio::test]
    async fn test_set_interval() {
        let event_loop = create_event_loop().await;

        let task_id = event_loop
            .set_interval("test_interval".to_string(), Duration::from_millis(100))
            .await
            .unwrap();

        assert!(!task_id.is_empty());

        let stats = event_loop.get_stats().await;
        assert_eq!(stats.timer_queue_size, 1);
    }

    #[tokio::test]
    async fn test_cancel_timer() {
        let event_loop = create_event_loop().await;

        let task_id = event_loop
            .set_timeout("test_timeout".to_string(), Duration::from_millis(100))
            .await
            .unwrap();

        let canceled = event_loop.cancel_timer(task_id).await.unwrap();
        assert!(canceled);
    }

    #[tokio::test]
    async fn test_event_loop_start_stop() {
        let event_loop = create_event_loop().await;

        event_loop.start().await.unwrap();
        assert!(event_loop.is_running().await);

        event_loop.stop().await.unwrap();
        assert!(!event_loop.is_running().await);
    }

    #[tokio::test]
    async fn test_macrotask_priority() {
        let event_loop = create_event_loop().await;

        event_loop
            .queue_macrotask("low_priority".to_string(), 10)
            .await
            .unwrap();

        event_loop
            .queue_macrotask("high_priority".to_string(), 1)
            .await
            .unwrap();

        event_loop
            .queue_macrotask("medium_priority".to_string(), 5)
            .await
            .unwrap();

        // Check order (high -> medium -> low)
        let queue = event_loop.macrotask_queue.lock().await;
        assert_eq!(queue[0].name, "high_priority");
        assert_eq!(queue[1].name, "medium_priority");
        assert_eq!(queue[2].name, "low_priority");
    }
}