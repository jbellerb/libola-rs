#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! A client for interacting with the Open Lighting Architecture.
//!
//! ## Feature flags
//!
//! * **`tokio`** *(enabled by default)* — Enable the Tokio-powered asynchronous client

mod buffer;
pub mod client;
pub mod config;
pub mod ola;

const PROTOCOL_VERSION: u32 = 1;
const VERSION_MASK: u32 = 0xf0000000;
const SIZE_MASK: u32 = 0x0fffffff;

pub use buffer::{DmxBuffer, TryFromBufferError};

use client::StreamingClient;
#[cfg(feature = "tokio")]
use client::StreamingClientAsync;
use config::{Config, ConnectError};
use std::net::TcpStream;
#[cfg(feature = "tokio")]
use tokio::net::TcpStream as TokioTcpStream;

/// Start a synchronous connection with OLA.
///
/// This is a convenience function for connecting to OLA using the default
/// configuration (auto-start and default port). See [`Config`] for changing
/// the port and auto-start behavior.
pub fn connect() -> Result<StreamingClient<TcpStream>, ConnectError> {
    Config::new().connect()
}

/// Start an asynchronous connection with OLA.
///
/// This is a convenience function for connecting to OLA using the default
/// configuration (auto-start and default port). See [`Config`] for changing
/// the port and auto-start behavior.
#[cfg(feature = "tokio")]
pub async fn connect_async() -> Result<StreamingClientAsync<TokioTcpStream>, ConnectError> {
    Config::new().connect_async().await
}
