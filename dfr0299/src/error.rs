use core::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum Error {
    BufferTooShort,
    BadChecksum,
    InvalidCommand,
    InvalidParameterValue,
}

impl<T: num_enum::TryFromPrimitive> From<num_enum::TryFromPrimitiveError<T>>
    for Error
{
    fn from(_: num_enum::TryFromPrimitiveError<T>) -> Self {
        Self::InvalidParameterValue
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{self:?}")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
