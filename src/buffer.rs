use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, Index, IndexMut};

/// A fixed-size byte buffer representing the state of a single DMX universe.
///
/// `DmxBuffer` holds exactly 512 bytes and is stored on the heap. All OLA
/// methods dealing with retrieving or setting universes are done with
/// `DmxBuffer`s. Various "buffer-like" types including `[u8; 512]` and
/// `Vec<u8>` can be converted to and from `DmxBuffer`s, but must be exactly
/// 512 bytes long for conversion to succeed.
///
/// # Examples
///
/// ```
/// # use ola::DmxBuffer;
/// let mut universe = DmxBuffer::new();
/// assert_eq!(*universe, [0; 512]);
/// assert_eq!(universe, [0; 512].into());
///
/// universe[0] = 255;
/// assert_eq!(universe[0], 255);
///
/// let mut other = universe.clone();
/// assert_eq!(other[0], 255);
///
/// other.zero();
/// assert_eq!(*other, [0; 512]);
/// assert_ne!(*universe, [0; 512]);
/// ```
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
    /// Construct a DMX buffer and move `b`'s values into it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ola::DmxBuffer;
    /// assert_eq!(DmxBuffer::from([0; 512]), DmxBuffer::new());
    /// ```
    fn from(b: [u8; 512]) -> Self {
        Self(Box::new(b))
    }
}

impl From<&[u8; 512]> for DmxBuffer {
    /// Construct a DMX buffer and copy `b`'s values into it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ola::DmxBuffer;
    /// assert_eq!(DmxBuffer::from(&[0; 512]), DmxBuffer::new());
    /// ```
    fn from(b: &[u8; 512]) -> Self {
        Self(Box::new(*b))
    }
}

impl From<DmxBuffer> for Vec<u8> {
    /// Convert a DMX buffer into a vector by transferring ownership of the
    /// existing heap allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ola::DmxBuffer;
    /// assert_eq!(Vec::<u8>::from(DmxBuffer::new()), vec![0; 512]);
    /// ```
    fn from(b: DmxBuffer) -> Self {
        (b.0 as Box<[u8]>).into()
    }
}

impl TryFrom<&[u8]> for DmxBuffer {
    type Error = TryFromBufferError;

    /// Tries to create a DMX buffer by copying from a slice. Succeeds if
    /// `b.len() == 512`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ola::{DmxBuffer, TryFromBufferError};
    /// # fn main() -> Result<(), TryFromBufferError> {
    /// let bytes = [0; 1024];
    ///
    /// assert_eq!(DmxBuffer::try_from(&bytes[0..512])?, DmxBuffer::new());
    /// # Ok(()) }
    /// ```
    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let buffer = b.try_into().map_err(|_| TryFromBufferError(()))?;

        Ok(Self(Box::new(buffer)))
    }
}

impl TryFrom<Vec<u8>> for DmxBuffer {
    type Error = TryFromBufferError;

    /// Construct a DMX buffer by transferring ownership of a vector's existing
    /// heap allocation. Succeeds if `b.len() == 512`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ola::{DmxBuffer, TryFromBufferError};
    /// # fn main() -> Result<(), TryFromBufferError> {
    /// let bytes = vec![0; 512];
    ///
    /// assert_eq!(DmxBuffer::try_from(bytes)?, DmxBuffer::new());
    /// # Ok(()) }
    /// ```
    fn try_from(b: Vec<u8>) -> Result<Self, Self::Error> {
        let buffer = b.try_into().map_err(|_| TryFromBufferError(()))?;

        Ok(Self(buffer))
    }
}

impl Deref for DmxBuffer {
    type Target = [u8; 512];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DmxBuffer {
    /// Construct an empty (zeroed) DMX buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ola::DmxBuffer;
    /// let universe = DmxBuffer::new();
    ///
    /// assert_eq!(*universe, [0; 512]);
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Set all values in a DMX buffer to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ola::DmxBuffer;
    /// let mut universe = DmxBuffer::new();
    ///
    /// universe[0] = 255;
    /// universe[1] = 128;
    /// assert_ne!(*universe, [0; 512]);
    ///
    /// universe.zero();
    /// assert_eq!(*universe, [0; 512]);
    /// ```
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

/// The error type returned when a conversion to a DMX buffer fails.
#[derive(Clone, Debug)]
pub struct TryFromBufferError(());

impl Display for TryFromBufferError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to convert to DMX buffer")
    }
}
