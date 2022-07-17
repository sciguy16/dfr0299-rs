// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt::{self, Display, Formatter};

/// Error states for dfr0299. Includes errors for both serialisation
/// parsing
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Serialisation buffer should be at least 10 bytes long
    BufferTooShort,
    /// A complete packet was received but its checksum was invalid
    BadChecksum,
    /// Command not recognised, and the raw value is returned
    InvalidCommand(u8),
    /// An attempt to parse a parameter value into one of the parameter
    /// enums failed
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
