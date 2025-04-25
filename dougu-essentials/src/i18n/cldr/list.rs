use icu::list::ListFormatter;
use icu::locid::Locale;
use icu_provider::AnyProvider;

impl CldrListFormatter for DefaultListFormatter {
    fn format_list(&self, items: &[String], locale: &LocaleId) -> String {
        let icu_locale = locale_str_to_icu_locale(locale.to_string().as_str());
        let provider = self.create_data_provider();

        let formatter = ListFormatter::try_new_with_any_provider(
            &provider,
            &icu_locale.into(),
            Default::default(),
        ).expect("Failed to create list formatter");

        formatter.format(items).to_string()
    }
} 