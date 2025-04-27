// Basic integration tests for ICU4X functionality

use dougu_essentials::i18n::{
    cldr::CldrDataFactory,
    CurrencyCode,
    LanguageId,
    LocaleId,
};
use dougu_essentials::time::{LocalDate, ZonedDateTime};

// Helper function to create a test locale
fn test_locale(lang: &str) -> LocaleId {
    let language = LanguageId::new(lang);
    LocaleId::new(language, None)
}

#[test]
#[ignore = "Requires ICU data files at ./data/manifest.json"]
fn test_date_formatting() {
    // Create a date to format (October 10, 2023)
    let date = LocalDate::of(2023, 10, 10).unwrap();

    // Create a formatter using the factory
    let factory = CldrDataFactory::new();
    let formatter = factory.get_datetime_formatter(&test_locale("en-US"));
    let locale = test_locale("en-US");

    // Format the date
    let formatted = formatter.format_date(&date, &locale);

    // Check the result (may vary depending on exact formatter implementation)
    assert!(!formatted.is_empty());
    println!("Formatted date: {}", formatted);
}

#[test]
#[ignore = "Requires ICU data files at ./data/manifest.json"]
fn test_number_formatting() {
    // Create a number to format
    let number = 12345.67;
    let locale = test_locale("en-US");

    // Create a formatter using the factory
    let factory = CldrDataFactory::new();
    let formatter = factory.get_number_formatter(&locale);

    // Format the number
    let formatted = formatter.format_number(number, &locale);

    // Check the result
    assert!(!formatted.is_empty());
    println!("Formatted number: {}", formatted);
}

#[test]
#[ignore = "Requires ICU data files at ./data/manifest.json"]
fn test_collation() {
    // Create strings to compare
    let a = "apple";
    let b = "banana";
    let locale = test_locale("en-US");

    // Create a collator using the factory
    let factory = CldrDataFactory::new();
    let collator = factory.get_collator(&locale);

    // Compare the strings
    let result = collator.compare(a, b, &locale);

    // Check the result (a should come before b)
    assert!(result == std::cmp::Ordering::Less);
    println!("Collation result: {:?}", result);
}

#[test]
#[ignore = "Requires ICU data files at ./data/manifest.json"]
fn test_currency_formatting() {
    // Create a currency amount
    let amount = 1234.56;
    let currency = CurrencyCode::new("USD");
    let locale = test_locale("en-US");

    // Create a formatter using the factory
    let factory = CldrDataFactory::new();
    let formatter = factory.get_number_formatter(&locale);

    // Format the currency
    let formatted = formatter.format_currency(amount, &currency, &locale);

    // Check the result
    assert!(!formatted.is_empty());
    println!("Formatted currency: {}", formatted);
}

#[test]
#[ignore = "Requires ICU data files at ./data/manifest.json"]
fn test_with_nonexistent_locale() {
    // Create a date to format
    let date = LocalDate::of(2023, 10, 10).unwrap();
    let locale = test_locale("xx-XX");

    // Create a formatter with a non-existent locale
    let factory = CldrDataFactory::new();
    let formatter = factory.get_datetime_formatter(&locale);

    // Format should still work with a fallback locale
    let formatted = formatter.format_date(&date, &locale);

    // Check that we got something
    assert!(!formatted.is_empty());
    println!("Formatted date with nonexistent locale: {}", formatted);
}

#[test]
#[ignore = "Requires ICU data files at ./data/manifest.json"]
fn test_datetime_provider() {
    // Use now() since of_local with components isn't implemented
    let dt = ZonedDateTime::now();
    let locale = test_locale("en-US");

    // Create a provider
    let factory = CldrDataFactory::new();
    let formatter = factory.get_datetime_formatter(&locale);

    // Format the date/time
    let formatted = formatter.format_datetime(&dt, &locale);

    // Check the result
    assert!(!formatted.is_empty());
    println!("Formatted datetime: {}", formatted);
} 