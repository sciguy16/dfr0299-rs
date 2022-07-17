// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{Error, Result};
use num_enum::TryFromPrimitive;

/// Possible messages we may receive from the DFR0299.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
pub enum Response {
    /// Response to any command that has the "request ACK" field set to
    /// `true`. Not documented in the datasheet, but present in the
    /// official arduino library and has been observed to be correct
    /// with a sample of one device:
    ///
    /// <https://github.com/DFRobot/DFRobotDFPlayerMini/blob/master/DFRobotDFPlayerMini.cpp#L152>
    ///
    /// "//handle the 0x41 ack feedback as a spcecial case, in case the
    /// pollusion of _handleCommand, _handleParameter, and _handleType."
    Ack,
    /// Report that the specified disk is connected and online.
    /// Attempting to play a track before the device sends this message
    /// may or may not be successfull.
    DiskOnline(Disk),
    /// Report that playback is complete from the UDisk source
    UDiskFinishPlayback(u16),
    /// Report that playback is complete from the SD card source
    TfFinishPlayback(u16),
    /// Report that playback is complete from the flash chip source
    FlashFinishPlayback(u16),
    /// The device encountered an error
    ModuleError(ModuleErrorType),
    /// Report that the specified disk has been inserted. Note that
    /// attempting to power an SD card off 3.3v will result in repeated
    /// 'inserted' and 'removed' messages and it won't work reliably.
    DiskInserted(Disk),
    /// Report that the specified disk has been removed
    DiskRemoved(Disk),
}

/// Disk types that the device might report the status of. Note that
/// the definitions here are slightly different to those of
/// `Control::PlaybackSource`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Disk {
    UDisk = 0x01,
    Tf = 0x02,
    /// I don't know what PC is here - perhaps something to do with USB
    /// playback?
    Pc = 0x03,
    Flash = 0x04,
    UDiskAndFlash = 0x05,
}

/// Possible error states reported by the device
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum ModuleErrorType {
    Busy = 0x00,
    IncompleteFrameReceived = 0x01,
    /// I don't know when this error is returned, as the module didn't
    /// send it when I intentionally sent it a packet with an invalid
    /// checksum
    ChecksumError = 0x02,
}

impl Response {
    /// Parse a command byte and its two parameter bytes into a
    /// `Response`. Response types which don't have parameters ignore
    /// the supplied parameter bytes. If the command byte does not match
    /// any known command then the byte is returned with an
    /// `Error::InvalidCommand`.
    pub fn parse(cmd: u8, param_h: u8, param_l: u8) -> Result<Self> {
        use Response::*;

        let param = u16::from_be_bytes([param_h, param_l]);

        Ok(match cmd {
            0x40 => ModuleError(ModuleErrorType::try_from(param_l)?),
            0x3a => DiskInserted(Disk::try_from(param_l)?),
            0x3b => DiskRemoved(Disk::try_from(param_l)?),
            0x3c => UDiskFinishPlayback(param),
            0x3d => TfFinishPlayback(param),
            0x3e => FlashFinishPlayback(param),
            0x3f => DiskOnline(Disk::try_from(param_l)?),
            0x41 => Ack,
            cmd => return Err(Error::InvalidCommand(cmd)),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_tf_insert() {
        let cmd = 0x3a;
        let param_h = 0x00;
        let param_l = 0x02;

        let resp = Response::parse(cmd, param_h, param_l).unwrap();
        assert_eq!(resp, Response::DiskInserted(Disk::Tf));
    }

    #[test]
    fn parse_checksum_error() {
        let cmd = 0x40;
        let param_h = 0x00;
        let param_l = 0x02;

        let resp = Response::parse(cmd, param_h, param_l).unwrap();
        assert_eq!(resp, Response::ModuleError(ModuleErrorType::ChecksumError));
    }
}
