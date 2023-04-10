use std::io::Write;

use super::{CallError, CallErrorKind};
use crate::ola::proto::{DmxData, OlaServerServiceCall};
use crate::ola::RpcContext;
use crate::DmxBuffer;

use bytes::BytesMut;

#[derive(Debug)]
pub struct StreamingClient<S: Write> {
    stream: S,
    ctx: RpcContext,
}

impl<S: Write> StreamingClient<S> {
    /// Send a DMX buffer to an OLA universe.
    pub fn send_dmx(&mut self, universe: u32, data: &DmxBuffer) -> Result<(), CallError> {
        self.send_dmx_with_priority(universe, data, 100)
    }

    /// Send a DMX buffer to an OLA universe with a priority value.
    pub fn send_dmx_with_priority(
        &mut self,
        universe: u32,
        data: &DmxBuffer,
        priority: u8,
    ) -> Result<(), CallError> {
        let request = OlaServerServiceCall::StreamDmxData(DmxData {
            universe: universe as i32,
            data: data.to_vec(),
            priority: Some(priority as i32),
        });

        let mut buf = BytesMut::new();
        self.ctx.encode(request, &mut buf).map_err(|e| CallError {
            kind: CallErrorKind::Encode(e),
        })?;
        self.stream
            .write_all(&buf.freeze())
            .map_err(|e| CallError {
                kind: CallErrorKind::Write(e),
            })?;
        Ok(())
    }

    /// Construct a new streaming client from an stream. The client is
    /// initialized with a fresh context. This usually, shouldn't be called
    /// directly, as `ClientConfig::connect()` will set up a stream for you
    /// before internally calling this.
    pub fn from_stream(stream: S) -> Self {
        Self {
            stream,
            ctx: RpcContext::new(),
        }
    }
}
