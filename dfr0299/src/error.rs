// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum Error {
    BufferTooShort,
    BadChecksum,
    InvalidCommand(u8),
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
