mod client;
mod contracts;
mod error;
mod http;
mod scraper;
mod types;
mod upstream;

pub use client::UnicumApi;
use error::ModuleError;
pub(crate) use error::ScraperError;
pub(super) use http::RequestBuilderExt;
pub(crate) use types::UserInResponse;
