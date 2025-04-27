// CLDR currency functionality

use crate::i18n::CurrencyCode;
use crate::i18n::LocaleId;
// Use public re-export
use std::path::Path;

use crate::math::FixedDecimal;

use crate::core::{Error as CoreError, ErrorTrait, Result as CoreResult};
use icu::locid::Locale;
use icu_decimal::{options::FixedDecimalFormatterOptions, options::GroupingStrategy, FixedDecimalFormatter};
use icu_provider::BufferProvider;
use icu_provider_fs::FsDataProvider;
use std::str::FromStr;

/// Custom error type for currency operations
#[derive(ErrorTrait, Debug)]
pub enum CurrencyError {
    #[error("Failed to parse currency code: {0}")]
    ParseError(String),
    #[error("ICU operation failed: {0}")]
    IcuError(#[from] icu::decimal::Error),
    #[error("Provider error: {0}")]
    ProviderError(#[from] icu_provider::DataError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Currency formatter trait
pub trait CurrencyFormatter {
    /// Format a currency value according to the locale and currency code
    fn format_currency(&self, value: f64, currency: &CurrencyCode, locale: &LocaleId) -> String;
}

/// Currency formatter implementation
#[derive(Clone)]
pub struct CurrencyFormatterImpl {
    data_provider_path: Option<String>,
}

impl CurrencyFormatterImpl {
    /// Create a new currency formatter
    pub fn new() -> Self {
        Self {
            data_provider_path: None,
        }
    }

    /// Set the data provider path
    pub fn with_data_path<P: Into<String>>(mut self, path: P) -> Self {
        self.data_provider_path = Some(path.into());
        self
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

    // Helper method to convert LocaleId to icu_locid::Locale
    fn to_icu_locale(&self, locale: &LocaleId) -> Locale {
        locale_str_to_icu_locale(locale.as_str())
    }

    // Helper method to create ICU formatter for number formatting
    fn create_icu_formatter(&self, _currency: &CurrencyCode, locale: &LocaleId) -> CoreResult<FixedDecimalFormatter> {
        let icu_locale = self.to_icu_locale(locale);
        let provider = self.create_data_provider()?;

        // Create options for number formatting with grouping
        let mut options = FixedDecimalFormatterOptions::default();
        options.grouping_strategy = GroupingStrategy::Auto;

        // Create the formatter
        FixedDecimalFormatter::try_new_with_buffer_provider(&*provider, &icu_locale.into(), options)
            .map_err(CoreError::new)
    }

    // Helper method to get currency symbol based on currency code and locale
    fn get_currency_symbol(&self, currency: &CurrencyCode, _locale: &LocaleId) -> String {
        // This is a simplification - in a real implementation we would use CLDR data
        match currency.as_str() {
            "USD" => "$".to_string(),
            "EUR" => "€".to_string(),
            "JPY" => "¥".to_string(),
            "GBP" => "£".to_string(),
            // For others, return the code
            _ => currency.as_str().to_string()
        }
    }
}

// Helper function to convert locale string to icu_locid::Locale
fn locale_str_to_icu_locale(locale_str: &str) -> Locale {
    Locale::try_from_bytes(locale_str.as_bytes())
        .unwrap_or_else(|_| panic!("Failed to parse locale string: {}", locale_str))
}

impl CurrencyFormatter for CurrencyFormatterImpl {
    fn format_currency(&self, value: f64, currency: &CurrencyCode, locale: &LocaleId) -> String {
        let formatter = self.create_icu_formatter(currency, locale)
            .expect("Failed to create ICU formatter");

        // Create FixedDecimal and convert to ICU FixedDecimal
        let fixed = FixedDecimal::try_from(value).expect("Failed to convert to fixed decimal");
        let icu_fixed = fixed.to_icu();

        // Format with ICU but add currency symbol based on locale and currency code
        let formatted_number = formatter.format(&icu_fixed).to_string();

        // Get the appropriate currency symbol
        let symbol = self.get_currency_symbol(currency, locale);

        // Format according to locale conventions (simplified)
        match locale.as_str() {
            // For Japanese locale, symbol goes before the number without space
            loc if loc.starts_with("ja") => format!("{}{}", symbol, formatted_number),
            // For most European locales, symbol goes after the number with space
            loc if loc.starts_with("fr") || loc.starts_with("it") || loc.starts_with("es") =>
                format!("{} {}", formatted_number, symbol),
            // For most other locales (including en-US), symbol goes before the number
            _ => format!("{}{}", symbol, formatted_number),
        }
    }
}

/// Data structure representing currency information
pub struct CurrencyInfo {
    pub code: CurrencyCode,
    pub name: String,
    pub symbol: String,
    pub decimal_places: u8,
}

/// Data structure for basic currency metadata (placeholder)
#[derive(Debug, Clone)]
pub struct CurrencyMetadata {
    pub code: String,
    pub decimal_digits: u8,
    pub rounding_increment: i32, // Or appropriate type
}

/// Repository of currency information
#[derive(Clone)]
pub struct CurrencyRepository {
    data_provider_path: Option<String>,
}

impl CurrencyRepository {
    /// Create a new currency repository
    pub fn new() -> Self {
        Self {
            data_provider_path: None,
        }
    }

    /// Set the data provider path
    pub fn with_data_path<P: Into<String>>(mut self, path: P) -> Self {
        self.data_provider_path = Some(path.into());
        self
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

    /// Get currency symbol for a given currency code and locale
    pub fn get_symbol(&self, code: &CurrencyCode, _locale: &LocaleId) -> CoreResult<String> {
        // Simplified approach that doesn't require currency formatter options
        let symbol = match code.as_str() {
            "USD" => "$",
            "EUR" => "€",
            "JPY" => "¥",
            "GBP" => "£",
            // For other currencies, return the code as fallback
            _ => code.as_str(),
        };

        Ok(symbol.to_string())
    }

    /// Get currency metadata (e.g., decimal places)
    pub fn get_basic_metadata(&self, code: &CurrencyCode) -> CoreResult<CurrencyMetadata> {
        // Determine decimal digits based on currency code
        let decimal_digits = match code.as_str() {
            "JPY" => 0,
            "BHD" | "IQD" | "JOD" | "KWD" | "LYD" | "OMR" | "TND" => 3,
            _ => 2,
        };

        Ok(CurrencyMetadata {
            code: code.as_str().to_string(),
            decimal_digits,
            rounding_increment: 0,
        })
    }

    /// Get information about a currency
    pub fn get_currency_info(&self, code: &CurrencyCode) -> CoreResult<CurrencyInfo> {
        let code_str = code.as_str();

        // Get symbol
        let symbol = self.get_symbol(code, &LocaleId::from_str("en").unwrap())?;

        // Get decimal places
        let metadata = self.get_basic_metadata(code)?;

        // Get currency name based on code
        let name = match code_str {
            "USD" => "US Dollar",
            "EUR" => "Euro",
            "JPY" => "Japanese Yen",
            "GBP" => "British Pound",
            _ => code_str, // Fallback to code for unknown currencies
        }.to_string();

        Ok(CurrencyInfo {
            code: code.clone(),
            name,
            symbol,
            decimal_places: metadata.decimal_digits,
        })
    }

    /// Parses a currency code string into a CurrencyCode instance.
    pub fn parse_code(code: &str) -> CoreResult<CurrencyCode> {
        CurrencyCode::from_str(code)
            .map_err(|_| CoreError::new(CurrencyError::ParseError(code.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_formatter() -> CurrencyFormatterImpl {
        // Use a path relative to the crate root or a test-specific setup
        // Instead of testdata provider, create a real one or mock if needed
        // For testing, ensure the ./testdata directory exists and has data, or use a different path.
        let test_data_path = "../testdata/icu"; // Adjust path as needed relative to Cargo.toml
        CurrencyFormatterImpl::new().with_data_path(test_data_path)
    }

    #[test]
    #[ignore = "Requires ICU data files at ../testdata/icu/manifest.json"]
    fn test_format_currency() {
        let formatter = setup_formatter();
        let formatted_usd = formatter.format_currency(1234.56, &CurrencyCode::from_str("USD").unwrap(), &LocaleId::from_str("en-US").unwrap());
        assert!(formatted_usd.contains("$"));
        assert!(formatted_usd.contains("1,234.56") || formatted_usd.contains("1234.56")); // Formatting depends on locale

        let formatted_jpy = formatter.format_currency(1234.0, &CurrencyCode::from_str("JPY").unwrap(), &LocaleId::from_str("ja-JP").unwrap());
        assert!(formatted_jpy.contains("¥"));
        assert!(formatted_jpy.contains("1,234") || formatted_jpy.contains("1234"));
    }
}

#[cfg(test)]
mod repo_tests {
    use super::*;
    // use crate::i18n::LocaleId; // Removed unused import in this module

    // Helper function to create a repository, potentially with test data path
    fn setup_repository() -> CurrencyRepository {
        let test_data_path = "../testdata/icu"; // Adjust path as needed
        CurrencyRepository::new().with_data_path(test_data_path)
    }

    #[test]
    #[ignore = "Requires ICU data files at ../testdata/icu/manifest.json"]
    fn test_get_currency_info() {
        let repo = setup_repository();
        let usd_info = repo.get_currency_info(&CurrencyCode::from_str("USD").unwrap()).unwrap();
        assert_eq!(usd_info.code, CurrencyCode::from_str("USD").unwrap());
        assert_eq!(usd_info.name, "US Dollar");
        assert_eq!(usd_info.symbol, "$");
        assert_eq!(usd_info.decimal_places, 2);

        let eur_info = repo.get_currency_info(&CurrencyCode::from_str("EUR").unwrap()).unwrap();
        assert_eq!(eur_info.code, CurrencyCode::from_str("EUR").unwrap());
        assert_eq!(eur_info.name, "Euro");
        assert_eq!(eur_info.symbol, "€");
        assert_eq!(eur_info.decimal_places, 2);
    }

    #[test]
    #[ignore = "Requires ICU data files at ../testdata/icu/manifest.json"]
    fn test_get_metadata() {
        let repo = setup_repository();
        let usd_code = CurrencyRepository::parse_code("USD").unwrap();
        let metadata = repo.get_basic_metadata(&usd_code).unwrap();
        assert_eq!(metadata.code, "USD");
        assert_eq!(metadata.decimal_digits, 2);

        let jpy_code = CurrencyRepository::parse_code("JPY").unwrap();
        let metadata = repo.get_basic_metadata(&jpy_code).unwrap();
        assert_eq!(metadata.code, "JPY");
        assert_eq!(metadata.decimal_digits, 0);
    }
} 