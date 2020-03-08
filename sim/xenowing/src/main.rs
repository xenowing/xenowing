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

    let mut ddr3_mem = vec![0; 0x20000];

    let mut xenowing = Xenowing::new();

    for i in 0..100000000 {
        if i == 0 {
            xenowing.reset();

            xenowing.bios_rom_bus_ready = true;
            xenowing.led_interface_bus_ready = true;
            xenowing.uart_interface_bus_ready = true;
            xenowing.ddr3_interface_bus_ready = true;
        } else {
            xenowing.posedge_clk();

            if xenowing.bios_rom_bus_enable {
                let byte_addr = xenowing.bios_rom_bus_addr << 4;
                if xenowing.bios_rom_bus_write {
                    panic!("Attempted write to BIOS ROM (byte addr: 0x{:08x})", byte_addr);
                }
                let byte_addr = byte_addr & 0x1fff;
                xenowing.bios_rom_bus_read_data = (0..16).fold(0, |acc, x| {
                    acc | ((bios_rom[(byte_addr + x) as usize] as u128) << (x * 8))
                });
                //println!("*** BIOS ROM read: byte addr: 0x{:08x} -> 0x{:032x}", byte_addr, xenowing.bios_rom_bus_read_data);
            }
            xenowing.bios_rom_bus_read_data_valid = xenowing.bios_rom_bus_enable && !xenowing.bios_rom_bus_write;

            if xenowing.led_interface_bus_enable {
                // TODO: Masking? It might help not to actually in order to catch errors
                let byte_addr = xenowing.led_interface_bus_addr << 4;
                if xenowing.led_interface_bus_write {
                    match byte_addr {
                        0x00000000 => {
                            let new_leds = xenowing.led_interface_bus_write_data;
                            if new_leds != leds {
                                println!("LEDs updated: 0b{:03b} -> 0b{:03b}", leds, new_leds);
                                leds = new_leds;
                            }
                        }
                        _ => panic!("Attempted write to LED interface (byte addr: 0x{:08x})", byte_addr)
                    };
                    //println!("*** LED interface write: byte addr: 0x{:08x} <- 0x{:032x} / 0b{:016b}", byte_addr, xenowing.led_interface_bus_write_data, xenowing.led_interface_bus_write_byte_enable);
                } else {
                    xenowing.led_interface_bus_read_data = match byte_addr {
                        0x00000000 => leds,
                        _ => panic!("Attempted read from LED interface (byte addr: 0x{:08x})", byte_addr)
                    };
                    //println!("*** LED interface read: byte addr: 0x{:08x} -> 0x{:032x}", byte_addr, xenowing.led_interface_bus_read_data);
                }
            }

            if xenowing.uart_interface_bus_enable {
                // TODO: Masking? It might help not to actually in order to catch errors
                let byte_addr = xenowing.uart_interface_bus_addr << 4;
                if xenowing.uart_interface_bus_write {
                    match byte_addr {
                        0x00000010 => {
                            // UART transmitter write
                            print!("{}", xenowing.uart_interface_bus_write_data as u8 as char);
                        }
                        _ => panic!("Attempted write to UART interface (byte addr: 0x{:08x})", byte_addr)
                    };
                    //println!("*** UART interface write: byte addr: 0x{:08x} <- 0x{:032x} / 0b{:016b}", byte_addr, xenowing.uart_interface_bus_write_data, xenowing.uart_interface_bus_write_byte_enable);
                } else {
                    xenowing.uart_interface_bus_read_data = match byte_addr {
                        0x00000000 => {
                            // UART transmitter status
                            1 // Always ready
                        }
                        _ => panic!("Attempted read from UART interface (byte addr: 0x{:08x})", byte_addr)
                    };
                    //println!("*** UART interface read: byte addr: 0x{:08x} -> 0x{:032x}", byte_addr, xenowing.uart_interface_bus_read_data);
                }
            }
            xenowing.uart_interface_bus_read_data_valid = xenowing.uart_interface_bus_enable && !xenowing.uart_interface_bus_write;

            if xenowing.ddr3_interface_bus_enable {
                let byte_addr = (xenowing.ddr3_interface_bus_addr << 4) & 0x1ffff;
                xenowing.ddr3_interface_bus_read_data = (0..16).fold(0, |acc, x| {
                    acc | ((ddr3_mem[(byte_addr + x) as usize] as u128) << (x * 8))
                });
                if xenowing.ddr3_interface_bus_write {
                    //println!("*** DDR3 mem write: byte addr: 0x{:08x} <- 0x{:032x} / 0b{:016b}", byte_addr, xenowing.ddr3_interface_bus_write_data, xenowing.ddr3_interface_bus_write_byte_enable);
                    for i in 0..16 {
                        if ((xenowing.ddr3_interface_bus_write_byte_enable >> i) & 1) != 0 {
                            ddr3_mem[(byte_addr + i) as usize] = (xenowing.ddr3_interface_bus_write_data >> (i * 8)) as _;
                        }
                    }
                } else {
                    //println!("*** DDR3 mem read: byte addr: 0x{:08x} -> 0x{:032x}", byte_addr, xenowing.ddr3_interface_bus_read_data);
                }
            }
            xenowing.ddr3_interface_bus_read_data_valid = xenowing.ddr3_interface_bus_enable && !xenowing.ddr3_interface_bus_write;

            xenowing.prop();
        }
    }
}
