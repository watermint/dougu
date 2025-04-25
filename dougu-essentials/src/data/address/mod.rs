pub mod email;
pub mod url;
pub mod uri;
pub mod error;
pub mod types;

pub use email::Email;
pub use url::Url;
pub use uri::Uri;
pub use error::{AddressError, Result};
pub use types::AddressType; 