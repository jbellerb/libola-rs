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
use proto::rpc::RpcMessage;

use bytes::{BufMut, BytesMut};
use prost::Message;

/// Methods that can be sent over an RPC channel.
pub trait RpcCall: Sized {
    /// Decode an RPC message as an RPC call.
    fn from_message(msg: RpcMessage) -> Result<(u32, Self), MessageDecodeError>;
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

/// Decodes an RPC message header to get it's protocol version and message
/// size.
pub fn decode_header(header: [u8; 4]) -> (u32, usize) {
    let header = u32::from_ne_bytes(header);
    let version = header >> 28;
    let size = (header & SIZE_MASK) as usize;

    (version, size)
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

        self.encode_message(message, dst)
    }

    /// Encode an RPC message.
    pub fn encode_message(
        &mut self,
        message: RpcMessage,
        dst: &mut BytesMut,
    ) -> Result<(), MessageEncodeError> {
        let size = message.encoded_len();

        dst.put_slice(&encode_header(PROTOCOL_VERSION, size));
        message.encode(dst).map_err(|e| MessageEncodeError {
            kind: MessageEncodeErrorKind::Capacity(e),
        })?;

        Ok(())
    }

    /// Encode a buffer containing a message as an RPC call.
    pub fn decode(buf: &[u8]) -> Result<(u32, proto::OlaClientServiceCall), MessageDecodeError> {
        let message = RpcMessage::decode(buf).map_err(|e| MessageDecodeError {
            kind: MessageDecodeErrorKind::Invalid(e),
        })?;

        proto::OlaClientServiceCall::from_message(message)
    }
}

/// The error type returned when encoding an API message fails.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct MessageEncodeError {
    pub kind: MessageEncodeErrorKind,
}

impl Display for MessageEncodeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

/// The error type returned when decoding an API message fails.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct MessageDecodeError {
    pub kind: MessageDecodeErrorKind,
}

impl Display for MessageDecodeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "failed to decode API message")
    }
}

impl Error for MessageDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            MessageDecodeErrorKind::Invalid(e) => Some(e),
            MessageDecodeErrorKind::Unrecognised => None,
        }
    }
}

/// Enum to store the various types of errors that can cause decoding a message
/// to fail.
#[derive(Clone, Debug)]
pub enum MessageDecodeErrorKind {
    /// Input buffer does not contain a valid message.
    Invalid(prost::DecodeError),
    /// Recieved message was of an unknown method or type.
    Unrecognised,
}
