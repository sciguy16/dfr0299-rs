// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{Error, Response, Result, START, STOP, VERSION};

const LEN: u8 = 6;

#[derive(Debug)]
enum ParserState {
    Idle,
    Start,
    Version,
    Len,
    Cmd,
    Feedback,
    ParamH,
    ParamL,
    ChecksumH,
    ChecksumL,
}

impl Default for ParserState {
    fn default() -> Self {
        Self::Idle
    }
}

/// After processing a byte the parser will return either `Incomplete`
/// to indicate that it requires more data or `Complete` to indicate
/// that a full message has been successfully processed
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseResult {
    /// Waiting for more bytes
    Incomplete,
    /// Complete message has been processed
    Complete(Response),
}

/// Parser for the DFR0299 response messages. After initialising, calls
/// to `Parser::process_byte` will advance the internal state machine
/// and return any complete messages.
///
/// ## Example
/// ```no_run
/// use dfr0299::{Parser, ParseResult};
/// fn process<R: std::io::Read>(mut uart: R) -> Result<(), Box<dyn std::error::Error>> {
///    let mut parser = Parser::new();
///    let mut buf = [0u8; 1];
///    loop {
///        uart.read_exact(&mut buf)?;
///        match parser.process_byte(buf[0]) {
///            Ok(ParseResult::Incomplete) => {}
///            Ok(ParseResult::Complete(msg)) => {
///                println!("Message received: {msg:?}");
///            }
///            Err(e) => {
///                println!("Parse error: {e}");
///            }
///        }
///    }
/// }
/// ```
#[derive(Debug, Default)]
pub struct Parser {
    state: ParserState,
    cmd: u8,
    feedback: u8,
    param_h: u8,
    param_l: u8,
    checksum_h: u8,
    checksum_l: u8,
}

impl Parser {
    /// Initialise a new `Parser`
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a single byte and advance the internal state machine
    /// accordingly. Returns a `ParseResult` indicating the parsing
    /// status or an `Error::BadChecksum` if a complete message has been
    /// received but the checksum is incorrect.
    pub fn process_byte(&mut self, byte: u8) -> Result<ParseResult> {
        use ParserState::*;
        self.state = match self.state {
            Idle => {
                if byte == START {
                    Start
                } else {
                    Idle
                }
            }
            Start => {
                if byte == VERSION {
                    Version
                } else {
                    Idle
                }
            }
            Version => {
                if byte == LEN {
                    Len
                } else {
                    Idle
                }
            }
            Len => {
                self.cmd = byte;
                Cmd
            }
            Cmd => {
                self.feedback = byte;
                Feedback
            }
            Feedback => {
                self.param_h = byte;
                ParamH
            }
            ParamH => {
                self.param_l = byte;
                ParamL
            }
            ParamL => {
                self.checksum_h = byte;
                ChecksumH
            }
            ChecksumH => {
                self.checksum_l = byte;
                ChecksumL
            }
            ChecksumL => {
                if byte == STOP {
                    // do the thing
                    self.state = Idle;

                    let checksum = self.calculate_checksum();
                    if checksum.to_be_bytes()
                        != [self.checksum_h, self.checksum_l]
                    {
                        return Err(Error::BadChecksum);
                    }

                    // checksum valid -> parse message
                    let response =
                        Response::parse(self.cmd, self.param_h, self.param_l)?;
                    return Ok(ParseResult::Complete(response));
                }
                Idle
            }
        };

        Ok(ParseResult::Incomplete)
    }

    fn calculate_checksum(&self) -> i16 {
        -[
            0xff,
            LEN,
            self.cmd,
            self.feedback,
            self.param_h,
            self.param_l,
        ]
        .into_iter()
        .map(i16::from)
        .sum::<i16>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_checksum() {
        let parser = Parser {
            cmd: 0x03,
            feedback: 0x00,
            param_h: 0x00,
            param_l: 0x01,
            ..Default::default()
        };

        let checksum = parser.calculate_checksum();
        assert_eq!(checksum.to_be_bytes(), [0xfe, 0xf7]);
    }

    #[test]
    fn parse_udisk_remove() {
        let msg = [0x7e, 0xff, 0x06, 0x3b, 0x00, 0x00, 0x01, 0xfe, 0xbf, 0xef];
        let mut parser = Parser::new();
        let expected = Response::DiskRemoved(crate::response::Disk::UDisk);
        let mut ok = false;
        for byte in msg {
            match parser.process_byte(byte).unwrap() {
                ParseResult::Incomplete => {}
                ParseResult::Complete(msg) => {
                    assert_eq!(msg, expected);
                    ok = true;
                }
            }
        }
        assert!(ok);
    }
}
