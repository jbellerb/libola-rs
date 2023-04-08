#[cfg(feature = "tokio")]
mod r#async;
mod sync;

#[cfg(feature = "tokio")]
pub use r#async::{connect_async, connect_async_with_config, StreamingClientAsync};
pub use sync::{connect, connect_with_config, StreamingClient};

use std::process::Command;
use std::time::Duration;

use crate::{Error, Result};

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

fn spawn_olad(config: &ClientConfig) -> Result<()> {
    let mut command = Command::new("olad");
    let command = command.args(["-r", &config.server_port.to_string(), "--syslog"]);

    #[cfg(not(target_os = "windows"))]
    let command = command.arg("--daemon");

    command.spawn().map_err(Error::AutoStart)?;

    Ok(())
}
