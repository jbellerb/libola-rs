#[cfg(feature = "tokio")]
mod r#async;
mod sync;

#[cfg(feature = "tokio")]
pub use r#async::ClientAsync;
pub use sync::StreamingClient;

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::ola::{MessageDecodeError, MessageEncodeError};
use crate::TryFromBufferError;

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
            CallErrorKind::Decode(e) => Some(e),
            CallErrorKind::InvalidBuffer(e) => Some(e),
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
    /// Failure decoding an RPC message.
    Decode(MessageDecodeError),
    /// RPC message contained an invalid DMX buffer.
    InvalidBuffer(TryFromBufferError),
    /// Failure writing an RPC message to the underlying socket.
    Write(std::io::Error),
}
