use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::ola::proto::DmxData;
use crate::ola::{RpcCall, RpcContext};
use crate::{DmxBuffer, Error, Result, OLA_DEFAULT_PORT};

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
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug)]
pub struct StreamingClient<S> {
    stream: S,
    ctx: RpcContext,
}

impl<S: Write> StreamingClient<S> {
    pub fn send_dmx(&mut self, universe: u32, data: &DmxBuffer) -> Result<()> {
        self.send_dmx_with_priority(universe, data, 100)
    }

    pub fn send_dmx_with_priority(
        &mut self,
        universe: u32,
        data: &DmxBuffer,
        priority: u8,
    ) -> Result<()> {
        let request = RpcCall::StreamDmxData(DmxData {
            universe: universe as i32,
            data: data.to_vec(),
            priority: Some(priority as i32),
        });

        self.stream.write_all(&self.ctx.encode(request))?;
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
                ctx: RpcContext::new(),
            });
        } else {
            let mut command = Command::new("olad");
            let command = command.args(["-r", &config.server_port.to_string(), "--syslog"]);

            #[cfg(not(target_os = "windows"))]
            let command = command.arg("--daemon");

            command.spawn().map_err(Error::AutoStart)?;

            // The official client sleeps for 1s
            sleep(Duration::from_secs(1));
        }
    }

    let stream = TcpStream::connect(("127.0.0.1", config.server_port))?;
    stream.set_nodelay(true)?;

    Ok(StreamingClient {
        stream,
        ctx: RpcContext::new(),
    })
}

pub fn connect() -> Result<StreamingClient<TcpStream>> {
    connect_with_config(Default::default())
}
