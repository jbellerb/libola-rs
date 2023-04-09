//! OLA protocol bits.
//!
//! This module contains types and helper functions for encoding and decoding
//! messages between the client and `olad`.

/// RPC types. These are all generated from the Protocol Buffer definitions
/// provided by OLA.
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/ola.proto.rs"));

    /// RPC message types.
    pub mod rpc {
        include!(concat!(env!("OUT_DIR"), "/ola.rpc.rs"));
    }
}

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{PROTOCOL_VERSION, SIZE_MASK, VERSION_MASK};

use bytes::{BufMut, BytesMut};
use prost::Message;

/// Methods that can be sent over an RPC channel.
pub trait RpcCall {
    /// Encode an RPC call as an RPC message.
    fn to_message(&self, id: u32) -> proto::rpc::RpcMessage;
}

/// Encode an RPC message header.
pub fn encode_header(version: u32, size: usize) -> [u8; 4] {
    let mut header = size as u32 & SIZE_MASK;
    header |= (version << 28) & VERSION_MASK;

    // `olad` uses host endianness for decoding the RPC header. See:
    // https://github.com/OpenLightingProject/ola/issues/1795
    header.to_ne_bytes()
}

/// Context for encoding and decoding RPC messages in a session.
#[derive(Clone, Debug, Default)]
pub struct RpcContext {
    sequence_number: u32,
}

impl RpcContext {
    /// Build a context for a new session.
    pub fn new() -> Self {
        Default::default()
    }

    fn next_sequence(&mut self) -> u32 {
        let number = self.sequence_number;
        self.sequence_number += 1;

        number
    }

    /// Encode an RPC call as a new message.
    pub fn encode(
        &mut self,
        item: proto::OlaServerServiceCall,
        dst: &mut BytesMut,
    ) -> Result<(), MessageEncodeError> {
        let message = item.to_message(self.next_sequence());
        let size = message.encoded_len();

        dst.put_slice(&encode_header(PROTOCOL_VERSION, size));
        message.encode(dst).map_err(|e| MessageEncodeError {
            kind: MessageEncodeErrorKind::Capacity(e),
        })?;

        Ok(())
    }
}

/// The error type returned when encoding an API message fails.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct MessageEncodeError {
    pub kind: MessageEncodeErrorKind,
}

impl Display for MessageEncodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to encode API message")
    }
}

impl Error for MessageEncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            MessageEncodeErrorKind::Capacity(e) => Some(e),
        }
    }
}

/// Enum to store the various types of errors that can cause encoding a message
/// to fail.
#[derive(Clone, Debug)]
pub enum MessageEncodeErrorKind {
    /// Destination buffer has insufficient capacity.
    Capacity(prost::EncodeError),
}
