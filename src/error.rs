#[cfg_attr(not(feature = "mio-serial"), derive(Copy, Clone))]
#[derive(Debug)]
pub enum Error {
    BufferTooShort,

    #[cfg(feature = "mio-serial")]
    MioSerialError(mio_serial::Error),
    #[cfg(feature = "mio-serial")]
    IoError(std::io::Error),
}

#[cfg(feature = "mio-serial")]
impl From<mio_serial::Error> for Error {
    fn from(e: mio_serial::Error) -> Self {
        Error::MioSerialError(e)
    }
}

#[cfg(feature = "mio-serial")]
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}
