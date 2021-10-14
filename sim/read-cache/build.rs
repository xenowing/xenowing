use kaze::*;
use rtl::buster::*;
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

    sim::generate(ReadCache::new(
        "read_cache",
        data_bit_width, addr_bit_width,
        cache_addr_bit_width,
        &c,
    ).m, sim::GenerationOptions {
        tracing: true,
        ..sim::GenerationOptions::default()
    }, &mut file)?;

    sim::generate(ReadCacheDelayedReturnPath::new(
        "read_cache_delayed_return_path",
        data_bit_width,
        addr_bit_width,
        cache_addr_bit_width,
        &c,
    ).m, sim::GenerationOptions {
        tracing: true,
        ..sim::GenerationOptions::default()
    }, file)
}

#[allow(unused)]
struct ReadCacheDelayedReturnPath<'a> {
    pub m: &'a Module<'a>,
    pub invalidate: &'a Input<'a>,
    pub client_port: ReplicaPort<'a>,
    pub system_port: PrimaryPort<'a>,
}

impl<'a> ReadCacheDelayedReturnPath<'a> {
    fn new(
        instance_name: impl Into<String>,
        data_bit_width: u32,
        addr_bit_width: u32,
        cache_addr_bit_width: u32,
        p: &'a impl ModuleParent<'a>,
    ) -> ReadCacheDelayedReturnPath<'a> {
        let m = p.module(instance_name, "ReadCacheDelayedReturnPath");

        let read_cache = ReadCache::new("read_cache", data_bit_width, addr_bit_width, cache_addr_bit_width, m);

        let invalidate = m.input("invalidate", 1);
        read_cache.invalidate.drive(invalidate);

        let client_port = read_cache.client_port.forward("client", m);

        let delay_cycles = 8;

        let system_bus_enable = m.output("system_bus_enable", read_cache.system_port.bus_enable);
        let system_bus_addr = m.output("system_bus_addr", read_cache.system_port.bus_addr);
        let system_bus_ready = m.input("system_bus_ready", 1);
        read_cache.system_port.bus_ready.drive(system_bus_ready);
        let system_bus_read_data = m.input("system_bus_read_data", data_bit_width);
        let system_bus_read_data_valid = m.input("system_bus_read_data_valid", 1);

        let mut read_data: &dyn Signal<'a> = system_bus_read_data;
        let mut read_data_valid: &dyn Signal<'a> = system_bus_read_data_valid;
        for i in 0..delay_cycles {
            read_data = read_data.reg_next(format!("read_data_delay_{}", i));
            read_data_valid = read_data_valid.reg_next_with_default(format!("read_data_valid_delay_{}", i), false);
        }
        read_cache.system_port.bus_read_data.drive(read_data);
        read_cache.system_port.bus_read_data_valid.drive(read_data_valid);

        ReadCacheDelayedReturnPath {
            m,
            invalidate,
            client_port,
            system_port: PrimaryPort {
                bus_enable: system_bus_enable,
                bus_addr: system_bus_addr,
                bus_write: m.output("system_bus_write", m.low()),
                bus_write_data: m.output("system_bus_write_data", m.lit(0u32, data_bit_width)),
                bus_write_byte_enable: m.output("system_bus_write_byte_enable", m.lit(0u32, data_bit_width / 8)),
                bus_ready: system_bus_ready,
                bus_read_data: system_bus_read_data,
                bus_read_data_valid: system_bus_read_data_valid,
            },
        }
    }
}
