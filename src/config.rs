//! Connection configuration.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::net::TcpStream;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[cfg(feature = "tokio")]
use crate::client::ClientAsync;
use crate::client::StreamingClient;

#[cfg(feature = "tokio")]
use tokio::{net::TcpStream as TokioTcpStream, time::sleep as tokio_sleep};

const OLA_DEFAULT_PORT: u16 = 9010;
const OLA_SPAWN_DELAY: Duration = Duration::from_secs(1);

/// Configuration for connecting to OLA.
///
/// `Config` is used to create a connection with OLA. By default, a connection
/// to OLA will be attempted on `127.0.0.1:9010`. If this fails, `olad` will be
/// started on that port. Automatically starting OLA can be disabled by setting
/// `auto_start` to false.
#[derive(Clone, Debug)]
pub struct Config {
    /// Whether to auto-start `olad` if a connection to OLA cannot be made.
    pub auto_start: bool,
    /// What port OLA's RPC is listening on. This is also the port OLA will be
    /// configured to listen on if auto-started.
    pub server_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_start: true,
            server_port: OLA_DEFAULT_PORT,
        }
    }
}

impl Config {
    /// Build a new configuration with auto-start enabled and the default port
    /// of 9010.
    pub fn new() -> Self {
        Default::default()
    }

    fn spawn_olad(&self) -> Result<(), SpawnOladError> {
        let mut command = Command::new("olad");
        let command = command.args(["-r", &self.server_port.to_string(), "--syslog"]);

        #[cfg(not(target_os = "windows"))]
        let command = command.arg("--daemon");

        command.spawn().map_err(SpawnOladError)?;

        Ok(())
    }

    /// Connect to OLA with the synchronous client. Fails if a connection
    /// cannot be established and (when `auto_start` is enabled) if `olad`
    /// cannot be started.
    pub fn connect(&self) -> Result<StreamingClient<TcpStream>, ConnectError> {
        let endpoint = ("127.0.0.1", self.server_port);

        if self.auto_start {
            let stream = TcpStream::connect(endpoint);

            if let Ok(stream) = stream {
                stream.set_nodelay(true).map_err(|e| ConnectError {
                    kind: ConnectErrorKind::NoDelay(e),
                })?;

                return Ok(StreamingClient::from_stream(stream));
            } else {
                self.spawn_olad().map_err(|e| ConnectError {
                    kind: ConnectErrorKind::Spawn(e),
                })?;
                sleep(OLA_SPAWN_DELAY);
            }
        }

        let stream = TcpStream::connect(endpoint).map_err(|e| ConnectError {
            kind: ConnectErrorKind::Connect(e),
        })?;
        stream.set_nodelay(true).map_err(|e| ConnectError {
            kind: ConnectErrorKind::NoDelay(e),
        })?;

        Ok(StreamingClient::from_stream(stream))
    }

    /// Connect to OLA with the asynchronous client. Fails if a connection
    /// cannot be established and (when `auto_start` is enabled) if `olad`
    /// cannot be started.
    #[cfg(feature = "tokio")]
    pub async fn connect_async(&self) -> Result<ClientAsync<TokioTcpStream>, ConnectError> {
        let endpoint = ("127.0.0.1", self.server_port);

        if self.auto_start {
            let stream = TokioTcpStream::connect(endpoint).await;

            if let Ok(stream) = stream {
                stream.set_nodelay(true).map_err(|e| ConnectError {
                    kind: ConnectErrorKind::NoDelay(e),
                })?;

                return Ok(ClientAsync::from_stream(stream));
            } else {
                self.spawn_olad().map_err(|e| ConnectError {
                    kind: ConnectErrorKind::Spawn(e),
                })?;
                tokio_sleep(OLA_SPAWN_DELAY).await;
            }
        }

        let stream = TokioTcpStream::connect(endpoint)
            .await
            .map_err(|e| ConnectError {
                kind: ConnectErrorKind::Connect(e),
            })?;
        stream.set_nodelay(true).map_err(|e| ConnectError {
            kind: ConnectErrorKind::NoDelay(e),
        })?;

        Ok(ClientAsync::from_stream(stream))
    }
}

/// The error type returned when connecting to OLA fails.
#[derive(Debug)]
#[non_exhaustive]
pub struct ConnectError {
    pub kind: ConnectErrorKind,
}

impl Display for ConnectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to connect to OLA")
    }
}

impl Error for ConnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            ConnectErrorKind::Connect(e) => Some(e),
            ConnectErrorKind::NoDelay(e) => Some(e),
            ConnectErrorKind::Spawn(e) => Some(e),
        }
    }
}

/// Enum to store the various types of errors that can cause connecting to
/// OLA to fail.
#[derive(Debug)]
pub enum ConnectErrorKind {
    /// Unable to establish TCP connection with OLA.
    Connect(std::io::Error),
    /// Problem while setting `TCP_NODELAY` on the underlying socket.
    NoDelay(std::io::Error),
    /// Failure while attempting to auto-start `olad`.
    Spawn(SpawnOladError),
}

/// The error type returned when spawning `olad` fails.
#[derive(Debug)]
pub struct SpawnOladError(std::io::Error);

impl Display for SpawnOladError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to start olad")
    }
}

impl Error for SpawnOladError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}
