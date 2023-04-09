use anyhow::Result;
use ola::DmxBuffer;

#[tokio::main]
async fn main() -> Result<()> {
    let mut channel = ola::connect_async().await?;
    let mut buffer = DmxBuffer::new();

    // Send 256 frames to the server, incrementing channel 0 each frame
    for i in 0..=255 {
        buffer[0] = i;
        channel.send_dmx(1, &buffer).await?;
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    Ok(())
}
