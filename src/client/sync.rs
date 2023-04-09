use std::io::Write;
use std::net::TcpStream;
use std::thread::sleep;

use super::{
    spawn_olad, CallError, CallErrorKind, ClientConfig, ConnectError, ConnectErrorKind,
    OLA_SPAWN_DELAY,
};
use crate::ola::proto::{DmxData, OlaServerServiceCall};
use crate::ola::RpcContext;
use crate::DmxBuffer;

use bytes::BytesMut;

#[derive(Debug)]
pub struct StreamingClient<S> {
    stream: S,
    context: RpcContext,
}

impl<S> StreamingClient<S> {
    /// Gets a reference to the underlying stream.
    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    /// Gets a mutable reference to the underlying stream.
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }
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
        self.context
            .encode(request, &mut buf)
            .map_err(|e| CallError {
                kind: CallErrorKind::Encode(e),
            })?;
        self.stream
            .write_all(&buf.freeze())
            .map_err(|e| CallError {
                kind: CallErrorKind::Write(e),
            })?;
        Ok(())
    }
}

pub fn connect_with_config(
    config: &ClientConfig,
) -> Result<StreamingClient<TcpStream>, ConnectError> {
    if config.auto_start {
        let stream = TcpStream::connect(("127.0.0.1", config.server_port));

        if let Ok(stream) = stream {
            stream.set_nodelay(true).map_err(|e| ConnectError {
                kind: ConnectErrorKind::NoDelay(e),
            })?;

            return Ok(StreamingClient {
                stream,
                context: RpcContext::new(),
            });
        } else {
            spawn_olad(config).map_err(|e| ConnectError {
                kind: ConnectErrorKind::Spawn(e),
            })?;
            sleep(OLA_SPAWN_DELAY);
        }
    }

    let stream =
        TcpStream::connect(("127.0.0.1", config.server_port)).map_err(|e| ConnectError {
            kind: ConnectErrorKind::Connect(e),
        })?;
    stream.set_nodelay(true).map_err(|e| ConnectError {
        kind: ConnectErrorKind::NoDelay(e),
    })?;

    Ok(StreamingClient {
        stream,
        context: RpcContext::new(),
    })
}

pub fn connect() -> Result<StreamingClient<TcpStream>, ConnectError> {
    connect_with_config(&ClientConfig::new())
}
