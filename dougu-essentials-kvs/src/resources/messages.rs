pub struct Messages;

impl Messages {
    pub const DATABASE_OPEN_ERROR: &'static str = "Failed to open database";
    pub const DATABASE_READ_ERROR: &'static str = "Failed to read from database";
    pub const DATABASE_WRITE_ERROR: &'static str = "Failed to write to database";
    pub const DATABASE_DELETE_ERROR: &'static str = "Failed to delete from database";
    pub const KEY_NOT_FOUND: &'static str = "Key not found in database";
    pub const TRANSACTION_ERROR: &'static str = "Transaction failed";
    pub const SERIALIZATION_ERROR: &'static str = "Failed to serialize data";
    pub const DESERIALIZATION_ERROR: &'static str = "Failed to deserialize data";
} 