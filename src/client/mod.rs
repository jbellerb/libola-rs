#[cfg(feature = "tokio")]
mod r#async;
mod sync;

#[cfg(feature = "tokio")]
pub use r#async::{connect_async, connect_async_with_config, StreamingClientAsync};
pub use sync::{connect, connect_with_config, StreamingClient};

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::process::Command;
use std::time::Duration;

use crate::ola::MessageEncodeError;

const OLA_DEFAULT_PORT: u16 = 9010;
const OLA_SPAWN_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub auto_start: bool,
    pub server_port: u16,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            server_port: OLA_DEFAULT_PORT,
        }
    }
}

impl ClientConfig {
    /// Create a new `StreamingClientConfig`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// The error type returned when spawning `olad` fails.
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

/// The error type returned when an RCP call fails.
#[derive(Debug)]
#[non_exhaustive]
pub struct CallError {
    pub kind: CallErrorKind,
}

impl Display for CallError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to call RPC function")
    }
}

impl Error for CallError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            CallErrorKind::Encode(e) => Some(e),
            CallErrorKind::Write(e) => Some(e),
        }
    }
}

/// Enum to store the various types of errors that can occur when making an RPC
/// call to OLA.
#[derive(Debug)]
#[non_exhaustive]
pub enum CallErrorKind {
    /// Failure encoding an RPC message.
    Encode(MessageEncodeError),
    /// Failure writing an RPC message to the underlying socket.
    Write(std::io::Error),
}

fn spawn_olad(config: &ClientConfig) -> Result<(), SpawnOladError> {
    let mut command = Command::new("olad");
    let command = command.args(["-r", &config.server_port.to_string(), "--syslog"]);

    #[cfg(not(target_os = "windows"))]
    let command = command.arg("--daemon");

    command.spawn().map_err(SpawnOladError)?;

    Ok(())
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
