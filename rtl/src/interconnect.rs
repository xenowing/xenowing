use kaze::*;

use crate::buster;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Interconnect");

    buster::generate(c, "Cpu", 1, 2, 28, 4, 128, 5);
    let cpu = m.instance("cpu", "Cpu");

    cpu.drive_input("primary0_bus_enable", m.input("marv_bus_enable", 1));
    cpu.drive_input("primary0_bus_addr", m.input("marv_bus_addr", 28));
    cpu.drive_input("primary0_bus_write", m.input("marv_bus_write", 1));
    cpu.drive_input("primary0_bus_write_data", m.input("marv_bus_write_data", 128));
    cpu.drive_input("primary0_bus_write_byte_enable", m.input("marv_bus_write_byte_enable", 16));
    m.output("marv_bus_ready", cpu.output("primary0_bus_ready"));
    m.output("marv_bus_read_data", cpu.output("primary0_bus_read_data"));
    m.output("marv_bus_read_data_valid", cpu.output("primary0_bus_read_data_valid"));

    // TODO: Better name?
    buster::generate(c, "MemCrossbar", 2, 1, 13, 0, 128, 5);
    let mem = m.instance("mem", "MemCrossbar");

    mem.drive_input("primary0_bus_enable", cpu.output("replica1_bus_enable"));
    mem.drive_input("primary0_bus_addr", cpu.output("replica1_bus_addr").bits(12, 0));
    mem.drive_input("primary0_bus_write", cpu.output("replica1_bus_write"));
    mem.drive_input("primary0_bus_write_data", cpu.output("replica1_bus_write_data"));
    mem.drive_input("primary0_bus_write_byte_enable", cpu.output("replica1_bus_write_byte_enable"));
    cpu.drive_input("replica1_bus_ready", mem.output("primary0_bus_ready"));
    cpu.drive_input("replica1_bus_read_data", mem.output("primary0_bus_read_data"));
    cpu.drive_input("replica1_bus_read_data_valid", mem.output("primary0_bus_read_data_valid"));

    mem.drive_input("primary1_bus_enable", m.input("color_thrust_replica_bus_enable", 1));
    mem.drive_input("primary1_bus_addr", m.input("color_thrust_replica_bus_addr", 13));
    mem.drive_input("primary1_bus_write", m.low());
    mem.drive_input("primary1_bus_write_data", m.lit(0u32, 128));
    mem.drive_input("primary1_bus_write_byte_enable", m.lit(0u32, 16));
    m.output("color_thrust_replica_bus_ready", mem.output("primary1_bus_ready"));
    m.output("color_thrust_replica_bus_read_data", mem.output("primary1_bus_read_data"));
    m.output("color_thrust_replica_bus_read_data_valid", mem.output("primary1_bus_read_data_valid"));

    m.output("ddr3_interface_bus_enable", mem.output("replica0_bus_enable"));
    m.output("ddr3_interface_bus_addr", mem.output("replica0_bus_addr"));
    m.output("ddr3_interface_bus_write", mem.output("replica0_bus_write"));
    m.output("ddr3_interface_bus_write_data", mem.output("replica0_bus_write_data"));
    m.output("ddr3_interface_bus_write_byte_enable", mem.output("replica0_bus_write_byte_enable"));
    mem.drive_input("replica0_bus_ready", m.input("ddr3_interface_bus_ready", 1));
    mem.drive_input("replica0_bus_read_data", m.input("ddr3_interface_bus_read_data", 128));
    mem.drive_input("replica0_bus_read_data_valid", m.input("ddr3_interface_bus_read_data_valid", 1));

    buster::generate(c, "Sys", 1, 7, 24, 4, 128, 5);
    let sys = m.instance("sys", "Sys");

    sys.drive_input("primary0_bus_enable", cpu.output("replica0_bus_enable"));
    sys.drive_input("primary0_bus_addr", cpu.output("replica0_bus_addr"));
    sys.drive_input("primary0_bus_write", cpu.output("replica0_bus_write"));
    sys.drive_input("primary0_bus_write_data", cpu.output("replica0_bus_write_data"));
    sys.drive_input("primary0_bus_write_byte_enable", cpu.output("replica0_bus_write_byte_enable"));
    cpu.drive_input("replica0_bus_ready", sys.output("primary0_bus_ready"));
    cpu.drive_input("replica0_bus_read_data", sys.output("primary0_bus_read_data"));
    cpu.drive_input("replica0_bus_read_data_valid", sys.output("primary0_bus_read_data_valid"));

    m.output("boot_rom_bus_enable", sys.output("replica0_bus_enable"));
    m.output("boot_rom_bus_addr", sys.output("replica0_bus_addr"));
    m.output("boot_rom_bus_write", sys.output("replica0_bus_write"));
    m.output("boot_rom_bus_write_data", sys.output("replica0_bus_write_data"));
    m.output("boot_rom_bus_write_byte_enable", sys.output("replica0_bus_write_byte_enable"));
    sys.drive_input("replica0_bus_ready", m.input("boot_rom_bus_ready", 1));
    sys.drive_input("replica0_bus_read_data", m.input("boot_rom_bus_read_data", 128));
    sys.drive_input("replica0_bus_read_data_valid", m.input("boot_rom_bus_read_data_valid", 1));

    m.output("program_ram_bus_enable", sys.output("replica1_bus_enable"));
    m.output("program_ram_bus_addr", sys.output("replica1_bus_addr"));
    m.output("program_ram_bus_write", sys.output("replica1_bus_write"));
    m.output("program_ram_bus_write_data", sys.output("replica1_bus_write_data"));
    m.output("program_ram_bus_write_byte_enable", sys.output("replica1_bus_write_byte_enable"));
    sys.drive_input("replica1_bus_ready", m.input("program_ram_bus_ready", 1));
    sys.drive_input("replica1_bus_read_data", m.input("program_ram_bus_read_data", 128));
    sys.drive_input("replica1_bus_read_data_valid", m.input("program_ram_bus_read_data_valid", 1));

    m.output("led_interface_bus_enable", sys.output("replica2_bus_enable"));
    m.output("led_interface_bus_addr", sys.output("replica2_bus_addr"));
    m.output("led_interface_bus_write", sys.output("replica2_bus_write"));
    m.output("led_interface_bus_write_data", sys.output("replica2_bus_write_data"));
    m.output("led_interface_bus_write_byte_enable", sys.output("replica2_bus_write_byte_enable"));
    sys.drive_input("replica2_bus_ready", m.input("led_interface_bus_ready", 1));
    sys.drive_input("replica2_bus_read_data", m.input("led_interface_bus_read_data", 128));
    sys.drive_input("replica2_bus_read_data_valid", m.input("led_interface_bus_read_data_valid", 1));

    m.output("uart_interface_bus_enable", sys.output("replica3_bus_enable"));
    m.output("uart_interface_bus_addr", sys.output("replica3_bus_addr"));
    m.output("uart_interface_bus_write", sys.output("replica3_bus_write"));
    m.output("uart_interface_bus_write_data", sys.output("replica3_bus_write_data"));
    m.output("uart_interface_bus_write_byte_enable", sys.output("replica3_bus_write_byte_enable"));
    sys.drive_input("replica3_bus_ready", m.input("uart_interface_bus_ready", 1));
    sys.drive_input("replica3_bus_read_data", m.input("uart_interface_bus_read_data", 128));
    sys.drive_input("replica3_bus_read_data_valid", m.input("uart_interface_bus_read_data_valid", 1));

    m.output("color_thrust_reg_bus_enable", sys.output("replica4_bus_enable"));
    m.output("color_thrust_reg_bus_addr", sys.output("replica4_bus_addr"));
    m.output("color_thrust_reg_bus_write", sys.output("replica4_bus_write"));
    m.output("color_thrust_reg_bus_write_data", sys.output("replica4_bus_write_data"));
    m.output("color_thrust_reg_bus_write_byte_enable", sys.output("replica4_bus_write_byte_enable"));
    sys.drive_input("replica4_bus_ready", m.input("color_thrust_reg_bus_ready", 1));
    sys.drive_input("replica4_bus_read_data", m.input("color_thrust_reg_bus_read_data", 128));
    sys.drive_input("replica4_bus_read_data_valid", m.input("color_thrust_reg_bus_read_data_valid", 1));

    m.output("color_thrust_color_buffer_bus_enable", sys.output("replica5_bus_enable"));
    m.output("color_thrust_color_buffer_bus_addr", sys.output("replica5_bus_addr"));
    m.output("color_thrust_color_buffer_bus_write", sys.output("replica5_bus_write"));
    m.output("color_thrust_color_buffer_bus_write_data", sys.output("replica5_bus_write_data"));
    m.output("color_thrust_color_buffer_bus_write_byte_enable", sys.output("replica5_bus_write_byte_enable"));
    sys.drive_input("replica5_bus_ready", m.input("color_thrust_color_buffer_bus_ready", 1));
    sys.drive_input("replica5_bus_read_data", m.input("color_thrust_color_buffer_bus_read_data", 128));
    sys.drive_input("replica5_bus_read_data_valid", m.input("color_thrust_color_buffer_bus_read_data_valid", 1));

    m.output("color_thrust_depth_buffer_bus_enable", sys.output("replica6_bus_enable"));
    m.output("color_thrust_depth_buffer_bus_addr", sys.output("replica6_bus_addr"));
    m.output("color_thrust_depth_buffer_bus_write", sys.output("replica6_bus_write"));
    m.output("color_thrust_depth_buffer_bus_write_data", sys.output("replica6_bus_write_data"));
    m.output("color_thrust_depth_buffer_bus_write_byte_enable", sys.output("replica6_bus_write_byte_enable"));
    sys.drive_input("replica6_bus_ready", m.input("color_thrust_depth_buffer_bus_ready", 1));
    sys.drive_input("replica6_bus_read_data", m.input("color_thrust_depth_buffer_bus_read_data", 128));
    sys.drive_input("replica6_bus_read_data_valid", m.input("color_thrust_depth_buffer_bus_read_data_valid", 1));

    m
}
