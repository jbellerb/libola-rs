use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::ola::proto::{DmxData, OlaServerServiceCall};
use crate::ola::RpcContext;
use crate::{DmxBuffer, Error, Result, OLA_DEFAULT_PORT};

use bytes::BytesMut;

#[cfg(feature = "tokio")]
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    net::TcpStream as TokioTcpStream,
    time::sleep as tokio_sleep,
};

#[derive(Clone, Debug)]
pub struct StreamingClientConfig {
    pub auto_start: bool,
    pub server_port: u16,
}

impl Default for StreamingClientConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            server_port: OLA_DEFAULT_PORT,
        }
    }
}

impl StreamingClientConfig {
    /// Create a new `StreamingClientConfig`.
    pub fn new() -> Self {
        Default::default()
    }
}

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

#[cfg(feature = "tokio")]
#[derive(Debug)]
pub struct StreamingClientAsync<S> {
    stream: S,
    ctx: RpcContext,
}

#[cfg(feature = "tokio")]
impl<S: AsyncWrite + Unpin> StreamingClientAsync<S> {
    pub async fn send_dmx(&mut self, universe: u32, data: &DmxBuffer) -> Result<()> {
        self.send_dmx_with_priority(universe, data, 100).await
    }

    pub async fn send_dmx_with_priority(
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
        self.ctx.encode(request, &mut buf)?;
        self.stream.write_all(&buf).await?;
        Ok(())
    }
}

pub fn connect_with_config(config: StreamingClientConfig) -> Result<StreamingClient<TcpStream>> {
    if config.auto_start {
        let stream = TcpStream::connect(("127.0.0.1", config.server_port));

        if let Ok(stream) = stream {
            stream.set_nodelay(true)?;

            return Ok(StreamingClient {
                stream,
                context: RpcContext::new(),
            });
        } else {
            let mut command = Command::new("olad");
            let command = command.args(["-r", &config.server_port.to_string(), "--syslog"]);

            #[cfg(not(target_os = "windows"))]
            let command = command.arg("--daemon");

            command.spawn().map_err(Error::AutoStart)?;

            // The official client sleeps for 1 second
            sleep(Duration::from_secs(1));
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
    connect_with_config(StreamingClientConfig::new())
}

#[cfg(feature = "tokio")]
pub async fn connect_async_with_config(
    config: StreamingClientConfig,
) -> Result<StreamingClientAsync<TokioTcpStream>> {
    if config.auto_start {
        let stream = TokioTcpStream::connect(("127.0.0.1", config.server_port)).await;

        if let Ok(stream) = stream {
            stream.set_nodelay(true)?;

            return Ok(StreamingClientAsync {
                stream,
                ctx: RpcContext::new(),
            });
        } else {
            let mut command = Command::new("olad");
            let command = command.args(["-r", &config.server_port.to_string(), "--syslog"]);

            #[cfg(not(target_os = "windows"))]
            let command = command.arg("--daemon");

            command.spawn().map_err(Error::AutoStart)?;

            // The official client sleeps for 1s
            tokio_sleep(Duration::from_secs(1)).await;
        }
    }

    let stream = TokioTcpStream::connect(("127.0.0.1", config.server_port)).await?;
    stream.set_nodelay(true)?;

    Ok(StreamingClientAsync {
        stream,
        ctx: RpcContext::new(),
    })
}

#[cfg(feature = "tokio")]
pub async fn connect_async() -> Result<StreamingClientAsync<TokioTcpStream>> {
    connect_async_with_config(StreamingClientConfig::new()).await
}
