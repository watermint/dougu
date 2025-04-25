use dougu_essentials::time::{
    ZonedDateTime, LocalDate, LocalTime,
    Instant, Duration, Clock, SystemClock, FixedClock
};
use std::collections::BinaryHeap;
use std::cmp::{Ordering, Reverse};
use std::sync::Arc;

/// Represents a task with a scheduled execution time
#[derive(Debug, Clone)]
struct ScheduledTask {
    id: u32,
    name: String,
    execution_time: ZonedDateTime,
    duration: Duration,
}

impl ScheduledTask {
    fn new(id: u32, name: &str, execution_time: ZonedDateTime, duration: Duration) -> Self {
        Self {
            id,
            name: name.to_string(),
            execution_time,
            duration,
        }
    }
    
    fn completion_time(&self) -> ZonedDateTime {
        self.execution_time.plus(self.duration)
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.execution_time.is_equal(&other.execution_time) && self.id == other.id
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
        // First compare by execution time
        match self.execution_time.get_epoch_second().cmp(&other.execution_time.get_epoch_second()) {
            Ordering::Equal => {
                // If execution times are equal, compare by nanoseconds
                match self.execution_time.get_epoch_nano().1.cmp(&other.execution_time.get_epoch_nano().1) {
                    Ordering::Equal => self.id.cmp(&other.id), // If still equal, compare by id
                    other_ordering => other_ordering,
                }
            },
            other_ordering => other_ordering,
        }
    }
}

/// A simple task scheduler that executes tasks at specified times
struct TaskScheduler {
    clock: Arc<dyn Clock>,
    tasks: BinaryHeap<Reverse<ScheduledTask>>,
    executed_tasks: Vec<ScheduledTask>,
}

impl TaskScheduler {
    fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            tasks: BinaryHeap::new(),
            executed_tasks: Vec::new(),
        }
    }
    
    fn schedule_task(&mut self, task: ScheduledTask) {
        self.tasks.push(Reverse(task));
    }
    
    fn run_until(&mut self, end_time: ZonedDateTime) {
        while let Some(Reverse(task)) = self.tasks.peek() {
            if !task.execution_time.is_before(&end_time) {
                break;
            }
            
            let task = self.tasks.pop().unwrap().0;
            self.executed_tasks.push(task);
        }
    }
    
    fn get_executed_tasks(&self) -> &[ScheduledTask] {
        &self.executed_tasks
    }
}

#[test]
fn test_task_scheduler() {
    // Create a fixed clock starting at a specific time
    let start_time = ZonedDateTime::parse("2024-04-25T08:00:00Z").unwrap();
    let clock = Arc::new(FixedClock::new(start_time.clone()));
    
    // Create a task scheduler
    let mut scheduler = TaskScheduler::new(clock.clone());
    
    // Schedule several tasks
    scheduler.schedule_task(ScheduledTask::new(
        1,
        "Morning Meeting",
        start_time.plus(Duration::of_hours(1)),
        Duration::of_minutes(30)
    ));
    
    scheduler.schedule_task(ScheduledTask::new(
        2,
        "Lunch Break",
        start_time.plus(Duration::of_hours(4)),
        Duration::of_hours(1)
    ));
    
    scheduler.schedule_task(ScheduledTask::new(
        3,
        "Project Review",
        start_time.plus(Duration::of_hours(6)),
        Duration::of_hours(2)
    ));
    
    scheduler.schedule_task(ScheduledTask::new(
        4,
        "Quick Check-in",
        start_time.plus(Duration::of_hours(2)),
        Duration::of_minutes(15)
    ));
    
    // Run scheduler until noon
    let noon = start_time.plus(Duration::of_hours(4));
    scheduler.run_until(noon);
    
    // Verify that only the tasks before noon were executed
    let executed = scheduler.get_executed_tasks();
    assert_eq!(executed.len(), 2);
    assert_eq!(executed[0].id, 1); // Morning Meeting at 9:00
    assert_eq!(executed[1].id, 4); // Quick Check-in at 10:00
    
    // Run until end of day
    let end_of_day = start_time.plus(Duration::of_hours(9));
    scheduler.run_until(end_of_day);
    
    // Verify all tasks were executed
    let executed = scheduler.get_executed_tasks();
    assert_eq!(executed.len(), 4);
    
    // Tasks executed in order of scheduled time:
    // 1. Morning Meeting (9:00)
    // 2. Quick Check-in (10:00)
    // 3. Lunch Break (12:00)
    // 4. Project Review (14:00)
    assert_eq!(executed[2].id, 2); // Lunch Break
    assert_eq!(executed[3].id, 3); // Project Review
    
    // Verify task completion time - Morning Meeting should end at 9:30
    let hours_1 = Duration::of_hours(1); 
    let minutes_30 = Duration::of_minutes(30);
    let combined = hours_1.plus(minutes_30);
    let expected_completion = start_time.plus(combined);
    
    assert_eq!(
        executed[0].completion_time().get_epoch_second(),
        expected_completion.get_epoch_second()
    );
}

#[test]
fn test_recurring_tasks() {
    // Create a fixed clock starting at Monday morning
    let monday = LocalDate::parse("2024-04-22").unwrap();
    let monday_morning = ZonedDateTime::of_unix(monday.get_epoch_second() + 8 * 3600).unwrap(); // 8:00 AM
    let clock = Arc::new(FixedClock::new(monday_morning.clone()));
    
    // Create a task scheduler
    let mut scheduler = TaskScheduler::new(clock.clone());
    
    // Schedule a daily standup meeting for 5 days
    let daily_duration = Duration::of_minutes(15);
    for day in 0..5 {
        let day_offset = Duration::of_days(day);
        let hour_offset = Duration::of_hours(1);
        let meeting_time = monday_morning.plus(day_offset.plus(hour_offset));
        scheduler.schedule_task(ScheduledTask::new(
            day as u32 + 1,
            &format!("Daily Standup Day {}", day + 1),
            meeting_time,
            daily_duration
        ));
    }
    
    // Add a weekly planning meeting
    scheduler.schedule_task(ScheduledTask::new(
        100,
        "Weekly Planning",
        monday_morning.plus(Duration::of_hours(2)),
        Duration::of_hours(1)
    ));
    
    // Add a weekly review meeting on Friday
    let friday_morning = monday_morning.plus(Duration::of_days(4));
    scheduler.schedule_task(ScheduledTask::new(
        101,
        "Weekly Review",
        friday_morning.plus(Duration::of_hours(3)),
        Duration::of_hours(1)
    ));
    
    // Run until Wednesday noon
    let wednesday_noon = monday_morning.plus(
        Duration::of_days(2).plus(Duration::of_hours(4))
    );
    scheduler.run_until(wednesday_noon);
    
    // Verify that Monday through Wednesday standups and the weekly planning were executed
    let executed = scheduler.get_executed_tasks();
    assert_eq!(executed.len(), 4); // 3 standups + 1 weekly planning
    
    // Run until end of week
    let end_of_week = friday_morning.plus(Duration::of_hours(8));
    scheduler.run_until(end_of_week);
    
    // Verify all tasks were executed
    let executed = scheduler.get_executed_tasks();
    assert_eq!(executed.len(), 7); // 5 standups + 2 weekly meetings
}

#[test]
fn test_time_conflicts() {
    // Create a fixed clock starting at a specific time
    let start_time = ZonedDateTime::parse("2024-04-25T10:00:00Z").unwrap();
    let _clock = Arc::new(FixedClock::new(start_time.clone()));
    
    // Create two tasks that have overlapping execution times
    let task1 = ScheduledTask::new(
        1,
        "First Task",
        start_time.clone(),
        Duration::of_hours(2)
    );
    
    let task2 = ScheduledTask::new(
        2,
        "Second Task",
        start_time.plus(Duration::of_hours(1)),
        Duration::of_hours(1)
    );
    
    // Verify that the second task starts during the execution of the first task
    assert!(task2.execution_time.is_after(&task1.execution_time));
    assert!(task2.execution_time.is_before(&task1.completion_time()));
    
    // In a real scheduling system, we might want to detect and handle these conflicts
    // This is just a demonstration of how the time components could be used for such checks
} 