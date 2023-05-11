use super::{CallError, CallErrorKind};
use crate::ola::proto::{
    rpc::{RpcMessage, Type},
    Ack, DmxData, OlaClientServiceCall, OlaServerServiceCall, RegisterAction, RegisterDmxRequest,
};
use crate::ola::{decode_header, RpcContext};
use crate::DmxBuffer;

use bytes::BytesMut;
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug)]
pub struct ClientAsync<S: AsyncRead + AsyncWrite + Unpin> {
    stream: S,
    ctx: RpcContext,
    buf: BytesMut,
}

impl<S: AsyncRead + AsyncWrite + Unpin> ClientAsync<S> {
    pub async fn send_dmx_streaming(
        &mut self,
        universe: u32,
        data: &DmxBuffer,
    ) -> Result<(), CallError> {
        self.send_dmx_streaming_with_priority(universe, data, 100)
            .await
    }

    pub async fn send_dmx_streaming_with_priority(
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

    pub async fn register_universe(&mut self, universe: u32) -> Result<(), CallError> {
        let request = OlaServerServiceCall::RegisterForDmx(RegisterDmxRequest {
            universe: universe as i32,
            action: RegisterAction::Register as i32,
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

    pub async fn recv(&mut self) -> Result<(i32, DmxBuffer), CallError> {
        loop {
            let _ = self.stream.read_buf(&mut self.buf).await;

            if self.buf.len() < 4 {
                // must load more to read header
                continue;
            }

            let mut header = [0; 4];
            header.copy_from_slice(&self.buf[0..4]);
            let (_version, size) = decode_header(header);
            if self.buf.len() < 4 + size {
                // must load more to read entire message
                continue;
            }

            let frame = self.buf.split_to(4 + size);
            let (id, call) = RpcContext::decode(&frame[4..]).map_err(|e| CallError {
                kind: CallErrorKind::Decode(e),
            })?;

            match call {
                OlaClientServiceCall::UpdateDmxData(data) => {
                    let mut buf = BytesMut::new();
                    let message = RpcMessage {
                        r#type: Type::Response as i32,
                        id: Some(id),
                        name: Some("Ack".to_string()),
                        buffer: Some(Ack {}.encode_to_vec()),
                    };
                    self.ctx
                        .encode_message(message, &mut buf)
                        .map_err(|e| CallError {
                            kind: CallErrorKind::Encode(e),
                        })?;
                    self.stream.write_all(&buf).await.map_err(|e| CallError {
                        kind: CallErrorKind::Write(e),
                    })?;

                    return Ok((
                        data.universe,
                        data.data.try_into().map_err(|e| CallError {
                            kind: CallErrorKind::InvalidBuffer(e),
                        })?,
                    ));
                }
            }
        }
    }

    /// Construct a new streaming async client from an async stream. The
    /// client is initialized with a fresh context. This usually shouldn't be
    /// called directly as `ClientConfig::connect_async()` will set up a
    /// stream for you before internally calling this.
    pub fn from_stream(stream: S) -> Self {
        Self {
            stream,
            ctx: RpcContext::new(),
            buf: BytesMut::new(),
        }
    }
}
