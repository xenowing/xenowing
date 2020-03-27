use crate::helpers::*;
use crate::interconnect;
use crate::led_interface;
use crate::marv;
use crate::marv_interconnect_bridge;
use crate::uart;
use crate::uart_interface;

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
    // "interconnect" is a SystemVerilog keyword
    let interconnect = m.instance("interconnect0", "Interconnect");

    interconnect.drive_input("marv_bus_enable", marv_interconnect_bridge.output("interconnect_bus_enable"));
    interconnect.drive_input("marv_bus_addr", marv_interconnect_bridge.output("interconnect_bus_addr"));
    interconnect.drive_input("marv_bus_write", marv_interconnect_bridge.output("interconnect_bus_write"));
    interconnect.drive_input("marv_bus_write_data", marv_interconnect_bridge.output("interconnect_bus_write_data"));
    interconnect.drive_input("marv_bus_write_byte_enable", marv_interconnect_bridge.output("interconnect_bus_write_byte_enable"));
    marv_interconnect_bridge.drive_input("interconnect_bus_ready", interconnect.output("marv_bus_ready"));
    marv_interconnect_bridge.drive_input("interconnect_bus_read_data", interconnect.output("marv_bus_read_data"));
    marv_interconnect_bridge.drive_input("interconnect_bus_read_data_valid", interconnect.output("marv_bus_read_data_valid"));

    const BIOS_ROM_BITS: u32 = 12;
    const BIOS_ROM_SIZE: u32 = 1 << BIOS_ROM_BITS;
    let bios_rom_contents_bytes = {
        let mut ret = include_bytes!("../../rom/rom.bin").iter().cloned().collect::<Vec<u8>>();
        if ret.len() as u32 > BIOS_ROM_SIZE {
            panic!("BIOS ROM cannot be larger than {} bytes", BIOS_ROM_SIZE);
        }
        // Zero-pad ROM to fill whole size
        while (ret.len() as u32) < BIOS_ROM_SIZE {
            ret.push(0);
        }
        ret
    };
    let bios_rom_contents = {
        let mut ret = Vec::new();
        for i in 0..BIOS_ROM_SIZE / 16 {
            let mut value = 0;
            for j in 0..16 {
                value |= (bios_rom_contents_bytes[(i * 16 + j) as usize] as u128) << (j * 8);
            }
            ret.push(value);
        }
        ret
    };

    let bios_rom = m.mem("bios_rom", BIOS_ROM_BITS - 4, 128);
    bios_rom.initial_contents(&bios_rom_contents);
    interconnect.drive_input("bios_rom_bus_ready", m.high());
    interconnect.drive_input("bios_rom_bus_read_data", bios_rom.read_port(interconnect.output("bios_rom_bus_addr").bits(BIOS_ROM_BITS - 5, 0), m.high()));
    let bios_rom_bus_enable = interconnect.output("bios_rom_bus_enable");
    let bios_rom_bus_write = interconnect.output("bios_rom_bus_write");
    interconnect.drive_input("bios_rom_bus_read_data_valid", reg_next_with_default("bios_rom_bus_read_data_valid", bios_rom_bus_enable & !bios_rom_bus_write, false, m));

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

    let ddr3_interface_addr_bit_width = 13;
    let ddr3_interface_bus_enable = interconnect.output("ddr3_interface_bus_enable");
    let ddr3_interface_bus_write = interconnect.output("ddr3_interface_bus_write");
    let ddr3_interface_bus_addr = interconnect.output("ddr3_interface_bus_addr").bits(ddr3_interface_addr_bit_width - 1, 0);
    let ddr3_interface_bus_write_data = interconnect.output("ddr3_interface_bus_write_data");
    let ddr3_interface_bus_write_byte_enable = interconnect.output("ddr3_interface_bus_write_byte_enable");
    interconnect.drive_input("ddr3_interface_bus_ready", m.high());
    let mut read_word = None;
    for byte_index in 0..16 {
        let mem = m.mem(format!("ddr3_mem_byte_{}", byte_index), ddr3_interface_addr_bit_width, 8);
        let byte_write_data = ddr3_interface_bus_write_data.bits((byte_index * 8) + 7, byte_index * 8);
        let byte_write_enable = ddr3_interface_bus_enable & ddr3_interface_bus_write & ddr3_interface_bus_write_byte_enable.bit(byte_index);
        mem.write_port(ddr3_interface_bus_addr, byte_write_data, byte_write_enable);
        let read_byte = mem.read_port(ddr3_interface_bus_addr, m.high());
        match read_word {
            Some(word) => {
                read_word = Some(read_byte.concat(word));
            }
            _ => {
                read_word = Some(read_byte);
            }
        }
    }
    interconnect.drive_input("ddr3_interface_bus_read_data", read_word.unwrap());
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
