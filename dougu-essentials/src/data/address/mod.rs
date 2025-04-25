pub mod email;
pub mod url;
pub mod uri;
pub mod error;
pub mod types;
pub mod geouri;

pub use email::Email;
pub use error::{AddressError, Result};
pub use geouri::GeoUri;
pub use types::AddressType;
pub use uri::Uri;
pub use url::Url;
