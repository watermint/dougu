use dougu_essentials::time::{
    ZonedDateTime, LocalDate, LocalTime, Duration, Period, 
    Instant, Clock, SystemClock, FixedClock, OffsetClock
};
use std::sync::Arc;
use std::thread;

/// Tests the integration between various time components
/// This specifically tests:
/// 1. Different clock implementations working together
/// 2. Time calculations across multiple components
/// 3. Conversion between different time representations
#[test]
fn test_time_components_integration() {
    // Create different clock implementations
    let system_clock = Arc::new(SystemClock::new());
    let fixed_time = ZonedDateTime::now();
    let fixed_clock = Arc::new(FixedClock::new(fixed_time.clone()));
    let offset = Duration::of_hours(2);
    let offset_clock = Arc::new(OffsetClock::new(system_clock.clone(), offset));

    // Test that different clocks can be used interchangeably through trait
    let clocks: Vec<Arc<dyn Clock>> = vec![
        system_clock.clone(),
        fixed_clock.clone(),
        offset_clock.clone(),
    ];

    for clock in &clocks {
        let instant = clock.instant();
        assert!(instant.get_epoch_second() > 0);
    }

    // Test offset clock is consistently ahead of system clock
    let system_time = system_clock.instant();
    let offset_time = offset_clock.instant();
    assert!(offset_time.is_after(&system_time));
    assert_eq!(
        offset_time.get_epoch_second() - system_time.get_epoch_second(),
        offset.get_seconds()
    );

    // Test fixed clock doesn't change over time
    let initial_time = fixed_clock.instant();
    thread::sleep(std::time::Duration::from_millis(10));
    let later_time = fixed_clock.instant();
    assert!(initial_time.is_equal(&later_time));

    // Test conversions between different time representations
    let _now = ZonedDateTime::now();
    let today = LocalDate::now();
    let current_time = LocalTime::now();

    // Convert between types
    let date_instant_seconds = today.get_epoch_second();
    let time_instant_seconds = current_time.get_epoch_second();
    
    // A date should refer to midnight
    assert_eq!(date_instant_seconds % 86400, 0);
    
    // Current time should be between 0 and 24 hours
    assert!(time_instant_seconds >= 0 && time_instant_seconds < 86400);

    // Test date calculations
    let tomorrow = today.plus_days(1);
    assert_eq!(tomorrow.get_epoch_second() - today.get_epoch_second(), 86400);

    // Test time arithmetic
    let hour_later = current_time.plus_hours(1);
    assert_eq!(
        (hour_later.get_epoch_second() - current_time.get_epoch_second()) % 86400,
        3600
    );

    // Test duration calculations
    let duration1 = Duration::of_hours(5);
    let duration2 = Duration::of_minutes(30);
    
    let combined = duration1.plus(duration2);
    assert_eq!(combined.get_minutes(), 5 * 60 + 30);
    
    let multiplied = duration2.multiplied_by(4);
    assert_eq!(multiplied.get_minutes(), 30 * 4);
}

/// Tests time-based scheduling simulations
#[test]
fn test_time_scheduling() {
    // Create a fixed clock at a specific time
    let start_time = ZonedDateTime::now();
    let _clock = Arc::new(FixedClock::new(start_time.clone()));
    
    // Simulate a schedule with different time intervals
    let intervals = vec![
        Duration::of_minutes(15),   // +15 minutes (15 min)
        Duration::of_minutes(30),   // +30 minutes (45 min total)
        Duration::of_hours(1),      // +60 minutes (105 min / 1h 45min total)
        Duration::of_hours(4),      // +240 minutes (345 min / 5h 45min total)
    ];
    
    let mut expected_time = start_time.clone();
    let mut accumulated_seconds = 0;
    
    // Verify that scheduled events occur at the right times
    for interval in intervals {
        // Calculate the next time
        expected_time = expected_time.plus(interval);
        accumulated_seconds += interval.get_seconds();
        
        // Verify the time calculation is correct
        let time_diff = expected_time.get_epoch_second() - start_time.get_epoch_second();
        assert_eq!(time_diff, accumulated_seconds);
    }
    
    // Test date-based scheduling
    let today = LocalDate::now();
    let one_week_later = today.plus_days(7);
    
    // Verify the date calculation is correct
    assert_eq!(
        one_week_later.get_epoch_second() - today.get_epoch_second(),
        7 * 24 * 60 * 60
    );
}

/// Tests handling of time zones and conversions
#[test]
fn test_time_zone_handling() {
    // Get current UTC time
    let utc_now = ZonedDateTime::now();
    
    // Convert to local time
    let local_now = utc_now.to_local();
    
    // Test that representation changes but the instant is the same
    assert_eq!(utc_now.get_epoch_second(), ZonedDateTime::of_local(local_now).get_epoch_second());
    
    // Test formatting
    let formatted = utc_now.format();
    assert!(formatted.contains('T'));
    assert!(formatted.contains('Z') || formatted.contains('+'));
    
    // Parse the formatted string back to a ZonedDateTime
    let parsed = ZonedDateTime::parse(&formatted).unwrap();
    
    // Verify that the parsed time is the same as the original
    assert_eq!(parsed.get_epoch_second(), utc_now.get_epoch_second());
    assert_eq!(parsed.get_epoch_nano().1, utc_now.get_epoch_nano().1);
} 