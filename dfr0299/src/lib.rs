// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(missing_docs)]

//! Serial protocol for the DFR0299 MP3 player module
//!
//! This crate provides zero-allocation, `no_std`-compatible
//! serialisation and deserialisation for the commands supported by the
//! DFR02999 MP3 player module.
//!
//! Communication with the module is via UART at 9600-8-N-1.
//!
//! ## Features
//! * `std`: implement `std::error::Error` for `Error`
//! * `use_defmt`: All types derive implementations of `defmt::Format`
//! to allow them to be formatted by `defmt` when used on embedded
//! devices
//!
//! ## Usage - serialisation
//! This example just demonstrates serialising commands into a buffer.
//! For implementations using `mio_serial` on Linux and a hardware UART
//! on an RP2040 microcontroller see
//! [the github repo](https://github.com/sciguy16/dfr0299-rs/tree/main/examples)
//!
//! ```no_run
//! # fn a() -> Result<(), dfr0299::Error> {
//! use dfr0299::{Command};
//! let mut buf = [0u8; 10];
//! Command::Reset.serialise(&mut buf)?;
//! // do something with the buffer, e.g. write to a uart peripheral
//! Command::Track(1).serialise(&mut buf)?;
//! // send the buffer, e.g.
//! // uart.write_full_blocking(&buf)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Usage - parsing
//! For a concrete application, see the `mio_serial` example
//! [on github](https://github.com/sciguy16/dfr0299-rs/tree/main/examples/with-mio-serial)
//!
//! ```no_run
//! use dfr0299::{Parser, ParseResult};
//! fn process<R: std::io::Read>(mut uart: R) -> Result<(), Box<dyn std::error::Error>> {
//!    let mut parser = Parser::new();
//!    let mut buf = [0u8; 1];
//!    loop {
//!        uart.read_exact(&mut buf)?;
//!        match parser.process_byte(buf[0]) {
//!            Ok(ParseResult::Incomplete) => {}
//!            Ok(ParseResult::Complete(msg)) => {
//!                println!("Message received: {msg:?}");
//!            }
//!            Err(e) => {
//!                println!("Parse error: {e}");
//!            }
//!        }
//!    }
//! }
//! ```
//!
//! ## Packet structure
//!
//! * START = 0x7e
//! * VERSION = 0xff
//! * LEN = number of bytes to follow, inc. LEN & VER, not inc. checksum
//! * CMD = the command
//! * FEEDBACK = whether to request ack/feedback
//! * PARAM_H = parameter high byte
//! * PARAM_L = parameter low byte
//! * CHECKSUM_H = checksum high byte
//! * CHECKSUM_L = checksum low byte
//! * STOP = 0xef
//!
//! The checksum is the twos complement of the sum over the packet bytes
//! (excluding START). Note that the example packets in the datasheet
//! have incorrect checksums.

mod control;
mod error;
mod parser;
mod response;

pub use control::*;
pub use error::Error;
pub use parser::*;
pub use response::*;

/// Newtype wrapping this crate's Error
pub type Result<T> = core::result::Result<T, Error>;

/// Packet start byte
pub const START: u8 = 0x7e;
/// Packet end byte
pub const STOP: u8 = 0xef;
/// Packet version field. This just seems to be hardcoded to `0xff` and
/// not actually used for anything
pub const VERSION: u8 = 0xff;
