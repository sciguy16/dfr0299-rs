use crate::{Error, Result};
use num_enum::TryFromPrimitive;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Response {
    DiskOnline(Disk),
    UDiskFinishPlayback(u16),
    TfFinishPlayback(u16),
    FlashFinishPlayback(u16),
    ModuleError(ModuleErrorType),
    DiskInserted(Disk),
    DiskRemoved(Disk),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Disk {
    UDisk = 0x01,
    Tf = 0x02,
    Pc = 0x03,
    Flash = 0x04,
    UDiskAndFlash = 0x05,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ModuleErrorType {
    Busy = 0x00,
    IncompleteFrameReceived = 0x01,
    ChecksumError = 0x02,
}

impl Response {
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
            _ => return Err(Error::InvalidCommand),
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
