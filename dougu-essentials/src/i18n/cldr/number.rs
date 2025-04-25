// Number system functionality

use super::{NumberFormatter, PluralCategory, PluralRules};
use crate::i18n::LocaleId;
use std::fmt;
use std::path::Path;


use crate::math::FixedDecimal;

use crate::core::{Error as CoreError, Result as CoreResult};
use crate::i18n::CurrencyCode;
use icu::locid::Locale;
use icu_decimal::options::FixedDecimalFormatterOptions;
use icu_decimal::FixedDecimalFormatter;
use icu_plurals::{PluralCategory as IcuPluralCategory, PluralRuleType, PluralRules as IcuPluralRules};
use icu_provider::BufferProvider;
use icu_provider_fs::FsDataProvider;


/// Number system identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberSystem {
    Arabic,
    Bengali,
    Chinese,
    Devanagari,
    Latin,
    Other(String),
}

impl fmt::Display for NumberSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberSystem::Latin => write!(f, "latn"),
            NumberSystem::Arabic => write!(f, "arab"),
            NumberSystem::Bengali => write!(f, "beng"),
            NumberSystem::Chinese => write!(f, "hans"),
            NumberSystem::Devanagari => write!(f, "deva"),
            NumberSystem::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Number formatter implementation
#[derive(Clone)]
pub struct DefaultNumberFormatter {
    data_provider_path: Option<String>,
}

impl DefaultNumberFormatter {
    /// Create a new number formatter with the specified number system
    pub fn new(data_provider_path: Option<String>) -> Self {
        Self { data_provider_path }
    }

    /// Create a new number formatter using the Latin number system
    pub fn latin() -> Self {
        Self::new(None)
    }

    /// Get the number system
    pub fn number_system(&self) -> &NumberSystem {
        match self.data_provider_path {
            Some(ref path) => {
                if path.contains("arab") {
                    &NumberSystem::Arabic
                } else if path.contains("beng") {
                    &NumberSystem::Bengali
                } else if path.contains("hans") {
                    &NumberSystem::Chinese
                } else if path.contains("deva") {
                    &NumberSystem::Devanagari
                } else {
                    &NumberSystem::Latin
                }
            }
            None => &NumberSystem::Latin,
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

    /// Convert LocaleId to ICU Locale
    fn to_icu_locale(&self, locale: &LocaleId) -> Locale {
        super::locale_str_to_icu_locale(locale.as_str())
    }

    /// Create ICU FixedDecimalFormatter
    fn create_decimal_formatter(&self, locale: &LocaleId) -> CoreResult<FixedDecimalFormatter> {
        let icu_locale = self.to_icu_locale(locale);
        let provider = self.create_data_provider()?;
        FixedDecimalFormatter::try_new_with_buffer_provider(&*provider, &icu_locale.into(), Default::default())
            .map_err(CoreError::new)
    }

    fn create_icu_formatter(
        &self,
        locale: &Locale,
        options: FixedDecimalFormatterOptions,
    ) -> CoreResult<FixedDecimalFormatter> {
        let provider = self.create_data_provider()?;
        FixedDecimalFormatter::try_new_with_buffer_provider(&*provider, &locale.into(), options)
            .map_err(CoreError::new)
    }

    fn create_icu_plural_rules(
        &self,
        locale: &Locale,
        rule_type: PluralRuleType,
    ) -> CoreResult<IcuPluralRules> {
        let provider = self.create_data_provider()?;
        IcuPluralRules::try_new_with_buffer_provider(&*provider, &locale.into(), rule_type)
            .map_err(CoreError::new)
    }
}

impl NumberFormatter for DefaultNumberFormatter {
    fn format_number(&self, number: f64, locale: &LocaleId) -> String {
        let formatter = self.create_decimal_formatter(locale).expect("Failed to create decimal formatter");
        // Create FixedDecimal from f64 and convert to ICU FixedDecimal
        let fixed_decimal = FixedDecimal::from(number);
        let icu_decimal = fixed_decimal.to_icu();
        formatter
            .format(&icu_decimal)
            .to_string()
    }

    fn format_percent(&self, number: f64, locale: &LocaleId) -> String {
        let formatter = self.create_decimal_formatter(locale).expect("Failed to create decimal formatter");
        // Create FixedDecimal from f64 and convert to ICU FixedDecimal
        let fixed_decimal = FixedDecimal::from(number);
        let icu_decimal = fixed_decimal.to_icu();
        format!(
            "{}%",
            formatter
                .format(&icu_decimal)
                .to_string()
        )
    }

    fn format_currency(&self, number: f64, currency_code: &CurrencyCode, locale: &LocaleId) -> String {
        let formatter = self.create_decimal_formatter(locale).expect("Failed to create decimal formatter");
        // Create FixedDecimal from f64 and convert to ICU FixedDecimal
        let fixed_decimal = FixedDecimal::from(number);
        let icu_decimal = fixed_decimal.to_icu();
        format!(
            "{} {}",
            currency_code.to_string(),
            formatter
                .format(&icu_decimal)
                .to_string()
        )
    }
}

/// Plural rules implementation
#[derive(Clone)] // Added Clone
pub struct DefaultPluralRules {
    data_provider_path: Option<String>,
}

impl DefaultPluralRules {
    /// Create a new plural rules implementation
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

    // Helper method to create ICU plural rules
    fn create_icu_rules(&self, locale: &LocaleId) -> CoreResult<IcuPluralRules> {
        let icu_locale = super::locale_str_to_icu_locale(locale.as_str());

        let provider = self.create_data_provider()?;

        IcuPluralRules::try_new_cardinal_with_buffer_provider(
            &*provider, // Pass &*Box<dyn BufferProvider>
            &icu_locale.into(),
        ).map_err(CoreError::new)
    }

    // Convert ICU plural category to our format
    fn convert_category(&self, category: IcuPluralCategory) -> PluralCategory {
        match category {
            IcuPluralCategory::Zero => PluralCategory::Zero,
            IcuPluralCategory::One => PluralCategory::One,
            IcuPluralCategory::Two => PluralCategory::Two,
            IcuPluralCategory::Few => PluralCategory::Few,
            IcuPluralCategory::Many => PluralCategory::Many,
            IcuPluralCategory::Other => PluralCategory::Other,
        }
    }
}

impl PluralRules for DefaultPluralRules {
    fn get_category(&self, number: f64, locale: &LocaleId) -> PluralCategory {
        let rules = self.create_icu_rules(locale).expect("Failed to create plural rules");
        // Try to create FixedDecimal and convert to ICU FixedDecimal
        let fixed = FixedDecimal::try_from(number).expect("Failed to create fixed decimal");
        let icu_fixed = fixed.to_icu();
        let category = rules.category_for(&icu_fixed);
        self.convert_category(category)
    }
} 