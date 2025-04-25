pub fn format(&self, _datetime: DateTime<Utc>) -> String {
    let _locale = self.locale.as_str();
    // TODO: Implement datetime formatting
    _datetime.to_rfc3339()
} 