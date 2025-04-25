// i18n module

// Submodules
pub mod locale;
pub mod script;
pub mod currency;
pub mod cldr;

pub use currency::CurrencyCode;
// Re-export common types
pub use locale::{LanguageId, LocaleId, LocaleService, RegionId};
pub use script::ScriptId;

pub use cldr::{CalendarType, Message, MessageArgs, MessageValue, NumberSystem, PluralCategory};
// Re-export from cldr
pub use cldr::{CldrDataFactory, LocaleDataProvider};
pub use cldr::{Collator, DateTimeFormatter, NumberFormatter, PluralRules};
