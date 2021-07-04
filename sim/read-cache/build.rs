use kaze::*;
use rtl::read_cache::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let c = Context::new();

    // TODO: Expose these to test driver somehow so we don't have to duplicate them
    let data_bit_width = 32;
    let addr_bit_width = 4;
    let cache_addr_bit_width = 2;

    sim::generate(read_cache::generate(&c, "ReadCache", data_bit_width, addr_bit_width, cache_addr_bit_width), sim::GenerationOptions {
        tracing: true,
        ..sim::GenerationOptions::default()
    }, &mut file)?;

    let m = c.module("ReadCacheDelayedReturnPath");

    let read_cache = m.instance("read_cache", "ReadCache");

    read_cache.drive_input("invalidate", m.input("invalidate", 1));
    m.output("primary_bus_ready", read_cache.output("primary_bus_ready"));
    read_cache.drive_input("primary_bus_enable", m.input("primary_bus_enable", 1));
    read_cache.drive_input("primary_bus_addr", m.input("primary_bus_addr", addr_bit_width));
    m.output("primary_bus_read_data", read_cache.output("primary_bus_read_data"));
    m.output("primary_bus_read_data_valid", read_cache.output("primary_bus_read_data_valid"));
    read_cache.drive_input("replica_bus_ready", m.input("replica_bus_ready", 1));
    m.output("replica_bus_enable", read_cache.output("replica_bus_enable"));
    m.output("replica_bus_addr", read_cache.output("replica_bus_addr"));
    let delay_cycles = 8;
    let mut replica_bus_read_data = m.input("replica_bus_read_data", data_bit_width);
    let mut replica_bus_read_data_valid = m.input("replica_bus_read_data_valid", 1);
    for i in 0..delay_cycles {
        replica_bus_read_data = replica_bus_read_data.reg_next(format!("replica_bus_read_data_delay_{}", i));
        replica_bus_read_data_valid = replica_bus_read_data_valid.reg_next_with_default(format!("replica_bus_read_data_valid_delay_{}", i), false);
    }
    read_cache.drive_input("replica_bus_read_data", replica_bus_read_data);
    read_cache.drive_input("replica_bus_read_data_valid", replica_bus_read_data_valid);

    sim::generate(m, sim::GenerationOptions {
        tracing: true,
        ..sim::GenerationOptions::default()
    }, file)
}
