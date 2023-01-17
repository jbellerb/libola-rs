use olaclient::DmxBuffer;

fn main() {
    let mut channel = olaclient::connect().unwrap();
    let mut buffer = DmxBuffer::new();

    // Send 256 frames to the server, incrementing channel 0 each frame
    for i in 0..=255 {
        buffer.set_channel(0, i);
        channel.send_dmx(1, &buffer).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
