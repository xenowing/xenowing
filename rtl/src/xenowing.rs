use crate::color_thrust;
use crate::helpers::*;
use crate::interconnect;
use crate::led_interface;
use crate::marv;
use crate::marv_interconnect_bridge;
use crate::uart;
use crate::uart_interface;
use crate::word_mem::*;

use kaze::*;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Xenowing");

    marv::generate(c);
    let marv = m.instance("marv", "Marv");

    marv_interconnect_bridge::generate(c);
    let marv_interconnect_bridge = m.instance("marv_interconnect_bridge", "MarvInterconnectBridge");

    marv_interconnect_bridge.drive_input("marv_bus_enable", marv.output("bus_enable"));
    marv_interconnect_bridge.drive_input("marv_bus_addr", marv.output("bus_addr"));
    marv_interconnect_bridge.drive_input("marv_bus_write", marv.output("bus_write"));
    marv_interconnect_bridge.drive_input("marv_bus_write_data", marv.output("bus_write_data"));
    marv_interconnect_bridge.drive_input("marv_bus_write_byte_enable", marv.output("bus_write_byte_enable"));
    marv.drive_input("bus_ready", marv_interconnect_bridge.output("marv_bus_ready"));
    marv.drive_input("bus_read_data", marv_interconnect_bridge.output("marv_bus_read_data"));
    marv.drive_input("bus_read_data_valid", marv_interconnect_bridge.output("marv_bus_read_data_valid"));

    interconnect::generate(c);
    let interconnect = m.instance("interconnect", "Interconnect");

    interconnect.drive_input("marv_bus_enable", marv_interconnect_bridge.output("interconnect_bus_enable"));
    interconnect.drive_input("marv_bus_addr", marv_interconnect_bridge.output("interconnect_bus_addr"));
    interconnect.drive_input("marv_bus_write", marv_interconnect_bridge.output("interconnect_bus_write"));
    interconnect.drive_input("marv_bus_write_data", marv_interconnect_bridge.output("interconnect_bus_write_data"));
    interconnect.drive_input("marv_bus_write_byte_enable", marv_interconnect_bridge.output("interconnect_bus_write_byte_enable"));
    marv_interconnect_bridge.drive_input("interconnect_bus_ready", interconnect.output("marv_bus_ready"));
    marv_interconnect_bridge.drive_input("interconnect_bus_read_data", interconnect.output("marv_bus_read_data"));
    marv_interconnect_bridge.drive_input("interconnect_bus_read_data_valid", interconnect.output("marv_bus_read_data_valid"));

    const BOOT_ROM_BITS: u32 = 12;
    const BOOT_ROM_SIZE: u32 = 1 << BOOT_ROM_BITS;
    let boot_rom_contents_bytes = {
        let mut ret = include_bytes!("../../boot_rom/boot_rom.bin").iter().cloned().collect::<Vec<u8>>();
        if ret.len() as u32 > BOOT_ROM_SIZE {
            panic!("BIOS ROM cannot be larger than {} bytes", BOOT_ROM_SIZE);
        }
        // Zero-pad ROM to fill whole size
        while (ret.len() as u32) < BOOT_ROM_SIZE {
            ret.push(0);
        }
        ret
    };
    let boot_rom_contents = {
        let mut ret = Vec::new();
        for i in 0..BOOT_ROM_SIZE / 16 {
            let mut value = 0;
            for j in 0..16 {
                value |= (boot_rom_contents_bytes[(i * 16 + j) as usize] as u128) << (j * 8);
            }
            ret.push(value);
        }
        ret
    };

    let boot_rom = m.mem("boot_rom", BOOT_ROM_BITS - 4, 128);
    boot_rom.initial_contents(&boot_rom_contents);
    interconnect.drive_input("boot_rom_bus_ready", m.high());
    interconnect.drive_input("boot_rom_bus_read_data", boot_rom.read_port(interconnect.output("boot_rom_bus_addr").bits(BOOT_ROM_BITS - 5, 0), m.high()));
    let boot_rom_bus_enable = interconnect.output("boot_rom_bus_enable");
    let boot_rom_bus_write = interconnect.output("boot_rom_bus_write");
    interconnect.drive_input("boot_rom_bus_read_data_valid", reg_next_with_default("boot_rom_bus_read_data_valid", boot_rom_bus_enable & !boot_rom_bus_write, false, m));

    let program_ram_addr_bit_width = 13;
    let program_ram_bus_enable = interconnect.output("program_ram_bus_enable");
    let program_ram_bus_write = interconnect.output("program_ram_bus_write");
    let program_ram_bus_addr = interconnect.output("program_ram_bus_addr").bits(program_ram_addr_bit_width - 1, 0);
    let program_ram_bus_write_data = interconnect.output("program_ram_bus_write_data");
    let program_ram_bus_write_byte_enable = interconnect.output("program_ram_bus_write_byte_enable");
    interconnect.drive_input("program_ram_bus_ready", m.high());
    let program_ram_mem = WordMem::new(m, "program_ram_mem", program_ram_addr_bit_width, 8, 16);
    program_ram_mem.write_port(program_ram_bus_addr, program_ram_bus_write_data, program_ram_bus_enable & program_ram_bus_write, program_ram_bus_write_byte_enable);
    interconnect.drive_input("program_ram_bus_read_data", program_ram_mem.read_port(program_ram_bus_addr, program_ram_bus_enable & !program_ram_bus_write));
    interconnect.drive_input("program_ram_bus_read_data_valid", reg_next_with_default("program_ram_bus_read_data_valid", program_ram_bus_enable & !program_ram_bus_write, false, m));

    led_interface::generate(c);
    let led_interface = m.instance("led_interface", "LedInterface");

    led_interface.drive_input("bus_enable", interconnect.output("led_interface_bus_enable"));
    led_interface.drive_input("bus_addr", interconnect.output("led_interface_bus_addr"));
    led_interface.drive_input("bus_write", interconnect.output("led_interface_bus_write"));
    led_interface.drive_input("bus_write_data", interconnect.output("led_interface_bus_write_data"));
    led_interface.drive_input("bus_write_byte_enable", interconnect.output("led_interface_bus_write_byte_enable"));
    interconnect.drive_input("led_interface_bus_ready", led_interface.output("bus_ready"));
    interconnect.drive_input("led_interface_bus_read_data", led_interface.output("bus_read_data"));
    interconnect.drive_input("led_interface_bus_read_data_valid", led_interface.output("bus_read_data_valid"));

    m.output("leds", led_interface.output("leds"));

    uart::generate_tx(c, 100000000, 460800);
    let uart_tx = m.instance("uart_tx", "UartTx");
    m.output("tx", uart_tx.output("tx"));

    uart::generate_rx(c, 100000000, 460800);
    let uart_rx = m.instance("uart_rx", "UartRx");
    uart_rx.drive_input("rx", m.input("rx", 1));

    uart_interface::generate(c);
    let uart_interface = m.instance("uart_interface", "UartInterface");

    uart_interface.drive_input("bus_enable", interconnect.output("uart_interface_bus_enable"));
    uart_interface.drive_input("bus_addr", interconnect.output("uart_interface_bus_addr"));
    uart_interface.drive_input("bus_write", interconnect.output("uart_interface_bus_write"));
    uart_interface.drive_input("bus_write_data", interconnect.output("uart_interface_bus_write_data"));
    uart_interface.drive_input("bus_write_byte_enable", interconnect.output("uart_interface_bus_write_byte_enable"));
    interconnect.drive_input("uart_interface_bus_ready", uart_interface.output("bus_ready"));
    interconnect.drive_input("uart_interface_bus_read_data", uart_interface.output("bus_read_data"));
    interconnect.drive_input("uart_interface_bus_read_data_valid", uart_interface.output("bus_read_data_valid"));

    uart_tx.drive_input("data", uart_interface.output("tx_data"));
    uart_tx.drive_input("enable", uart_interface.output("tx_enable"));
    uart_interface.drive_input("tx_ready", uart_tx.output("ready"));

    uart_interface.drive_input("rx_data", uart_rx.output("data"));
    uart_interface.drive_input("rx_data_valid", uart_rx.output("data_valid"));

    color_thrust::generate(c);
    let color_thrust = m.instance("color_thrust", "ColorThrust");

    color_thrust.drive_input("reg_bus_enable", interconnect.output("color_thrust_reg_bus_enable"));
    color_thrust.drive_input("reg_bus_addr", interconnect.output("color_thrust_reg_bus_addr").bits(color_thrust::REG_BUS_ADDR_BIT_WIDTH - 1, 0));
    color_thrust.drive_input("reg_bus_write", interconnect.output("color_thrust_reg_bus_write"));
    color_thrust.drive_input("reg_bus_write_data", interconnect.output("color_thrust_reg_bus_write_data").bits(31, 0));
    //color_thrust.drive_input("reg_bus_write_byte_enable", interconnect.output("color_thrust_reg_bus_write_byte_enable"));
    interconnect.drive_input("color_thrust_reg_bus_ready", color_thrust.output("reg_bus_ready"));
    interconnect.drive_input("color_thrust_reg_bus_read_data", m.lit(0u32, 96).concat(color_thrust.output("reg_bus_read_data")));
    interconnect.drive_input("color_thrust_reg_bus_read_data_valid", color_thrust.output("reg_bus_read_data_valid"));

    color_thrust.drive_input("color_buffer_bus_enable", interconnect.output("color_thrust_color_buffer_bus_enable"));
    color_thrust.drive_input("color_buffer_bus_addr", interconnect.output("color_thrust_color_buffer_bus_addr").bits(color_thrust::TILE_PIXELS_WORDS_BITS - 1, 0));
    color_thrust.drive_input("color_buffer_bus_write", interconnect.output("color_thrust_color_buffer_bus_write"));
    color_thrust.drive_input("color_buffer_bus_write_data", interconnect.output("color_thrust_color_buffer_bus_write_data"));
    color_thrust.drive_input("color_buffer_bus_write_byte_enable", interconnect.output("color_thrust_color_buffer_bus_write_byte_enable"));
    interconnect.drive_input("color_thrust_color_buffer_bus_ready", color_thrust.output("color_buffer_bus_ready"));
    interconnect.drive_input("color_thrust_color_buffer_bus_read_data", color_thrust.output("color_buffer_bus_read_data"));
    interconnect.drive_input("color_thrust_color_buffer_bus_read_data_valid", color_thrust.output("color_buffer_bus_read_data_valid"));

    color_thrust.drive_input("depth_buffer_bus_enable", interconnect.output("color_thrust_depth_buffer_bus_enable"));
    color_thrust.drive_input("depth_buffer_bus_addr", interconnect.output("color_thrust_depth_buffer_bus_addr").bits(color_thrust::TILE_PIXELS_WORDS_BITS - 2, 0));
    color_thrust.drive_input("depth_buffer_bus_write", interconnect.output("color_thrust_depth_buffer_bus_write"));
    color_thrust.drive_input("depth_buffer_bus_write_data", interconnect.output("color_thrust_depth_buffer_bus_write_data"));
    color_thrust.drive_input("depth_buffer_bus_write_byte_enable", interconnect.output("color_thrust_depth_buffer_bus_write_byte_enable"));
    interconnect.drive_input("color_thrust_depth_buffer_bus_ready", color_thrust.output("depth_buffer_bus_ready"));
    interconnect.drive_input("color_thrust_depth_buffer_bus_read_data", color_thrust.output("depth_buffer_bus_read_data"));
    interconnect.drive_input("color_thrust_depth_buffer_bus_read_data_valid", color_thrust.output("depth_buffer_bus_read_data_valid"));

    color_thrust.drive_input("tex_buffer_bus_enable", interconnect.output("color_thrust_tex_buffer_bus_enable"));
    color_thrust.drive_input("tex_buffer_bus_addr", interconnect.output("color_thrust_tex_buffer_bus_addr").bits(color_thrust::TEX_PIXELS_WORDS_BITS - 1, 0));
    //color_thrust.drive_input("tex_buffer_bus_write", interconnect.output("color_thrust_tex_buffer_bus_write"));
    color_thrust.drive_input("tex_buffer_bus_write_data", interconnect.output("color_thrust_tex_buffer_bus_write_data"));
    color_thrust.drive_input("tex_buffer_bus_write_byte_enable", interconnect.output("color_thrust_tex_buffer_bus_write_byte_enable"));
    interconnect.drive_input("color_thrust_tex_buffer_bus_ready", color_thrust.output("tex_buffer_bus_ready"));
    interconnect.drive_input("color_thrust_tex_buffer_bus_read_data", m.lit(0u32, 128));
    interconnect.drive_input("color_thrust_tex_buffer_bus_read_data_valid", m.low());

    let ddr3_interface_addr_bit_width = 13;
    let ddr3_interface_bus_enable = interconnect.output("ddr3_interface_bus_enable");
    let ddr3_interface_bus_write = interconnect.output("ddr3_interface_bus_write");
    let ddr3_interface_bus_addr = interconnect.output("ddr3_interface_bus_addr").bits(ddr3_interface_addr_bit_width - 1, 0);
    let ddr3_interface_bus_write_data = interconnect.output("ddr3_interface_bus_write_data");
    let ddr3_interface_bus_write_byte_enable = interconnect.output("ddr3_interface_bus_write_byte_enable");
    interconnect.drive_input("ddr3_interface_bus_ready", m.high());
    let ddr3_mem = WordMem::new(m, "ddr3_mem", ddr3_interface_addr_bit_width, 8, 16);
    ddr3_mem.write_port(ddr3_interface_bus_addr, ddr3_interface_bus_write_data, ddr3_interface_bus_enable & ddr3_interface_bus_write, ddr3_interface_bus_write_byte_enable);
    interconnect.drive_input("ddr3_interface_bus_read_data", ddr3_mem.read_port(ddr3_interface_bus_addr, ddr3_interface_bus_enable & !ddr3_interface_bus_write));
    interconnect.drive_input("ddr3_interface_bus_read_data_valid", reg_next_with_default("ddr3_interface_bus_read_data_valid", ddr3_interface_bus_enable & !ddr3_interface_bus_write, false, m));

    /*m.output("ddr3_interface_bus_enable", interconnect.output("ddr3_interface_bus_enable"));
    m.output("ddr3_interface_bus_addr", interconnect.output("ddr3_interface_bus_addr"));
    m.output("ddr3_interface_bus_write", interconnect.output("ddr3_interface_bus_write"));
    m.output("ddr3_interface_bus_write_data", interconnect.output("ddr3_interface_bus_write_data"));
    m.output("ddr3_interface_bus_write_byte_enable", interconnect.output("ddr3_interface_bus_write_byte_enable"));
    interconnect.drive_input("ddr3_interface_bus_ready", m.input("ddr3_interface_bus_ready", 1));
    interconnect.drive_input("ddr3_interface_bus_read_data", m.input("ddr3_interface_bus_read_data", 128));
    interconnect.drive_input("ddr3_interface_bus_read_data_valid", m.input("ddr3_interface_bus_read_data_valid", 1));*/

    m
}
