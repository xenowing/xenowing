use notify::{self, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

use serialport::prelude::*;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use std::env;
use std::io::{self, Read, Write};
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::{self, channel};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Error {
    Notify(notify::Error),
    SerialPort(serialport::Error),
    Io(io::Error),
    Recv(mpsc::RecvError),
    Other(String),
}

impl From<notify::Error> for Error {
    fn from(error: notify::Error) -> Error {
        Error::Notify(error)
    }
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

impl From<mpsc::RecvError> for Error {
    fn from(error: mpsc::RecvError) -> Error {
        Error::Recv(error)
    }
}

fn main() -> Result<(), Error> {
    let rom_file_path_arg = env::args().skip(1).nth(0).ok_or(Error::Other("ROM file path not specified".into()))?;
    let rom_file_path = Path::new(&rom_file_path_arg);

    let port_name = "COM4";
    let baud_rate: u32 = 115200;

    let mut settings: SerialPortSettings = Default::default();
    settings.baud_rate = baud_rate.into();

    let mut port = serialport::open_with_settings(port_name, &settings)?;
    port.write_data_terminal_ready(true)?;

    let mut receive_port = port.try_clone()?;
    thread::spawn(move || {
        let mut read_buf = vec![0; 1000];

        loop {
            match receive_port.read(read_buf.as_mut_slice()) {
                Ok(t) => {
                    print!("{}", String::from_utf8_lossy(&read_buf[..t]));
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => panic!("{}", e)
            }
        }
    });

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
    watcher.watch(rom_file_path, RecursiveMode::NonRecursive)?;

    println!("XENOWING BLASTER ENGAGED");
    println!("ALL SYSTEMS ARE GO");
    println!();

    loop {
        match rx.recv() {
            Ok(event) => {
                if match event {
                    DebouncedEvent::Create(_) => true,
                    DebouncedEvent::NoticeWrite(_) => true,
                    DebouncedEvent::Write(_) => true,
                    _ => false
                } {
                    let mut stdout = StandardStream::stdout(ColorChoice::Always);
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_intense(true))?;

                    writeln!(&mut stdout, "")?;
                    writeln!(&mut stdout, "ROM file changed, attempting reload")?;

                    if let Err(e) = reload_file(rom_file_path, port.as_mut()) {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_intense(true))?;
                        writeln!(&mut stdout, "Error reloading ROM file: {:?}", e)?;
                    } else {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_intense(true))?;
                        writeln!(&mut stdout, "ROM reload successful!")?;
                    }

                    writeln!(&mut stdout, "")?;

                    stdout.reset()?;
                }
            }
            Err(e) => Err(e)?
        }
    }
}

fn reload_file<P: AsRef<Path>>(rom_file_path: P, port: &mut SerialPort) -> Result<(), Error> {
    let input = {
        let mut input = Vec::new();
        File::open(rom_file_path)?.read_to_end(&mut input)?;
        input
    };
    let input_len = input.len();

    println!("ROM file size: 0x{:08x} bytes", input_len);

    if input.is_empty() {
        return Err(Error::Other("ROM file is empty".into()));
    }

    if (input_len % 4) != 0 {
        return Err(Error::Other(format!("ROM file size ({} bytes) is not divisible by 4", input_len)));
    }

    if input_len >= 0x2000 {
        return Err(Error::Other(format!("ROM file size ({} bytes) is too large", input_len)));
    }

    let input_len_bytes = [
        input_len as u8,
        (input_len >> 8) as u8,
        (input_len >> 16) as u8,
        (input_len >> 24) as u8,
    ];
    port.write_all(&input_len_bytes)?;
    port.write_all(&input)?;

    Ok(())
}
