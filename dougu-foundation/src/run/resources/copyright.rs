/// Constants for copyright information display
/// These constants are used to represent copyright information in a 
/// language-independent way
pub const LICENSE_TEXT: &str = "Licensed under open source licenses. Use the `license` command for more detail.";

/// Format to use for displaying copyright when start year equals current year
pub const COPYRIGHT_SINGLE_YEAR_FORMAT: &str = "© {year} {owner}";

/// Format to use for displaying copyright when showing a range of years 
pub const COPYRIGHT_YEAR_RANGE_FORMAT: &str = "© {start_year}-{end_year} {owner}"; 