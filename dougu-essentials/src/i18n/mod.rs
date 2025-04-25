// Internationalization module
// This module provides tools for internationalization (i18n) and localization (l10n).

// Submodules
pub mod locale;
pub mod script;
pub mod currency;
pub mod cldr;
pub mod msg;

pub use currency::CurrencyCode;
// Re-export common types
pub use locale::{LanguageId, LocaleId, LocaleService, RegionId};
pub use script::ScriptId;

pub use cldr::{CalendarType, Message, MessageArgs, MessageValue, NumberSystem, PluralCategory};
// Re-export from cldr
pub use cldr::{CldrDataFactory, LocaleDataProvider};
pub use cldr::{Collator, DateTimeFormatter, NumberFormatter, PluralRules};

pub use msg::MessageArgs as MsgArgs;
// Re-export from msg
pub use msg::{MessageBundle, MessageFormatter, ResourceManager};
// Renamed to avoid conflict
pub use msg::format::MessageFormat;
