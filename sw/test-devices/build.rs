use kaze::*;

use color_thrust_meta::*;

use rtl::buster::*;
use rtl::byte_ram::*;
use rtl::color_thrust::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let file = File::create(&dest_path).unwrap();

    let p = Context::new();

    let m = p.module("top", "Top");

    let color_thrust = ColorThrust::new("color_thrust", m);

    let mem = ByteRam::new("mem", SYSTEM_BUS_ADDR_BITS, SYSTEM_BUS_ADDR_BITS, m);

    // Interconnect
    color_thrust.reg_port.forward("reg", m);
    color_thrust.color_buffer_port.forward("color_buffer", m);
    color_thrust.depth_buffer_port.forward("depth_buffer", m);

    let mem_crossbar = Crossbar::new("mem_crossbar", 2, 1, SYSTEM_BUS_ADDR_BITS, 0, 128, 5, m);

    mem_crossbar.replica_ports[0].forward("mem", m);
    color_thrust.tex_cache_system_port.connect(&mem_crossbar.replica_ports[1]);

    mem_crossbar.primary_ports[0].connect(&mem.client_port);

    sim::generate(m, sim::GenerationOptions::default(), file)
}
