mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use serialport::prelude::*;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use std::env;
use std::fs;
use std::io::{self, Write};
use std::str;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::thread;

#[derive(Debug)]
enum Error {
    Io(io::Error),
    Other(String),
    Recv(mpsc::RecvError),
    Send(mpsc::SendError<u8>),
    SerialPort(serialport::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Error {
        Error::Other(error)
    }
}

impl From<mpsc::RecvError> for Error {
    fn from(error: mpsc::RecvError) -> Error {
        Error::Recv(error)
    }
}

impl From<mpsc::SendError<u8>> for Error {
    fn from(error: mpsc::SendError<u8>) -> Error {
        Error::Send(error)
    }
}

impl From<serialport::Error> for Error {
    fn from(error: serialport::Error) -> Error {
        Error::SerialPort(error)
    }
}

trait Device {
    fn read_byte(&mut self) -> Result<u8, Error>;
    fn write_byte(&mut self, value: u8) -> Result<(), Error>;
}

struct SimDevice {
    host_command_rx: Receiver<u8>,
    host_response_tx: Sender<u8>,
}

impl SimDevice {
    fn new() -> SimDevice {
        let (host_command_tx, host_command_rx) = channel();
        let (host_response_tx, host_response_rx) = channel();

        // TODO: This is leaky, but I guess it doesn't matter :)
        thread::spawn(move|| {
            let mut leds = 0b000;

            let mut is_sending_byte = false;

            let mut top = Top::new();

            let mut is_first_cycle = true;
            loop {
                if is_first_cycle {
                    top.reset();

                    is_first_cycle = false;
                } else {
                    top.posedge_clk();

                    let new_leds = top.leds;
                    if new_leds != leds {
                        println!("LEDs updated: 0b{:08b} -> 0b{:08b}", leds, new_leds);
                        leds = new_leds;
                    }

                    if top.uart_tx_data_valid {
                        host_command_tx.send(top.uart_tx_data as _).unwrap();
                    }

                    // TODO: This isn't necessarily the best way to use this interface, but it should work :)
                    if is_sending_byte && top.uart_rx_ready {
                        is_sending_byte = false;
                        top.uart_rx_enable = false;
                    }
                    if !is_sending_byte {
                        if let Ok(value) = host_response_rx.try_recv() {
                            is_sending_byte = true;
                            top.uart_rx_enable = true;
                            top.uart_rx_data = value as u32;
                        }
                    }
                }

                top.prop();
            }
        });

        SimDevice {
            host_command_rx,
            host_response_tx,
        }
    }
}

impl Device for SimDevice {
    fn read_byte(&mut self) -> Result<u8, Error> {
        Ok(self.host_command_rx.recv()?)
    }

    fn write_byte(&mut self, value: u8) -> Result<(), Error> {
        self.host_response_tx.send(value)?;

        Ok(())
    }
}

struct SerialDevice {
    port: Box<dyn SerialPort>,
}

impl SerialDevice {
    fn new(port_name: String) -> Result<SerialDevice, Error> {
        let baud_rate: u32 = 460800;

        let mut settings: SerialPortSettings = Default::default();
        settings.baud_rate = baud_rate.into();

        let port = serialport::open_with_settings(&port_name, &settings)?;
        let actual_baud_rate = port.baud_rate()?;
        if actual_baud_rate != baud_rate {
            return Err(format!("Unable to achieve specified baud rate: got {}, expected {}", actual_baud_rate, baud_rate).into());
        }

        Ok(SerialDevice {
            port,
        })
    }
}

impl Device for SerialDevice {
    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut buf = [0];
        loop {
            match self.port.read(&mut buf) {
                Ok(t) => {
                    if t > 0 {
                        return Ok(buf[0]);
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    fn write_byte(&mut self, value: u8) -> Result<(), Error> {
        self.port.write_all(&[value])?;

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let mut device: Box<dyn Device> = if let Some(port_name) = env::args().nth(1) {
        println!("Creating serial device on port {}", port_name);
        Box::new(SerialDevice::new(port_name)?)
    } else {
        println!("Creating sim device");
        Box::new(SimDevice::new())
    };
    println!();

    println!("XENOWING BLASTER ENGAGED");
    println!("ALL SYSTEMS ARE GO");
    println!();

    loop {
        match device.read_byte()? {
            0x00 => {
                // XW_UART_COMMAND_PUTC
                print!("{}", device.read_byte()? as char);
            }
            0x01 => {
                let mut stdout = StandardStream::stdout(ColorChoice::Always);
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_intense(true))?;

                // File read test
                let mut filename = Vec::new();
                loop {
                    let c = device.read_byte()?;
                    if c == 0 {
                        break;
                    }

                    filename.push(c);
                }
                let filename = str::from_utf8(&filename).unwrap();
                writeln!(&mut stdout, "file requested: {}", filename)?;
                let file = fs::read(filename)?;
                let len = file.len();
                device.write_byte((len >> 0) as _)?;
                device.write_byte((len >> 8) as _)?;
                device.write_byte((len >> 16) as _)?;
                device.write_byte((len >> 24) as _)?;
                for byte in file {
                    device.write_byte(byte)?;
                }

                stdout.reset()?;
            }
            command_byte => {
                return Err(format!("Invalid UART command byte received: 0x{:02x}", command_byte).into());
            }
        }
    }
}
