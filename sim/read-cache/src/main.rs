mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use modules::*;

#[cfg(test)]
mod tests;

use kaze::runtime::tracing::*;
use kaze::runtime::tracing::vcd::*;

use rand::{Rng, SeedableRng};
use rand::distributions::{Distribution, Uniform};

use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io;

fn build_trace(test_name: &'static str) -> io::Result<impl Trace> {
    let mut path = env::temp_dir();
    path.push(format!("{}.vcd", test_name));
    println!("Writing trace to {:?}", path);
    let file = File::create(path)?;
    VcdTrace::new(file, 10, TimeScaleUnit::Ns)
}

fn main() -> io::Result<()> {
    let seed = env::args().skip(1).nth(0).expect("seed not specified").parse().expect("Couldn't parse seed");
    let num_cycles = env::args().skip(2).nth(0).expect("num cycles not specified").parse().expect("Couldn't parse num cycles");

    let data_bit_width = 32;

    let mem_addr_bit_width = 4;
    let mem_num_elements = 1 << mem_addr_bit_width;
    let mem_data = (0..mem_num_elements).collect::<Vec<_>>();

    let cache_addr_bit_width = 2;
    let cache_num_elements = 1 << cache_addr_bit_width;

    println!("Testing ReadCache with seed = {}, num cycles = {}, mem size = {} bytes, cache size = {} bytes", seed, num_cycles, mem_num_elements * data_bit_width / 8, cache_num_elements * data_bit_width / 8);

    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

    let trace = build_trace("ReadCache__fuzz")?;

    let mut m = ReadCacheDelayedReturnPath::new(trace)?;
    let mut time_stamp = 0;

    m.reset();

    let mut issued_cache_addrs = VecDeque::new();
    let mut successful_reads = 0;

    let mut last_mem_addr = None;

    while time_stamp < num_cycles {
        // Invalidate
        m.invalidate = Uniform::new_inclusive(0.0, 1.0).sample(&mut rng) < 0.05;

        // Mem read return (to cache)
        if let Some(addr) = last_mem_addr {
            m.system_bus_read_data = mem_data[addr as usize];
            m.system_bus_read_data_valid = true;
        } else {
            m.system_bus_read_data_valid = false;
        }

        m.prop();

        // Cache read issue (from user)
        if rng.gen() {
            m.client_bus_enable = true;
            let addr = rng.gen::<u32>() % mem_num_elements;
            m.client_bus_addr = addr;
            if m.client_bus_ready {
                issued_cache_addrs.push_back(addr);
            }
        } else {
            m.client_bus_enable = false;
        }

        // Cache read return (to user)
        if m.client_bus_read_data_valid {
            let addr = issued_cache_addrs.pop_front().expect("Cache returned data but no corresponding read was issued");
            assert_eq!(mem_data[addr as usize], m.client_bus_read_data);
            successful_reads += 1;
        }

        // Mem read issue (from cache)
        m.system_bus_ready = rng.gen();
        last_mem_addr = if m.system_bus_enable && m.system_bus_ready {
            Some(m.system_bus_addr)
        } else {
            None
        };

        m.prop();
        m.update_trace(time_stamp)?;

        m.posedge_clk();
        time_stamp += 1;
    }

    println!("Test successful after {} cycles", time_stamp);
    println!("Successful reads: {}", successful_reads);

    Ok(())
}
