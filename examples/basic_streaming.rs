use anyhow::Result;
use ola::DmxBuffer;

fn main() -> Result<()> {
    let mut channel = ola::connect()?;
    let mut buffer = DmxBuffer::new();

    // Send 256 frames to the server, incrementing channel 0 each frame
    for i in 0..=255 {
        buffer.set_channel(0, i);
        channel.send_dmx(1, &buffer)?;
        std::thread::sleep(std::time::Duration::from_millis(25));
    }

    Ok(())
}
