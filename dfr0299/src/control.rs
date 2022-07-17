// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

//! Definitions for Command and Control packet types.

use crate::{Error, Result, START, STOP, VERSION};

/// Available commands supported by the DFR0299
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
pub enum Command {
    // "Control" messages
    /// Advance to the next track (loops back to the first track if it
    /// is currently playing the last track)
    Next,
    /// Go to the previous track, looping to the final track if it is
    /// currently playing the first track
    Previous,
    /// Play the specified track. According to the datasheet the
    /// parameter for this command should be in the range 0-2999,
    /// however this crate does not restrict the values
    Track(u16),
    /// Increase the volume
    IncreaseVolume,
    /// Decrease the volume
    DecreaseVolume,
    /// Set the volume to the specified level. According to the
    /// datasheet the volume should be in the range 0-30, however this
    /// is not enforced by this crate.
    SetVolume(u16),
    /// Set the internal equaliser to the specified preset
    SetEq(EqMode),
    /// Set the repeat playback mode
    SetPlaybackMode(PlaybackMode),
    /// Set the playback source
    SetPlaybackSource(PlaybackSource),
    /// Enter standby mode. This does not cause the controller to enter
    /// any kind of sleep state, instead it disables the playback
    /// function
    /// <https://github.com/DFRobot/DFPlayer-Mini-mp3/issues/2>
    Standby,
    /// Wake from sleep (i.e. enter "normal" mode)
    Wake,
    /// Reset the controller
    Reset,
    /// Playback mode (I think this is as opposed to 'Pause')
    Playback,
    /// Pause current track
    Pause,
    /// Play the specified track from the given folder. Note that the
    /// folder name MUST be two ascii digits and the file name MUST be
    /// four ascii digits. For example `SetFolder { folder: 4, file: 123 }`
    /// refers to the file named '0123.mp3' in the folder '04'.
    SetFolder {
        /// Folder name (0-2999)
        folder: u8,
        /// File name
        file: u8,
    },
    /// Set some sort of gain parameter. According to the datasheet,
    /// `gain` should be in the range 0-31, but this is not checked here
    SetVolumeAdjust {
        /// Enable this gain parameter
        enable: bool,
        /// Gain (0-31)
        gain: u8,
    },
    /// `true` to start repeat play, `false` to stop repeat play
    RepeatPlay(bool),

    // "Command" messages
    /// Datasheet just says "STAY"
    Stay1,
    /// Datasheet just says "STAY"
    Stay2,
    /// Datasheet just says "STAY"
    Stay3,
    /// Send initialisation parameters to the device. According to the
    /// datasheet the parameter should be in the range 0x00-0x0f, but
    /// it doesn't say what any of these values actually represent.
    InitialisationParameters(u16),
    /// Datasheet says "returns an error, request retransmission"
    RequestRetransmission,
    /// Datasheet says "reply". This is the command number used by 'ACK'
    /// responses, but I'm not sure whether that's relevant here.
    /// Perhaps the DFR02999 will sometimes request an ACK from us?
    Reply,
    /// Query the current status
    GetStatus,
    /// Query the current volume
    GetVolume,
    /// Query the current EQ preset
    GetEq,
    /// Query the current playback mode
    GetPlaybackMode,
    /// Query the current software version
    GetSoftwareVersion,
    /// Count the number of files on the attached SD card
    GetTfFileCount,
    /// Count the number of files on the attached UDisk
    GetUDiskFileCount,
    /// Count the number of files on the attached flash chip
    GetFlashFileCount,
    /// Datasheet just says "keep on", whatever that means. Is there an
    /// automatic standby mode?
    KeepOn,
    /// Query the currently selected track on the SD card source
    GetTfCurrentTrack,
    /// Query the currently selected track on the UDisk source
    GetUDiskCurrentTrack,
    /// Query the currently selected track on the flash chip source
    GetFlashCurrentTrack,
}

/// EQ presets supported by the device
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
#[repr(u16)]
#[allow(missing_docs)]
pub enum EqMode {
    Normal = 0x00,
    Pop = 0x01,
    Rock = 0x02,
    Jazz = 0x03,
    Classic = 0x04,
    Base = 0x05,
}

/// Repeat modes supported by the device
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
#[repr(u16)]
#[allow(missing_docs)]
pub enum PlaybackMode {
    Repeat = 0x00,
    FolderRepeat = 0x01,
    SingleRepeat = 0x02,
    Random = 0x03,
}

/// Input data sources supported by the device. I don't know what
/// `Sleep` means here.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
#[repr(u16)]
#[allow(missing_docs)]
pub enum PlaybackSource {
    UDisk = 0x00,
    Tf = 0x01,
    Aux = 0x02,
    Sleep = 0x03,
    Flash = 0x04,
}

/// Whether to request an ACK from the device
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use_defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum RequestAck {
    No = 0x00,
    Yes = 0x01,
}

impl Command {
    /// Serialise the command into the provided buffer. The buffer size
    /// should be at least 10 bytes, otherwise an `Error::BufferTooShort`
    /// will be returned. On success returns the number of bytes written
    /// (this should always be 10 for the currently known packet types).
    pub fn serialise(&self, buf: &mut [u8]) -> Result<usize> {
        self.serialise_with_ack(buf, RequestAck::No)
    }

    /// Serialise the command into the provided buffer, optionally
    /// requesting an ACK from the device. The buffer size should be at
    /// least 10 bytes, otherwise an `Error::BufferTooShort` will be
    /// returned. On success returns the number of bytes written (this
    /// should always be 10 for the currently known packet types).
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

    /// Get the u8 value associated with the current Command.
    /// Unfortunately this has to be a big match statement rather than
    /// a simple cast from custom discriminants because several of the
    /// commands have parameters, and support for this situation is
    /// still [in development](https://github.com/rust-lang/rust/issues/60553).
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
            SetFolder { .. } => 0x0f,
            SetVolumeAdjust { .. } => 0x10,
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

    /// Get the u16 parameter value for the current command. For
    /// commands which require a parameter this will be that value (or
    /// two u8 values concatenated into a u16), otherwise returns 0.
    pub fn param(&self) -> u16 {
        use Command::*;
        match self {
            Track(t) => *t,
            SetVolume(v) => *v,
            SetEq(e) => *e as u16,
            SetPlaybackMode(m) => *m as u16,
            SetPlaybackSource(s) => *s as u16,
            SetFolder { folder, file } => u16::from_be_bytes([*folder, *file]),
            SetVolumeAdjust { enable, gain } => {
                u16::from_be_bytes([*enable as u8, *gain])
            }
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
