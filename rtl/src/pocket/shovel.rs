use crate::buster::*;
use crate::byte_ram::*;
use crate::marv::*;
use crate::pocket::char_display::*;
use crate::read_cache::*;

use kaze::*;

pub struct Shovel<'a> {
    pub m: &'a Module<'a>,

    pub system_write_vsync_pulse: &'a Input<'a>,
    pub system_write_line_pulse: &'a Input<'a>,

    pub video_line_buffer_write_enable: &'a Output<'a>,
    pub video_line_buffer_write_data: &'a Output<'a>,

    pub bootloader: PrimaryPort<'a>,
}

impl<'a> Shovel<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> Shovel<'a> {
        let m = p.module(instance_name, "Shovel");

        let marv = Marv::new("marv", m);

        let char_display = CharDisplay::new("char_display", m);
        let system_write_vsync_pulse = m.input("system_write_vsync_pulse", 1);
        let system_write_line_pulse = m.input("system_write_line_pulse", 1);
        char_display
            .system_write_vsync_pulse
            .drive(system_write_vsync_pulse);
        char_display
            .system_write_line_pulse
            .drive(system_write_line_pulse);

        let byte_ram = ByteRam::new("byte_ram", 14 - 2, 26, m);

        // Interconnect
        let cpu_crossbar = Crossbar::new("cpu_crossbar", 2, 2, 30, 4, 32, 2, m);
        let marv_instruction_cache = ReadCache::new("marv_instruction_cache", 32, 30, 12 - 2, m);
        marv.instruction_port
            .connect(&marv_instruction_cache.client_port);
        marv_instruction_cache.invalidate.drive(m.low()); // TODO: Expose this to the CPU somehow
        marv_instruction_cache.system_port.connect(
            &cpu_crossbar.replica_ports[0].skid_buffer("instruction_cache_to_cpu_crossbar", m),
        );
        marv.data_port
            .connect(&cpu_crossbar.replica_ports[1].skid_buffer("data_port_to_cpu_crossbar", m));

        let mem_crossbar = Crossbar::new("mem_crossbar", 1, 1, 26, 0, 32, 2, m);
        cpu_crossbar.primary_ports[1]
            .connect(&mem_crossbar.replica_ports[0].skid_buffer("cpu_crossbar_to_mem_crossbar", m));
        mem_crossbar.primary_ports[0].connect(&byte_ram.client_port);

        let sys_crossbar = Crossbar::new("sys_crossbar", 1, 2, 26, 4, 32, 2, m);
        cpu_crossbar.primary_ports[0]
            .connect(&sys_crossbar.replica_ports[0].skid_buffer("cpu_crossbar_to_sys_crossbar", m));
        let bootloader = sys_crossbar.primary_ports[0].forward("bootloader", m);
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

            bootloader,
        }
    }
}
