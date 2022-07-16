use crate::{Error, Result, START, STOP, VERSION};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Control {
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
    Normal,
    Reset,
    Playback,
    Pause,
    SetFolder(u8, u8),
    SetVolumeAdjust(u16),
    RepeatPlay(u16),
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

impl Control {
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
    pub fn serialise(&self, buf: &mut [u8]) -> Result<usize> {
        const LEN: usize = 10;

        if buf.len() < LEN {
            return Err(Error::BufferTooShort);
        }

        let param = self.param();

        buf[0] = START;
        buf[1] = VERSION;
        buf[2] = 0x06; // LEN
        buf[3] = self.command_byte();
        buf[4] = RequestAck::No as u8;
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
        match self {
            Control::Next => 0x01,
            Control::Previous => 0x02,
            Control::Track(_) => 0x03,
            Control::IncreaseVolume => 0x04,
            Control::DecreaseVolume => 0x05,
            Control::SetVolume(_) => 0x06,
            Control::SetEq(_) => 0x07,
            Control::SetPlaybackMode(_) => 0x08,
            Control::SetPlaybackSource(_) => 0x09,
            Control::Standby => 0x0a,
            Control::Normal => 0x0b,
            Control::Reset => 0x0c,
            Control::Playback => 0x0d,
            Control::Pause => 0x0e,
            Control::SetFolder(..) => 0x0f,
            Control::SetVolumeAdjust(_) => 0x10,
            Control::RepeatPlay(_) => 0x11,
        }
    }

    pub fn param(&self) -> u16 {
        match self {
            Control::Track(t) => *t,
            Control::SetVolume(v) => *v,
            Control::SetEq(e) => *e as u16,
            Control::SetPlaybackMode(m) => *m as u16,
            Control::SetPlaybackSource(s) => *s as u16,
            Control::SetFolder(folder, file) => {
                u16::from_be_bytes([*folder, *file])
            }
            Control::SetVolumeAdjust(v) => *v as u16,
            Control::RepeatPlay(r) => *r as u16,
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
        let cmd = Control::Track(1);
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
        let cmd = Control::SetPlaybackSource(PlaybackSource::Flash);
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
