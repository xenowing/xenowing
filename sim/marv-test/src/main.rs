mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let rom_file_name = env::args().nth(1).expect("No ROM file name specified");

    let rom_buf = {
        let mut file = File::open(rom_file_name).expect("Couldn't open ROM file");
        let mut ret = vec![0; 0x10000];
        file.read_to_end(&mut ret).expect("Couldn't read ROM file");

        ret
    };

    let mut marv = marv::new();
    marv.reset();

    marv.alu_res = 0xfadebabe;
    marv.bus_read_data = 0xfadebabe;
    marv.bus_read_data_valid = false;
    marv.bus_ready = true;
    marv.register_file_read_data1 = 0xfadebabe;
    marv.register_file_read_data2 = 0xfadebabe;

    marv.prop();

    println!("alu_lhs: 0x{:08x}", marv.alu_lhs);
    println!("alu_op: 0b{:03b}", marv.alu_op);
    println!("alu_op_mod: {}", marv.alu_op_mod);
    println!("alu_rhs: 0x{:08x}", marv.alu_rhs);
    println!("bus_addr: 0x{:08x} (byte addr: 0x{:08x})", marv.bus_addr, marv.bus_addr << 2);
    println!("bus_byte_enable: 0b{:04b}", marv.bus_byte_enable);
    println!("bus_read_req: {}", marv.bus_read_req);
    println!("bus_write_data: 0x{:08x}", marv.bus_write_data);
    println!("bus_write_req: {}", marv.bus_write_req);
    println!("register_file_read_addr1: 0x{:02x}", marv.register_file_read_addr1);
    println!("register_file_read_addr2: 0x{:02x}", marv.register_file_read_addr2);
    println!("register_file_write_addr: 0x{:02x}", marv.register_file_write_addr);
    println!("register_file_write_data: 0x{:08x}", marv.register_file_write_data);
    println!("register_file_write_enable: {}", marv.register_file_write_enable);
}
