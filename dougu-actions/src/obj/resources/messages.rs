pub const CMD_OBJ_DESCRIPTION: &str = "Work with object notation formats (JSON, BSON, XML, CBOR) and execute queries";
pub const CMD_QUERY_DESCRIPTION: &str = "Execute a query on an object notation file";
pub const CMD_CONVERT_DESCRIPTION: &str = "Convert between object notation formats";
pub const CMD_EXTRACT_DESCRIPTION: &str = "Extract raw value from JSON input (for scripting)";

pub const ARG_FORMAT_DESCRIPTION: &str = "Input format (json, bson, xml, cbor)";
pub const ARG_OUTPUT_FORMAT_DESCRIPTION: &str = "Output format (json, bson, xml, cbor)";
pub const ARG_FILE_DESCRIPTION: &str = "Input file path (use - for stdin)";
pub const ARG_QUERY_DESCRIPTION: &str = "Query string in jq-like format";

pub const ERROR_FILE_NOT_FOUND: &str = "File not found";
pub const ERROR_INVALID_FORMAT: &str = "Invalid format specified";
pub const ERROR_DECODE_FAILED: &str = "Failed to decode input";
pub const ERROR_ENCODE_FAILED: &str = "Failed to encode output";
pub const ERROR_INVALID_UTF8: &str = "Invalid UTF-8 in output";
pub const ERROR_QUERY_PARSE: &str = "Failed to parse query";
pub const ERROR_QUERY_FAILED: &str = "Failed to execute query";
pub const ERROR_QUERY_EXECUTION: &str = "Query execution failed";
pub const ERROR_VALUE_CONVERSION: &str = "Failed to convert value";
pub const ERROR_STDIN_READ: &str = "Failed to read from stdin";
pub const ERROR_RESULT_CONVERSION: &str = "Failed to convert query result"; 