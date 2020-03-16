mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use std::env;
use std::fs;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::thread;

fn main() {
    let bios_rom_file_name = env::args().nth(1).expect("No BIOS ROM file name specified");

    let bios_rom = {
        let mut ret = fs::read(bios_rom_file_name).expect("Couldn't read BIOS ROM file");
        // Zero-pad ROM, since all ROM reads are interpreted as 128-bit reads in sim
        while (ret.len() % 16) != 0 {
            ret.push(0);
        }
        ret
    };

    enum HostCommand {
        Byte(u8),
        Complete,
    }

    let (host_command_tx, host_command_rx) = channel();
    let (host_response_tx, host_response_rx) = channel();

    let thread = thread::spawn(move|| {
        let mut leds = 0b000;

        let mut ddr3_mem = vec![0; 0x20000];

        let mut is_sending_byte = false;

        let mut top = Top::new();

        for i in 0..100000000 {
            if i == 0 {
                top.reset();

                top.bios_rom_bus_ready = true;
                top.ddr3_interface_bus_ready = true;
            } else {
                top.posedge_clk();

                if top.bios_rom_bus_enable {
                    let byte_addr = top.bios_rom_bus_addr << 4;
                    if top.bios_rom_bus_write {
                        panic!("Attempted write to BIOS ROM (byte addr: 0x{:08x})", byte_addr);
                    }
                    let byte_addr = byte_addr & 0x1fff;
                    top.bios_rom_bus_read_data = (0..16).fold(0, |acc, x| {
                        acc | ((bios_rom[(byte_addr + x) as usize] as u128) << (x * 8))
                    });
                    //println!("*** BIOS ROM read: byte addr: 0x{:08x} -> 0x{:032x}", byte_addr, top.bios_rom_bus_read_data);
                }
                top.bios_rom_bus_read_data_valid = top.bios_rom_bus_enable && !top.bios_rom_bus_write;

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

                if top.ddr3_interface_bus_enable {
                    let byte_addr = (top.ddr3_interface_bus_addr << 4) & 0x1ffff;
                    top.ddr3_interface_bus_read_data = (0..16).fold(0, |acc, x| {
                        acc | ((ddr3_mem[(byte_addr + x) as usize] as u128) << (x * 8))
                    });
                    if top.ddr3_interface_bus_write {
                        //println!("*** DDR3 mem write: byte addr: 0x{:08x} <- 0x{:032x} / 0b{:016b}", byte_addr, top.ddr3_interface_bus_write_data, top.ddr3_interface_bus_write_byte_enable);
                        for i in 0..16 {
                            if ((top.ddr3_interface_bus_write_byte_enable >> i) & 1) != 0 {
                                ddr3_mem[(byte_addr + i) as usize] = (top.ddr3_interface_bus_write_data >> (i * 8)) as _;
                            }
                        }
                    } else {
                        //println!("*** DDR3 mem read: byte addr: 0x{:08x} -> 0x{:032x}", byte_addr, top.ddr3_interface_bus_read_data);
                    }
                }
                top.ddr3_interface_bus_read_data_valid = top.ddr3_interface_bus_enable && !top.ddr3_interface_bus_write;
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
