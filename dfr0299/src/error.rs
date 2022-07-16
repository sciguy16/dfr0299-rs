use core::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum Error {
    BufferTooShort,
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{self:?}")
    }
}

#[cfg(feature = "use_std")]
impl std::error::Error for Error {}
