#![deny(missing_debug_implementations)]

//! A client for interacting with the Open Lighting Architecture.

pub mod buffer;
pub mod client;
pub mod error;
pub mod ola;

const OLA_DEFAULT_PORT: u16 = 9010;

const PROTOCOL_VERSION: u32 = 1;
const VERSION_MASK: u32 = 0xf0000000;
const SIZE_MASK: u32 = 0x0fffffff;

pub use crate::buffer::DmxBuffer;
pub use crate::client::{connect, connect_with_config, StreamingClient};
pub use crate::error::{Error, Result};
