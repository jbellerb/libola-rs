use std::io::Write;
use std::net::TcpStream;
use std::process::Command;

use crate::ola::proto::DmxData;
use crate::ola::rpc::{RpcMessage, Type as RpcType};
use crate::{DmxBuffer, Result, OLA_DEFAULT_PORT, PROTOCOL_VERSION, SIZE_MASK, VERSION_MASK};

use bytes::{BufMut, BytesMut};
use prost::Message;

#[derive(Clone, Debug)]
pub struct StreamingClient {
    auto_start: bool,
    server_port: u16,
}

impl Default for StreamingClient {
    fn default() -> Self {
        Self {
            auto_start: true,
            server_port: OLA_DEFAULT_PORT,
        }
    }
}

impl StreamingClient {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn auto_start(self, auto_start: bool) -> Self {
        Self { auto_start, ..self }
    }

    pub fn port(self, port: u16) -> Self {
        Self {
            server_port: port,
            ..self
        }
    }

    pub fn connect(self) -> Result<StreamingClientChannel<TcpStream>> {
        if self.auto_start {
            let stream = TcpStream::connect(("127.0.0.1", self.server_port));

            if let Ok(stream) = stream {
                return Ok(StreamingClientChannel {
                    stream,
                    sequence_number: 0,
                });
            } else {
                let mut command = Command::new("olad");
                let command = command.arg("--syslog");

                #[cfg(not(target_os = "windows"))]
                let command = command.arg("--daemon");

                command.spawn()?;
            }
        }

        let stream = TcpStream::connect(("127.0.0.1", self.server_port))?;

        Ok(StreamingClientChannel {
            stream,
            sequence_number: 0,
        })
    }
}

pub struct StreamingClientChannel<Stream> {
    stream: Stream,
    sequence_number: u32,
}

impl<Stream: Write> StreamingClientChannel<Stream> {
    fn next_sequence(&mut self) -> u32 {
        let number = self.sequence_number;
        self.sequence_number += 1;

        number
    }

    pub fn send_dmx(&mut self, universe: u32, data: &DmxBuffer) -> Result<()> {
        self.send_dmx_with_priority(universe, data, 100)
    }

    pub fn send_dmx_with_priority(
        &mut self,
        universe: u32,
        data: &DmxBuffer,
        priority: u8,
    ) -> Result<()> {
        let request = DmxData {
            universe: universe as i32,
            data: data.to_vec(),
            priority: Some(priority as i32),
        };

        let message = RpcMessage {
            r#type: RpcType::StreamRequest as i32,
            id: Some(self.next_sequence()),
            name: Some("StreamDmxData".to_string()),
            buffer: Some(request.encode_to_vec()),
        };

        let size = message.encoded_len();
        let mut bytes = BytesMut::with_capacity(size + 4);

        bytes.put_u32_le(encode_header(size));
        let mut header = bytes.split();
        message.encode(&mut bytes).unwrap();
        header.unsplit(bytes);

        self.stream.write_all(&header)?;
        Ok(())
    }
}

fn encode_header(size: usize) -> u32 {
    let mut header = size as u32 & SIZE_MASK;
    header |= (PROTOCOL_VERSION << 28) & VERSION_MASK;

    header
}
