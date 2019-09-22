use serialport::prelude::*;

use std::env;
use std::io::{self, Write};

#[derive(Debug)]
enum Error {
    SerialPort(serialport::Error),
    Io(io::Error),
}

impl From<serialport::Error> for Error {
    fn from(error: serialport::Error) -> Error {
        Error::SerialPort(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

fn main() -> Result<(), Error> {
    let port_name = env::args().nth(1).expect("No COM port name specified");
    let baud_rate: u32 = 115200;

    let mut settings: SerialPortSettings = Default::default();
    settings.baud_rate = baud_rate.into();

    let mut port = serialport::open_with_settings(&port_name, &settings)?;
    let mut buf = vec![0; 1000];

    let start_state = 0xace1u16;
    let mut lfsr = start_state;

    let mut total_bytes = 0u64;
    let mut total_errors = 0u64;

    loop {
        match port.read(buf.as_mut_slice()) {
            Ok(t) => {
                for data in &buf[..t] {
                    let expected = lfsr as u8;

                    let bit = (lfsr >> 0) ^ (lfsr >> 2) ^ (lfsr >> 3) ^ (lfsr >> 5);
                    lfsr = (lfsr >> 1) | (bit << 15);

                    total_bytes += 1;

                    if *data != expected {
                        total_errors += 1;
                    }
                }

                port.write_all(&buf[..t])?;

                print!("\r{} bytes, {} errors ({:.*}%)            ", total_bytes, total_errors, 2, (total_errors as f64) / (total_bytes as f64) * 100.0);
                io::stdout().flush()?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => Err(e)?
        }
    }
}
