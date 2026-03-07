//! Unit tests for the Scheduler module

use vantisweb::core::scheduler::Scheduler;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.active_task_count(), 0);
    }

    #[test]
    fn test_scheduler_add_task() {
        let mut scheduler = Scheduler::new();
        let task_id = scheduler.add_task(|| {
            // Simple task
            42
        });
        assert!(!task_id.is_empty());
        assert_eq!(scheduler.active_task_count(), 1);
    }

    #[test]
    fn test_scheduler_remove_task() {
        let mut scheduler = Scheduler::new();
        let task_id = scheduler.add_task(|| 42);
        assert!(scheduler.remove_task(&task_id));
        assert_eq!(scheduler.active_task_count(), 0);
    }

    #[test]
    fn test_scheduler_clear_all() {
        let mut scheduler = Scheduler::new();
        scheduler.add_task(|| 1);
        scheduler.add_task(|| 2);
        scheduler.add_task(|| 3);
        scheduler.clear_all();
        assert_eq!(scheduler.active_task_count(), 0);
    }

    #[test]
    fn test_scheduler_priority() {
        let mut scheduler = Scheduler::new();
        scheduler.add_task_with_priority(|| 1, 1);
        scheduler.add_task_with_priority(|| 2, 5);
        scheduler.add_task_with_priority(|| 3, 3);
        let priorities = scheduler.get_task_priorities();
        assert!(priorities.len() == 3);
    }

    #[test]
    fn test_scheduler_pause_resume() {
        let mut scheduler = Scheduler::new();
        let task_id = scheduler.add_task(|| 42);
        assert!(scheduler.pause_task(&task_id));
        assert!(scheduler.is_task_paused(&task_id));
        assert!(scheduler.resume_task(&task_id));
        assert!(!scheduler.is_task_paused(&task_id));
    }
}