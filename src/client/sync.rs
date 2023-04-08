use std::io::Write;
use std::net::TcpStream;
use std::thread::sleep;

use super::{spawn_olad, ClientConfig, OLA_SPAWN_DELAY};
use crate::ola::proto::{DmxData, OlaServerServiceCall};
use crate::ola::RpcContext;
use crate::{DmxBuffer, Result};

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
    pub fn send_dmx(&mut self, universe: u32, data: &DmxBuffer) -> Result<()> {
        self.send_dmx_with_priority(universe, data, 100)
    }

    /// Send a DMX buffer to an OLA universe with a priority value.
    pub fn send_dmx_with_priority(
        &mut self,
        universe: u32,
        data: &DmxBuffer,
        priority: u8,
    ) -> Result<()> {
        let request = OlaServerServiceCall::StreamDmxData(DmxData {
            universe: universe as i32,
            data: data.to_vec(),
            priority: Some(priority as i32),
        });

        let mut buf = BytesMut::new();
        self.context.encode(request, &mut buf)?;
        self.stream.write_all(&buf.freeze())?;
        Ok(())
    }
}

pub fn connect_with_config(config: &ClientConfig) -> Result<StreamingClient<TcpStream>> {
    if config.auto_start {
        let stream = TcpStream::connect(("127.0.0.1", config.server_port));

        if let Ok(stream) = stream {
            stream.set_nodelay(true)?;

            return Ok(StreamingClient {
                stream,
                context: RpcContext::new(),
            });
        } else {
            spawn_olad(config)?;
            sleep(OLA_SPAWN_DELAY);
        }
    }

    let stream = TcpStream::connect(("127.0.0.1", config.server_port))?;
    stream.set_nodelay(true)?;

    Ok(StreamingClient {
        stream,
        context: RpcContext::new(),
    })
}

pub fn connect() -> Result<StreamingClient<TcpStream>> {
    connect_with_config(&ClientConfig::new())
}
