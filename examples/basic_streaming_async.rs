use ola::DmxBuffer;

#[tokio::main]
async fn main() {
    let mut channel = ola::connect_async().await.unwrap();
    let mut buffer = DmxBuffer::new();

    // Send 256 frames to the server, incrementing channel 0 each frame
    for i in 0..=255 {
        buffer.set_channel(0, i);
        channel.send_dmx(1, &buffer).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
}
