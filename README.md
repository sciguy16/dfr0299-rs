# dfr0299

[![Crates.io](https://img.shields.io/crates/v/dcc-rs)](https://crates.io/crates/dcc-rs)
[![docs.rs](https://img.shields.io/docsrs/dcc-rs)](https://docs.rs/dcc-rs)

Rust implementation of the serial protocol for the
[DFR0299 MP3 player module](https://wiki.dfrobot.com/DFPlayer_Mini_SKU_DFR0299)

This crate provides zero-allocation, `no_std`-compatible serialisation
and deserialisation for the commands supported by the DFR02999 MP3
player module.

Communication with the module is via UART at 9600-8-N-1.

## Examples
Two example are provided: one using `mio_serial` and one for the RP2040

```bash
cargo run --package serial_port
cargo run --release --package pico --target thumbv6m-none-eabi
```

Serialise commands into a buffer:
```rust
use dfr0299::{Command};
let mut buf = [0u8; 10];
Command::Reset.serialise(&mut buf)?;
// do something with the buffer, e.g. write to a uart peripheral
Command::Track(1).serialise(&mut buf)?;
// send the buffer, e.g.
// uart.write_full_blocking(&buf)?;
```

Parse response messages:
```rust
use dfr0299::{Parser, ParseResult};
fn process<R: std::io::Read>(mut uart: R) ->
    Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    let mut buf = [0u8; 1];
    loop {
        uart.read_exact(&mut buf)?;
        match parser.process_byte(buf[0]) {
            Ok(ParseResult::Incomplete) => {}
            Ok(ParseResult::Complete(msg)) => {
                println!("Message received: {msg:?}");
            }
            Err(e) => {
                println!("Parse error: {e}");
            }
        }
    }
}
```

Note that the example serialised messages in the datasheet are have
incorrect checksums. The checksum algorithm is not described in the
datasheet but is present in the
[official Arduino library code](https://github.com/DFRobot/DFRobotDFPlayerMini/blob/master/DFRobotDFPlayerMini.cpp)

# License
This crate is distributed under the terms of the Mozilla Public License
Version 2.0.
