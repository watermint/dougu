use chrono::NaiveDate;
use dougu_essentials::time::{
    Duration, Instant, Period, ZonedDateTime,
};
use std::collections::HashMap;

/// Represents a timestamped data record
#[derive(Debug, Clone)]
struct TimeseriesRecord {
    timestamp: ZonedDateTime,
    device_id: String,
    value: f64,
}

impl TimeseriesRecord {
    fn new(timestamp: ZonedDateTime, device_id: &str, value: f64) -> Self {
        Self {
            timestamp,
            device_id: device_id.to_string(),
            value,
        }
    }
}

/// A processor for time-series data that performs operations like
/// filtering, aggregation, and statistical analysis
struct TimeseriesProcessor {
    records: Vec<TimeseriesRecord>,
}

impl TimeseriesProcessor {
    fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    fn add_record(&mut self, record: TimeseriesRecord) {
        self.records.push(record);
    }

    fn add_records(&mut self, records: Vec<TimeseriesRecord>) {
        self.records.extend(records);
    }

    /// Filters records that fall within the specified time period
    fn filter_time_range(&self, start: &ZonedDateTime, end: &ZonedDateTime) -> Vec<&TimeseriesRecord> {
        self.records.iter()
            .filter(|record| {
                record.timestamp.is_after(start) &&
                    record.timestamp.is_before(end)
            })
            .collect()
    }

    /// Groups records by a time bucket (e.g., hourly, daily)
    fn group_by_time_bucket(&self, bucket_duration: Duration) -> HashMap<i64, Vec<&TimeseriesRecord>> {
        let mut result = HashMap::new();

        for record in &self.records {
            // Calculate the bucket key by truncating to the nearest bucket
            let bucket_key = (record.timestamp.get_epoch_second() / bucket_duration.get_seconds())
                * bucket_duration.get_seconds();

            result.entry(bucket_key)
                .or_insert_with(Vec::new)
                .push(record);
        }

        result
    }

    /// Calculates average values per device within each time bucket
    fn calculate_averages_by_device(&self, bucket_duration: Duration)
                                    -> HashMap<i64, HashMap<String, f64>> {
        let buckets = self.group_by_time_bucket(bucket_duration);
        let mut result = HashMap::new();

        for (bucket_key, records) in buckets {
            let mut device_sums = HashMap::new();
            let mut device_counts = HashMap::new();

            // Sum values and count records for each device
            for record in records {
                *device_sums.entry(record.device_id.clone()).or_insert(0.0) += record.value;
                *device_counts.entry(record.device_id.clone()).or_insert(0) += 1;
            }

            // Calculate averages
            let mut device_averages = HashMap::new();
            for (device_id, sum) in device_sums {
                let count = *device_counts.get(&device_id).unwrap();
                device_averages.insert(device_id, sum / count as f64);
            }

            result.insert(bucket_key, device_averages);
        }

        result
    }
}

/// Helper function to create a test dataset
fn create_test_dataset() -> Vec<TimeseriesRecord> {
    let base_time = ZonedDateTime::parse("2024-04-25T00:00:00Z").unwrap();
    let mut records = Vec::new();

    // Create records for device A (every hour)
    for hour in 0..24 {
        let timestamp = base_time.plus(Duration::of_hours(hour));
        let value = 20.0 + (hour as f64 * 0.5); // Values increasing over time
        records.push(TimeseriesRecord::new(timestamp, "Device-A", value));
    }

    // Create records for device B (every 30 minutes)
    for half_hour in 0..48 {
        let timestamp = base_time.plus(Duration::of_minutes(half_hour * 30));
        let value = 25.0 + (half_hour as f64 * 0.25); // Different pattern
        records.push(TimeseriesRecord::new(timestamp, "Device-B", value));
    }

    // Create records for device C (every 3 hours, with some missing data)
    for period in 0..8 {
        if period != 3 && period != 5 { // Simulate missing data
            let timestamp = base_time.plus(Duration::of_hours(period * 3));
            let value = 30.0 - (period as f64 * 1.0); // Decreasing pattern
            records.push(TimeseriesRecord::new(timestamp, "Device-C", value));
        }
    }

    records
}

#[test]
fn test_timeseries_filtering() {
    let data = create_test_dataset();
    let processor = TimeseriesProcessor {
        records: data,
    };

    // Filter records from 6 AM to 12 PM
    let start = ZonedDateTime::parse("2024-04-25T06:00:00Z").unwrap();
    let end = ZonedDateTime::parse("2024-04-25T12:00:00Z").unwrap();

    let filtered = processor.filter_time_range(&start, &end);

    // Count expected records in this time range
    // Device A: 6 records (hourly from 6 AM to 11 AM)
    // Device B: 12 records (every 30 mins from 6:00 AM to 11:30 AM)
    // Device C: 2 records (at 6 AM and 9 AM, remembering 2 are missing)
    assert_eq!(filtered.len(), 16);

    // Verify all records are within the time range
    for record in filtered {
        assert!(record.timestamp.is_after(&start));
        assert!(record.timestamp.is_before(&end));
    }
}

#[test]
fn test_timeseries_grouping() {
    let data = create_test_dataset();
    let processor = TimeseriesProcessor {
        records: data,
    };

    // Group by 4-hour periods
    let buckets = processor.group_by_time_bucket(Duration::of_hours(4));

    // Expect 6 buckets covering 24 hours
    assert_eq!(buckets.len(), 6);

    // Check the content of the first bucket (00:00 - 04:00)
    let first_bucket_key = ZonedDateTime::parse("2024-04-25T00:00:00Z")
        .unwrap()
        .get_epoch_second();

    let first_bucket = buckets.get(&first_bucket_key).unwrap();

    // In the first 4 hours, expect:
    // Device A: 4 records (hourly)
    // Device B: 8 records (every 30 mins from 0:00 to 3:30)
    // Device C: 2 records (at 0h and 3h)
    assert_eq!(first_bucket.len(), 14);
}

#[test]
fn test_timeseries_aggregation() {
    let data = create_test_dataset();
    let processor = TimeseriesProcessor {
        records: data,
    };

    // Calculate hourly averages per device
    let hourly_avgs = processor.calculate_averages_by_device(Duration::of_hours(1));

    // Expect 24 hourly buckets
    assert_eq!(hourly_avgs.len(), 24);

    // Check a specific hour
    let hour_12_key = ZonedDateTime::parse("2024-04-25T12:00:00Z")
        .unwrap()
        .get_epoch_second();

    let hour_12_data = hourly_avgs.get(&hour_12_key).unwrap();

    // At 12:00, expect:
    // Device A: 1 record with value 26.0
    // Device B: 2 records with values around 31.0
    assert!(hour_12_data.contains_key("Device-A"));
    assert!(hour_12_data.contains_key("Device-B"));

    // The exact values should match our data generation formula
    assert!((hour_12_data.get("Device-A").unwrap() - 26.0).abs() < 0.001);
    // For device B, it's the average of two readings at 12:00 and 12:30
    let expected_b_avg = (31.0 + 31.25) / 2.0;
    assert!((hour_12_data.get("Device-B").unwrap() - expected_b_avg).abs() < 0.001);
}

#[test]
fn test_time_period_processing() {
    let start_date_zoned = ZonedDateTime::parse("2024-04-01T00:00:00Z").unwrap();
    let end_date_zoned = ZonedDateTime::parse("2024-04-30T23:59:59Z").unwrap();

    // Convert ZonedDateTime to NaiveDate for Period::between
    let start_date = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 4, 30).unwrap();

    // Create a period representing the month of April 2024
    let _april_period = Period::between(start_date, end_date);

    // Create test data
    let mut processor = TimeseriesProcessor::new();

    // Add some records that fall within our period
    let mid_april = ZonedDateTime::parse("2024-04-15T12:00:00Z").unwrap();
    processor.add_record(TimeseriesRecord::new(mid_april.clone(), "Device-X", 42.0));

    // Add some records outside our period
    let march = ZonedDateTime::parse("2024-03-15T12:00:00Z").unwrap();
    processor.add_record(TimeseriesRecord::new(march, "Device-X", 30.0));

    let may = ZonedDateTime::parse("2024-05-15T12:00:00Z").unwrap();
    processor.add_record(TimeseriesRecord::new(may, "Device-X", 50.0));

    // Filter for our period
    let filtered = processor.filter_time_range(&start_date_zoned, &end_date_zoned);

    // Should only have the April record
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].value, 42.0);
} 