use crate::marv;
use crate::marv_interconnect_bridge;
use crate::interconnect;

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

    m.output("ddr3_interface_bus_enable", interconnect.output("ddr3_interface_bus_enable"));
    m.output("ddr3_interface_bus_addr", interconnect.output("ddr3_interface_bus_addr"));
    m.output("ddr3_interface_bus_write", interconnect.output("ddr3_interface_bus_write"));
    m.output("ddr3_interface_bus_write_data", interconnect.output("ddr3_interface_bus_write_data"));
    m.output("ddr3_interface_bus_write_byte_enable", interconnect.output("ddr3_interface_bus_write_byte_enable"));
    interconnect.drive_input("ddr3_interface_bus_ready", m.input("ddr3_interface_bus_ready", 1));
    interconnect.drive_input("ddr3_interface_bus_read_data", m.input("ddr3_interface_bus_read_data", 128));
    interconnect.drive_input("ddr3_interface_bus_read_data_valid", m.input("ddr3_interface_bus_read_data_valid", 1));

    m.output("bios_rom_bus_enable", interconnect.output("bios_rom_bus_enable"));
    m.output("bios_rom_bus_addr", interconnect.output("bios_rom_bus_addr"));
    m.output("bios_rom_bus_write", interconnect.output("bios_rom_bus_write"));
    m.output("bios_rom_bus_write_data", interconnect.output("bios_rom_bus_write_data"));
    m.output("bios_rom_bus_write_byte_enable", interconnect.output("bios_rom_bus_write_byte_enable"));
    interconnect.drive_input("bios_rom_bus_ready", m.input("bios_rom_bus_ready", 1));
    interconnect.drive_input("bios_rom_bus_read_data", m.input("bios_rom_bus_read_data", 128));
    interconnect.drive_input("bios_rom_bus_read_data_valid", m.input("bios_rom_bus_read_data_valid", 1));

    m.output("led_interface_bus_enable", interconnect.output("led_interface_bus_enable"));
    m.output("led_interface_bus_addr", interconnect.output("led_interface_bus_addr"));
    m.output("led_interface_bus_write", interconnect.output("led_interface_bus_write"));
    m.output("led_interface_bus_write_data", interconnect.output("led_interface_bus_write_data"));
    m.output("led_interface_bus_write_byte_enable", interconnect.output("led_interface_bus_write_byte_enable"));
    interconnect.drive_input("led_interface_bus_ready", m.input("led_interface_bus_ready", 1));
    interconnect.drive_input("led_interface_bus_read_data", m.input("led_interface_bus_read_data", 128));
    interconnect.drive_input("led_interface_bus_read_data_valid", m.input("led_interface_bus_read_data_valid", 1));

    m.output("uart_interface_bus_enable", interconnect.output("uart_interface_bus_enable"));
    m.output("uart_interface_bus_addr", interconnect.output("uart_interface_bus_addr"));
    m.output("uart_interface_bus_write", interconnect.output("uart_interface_bus_write"));
    m.output("uart_interface_bus_write_data", interconnect.output("uart_interface_bus_write_data"));
    m.output("uart_interface_bus_write_byte_enable", interconnect.output("uart_interface_bus_write_byte_enable"));
    interconnect.drive_input("uart_interface_bus_ready", m.input("uart_interface_bus_ready", 1));
    interconnect.drive_input("uart_interface_bus_read_data", m.input("uart_interface_bus_read_data", 128));
    interconnect.drive_input("uart_interface_bus_read_data_valid", m.input("uart_interface_bus_read_data_valid", 1));

    m
}
