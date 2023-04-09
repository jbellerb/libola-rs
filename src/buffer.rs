use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, Index, IndexMut};

#[derive(Clone, Debug, Eq)]
pub struct DmxBuffer(Box<[u8; 512]>);

impl Default for DmxBuffer {
    fn default() -> Self {
        Self(Box::new([0; 512]))
    }
}

impl PartialEq for DmxBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl From<[u8; 512]> for DmxBuffer {
    fn from(value: [u8; 512]) -> Self {
        Self(Box::new(value))
    }
}

impl TryFrom<&[u8]> for DmxBuffer {
    type Error = TryFromBufferError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let buffer = value.try_into().map_err(|_| TryFromBufferError(()))?;

        Ok(Self(Box::new(buffer)))
    }
}

impl TryFrom<Vec<u8>> for DmxBuffer {
    type Error = TryFromBufferError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let buffer = value.try_into().map_err(|_| TryFromBufferError(()))?;

        Ok(Self(Box::new(buffer)))
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
}

impl Index<usize> for DmxBuffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for DmxBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

/// The error type returned when a conversion to a dmx buffer fails.
#[derive(Clone, Debug)]
pub struct TryFromBufferError(());

impl Display for TryFromBufferError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to convert to dmx buffer")
    }
}
