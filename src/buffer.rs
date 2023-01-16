use std::ops::Deref;

pub struct DmxBuffer(Box<[u8; 512]>);

impl Default for DmxBuffer {
    fn default() -> Self {
        Self(Box::new([0; 512]))
    }
}

impl Deref for DmxBuffer {
    type Target = [u8; 512];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DmxBuffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn zero(&mut self) {
        self.0.fill(0);
    }

    pub fn get_channel(&mut self, channel: usize) -> u8 {
        self.0[channel]
    }

    pub fn set_channel(&mut self, channel: usize, value: u8) {
        self.0[channel] = value;
    }
}
