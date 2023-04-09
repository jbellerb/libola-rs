#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! A client for interacting with the Open Lighting Architecture.
//!
//! ## Feature flags
//!
//! * **`tokio`** *(enabled by default)* — Enable to Tokio-powered asynchronous client

mod buffer;
mod client;
pub mod ola;

const PROTOCOL_VERSION: u32 = 1;
const VERSION_MASK: u32 = 0xf0000000;
const SIZE_MASK: u32 = 0x0fffffff;

pub use crate::client::{
    connect, connect_with_config, CallError, CallErrorKind, ConnectError, ConnectErrorKind,
    SpawnOladError, StreamingClient,
};
#[cfg(feature = "tokio")]
pub use crate::client::{connect_async, connect_async_with_config, StreamingClientAsync};
pub use buffer::{DmxBuffer, TryFromBufferError};
