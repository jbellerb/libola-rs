/// Result type returned from methods with olaclient `Errors`s.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents errors that can occur when talking to `olad`.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O errors, usually with the underlying connection.
    #[error("connection error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors related to auto-starting `olad`.
    #[error("failed to start olad: {0}")]
    AutoStart(#[source] std::io::Error),

    /// Encode buffer is too small.
    #[error("buffer too small to write message to")]
    Encode(),
}
