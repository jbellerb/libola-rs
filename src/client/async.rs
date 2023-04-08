use super::{spawn_olad, ClientConfig, OLA_SPAWN_DELAY};
use crate::ola::proto::{DmxData, OlaServerServiceCall};
use crate::ola::RpcContext;
use crate::{DmxBuffer, Result};

use bytes::BytesMut;
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    time::sleep,
};

#[derive(Debug)]
pub struct StreamingClientAsync<S> {
    stream: S,
    ctx: RpcContext,
}

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

pub async fn connect_async_with_config(
    config: &ClientConfig,
) -> Result<StreamingClientAsync<TcpStream>> {
    if config.auto_start {
        let stream = TcpStream::connect(("127.0.0.1", config.server_port)).await;

        if let Ok(stream) = stream {
            stream.set_nodelay(true)?;

            return Ok(StreamingClientAsync {
                stream,
                ctx: RpcContext::new(),
            });
        } else {
            spawn_olad(config)?;
            sleep(OLA_SPAWN_DELAY).await;
        }
    }

    let stream = TcpStream::connect(("127.0.0.1", config.server_port)).await?;
    stream.set_nodelay(true)?;

    Ok(StreamingClientAsync {
        stream,
        ctx: RpcContext::new(),
    })
}

pub async fn connect_async() -> Result<StreamingClientAsync<TcpStream>> {
    connect_async_with_config(&ClientConfig::new()).await
}
