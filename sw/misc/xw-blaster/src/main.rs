#![feature(stdarch)]

mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use minifb::{Scale, ScaleMode, Window, WindowOptions};
use serialport::prelude::*;
use strugl::{WIDTH, HEIGHT};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use std::env;
use std::fs;
use std::io::{self, Write};
use std::str;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::thread;

const PIXELS: usize = WIDTH * HEIGHT;

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

    fn read_u32(&mut self) -> Result<u32, Error> {
        let mut ret = 0x00;
        for i in 0..4 {
            ret |= (self.read_byte()? as u32) << i * 8;
        }

        Ok(ret)
    }

    fn write_u32(&mut self, value: u32) -> Result<(), Error> {
        self.write_byte((value >> 0) as _)?;
        self.write_byte((value >> 8) as _)?;
        self.write_byte((value >> 16) as _)?;
        self.write_byte((value >> 24) as _)?;

        Ok(())
    }
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

// TODO: De-dupe this code (via a trait or something, perhaps?)
struct SimInnerDevice {
    host_command_rx: Receiver<u8>,
    host_response_tx: Sender<u8>,
}

impl SimInnerDevice {
    fn new() -> SimInnerDevice {
        let (host_command_tx, host_command_rx) = channel();
        let (host_response_tx, host_response_rx) = channel();

        // TODO: This is leaky, but I guess it doesn't matter :)
        thread::spawn(move|| {
            let mut leds = 0b000;

            let mut is_sending_byte = false;

            let mut top = TopInner::new();

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

                    if top.uart_tx_enable {
                        host_command_tx.send(top.uart_tx_data as _).unwrap();
                    }

                    // TODO: This isn't necessarily the best way to use this interface, but it should work :)
                    if is_sending_byte && top.uart_rx_ready {
                        is_sending_byte = false;
                        top.uart_rx_data_valid = false;
                    }
                    if !is_sending_byte {
                        if let Ok(value) = host_response_rx.try_recv() {
                            is_sending_byte = true;
                            top.uart_rx_data = value as u32;
                            top.uart_rx_data_valid = true;
                        }
                    }
                }

                top.prop();
            }
        });

        SimInnerDevice {
            host_command_rx,
            host_response_tx,
        }
    }
}

impl Device for SimInnerDevice {
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
    let device_type = env::args().nth(1).expect("Missing device type arg");

    let mut device: Box<dyn Device> = match device_type.as_str() {
        "serial" => {
            let port_name = env::args().nth(2).expect("Missing port name arg");
            println!("Creating serial device on port {}", port_name);
            Box::new(SerialDevice::new(port_name)?)
        }
        "sim" => {
            println!("Creating sim device");
            Box::new(SimDevice::new())
        }
        "sim-inner" => {
            println!("Creating sim inner device");
            Box::new(SimInnerDevice::new())
        }
        _ => panic!("Invalid device type argument")
    };
    println!();

    let mut back_buffer = vec![0xffff00ff; PIXELS];

    let mut window = Window::new("trim", WIDTH, HEIGHT, WindowOptions {
        scale: Scale::X4,
        scale_mode: ScaleMode::AspectRatioStretch,
        ..WindowOptions::default()
    }).unwrap();

    let tex = image::open("tex.png").unwrap();

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
                device.write_u32(len as _)?;
                for byte in file {
                    device.write_byte(byte)?;
                }

                stdout.reset()?;
            }
            0x02 => {
                let mut stdout = StandardStream::stdout(ColorChoice::Always);
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_intense(true))?;

                writeln!(&mut stdout, "commands requested, rendering entire frame")?;

                device.write_byte(0x06)?;

                stdout.reset()?;
            }
            0x03 => {
                let mut stdout = StandardStream::stdout(ColorChoice::Always);
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_intense(true))?;

                let mut elapsed_cycles = 0;
                for i in 0..8 {
                    elapsed_cycles |= (device.read_byte()? as u64) << i * 8;
                }
                writeln!(&mut stdout, "  elapsed cycles: {}", elapsed_cycles)?;

                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        back_buffer[(HEIGHT - 1 - y) * WIDTH + x] = device.read_u32()?;
                    }
                }

                window.update_with_buffer(&back_buffer, WIDTH, HEIGHT).unwrap();

                writeln!(&mut stdout, "frame complete")?;

                stdout.reset()?;
            }
            command_byte => {
                return Err(format!("Invalid UART command byte received: 0x{:02x}", command_byte).into());
            }
        }
    }
}
