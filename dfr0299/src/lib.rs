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
