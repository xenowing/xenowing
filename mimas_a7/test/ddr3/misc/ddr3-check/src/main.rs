use serialport::prelude::*;

use std::env;
use std::io;

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
    let baud_rate: u32 = 460800;

    let mut settings: SerialPortSettings = Default::default();
    settings.baud_rate = baud_rate.into();

    let mut port = serialport::open_with_settings(&port_name, &settings)?;
    let actual_baud_rate = port.baud_rate()?;
    if actual_baud_rate != baud_rate {
        panic!("Unable to achieve specified baud rate: got {}, expected {}", actual_baud_rate, baud_rate);
    }

    loop {
        let write_cycles = read_u64(&mut port)?;
        println!("Write cycles: {} (0x{:016x})", write_cycles, write_cycles);
        let read_cycles = read_u64(&mut port)?;
        println!("Read cycles: {} (0x{:016x})", read_cycles, read_cycles);
        let total_cycles = write_cycles + read_cycles;
        println!("Total cycles: {} (0x{:016x})", total_cycles, total_cycles);
        println!("");
    }
}

fn read_u64<R: io::Read>(port: &mut R) -> Result<u64, Error> {
    loop {
        let mut buf = [0; 8];
        match port.read_exact(&mut buf) {
            Ok(()) => {
                return Ok(u64::from_le_bytes(buf));
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => Err(e)?
        }
    }
}
