use crate::boot_rom::*;
use crate::buster::*;
use crate::byte_ram::*;
use crate::marv::*;
use crate::marv_system_bridge::*;
use crate::pocket::char_display::*;
use crate::read_cache::*;

use kaze::*;

pub struct Shovel<'a> {
    pub m: &'a Module<'a>,

    pub system_write_vsync_pulse: &'a Input<'a>,
    pub system_write_line_pulse: &'a Input<'a>,

    pub video_line_buffer_write_enable: &'a Output<'a>,
    pub video_line_buffer_write_data: &'a Output<'a>,
}

impl<'a> Shovel<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> Shovel<'a> {
        let m = p.module(instance_name, "Shovel");

        let marv = Marv::new("marv", m);

        let boot_rom = BootRom::new("boot_rom", m);

        let char_display = CharDisplay::new("char_display", m);
        let system_write_vsync_pulse = m.input("system_write_vsync_pulse", 1);
        let system_write_line_pulse = m.input("system_write_line_pulse", 1);
        char_display
            .system_write_vsync_pulse
            .drive(system_write_vsync_pulse);
        char_display
            .system_write_line_pulse
            .drive(system_write_line_pulse);

        let byte_ram = ByteRam::new("byte_ram", 14 - 4, 24, m);

        // Interconnect
        let cpu_crossbar = Crossbar::new("cpu_crossbar", 2, 2, 28, 4, 128, 2, m);
        let marv_instruction_bridge = MarvSystemBridge::new("marv_instruction_bridge", m);
        marv.instruction_port
            .connect(&marv_instruction_bridge.marv_port);
        let marv_instruction_cache = ReadCache::new("marv_instruction_cache", 128, 28, 12 - 4, m);
        marv_instruction_cache.invalidate.drive(m.low()); // TODO: Expose this to the CPU somehow
        marv_instruction_bridge
            .system_port
            .connect(&marv_instruction_cache.client_port);
        marv_instruction_cache.system_port.connect(
            &cpu_crossbar.replica_ports[0]
                .skid_buffer("instruction_cache_to_cpu_crossbar_0", m)
                .skid_buffer("instruction_cache_to_cpu_crossbar_1", m)
                .skid_buffer("instruction_cache_to_cpu_crossbar_2", m),
        );
        let marv_data_bridge = MarvSystemBridge::new("marv_data_bridge", m);
        marv.data_port.connect(&marv_data_bridge.marv_port);
        marv_data_bridge.system_port.connect(
            &cpu_crossbar.replica_ports[1]
                .skid_buffer("data_port_to_cpu_crossbar_0", m)
                .skid_buffer("data_port_to_cpu_crossbar_1", m)
                .skid_buffer("data_port_to_cpu_crossbar_2", m),
        );

        let mem_crossbar = Crossbar::new("mem_crossbar", 1, 1, 24, 0, 128, 2, m);
        cpu_crossbar.primary_ports[1].connect(
            &mem_crossbar.replica_ports[0]
                .skid_buffer("cpu_crossbar_to_mem_crossbar_0", m)
                .skid_buffer("cpu_crossbar_to_mem_crossbar_1", m),
        );
        mem_crossbar.primary_ports[0].connect(&byte_ram.client_port);

        let sys_crossbar = Crossbar::new("sys_crossbar", 1, 2, 24, 4, 128, 2, m);
        cpu_crossbar.primary_ports[0].connect(
            &sys_crossbar.replica_ports[0]
                .skid_buffer("cpu_crossbar_to_sys_crossbar_0", m)
                .skid_buffer("cpu_crossbar_to_sys_crossbar_1", m),
        );
        sys_crossbar.primary_ports[0].connect(&boot_rom.client_port);
        sys_crossbar.primary_ports[1].connect(&char_display.client_port);

        Shovel {
            m,

            system_write_vsync_pulse,
            system_write_line_pulse,

            video_line_buffer_write_enable: m.output(
                "video_line_buffer_write_enable",
                char_display.video_line_buffer_write_enable,
            ),
            video_line_buffer_write_data: m.output(
                "video_line_buffer_write_data",
                char_display.video_line_buffer_write_data,
            ),
        }
    }
}
