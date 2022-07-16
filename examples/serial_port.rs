use dfr0299::{mio::Serial, Control::*, PlaybackSource};

fn main() {
    let mut port = Serial::new("/dev/ttyUSB0").unwrap();
    for msg in [
        SetPlaybackSource(PlaybackSource::Tf),
        Track(1),
        SetVolume(20),
        Playback,
        //Normal,
    ] {
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("Send message: {msg:?}");
        port.send_control(msg).unwrap();
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
}
