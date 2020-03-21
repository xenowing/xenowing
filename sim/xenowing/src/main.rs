mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::thread;

fn main() {
    enum HostCommand {
        Byte(u8),
        Complete,
    }

    let (host_command_tx, host_command_rx) = channel();
    let (host_response_tx, host_response_rx) = channel();

    let thread = thread::spawn(move|| {
        let mut leds = 0b000;

        let mut is_sending_byte = false;

        let mut top = Top::new();

        for i in 0..100000000 {
            if i == 0 {
                top.reset();
            } else {
                top.posedge_clk();

                let new_leds = top.leds;
                if new_leds != leds {
                    println!("LEDs updated: 0b{:03b} -> 0b{:03b}", leds, new_leds);
                    leds = new_leds;
                }

                if top.uart_tx_data_valid {
                    host_command_tx.send(HostCommand::Byte(top.uart_tx_data as _)).unwrap();
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

        host_command_tx.send(HostCommand::Complete).unwrap();
    });

    loop {
        enum ReadByteError {
            Complete,
            RecvError(RecvError),
        }

        fn read_byte(host_command_rx: &Receiver<HostCommand>) -> Result<u8, ReadByteError> {
            match host_command_rx.recv() {
                Ok(command) => match command {
                    HostCommand::Byte(value) => Ok(value),
                    HostCommand::Complete => Err(ReadByteError::Complete),
                }
                Err(e) => Err(ReadByteError::RecvError(e)),
            }
        }

        fn write_byte(host_response_tx: &Sender<u8>, value: u8) {
            host_response_tx.send(value).unwrap();
        }

        fn process_command(host_command_rx: &Receiver<HostCommand>, host_response_tx: &Sender<u8>) -> Result<(), ReadByteError> {
            match read_byte(host_command_rx)? {
                0x00 => {
                    // XW_UART_COMMAND_PUTC
                    print!("{}", read_byte(host_command_rx)? as char);
                }
                0x01 => {
                    // File read test
                    let file = "WHADDUP DOES THIS ACTUALLY WORK YOOOO???!??!?!!?\r\n";
                    let len = file.len();
                    write_byte(&host_response_tx, (len >> 0) as _);
                    write_byte(&host_response_tx, (len >> 8) as _);
                    write_byte(&host_response_tx, (len >> 16) as _);
                    write_byte(&host_response_tx, (len >> 24) as _);
                    for byte in file.bytes() {
                        write_byte(&host_response_tx, byte);
                    }
                }
                command_byte => panic!("Invalid UART command byte received: 0x{:02x}", command_byte)
            }

            Ok(())
        }

        if let Err(e) = process_command(&host_command_rx, &host_response_tx) {
            match e {
                ReadByteError::Complete => break,
                ReadByteError::RecvError(e) => panic!("xenowing thread errored: {}", e),
            }
        }
    }

    thread.join().unwrap();
}
