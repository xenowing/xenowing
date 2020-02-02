mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use std::env;
use std::fs::File;
use std::io::Read;

fn alu(lhs: u32, rhs: u32, shift_amt: u32, op: u32, op_mod: bool) -> u32 {
    match op {
        0b000 => {
            if !op_mod {
                // ADD
                lhs.wrapping_add(rhs)
            } else {
                // SUB
                lhs.wrapping_sub(rhs)
            }
        }
        0b001 => lhs << shift_amt, // SLL
        0b010 => if (lhs as i32) < (rhs as i32) { 1 } else { 0 }, // LT
        0b011 => if lhs < rhs { 1 } else { 0 }, // LTU
        0b100 => lhs ^ rhs, // XOR
        0b101 => {
            if !op_mod {
                // SRL
                lhs >> shift_amt
            }
            else {
                // SRA
                ((lhs as i32) >> shift_amt) as _
            }
        }
        0b110 => lhs | rhs, // OR
        0b111 => lhs & rhs, // AND
        _ => unreachable!()
    }
}

fn main() {
    let program_rom_file_name = env::args().nth(1).expect("No program ROM file name specified");

    let mut register_file = vec![0; 32];

    let program_rom = {
        let mut file = File::open(program_rom_file_name).expect("Couldn't open program ROM file");
        let mut ret = Vec::with_capacity(0x10000);
        file.read_to_end(&mut ret).expect("Couldn't read program ROM file");

        ret
    };

    let mut mem = vec![0; 0x2000000];

    let mut leds = 0b000;

    let mut marv = Marv::new();
    marv.reset();

    for _ in 0..100000000 {
        //println!("*** CYCLE {} ***", i);

        marv.bus_ready = true;

        marv.prop();

        marv.alu_res = alu(marv.alu_lhs, marv.alu_rhs, marv.alu_shift_amt, marv.alu_op, marv.alu_op_mod);
        //println!("alu_res: 0x{:08x}", marv.alu_res);

        marv.register_file_read_data1 = register_file[marv.register_file_read_addr1 as usize];
        marv.register_file_read_data2 = register_file[marv.register_file_read_addr2 as usize];
        /*println!("register_file_read_data1: 0x{:08x}", marv.register_file_read_data1);
        println!("register_file_read_data2: 0x{:08x}", marv.register_file_read_data2);*/

        marv.prop();

        /*println!("alu_lhs: 0x{:08x}", marv.alu_lhs);
        println!("alu_op: 0b{:03b}", marv.alu_op);
        println!("alu_op_mod: {}", marv.alu_op_mod);
        println!("alu_rhs: 0x{:08x}", marv.alu_rhs);
        println!("bus_addr: 0x{:08x} (byte addr: 0x{:08x})", marv.bus_addr, marv.bus_addr << 2);
        println!("bus_write_byte_enable: 0b{:04b}", marv.bus_write_byte_enable);
        println!("bus_read_req: {}", marv.bus_read_req);
        println!("bus_write_data: 0x{:08x}", marv.bus_write_data);
        println!("bus_write_req: {}", marv.bus_write_req);
        println!("register_file_read_addr1: 0x{:02x}", marv.register_file_read_addr1);
        println!("register_file_read_addr2: 0x{:02x}", marv.register_file_read_addr2);
        println!("register_file_write_addr: 0x{:02x}", marv.register_file_write_addr);
        println!("register_file_write_data: 0x{:08x}", marv.register_file_write_data);
        println!("register_file_write_enable: {}", marv.register_file_write_enable);*/

        marv.posedge_clk();

        if marv.register_file_write_enable {
            register_file[marv.register_file_write_addr as usize] = marv.register_file_write_data;
        }

        match marv.bus_addr >> 26 {
            0x1 => {
                if marv.bus_write_req {
                    panic!("Attempted write to program ROM");
                }
                let byte_addr = ((marv.bus_addr << 2) & 0x3fff) as usize;
                marv.bus_read_data =
                    ((program_rom[byte_addr + 0] as u32) << 0) |
                    ((program_rom[byte_addr + 1] as u32) << 8) |
                    ((program_rom[byte_addr + 2] as u32) << 16) |
                    ((program_rom[byte_addr + 3] as u32) << 24);
            }
            0x2 => {
                let byte_addr = marv.bus_addr << 2;
                if marv.bus_read_req {
                    marv.bus_read_data = match byte_addr {
                        0x21000000 => {
                            // UART transmitter status
                            1 // Always return ready
                        }
                        _ => panic!("Attempted read unknown system reg (byte addr: 0x{:08x})", byte_addr)
                    };
                }
                if marv.bus_write_req {
                    match byte_addr {
                        0x20000000 => {
                            // LED interface
                            let new_leds = marv.bus_write_data & 0b111;
                            if new_leds != leds {
                                leds = new_leds;
                                println!("LEDs changed: 0b{:03b}", leds);
                            }
                        }
                        0x21000004 => {
                            // UART transmitter write
                            print!("{}", marv.bus_write_data as u8 as char);
                        }
                        _ => panic!("Attempted write to system regs (byte addr: 0x{:08x})", marv.bus_addr << 2)
                    }
                }
            }
            0x3 => {
                let mem_addr = (marv.bus_addr & 0x1ffffff) as usize;
                marv.bus_read_data = mem[mem_addr];
                if marv.bus_write_req {
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
                if marv.bus_read_req || marv.bus_write_req {
                    panic!("Attempted read/write to unmapped address: 0x{:08x}", marv.bus_addr << 2);
                }
            }
        }
        marv.bus_read_data_valid = marv.bus_read_req;

        //println!("");
    }
}
