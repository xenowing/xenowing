use kaze::*;
use rtl::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let file = File::create(&dest_path).unwrap();

    let c = Context::new();

    let m = c.module("Top");

    color_thrust::generate(&c);
    let color_thrust = m.instance("color_thrust", "ColorThrust");

    color_thrust.drive_input("reg_bus_enable", m.input("reg_bus_enable", 1));
    color_thrust.drive_input("reg_bus_addr", m.input("reg_bus_addr", color_thrust::REG_BUS_ADDR_BIT_WIDTH));
    color_thrust.drive_input("reg_bus_write", m.input("reg_bus_write", 1));
    color_thrust.drive_input("reg_bus_write_data", m.input("reg_bus_write_data", 32));
    //color_thrust.drive_input("reg_bus_write_byte_enable", m.input("reg_bus_write_byte_enable", 4));
    m.output("reg_bus_ready", color_thrust.output("reg_bus_ready"));
    m.output("reg_bus_read_data", color_thrust.output("reg_bus_read_data"));
    m.output("reg_bus_read_data_valid", color_thrust.output("reg_bus_read_data_valid"));

    color_thrust.drive_input("color_buffer_bus_enable", m.input("color_buffer_bus_enable", 1));
    color_thrust.drive_input("color_buffer_bus_addr", m.input("color_buffer_bus_addr", color_thrust::TILE_PIXELS_WORDS_BITS));
    color_thrust.drive_input("color_buffer_bus_write", m.input("color_buffer_bus_write", 1));
    color_thrust.drive_input("color_buffer_bus_write_data", m.input("color_buffer_bus_write_data", 128));
    color_thrust.drive_input("color_buffer_bus_write_byte_enable", m.input("color_buffer_bus_write_byte_enable", 16));
    m.output("color_buffer_bus_ready", color_thrust.output("color_buffer_bus_ready"));
    m.output("color_buffer_bus_read_data", color_thrust.output("color_buffer_bus_read_data"));
    m.output("color_buffer_bus_read_data_valid", color_thrust.output("color_buffer_bus_read_data_valid"));

    color_thrust.drive_input("depth_buffer_bus_enable", m.input("depth_buffer_bus_enable", 1));
    color_thrust.drive_input("depth_buffer_bus_addr", m.input("depth_buffer_bus_addr", color_thrust::TILE_PIXELS_WORDS_BITS - 1));
    color_thrust.drive_input("depth_buffer_bus_write", m.input("depth_buffer_bus_write", 1));
    color_thrust.drive_input("depth_buffer_bus_write_data", m.input("depth_buffer_bus_write_data", 128));
    color_thrust.drive_input("depth_buffer_bus_write_byte_enable", m.input("depth_buffer_bus_write_byte_enable", 16));
    m.output("depth_buffer_bus_ready", color_thrust.output("depth_buffer_bus_ready"));
    m.output("depth_buffer_bus_read_data", color_thrust.output("depth_buffer_bus_read_data"));
    m.output("depth_buffer_bus_read_data_valid", color_thrust.output("depth_buffer_bus_read_data_valid"));

    // TODO: Better name?
    buster::generate(&c, "MemCrossbar", 2, 1, 13, 0, 128, 5);
    let mem = m.instance("mem", "MemCrossbar");

    let ddr3_interface_addr_bit_width = 13;
    let ddr3_interface_bus_enable = mem.output("replica0_bus_enable");
    let ddr3_interface_bus_write = mem.output("replica0_bus_write");
    let ddr3_interface_bus_addr = mem.output("replica0_bus_addr").bits(ddr3_interface_addr_bit_width - 1, 0);
    let ddr3_interface_bus_write_data = mem.output("replica0_bus_write_data");
    let ddr3_interface_bus_write_byte_enable = mem.output("replica0_bus_write_byte_enable");
    mem.drive_input("replica0_bus_ready", m.high());
    let ddr3_mem = word_mem::WordMem::new(m, "ddr3_mem", ddr3_interface_addr_bit_width, 8, 16);
    ddr3_mem.write_port(ddr3_interface_bus_addr, ddr3_interface_bus_write_data, ddr3_interface_bus_enable & ddr3_interface_bus_write, ddr3_interface_bus_write_byte_enable);
    mem.drive_input("replica0_bus_read_data", ddr3_mem.read_port(ddr3_interface_bus_addr, ddr3_interface_bus_enable & !ddr3_interface_bus_write));
    mem.drive_input("replica0_bus_read_data_valid", (ddr3_interface_bus_enable & !ddr3_interface_bus_write).reg_next_with_default("ddr3_interface_bus_read_data_valid", false));

    mem.drive_input("primary0_bus_enable", m.input("mem_bus_enable", 1));
    mem.drive_input("primary0_bus_addr", m.input("mem_bus_addr", 13));
    mem.drive_input("primary0_bus_write", m.input("mem_bus_write", 1));
    mem.drive_input("primary0_bus_write_data", m.input("mem_bus_write_data", 128));
    mem.drive_input("primary0_bus_write_byte_enable", m.input("mem_bus_write_byte_enable", 16));
    m.output("mem_bus_ready", mem.output("primary0_bus_ready"));
    m.output("mem_bus_read_data", mem.output("primary0_bus_read_data"));
    m.output("mem_bus_read_data_valid", mem.output("primary0_bus_read_data_valid"));

    mem.drive_input("primary1_bus_enable", color_thrust.output("replica_bus_enable"));
    mem.drive_input("primary1_bus_addr", color_thrust.output("replica_bus_addr"));
    mem.drive_input("primary1_bus_write", m.low());
    mem.drive_input("primary1_bus_write_data", m.lit(0u32, 128));
    mem.drive_input("primary1_bus_write_byte_enable", m.lit(0u32, 16));
    color_thrust.drive_input("replica_bus_ready", mem.output("primary1_bus_ready"));
    color_thrust.drive_input("replica_bus_read_data", mem.output("primary1_bus_read_data"));
    color_thrust.drive_input("replica_bus_read_data_valid", mem.output("primary1_bus_read_data_valid"));

    sim::generate(m, sim::GenerationOptions::default(), file)
}
