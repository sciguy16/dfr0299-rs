// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{Error, Result, START, STOP, VERSION};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Command {
    // "Control" messages
    Next,
    Previous,
    Track(u16),
    IncreaseVolume,
    DecreaseVolume,
    SetVolume(u16),
    SetEq(EqMode),
    SetPlaybackMode(PlaybackMode),
    SetPlaybackSource(PlaybackSource),
    Standby,
    Wake,
    Reset,
    Playback,
    Pause,
    SetFolder(u8, u8),
    SetVolumeAdjust(u16),
    RepeatPlay(u16),

    // "Command" messages
    Stay1,
    Stay2,
    Stay3,
    InitialisationParameters(u16),
    RequestRetransmission,
    Reply,
    GetStatus,
    GetVolume,
    GetEq,
    GetPlaybackMode,
    GetSoftwareVersion,
    GetTfFileCount,
    GetUDiskFileCount,
    GetFlashFileCount,
    KeepOn,
    GetTfCurrentTrack,
    GetUDiskCurrentTrack,
    GetFlashCurrentTrack,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum EqMode {
    Normal = 0x00,
    Pop = 0x01,
    Rock = 0x02,
    Jazz = 0x03,
    Classic = 0x04,
    Base = 0x05,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum PlaybackMode {
    Repeat = 0x00,
    FolderRepeat = 0x01,
    SingleRepeat = 0x02,
    Random = 0x03,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum PlaybackSource {
    UDisk = 0x00,
    Tf = 0x01,
    Aux = 0x02,
    Sleep = 0x03,
    Flash = 0x04,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum RequestAck {
    No = 0x00,
    Yes = 0x01,
}

impl Command {
    pub fn serialise(&self, buf: &mut [u8]) -> Result<usize> {
        self.serialise_with_ack(buf, RequestAck::No)
    }

    /// serialisation format is:
    /// * START = 0x7e
    /// * VERSION = 0xff
    /// * LEN = number of bytes to follow, inc. LEN & VER, not inc. checksum
    /// * CMD = the command
    /// * FEEDBACK = whether to request ack/feedback
    /// * PARAM_H = parameter high byte
    /// * PARAM_L = parameter low byte
    /// * CHECKSUM_H = checksum high byte
    /// * CHECKSUM_L = checksum low byte
    /// * STOP = 0xef
    pub fn serialise_with_ack(
        &self,
        buf: &mut [u8],
        request_ack: RequestAck,
    ) -> Result<usize> {
        const LEN: usize = 10;

        if buf.len() < LEN {
            return Err(Error::BufferTooShort);
        }

        let param = self.param();

        buf[0] = START;
        buf[1] = VERSION;
        buf[2] = 0x06; // LEN
        buf[3] = self.command_byte();
        buf[4] = request_ack as u8;
        buf[5] = (param >> 8) as u8;
        buf[6] = param as u8;

        // checksum is twos complement of the sum of the data bytes
        let checksum: i16 = buf[1..7].iter().cloned().map(i16::from).sum();
        let checksum = -checksum;

        buf[7] = (checksum >> 8) as u8;
        buf[8] = checksum as u8;
        buf[9] = STOP;
        Ok(LEN)
    }

    pub fn command_byte(&self) -> u8 {
        use Command::*;
        match self {
            // "Control" messages
            Next => 0x01,
            Previous => 0x02,
            Track(_) => 0x03,
            IncreaseVolume => 0x04,
            DecreaseVolume => 0x05,
            SetVolume(_) => 0x06,
            SetEq(_) => 0x07,
            SetPlaybackMode(_) => 0x08,
            SetPlaybackSource(_) => 0x09,
            Standby => 0x0a,
            Wake => 0x0b,
            Reset => 0x0c,
            Playback => 0x0d,
            Pause => 0x0e,
            SetFolder(..) => 0x0f,
            SetVolumeAdjust(_) => 0x10,
            RepeatPlay(_) => 0x11,

            // "Command messages"
            Stay1 => 0x3c,
            Stay2 => 0x3d,
            Stay3 => 0x3e,
            InitialisationParameters(_) => 0x3f,
            RequestRetransmission => 0x40,
            Reply => 0x41,
            GetStatus => 0x41,
            GetVolume => 0x43,
            GetEq => 0x44,
            GetPlaybackMode => 0x45,
            GetSoftwareVersion => 0x46,
            GetTfFileCount => 0x47,
            GetUDiskFileCount => 0x48,
            GetFlashFileCount => 0x49,
            KeepOn => 0x4a,
            GetTfCurrentTrack => 0x4b,
            GetUDiskCurrentTrack => 0x4c,
            GetFlashCurrentTrack => 0x4d,
        }
    }

    pub fn param(&self) -> u16 {
        use Command::*;
        match self {
            Track(t) => *t,
            SetVolume(v) => *v,
            SetEq(e) => *e as u16,
            SetPlaybackMode(m) => *m as u16,
            SetPlaybackSource(s) => *s as u16,
            SetFolder(folder, file) => u16::from_be_bytes([*folder, *file]),
            SetVolumeAdjust(v) => *v as u16,
            RepeatPlay(r) => *r as u16,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn serialise_command_track() {
        let mut buf = [0; 16];
        let cmd = Command::Track(1);
        let len = cmd.serialise(&mut buf).unwrap();
        assert_eq!(len, 10);
        let expected: &[u8] = &[
            0x7e, // START
            0xff, // VERSION
            0x06, // LEN
            0x03, // command
            0x00, // request ack
            0x00, // param high
            0x01, // param low
            0xfe, // checksum high (datasheet says 0xff but is wrong)
            0xf7, // checksum low (datasheet says 0xe6)
            0xef, // STOP
        ];
        eprintln!("< calculated / expected >");
        assert_eq!(&buf[..len], expected);
    }

    #[test]
    fn serialise_command_norflash() {
        let mut buf = [0; 16];
        let cmd = Command::SetPlaybackSource(PlaybackSource::Flash);
        let len = cmd.serialise(&mut buf).unwrap();
        assert_eq!(len, 10);
        let expected: &[u8] = &[
            0x7e, // START
            0xff, // VERSION
            0x06, // LEN
            0x09, // command
            0x00, // request ack
            0x00, // param high
            0x04, // param low
            0xfe, // checksum high (datasheet says 0xff)
            0xee, // checksum low (datasheet says 0xdd)
            0xef, // STOP
        ];
        eprintln!("< calculated / expected >");
        assert_eq!(&buf[..len], expected);
    }
}
