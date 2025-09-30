pub mod errors;
pub mod types;
pub mod executor;
pub mod bridged_executor;

pub use errors::{NixError, Result};
pub use types::{Package, SearchResult, InstalledPackage};
pub use executor::NixExecutor;
pub use bridged_executor::BridgedNixExecutor;
