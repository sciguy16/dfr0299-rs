// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use color_eyre::Result;
use dfr0299::{
    Control::{self, *},
    Disk, ParseResult, Parser, PlaybackSource, RequestAck,
    Response::{self, *},
};
use mio_serial::SerialPort;
use std::io::Write;
use std::sync::mpsc;
use std::time::Duration;

const USAGE: &str = "Usage: ./with-mio-serial PORT\n\
    e.g. ./with-mio-serial /dev/ttyUSB0";

fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let port = std::env::args().nth(1).expect(USAGE);

    let (tx, mut rx) = mpsc::channel::<Response>();

    let mut port = mio_serial::new(port, 9600).open()?;
    let read_half = port.try_clone()?;
    // start read thread
    std::thread::spawn(move || read_loop(read_half, tx).unwrap());

    // for msg in [
    //     Reset,
    //     SetPlaybackSource(PlaybackSource::Tf),
    //     Track(1),
    //     SetVolume(20),
    //     Playback,
    //     //Normal,
    // ] {
    //     std::thread::sleep(std::time::Duration::from_millis(2000));
    //     send(&mut port, msg)?;
    // }

    send(&mut port, Reset)?;
    wait_for_ack(&mut rx)?;

    loop {
        let resp = rx.recv()?;
        if resp == DiskOnline(Disk::Tf) {
            println!("SD online");
            break;
        }
    }

    send(&mut port, Track(1))?;

    // println!("buf from datasheet");
    // port.send_buf(&[
    //     0x7e, // START
    //     0xff, // VERSION
    //     0x06, // LEN
    //     0x03, // command
    //     0x00, // request ack
    //     0x00, // param high
    //     0x01, // param low
    //     0xff, // checksum high
    //     0xe6, // checksum low
    //     0xef, // STOP
    // ])
    // .unwrap();
    loop {
        let resp = rx.recv()?;
        match resp {
            DiskInserted(Disk::Tf) | DiskOnline(Disk::Tf) => {
                send(&mut port, Track(1))?;
            }
            _ => {}
        }
    }
}

fn send(port: &mut Box<dyn SerialPort>, msg: Control) -> Result<()> {
    let mut buf = [0; 10];
    println!("Send message: {msg:?}");
    msg.serialise_with_ack(&mut buf, RequestAck::Yes)?;
    port.write_all(&buf)?;
    Ok(())
}

fn wait_for_ack(rx: &mut mpsc::Receiver<Response>) -> Result<()> {
    loop {
        let resp = rx.recv()?;
        if resp == Response::Ack {
            return Ok(());
        }
        println!("Non-ack response: {resp:?}");
    }
}

fn read_loop(
    mut port: Box<dyn SerialPort>,
    tx: mpsc::Sender<Response>,
) -> Result<()> {
    println!("Read thread started");
    let mut parser = Parser::new();
    let mut buf = [0u8; 1];
    loop {
        match port.read_exact(&mut buf) {
            Ok(()) => {
                // println!("Received: {:02x}", buf[0]);
                match parser.process_byte(buf[0]) {
                    Ok(ParseResult::Incomplete) => {}
                    Ok(ParseResult::Complete(msg)) => {
                        println!("Received: {msg:?}");
                        tx.send(msg)?;
                    }
                    Err(e) => println!("Error: {e}"),
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                drop(tx);
                break Ok(());
            }
            Err(e) => println!("Error {e}"),
        }
    }
}
