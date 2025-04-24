pub struct Messages;

impl Messages {
    pub const DATABASE_OPEN_ERROR: &'static str = "Failed to open database";
    #[allow(dead_code)]
    pub const DATABASE_CLOSE_ERROR: &'static str = "Failed to close database";
    pub const QUERY_EXECUTION_ERROR: &'static str = "Failed to execute query";
    pub const TRANSACTION_BEGIN_ERROR: &'static str = "Failed to begin transaction";
    pub const TRANSACTION_COMMIT_ERROR: &'static str = "Failed to commit transaction";
    pub const TRANSACTION_ROLLBACK_ERROR: &'static str = "Failed to rollback transaction";
    #[allow(dead_code)]
    pub const PARAMETER_BINDING_ERROR: &'static str = "Failed to bind parameters";
    pub const ROW_FETCH_ERROR: &'static str = "Failed to fetch row";
    #[allow(dead_code)]
    pub const COLUMN_FETCH_ERROR: &'static str = "Failed to fetch column";
    pub const SERIALIZATION_ERROR: &'static str = "Failed to serialize data";
    pub const DESERIALIZATION_ERROR: &'static str = "Failed to deserialize data";
} 