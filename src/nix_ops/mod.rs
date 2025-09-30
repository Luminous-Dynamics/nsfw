pub mod errors;
pub mod types;
pub mod executor;

pub use errors::{NixError, Result};
pub use types::{Package, SearchResult, InstalledPackage};
pub use executor::NixExecutor;
