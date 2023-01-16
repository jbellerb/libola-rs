mod buffer;
pub mod client;
mod error;
pub mod ola {
    //! RPC types for talking to `olad`. These are all generated from the
    //! Protocol Buffer definitions provided by OLA.

    pub mod proto {
        include!(concat!(env!("OUT_DIR"), "/ola.proto.rs"));
    }

    pub mod rpc {
        include!(concat!(env!("OUT_DIR"), "/ola.rpc.rs"));
    }
}

const OLA_DEFAULT_PORT: u16 = 9010;

const PROTOCOL_VERSION: u32 = 1;
const VERSION_MASK: u32 = 0xf0000000;
const SIZE_MASK: u32 = 0x0fffffff;

pub use crate::buffer::DmxBuffer;
pub use crate::client::StreamingClient;
pub use crate::error::{Error, Result};
