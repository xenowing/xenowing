mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use std::env;
use std::fs;

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

    let mut leds = 0b000;

    enum UartCommand {
        Putc,
    }
    let mut uart_command = None;
    const XW_UART_COMMAND_PUTC: u8 = 0x00;

    let mut ddr3_mem = vec![0; 0x20000];

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

            if top.uart_data_valid {
                let data = top.uart_data as u8;
                if let Some(ref command) = uart_command {
                    match command {
                        UartCommand::Putc => print!("{}", data as char),
                    }
                } else {
                    uart_command = Some(match data {
                        XW_UART_COMMAND_PUTC => UartCommand::Putc,
                        _ => panic!("Invalid UART command received: 0x{:02x}", data)
                    });
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

            top.prop();
        }
    }
}
