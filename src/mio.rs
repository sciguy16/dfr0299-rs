use crate::{Control, Result};
use mio_serial::SerialPort;
use std::io::Write;

pub struct Serial {
    port: Box<dyn SerialPort>,
    buf: [u8; 10],
}

impl Serial {
    pub fn new(port: &str) -> Result<Self> {
        Ok(Self {
            port: mio_serial::new(port, 9600).open()?,
            buf: Default::default(),
        })
    }

    pub fn send_control(&mut self, msg: Control) -> Result<()> {
        msg.serialise(&mut self.buf)?;
        self.port.write_all(&self.buf)?;

        Ok(())
    }

    pub fn send_buf(&mut self, buf: &[u8]) -> Result<()> {
        self.port.write_all(buf)?;
        Ok(())
    }
}
