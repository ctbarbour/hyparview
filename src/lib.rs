pub mod server;

pub mod state;
use state::Config;
use state::PeerState;

pub mod connection;
pub use connection::Connection;

pub mod message;
pub use message::ProtocolMessage;

pub mod action;
pub use action::Action;

pub mod client;
pub use client::Client;

/// Copied from https://github.com/tokio-rs/mini-redis/blob/master/src/lib.rs
///
/// Error returned by most functions.
///
/// When writing a real application, one might want to consider a specialized
/// error handling crate or defining an error type as an `enum` of causes.
/// However, for our example, using a boxed `std::error::Error` is sufficient.
///
/// For performance reasons, boxing is avoided in any hot path. For example, in
/// `parse`, a custom error `enum` is defined. This is because the error is hit
/// and handled during normal execution when a partial frame is received on a
/// socket. `std::error::Error` is implemented for `parse::Error` which allows
/// it to be converted to `Box<dyn std::error::Error>`.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;
