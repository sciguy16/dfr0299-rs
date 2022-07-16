use dfr0299::{Control::*, PlaybackSource};
use std::io::Write;

const USAGE: &str = "Usage: ./with-mio-serial PORT\n\
    e.g. ./with-mio-serial /dev/ttyUSB0";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let port = std::env::args().nth(1).expect(USAGE);

    let mut port = mio_serial::new(port, 9600).open()?;
    let mut buf = [0; 10];
    for msg in [
        SetPlaybackSource(PlaybackSource::Tf),
        Track(1),
        SetVolume(20),
        Playback,
        //Normal,
    ] {
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("Send message: {msg:?}");
        msg.serialise(&mut buf)?;
        port.write_all(&buf)?;
    }

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

    Ok(())
}
