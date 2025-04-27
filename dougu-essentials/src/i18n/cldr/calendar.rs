// Calendar functionality

use super::DateTimeFormatter as CldrDateTimeFormatter;
use crate::i18n::LocaleId;
use crate::time::{LocalDate, LocalTime, ZonedDateTime};
use std::fmt;

use crate::core::{Error as CoreError, Result as CoreResult};
use icu::calendar::types::{IsoHour, IsoMinute, IsoSecond, NanoSecond};
// ICU4X imports
use icu::calendar::{DateTime, Iso};
use icu::datetime::{DateTimeFormatter, DateTimeFormatterOptions};
use icu::locid::Locale;
use icu_provider::BufferProvider;
use icu_provider_fs::FsDataProvider;
use std::path::Path;

/// Calendar type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalendarType {
    Gregorian,
    Japanese,
    Buddhist,
    Chinese,
    Hebrew,
    Islamic,
    Persian,
    Indian,
    Coptic,
    Ethiopic,
    Other(String),
}

impl fmt::Display for CalendarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarType::Gregorian => write!(f, "gregorian"),
            CalendarType::Japanese => write!(f, "japanese"),
            CalendarType::Buddhist => write!(f, "buddhist"),
            CalendarType::Chinese => write!(f, "chinese"),
            CalendarType::Hebrew => write!(f, "hebrew"),
            CalendarType::Islamic => write!(f, "islamic"),
            CalendarType::Persian => write!(f, "persian"),
            CalendarType::Indian => write!(f, "indian"),
            CalendarType::Coptic => write!(f, "coptic"),
            CalendarType::Ethiopic => write!(f, "ethiopic"),
            CalendarType::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Calendar formatter implementation
pub struct CalendarFormatter {
    calendar_type: CalendarType,
    data_provider_path: Option<String>,
}

impl CalendarFormatter {
    /// Create a new calendar formatter with the specified calendar type
    pub fn new(calendar_type: CalendarType) -> Self {
        Self {
            calendar_type,
            data_provider_path: None,
        }
    }

    /// Create a new Gregorian calendar formatter
    pub fn gregorian() -> Self {
        Self::new(CalendarType::Gregorian)
    }

    /// Set the data provider path
    pub fn with_data_path<P: Into<String>>(mut self, path: P) -> Self {
        self.data_provider_path = Some(path.into());
        self
    }

    /// Get the calendar type
    pub fn calendar_type(&self) -> &CalendarType {
        &self.calendar_type
    }

    /// Create a data provider for ICU4X
    fn create_data_provider(&self) -> CoreResult<Box<dyn BufferProvider>> {
        let fs_provider: Box<dyn BufferProvider> = if let Some(path) = &self.data_provider_path {
            Box::new(FsDataProvider::try_new(Path::new(path)).map_err(CoreError::new)?)
        } else {
            Box::new(FsDataProvider::try_new("./data").map_err(CoreError::new)?)
        };
        Ok(fs_provider)
    }

    // Convert our LocaleId to icu_locid::Locale
    fn to_icu_locale(&self, locale: &LocaleId) -> Locale {
        locale_str_to_icu_locale(locale.as_str())
    }

    // Convert our LocalDate to ICU DateTime
    fn to_icu_date(&self, date: &LocalDate) -> DateTime<Iso> {
        // Calendar type selection might need revisiting if non-Gregorian formatting is required
        // let calendar_code = CalendarCode::from(self.calendar_type.clone());
        // let calendar = Gregorian::new(); // Using Gregorian explicitly for Iso DateTime
        DateTime::try_new_iso_datetime(
            date.year() as i32,
            date.month() as u8,
            date.day() as u8,
            0, // Default hour
            0, // Default minute
            0, // Default second
        )
            .expect("Failed to create Iso DateTime from LocalDate components")
    }

    // Convert our LocalTime to components for an ICU DateTime - No longer directly used by to_icu_datetime
    fn time_components(&self, time: &LocalTime) -> (IsoHour, IsoMinute, IsoSecond, NanoSecond) {
        (
            IsoHour::try_from(time.hour() as u8).unwrap(),
            IsoMinute::try_from(time.minute() as u8).unwrap(),
            IsoSecond::try_from(time.second() as u8).unwrap(),
            NanoSecond::zero() // Use NanoSecond::zero() instead of try_new(0)
        )
    }

    // Convert our ZonedDateTime to ICU DateTime<Iso>
    // Removed generic <A>, returning DateTime<Iso>
    fn to_icu_datetime(&self, datetime: &ZonedDateTime) -> DateTime<Iso> {
        // let calendar_code = CalendarCode::from(self.calendar_type.clone());
        // let calendar = Gregorian::new(); // Explicitly use Gregorian/Iso calendar
        // let (hour, minute, second, _) = self.time_components(&datetime.time()); // Incorrect: ZonedDateTime has direct accessors
        DateTime::try_new_iso_datetime(
            datetime.year() as i32,
            datetime.month() as u8,
            datetime.day() as u8,
            datetime.hour() as u8, // Get components directly
            datetime.minute() as u8,
            datetime.second() as u8,
        )
            .expect("Failed to create Iso DateTime from ZonedDateTime components")
    }
}

// Helper function to convert locale string to icu_locid::Locale
fn locale_str_to_icu_locale(locale_str: &str) -> Locale {
    Locale::try_from_bytes(locale_str.as_bytes())
        .unwrap_or_else(|_| panic!("Failed to parse locale string: {}", locale_str))
}

impl CldrDateTimeFormatter for CalendarFormatter {
    fn format_date(&self, date: &LocalDate, locale: &LocaleId) -> String {
        let icu_locale = self.to_icu_locale(locale);
        let provider = self.create_data_provider()
            .expect("Failed to create data provider");

        let formatter = DateTimeFormatter::try_new_with_buffer_provider(
            &provider,
            &icu_locale.into(),
            DateTimeFormatterOptions::default(),
        ).expect("Failed to create date formatter");

        let icu_date_iso = self.to_icu_date(date);
        let icu_date_any = icu_date_iso.to_any();

        formatter
            .format_to_string(&icu_date_any)
            .expect("Failed to format date")
    }

    fn format_time(&self, time: &LocalTime, locale: &LocaleId) -> String {
        let icu_locale = self.to_icu_locale(locale);
        let provider = self.create_data_provider()
            .expect("Failed to create data provider");

        let formatter = DateTimeFormatter::try_new_with_buffer_provider(
            &provider,
            &icu_locale.into(),
            DateTimeFormatterOptions::default(),
        ).expect("Failed to create time formatter");

        let today = LocalDate::now();
        let (hour, minute, second, _) = self.time_components(time);

        let icu_datetime_iso = DateTime::try_new_iso_datetime(
            today.year() as i32,
            today.month() as u8,
            today.day() as u8,
            hour.number(),
            minute.number(),
            second.number(),
        )
            .expect("Failed to create ICU datetime for time formatting");

        let icu_datetime_any = icu_datetime_iso.to_any();

        formatter
            .format_to_string(&icu_datetime_any)
            .expect("Failed to format time")
    }

    fn format_datetime(&self, datetime: &ZonedDateTime, locale: &LocaleId) -> String {
        let icu_locale = self.to_icu_locale(locale);
        let provider = self.create_data_provider()
            .expect("Failed to create data provider");

        let formatter = DateTimeFormatter::try_new_with_buffer_provider(
            &provider,
            &icu_locale.into(),
            DateTimeFormatterOptions::default(),
        ).expect("Failed to create datetime formatter");

        let icu_datetime_iso = self.to_icu_datetime(datetime);
        let icu_datetime_any = icu_datetime_iso.to_any();

        formatter
            .format_to_string(&icu_datetime_any)
            .expect("Failed to format datetime")
    }
} 