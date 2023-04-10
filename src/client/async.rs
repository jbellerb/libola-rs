use super::{CallError, CallErrorKind};
use crate::ola::proto::{DmxData, OlaServerServiceCall};
use crate::ola::RpcContext;
use crate::DmxBuffer;

use bytes::BytesMut;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Debug)]
pub struct StreamingClientAsync<S: AsyncWrite + Unpin> {
    stream: S,
    ctx: RpcContext,
}

impl<S: AsyncWrite + Unpin> StreamingClientAsync<S> {
    pub async fn send_dmx(&mut self, universe: u32, data: &DmxBuffer) -> Result<(), CallError> {
        self.send_dmx_with_priority(universe, data, 100).await
    }

    pub async fn send_dmx_with_priority(
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
        self.stream.write_all(&buf).await.map_err(|e| CallError {
            kind: CallErrorKind::Write(e),
        })?;
        Ok(())
    }

    /// Construct a new streaming async client from an async stream. The
    /// client is initialized with a fresh context. This usually, shouldn't
    /// be called directly, as `ClientConfig::connect_async()` will set up a
    /// stream for you before internally calling this.
    pub fn from_stream(stream: S) -> Self {
        Self {
            stream,
            ctx: RpcContext::new(),
        }
    }
}
