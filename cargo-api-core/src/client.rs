pub mod connection;
#[cfg(any(feature = "blocking_reqwest", feature = "async_reqwest"))]
pub mod reqwest;
pub mod token;

pub use connection::{CratesIoConnection, CRATES_IO_API};
#[cfg(feature = "async_reqwest")]
pub use reqwest::ReqwestAsyncClient;
#[cfg(feature = "blocking_reqwest")]
pub use reqwest::ReqwestClient;
#[cfg(any(feature = "blocking_reqwest", feature = "async_reqwest"))]
pub use reqwest::{Error, NewClientError};
pub use token::{InvalidToken, Token};
