use icu::locid::Locale;
use icu::plurals::{PluralCategory, PluralRules};
use icu_provider::AnyProvider;

impl CldrPluralRules for DefaultPluralRules {
    fn select(&self, number: f64, locale: &LocaleId) -> PluralCategory {
        let icu_locale = locale_str_to_icu_locale(locale.to_string().as_str());
        let provider = self.create_data_provider();

        let rules = PluralRules::try_new_with_any_provider(
            &provider,
            &icu_locale.into(),
            Default::default(),
        ).expect("Failed to create plural rules");

        rules.select(number).into()
    }
} 