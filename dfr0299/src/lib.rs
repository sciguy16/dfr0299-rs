// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

// 9600 baud
// data bits 1
// checkout none
// flow control none

mod control;
mod error;
mod parser;
mod response;

pub use control::*;
pub use error::Error;
pub use parser::*;
pub use response::*;
pub type Result<T> = core::result::Result<T, Error>;

pub const START: u8 = 0x7e;
pub const STOP: u8 = 0xef;
pub const VERSION: u8 = 0xff;
