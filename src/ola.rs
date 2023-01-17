//! OLA communication.
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

use crate::{PROTOCOL_VERSION, SIZE_MASK, VERSION_MASK};

use bytes::{BufMut, Bytes, BytesMut};
use prost::Message;

/// RPC methods supported by `olad`.
// TODO: Generate from .proto
#[derive(Clone, Debug)]
pub enum RpcCall {
    StreamDmxData(proto::DmxData),
}

impl RpcCall {
    pub fn encode(&self, id: u32) -> proto::rpc::RpcMessage {
        let (rpc_type, name, buffer) = match self {
            RpcCall::StreamDmxData(r) => (
                proto::rpc::Type::StreamRequest,
                "StreamDmxData",
                r.encode_to_vec(),
            ),
        };

        proto::rpc::RpcMessage {
            r#type: rpc_type as i32,
            id: Some(id),
            name: Some(name.to_string()),
            buffer: Some(buffer),
        }
    }
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
    pub fn encode(&mut self, call: RpcCall) -> Bytes {
        let message = call.encode(self.next_sequence());

        let size = message.encoded_len();
        let mut bytes = BytesMut::with_capacity(size + 4);

        bytes.put(&encode_header(PROTOCOL_VERSION, size)[..]);
        let mut header = bytes.split();
        message.encode(&mut bytes).unwrap();
        header.unsplit(bytes);

        header.freeze()
    }
}
