mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use goblin::Object;

use std::env;
use std::fs::{self, File};
use std::io::Write;

fn main() {
    let program_rom_file_name = env::args().nth(1).expect("No program ROM file name specified");
    let program_elf_file_name = env::args().nth(2).expect("No program elf file name specified");
    let signature_file_name = env::args().nth(3).expect("No signature file name specified");

    let program_rom = {
        let mut ret = fs::read(program_rom_file_name).expect("Couldn't read program ROM file");
        // Zero-pad ROM, since all ROM reads are interpreted as 32-bit reads in sim
        while (ret.len() % 4) != 0 {
            ret.push(0);
        }
        ret
    };

    let mut mem = vec![0; 0x20000 / 4];

    let mut marv = Marv::new();

    for i in 0..100000000 {
        //println!("*** CYCLE {} ***", i);

        if i == 0 {
            marv.reset();
            marv.bus_ready = true;
        } else {
            marv.posedge_clk();

            match marv.bus_addr >> 26 {
                0x1 => {
                    if marv.bus_enable && marv.bus_write {
                        println!("WARNING: write to program ROM (byte addr: 0x{:08x})", marv.bus_addr << 2);
                    }
                    let byte_addr = ((marv.bus_addr << 2) & 0xffff) as usize;
                    marv.bus_read_data =
                        ((program_rom[byte_addr + 0] as u32) << 0) |
                        ((program_rom[byte_addr + 1] as u32) << 8) |
                        ((program_rom[byte_addr + 2] as u32) << 16) |
                        ((program_rom[byte_addr + 3] as u32) << 24);
                }
                0x2 => {
                    let byte_addr = marv.bus_addr << 2;
                    if marv.bus_enable {
                        if marv.bus_write {
                            match byte_addr {
                                0x20000000 => {
                                    // Test complete!
                                    println!("");
                                    println!("Test complete!");
                                    println!("");
                                    let return_code = marv.bus_write_data & 0xff;
                                    if return_code == 0 {
                                        println!("Parsing program elf file {}", program_elf_file_name);

                                        let program_elf = fs::read(program_elf_file_name).expect("Couldn't read program elf file");
                                        let mut begin_signature = None;
                                        let mut end_signature = None;
                                        match Object::parse(&program_elf).expect("Couldn't parse program elf file") {
                                            Object::Elf(elf) => {
                                                for sym in &elf.syms {
                                                    let name = elf.strtab.get(sym.st_name).unwrap().unwrap();
                                                    if name == "begin_signature" {
                                                        begin_signature = Some(sym.st_value as u32);
                                                    }
                                                    if name == "end_signature" {
                                                        end_signature = Some(sym.st_value as u32);
                                                    }
                                                }
                                            }
                                            _ => panic!("Program elf file is not an elf file")
                                        };

                                        let begin_signature = ((begin_signature.expect("Couldn't find `begin_signature` symbol") >> 2) & 0x1ffffff) as usize;
                                        let end_signature = ((end_signature.expect("Couldn't find `end_signature` symbol") >> 2) & 0x1ffffff) as usize;

                                        println!("Dumping signature to file {}", signature_file_name);

                                        let mut f = File::create(signature_file_name).expect("Couldn't open signature file");
                                        for i in begin_signature..end_signature {
                                            writeln!(f, "{:08x}", mem[i]).expect("Couldn't write to signature file");
                                        }

                                        println!("SUCCESS");
                                    } else {
                                        println!("FAIL, return code: 0x{:02x}", return_code);
                                    }
                                    println!("");
                                    return;
                                }
                                0x21000000 => {
                                    // Serial write
                                    print!("{}", marv.bus_write_data as u8 as char);
                                }
                                _ => panic!("Attempted write to system regs (byte addr: 0x{:08x})", marv.bus_addr << 2)
                            }
                        } else {
                            panic!("Attempted read unknown system reg (byte addr: 0x{:08x})", byte_addr);
                        }
                    }
                }
                0x3 => {
                    let mem_addr = (marv.bus_addr & 0x1ffffff) as usize;
                    marv.bus_read_data = mem[mem_addr];
                    if marv.bus_enable && marv.bus_write {
                        let read_data = mem[mem_addr];
                        let mut write_data = 0;
                        for i in 0..4 {
                            let sel = (marv.bus_write_byte_enable & (1 << i)) != 0;
                            write_data |= if sel { marv.bus_write_data } else { read_data } & (0xff << (8 * i));
                        }
                        mem[mem_addr] = write_data;
                    }
                }
                _ => {
                    if marv.bus_enable {
                        if marv.bus_write {
                            panic!("Attempted write to unmapped address: 0x{:08x}", marv.bus_addr << 2);
                        } else {
                            panic!("Attempted read from unmapped address: 0x{:08x}", marv.bus_addr << 2);
                        }
                    }
                }
            }
            marv.bus_read_data_valid = marv.bus_enable && !marv.bus_write;
        }

        marv.prop();

        /*println!("bus_addr: 0x{:08x} (byte addr: 0x{:08x})", marv.bus_addr, marv.bus_addr << 2);
        println!("bus_write_byte_enable: 0b{:04b}", marv.bus_write_byte_enable);
        println!("bus_write_data: 0x{:08x}", marv.bus_write_data);
        println!("bus_enable: {}", marv.bus_enable);
        println!("bus_write: {}", marv.bus_write);*/

        //println!("");
    }
}
