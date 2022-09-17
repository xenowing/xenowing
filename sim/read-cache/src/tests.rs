use crate::modules::*;

use kaze::runtime::tracing::*;
use kaze::runtime::tracing::vcd::*;

use std::env;
use std::fs::File;
use std::io::{self, BufWriter};

fn build_trace(test_name: &'static str) -> io::Result<impl Trace> {
    let mut path = env::temp_dir();
    path.push(format!("{}.vcd", test_name));
    println!("Writing trace to {:?}", path);
    let file = File::create(path)?;
    VcdTrace::new(BufWriter::new(file), 10, TimeScaleUnit::Ns)
}

#[test]
fn read_first_addr() -> io::Result<()> {
    let trace = build_trace("ReadCache__read_first_addr")?;

    let mut m = ReadCache::new(trace)?;
    let mut time_stamp = 0;

    m.reset();
    m.system_bus_ready = true;
    m.system_bus_read_data = 0;
    m.system_bus_read_data_valid = false;

    // Issue read to addr 0 (should cause a cache miss)
    m.client_bus_enable = true;
    m.client_bus_addr = 0;
    loop {
        m.prop();
        m.update_trace(time_stamp)?;

        let client_bus_ready = m.client_bus_ready;
        m.posedge_clk();
        time_stamp += 1;

        if client_bus_ready {
            break;
        }
    }

    // Note that we're still issuing a read to addr 0 - it should be accepted on the cycle in which we get data returned to us

    // Cache miss should be observable on the cycle following the read issue
    m.prop();
    m.update_trace(time_stamp)?;
    // Cannot accept further reads while cache miss is being processed
    assert_eq!(m.client_bus_ready, false);
    // Cache miss causes the read to be issued to the replica's bus
    assert_eq!(m.system_bus_enable, true);
    assert_eq!(m.system_bus_addr, 0);

    // Return read from replica on following cycle
    m.posedge_clk();
    time_stamp += 1;
    m.system_bus_read_data = 0xdeadbeef;
    m.system_bus_read_data_valid = true;
    m.prop();
    m.update_trace(time_stamp)?;

    // Cache should propagate replica return data in the same cycle, and should accept the next read
    assert_eq!(m.client_bus_read_data, 0xdeadbeef);
    assert_eq!(m.client_bus_read_data_valid, true);
    assert_eq!(m.client_bus_ready, true);

    // Cache should return data again on the following cycle due to the second issued read (which should be in-cache now)
    m.posedge_clk();
    time_stamp += 1;
    // De-assert read after handshake
    m.client_bus_enable = false;
    //  Bogus bus addr after correct issue to make sure this isn't used by mistake somehow
    m.client_bus_addr = 0xfadebabe;
    // De-assert returned data after handshake
    m.system_bus_read_data = 0;
    m.system_bus_read_data_valid = false;
    m.prop();
    m.update_trace(time_stamp)?;
    assert_eq!(m.client_bus_read_data, 0xdeadbeef);
    assert_eq!(m.client_bus_read_data_valid, true);
    assert_eq!(m.client_bus_ready, true);

    // Cache should not return more data on the following cycle since no read was issued
    m.posedge_clk();
    time_stamp += 1;
    m.prop();
    m.update_trace(time_stamp)?;
    assert_eq!(m.client_bus_read_data_valid, false);

    // Issue another read to addr 0 (should cause another cache hit)
    m.client_bus_enable = true;
    m.client_bus_addr = 0;
    loop {
        m.prop();
        m.update_trace(time_stamp)?;

        let client_bus_ready = m.client_bus_ready;
        m.posedge_clk();
        time_stamp += 1;

        if client_bus_ready {
            break;
        }
    }

    // De-assert read after handshake
    m.client_bus_enable = false;
    //  Bogus bus addr after correct issue to make sure this isn't used by mistake somehow
    m.client_bus_addr = 0xabad1dea;

    // Data should be in cache, so it should be returned the following cycle, and the cache should be able to accept reads again
    m.prop();
    m.update_trace(time_stamp)?;
    assert_eq!(m.client_bus_read_data, 0xdeadbeef);
    assert_eq!(m.client_bus_read_data_valid, true);
    assert_eq!(m.system_bus_enable, false);

    // Cache should not return more data on the following cycle since no read was issued
    m.posedge_clk();
    time_stamp += 1;
    m.prop();
    m.update_trace(time_stamp)?;
    assert_eq!(m.client_bus_read_data_valid, false);

    Ok(())
}

#[test]
fn read_all() -> io::Result<()> {
    let addr_bit_width = 4;
    let num_elements = 1 << addr_bit_width;
    let data = (0..num_elements).collect::<Vec<_>>();

    let mut client_read_addr = 0;
    let mut client_read_data = Vec::new();

    let mut system_read_addr = None;

    let trace = build_trace("ReadCache__read_all")?;

    let mut m = ReadCache::new(trace)?;
    let mut time_stamp = 0;

    m.reset();

    loop {
        m.prop();
        m.update_trace(time_stamp)?;

        if m.client_bus_read_data_valid {
            client_read_data.push(m.client_bus_read_data);
            if client_read_data.len() == data.len() {
                assert_eq!(client_read_data, data);
                break;
            }
        }

        if let Some(addr) = system_read_addr {
            m.system_bus_read_data = data[addr as usize];
            m.system_bus_read_data_valid = true;
        } else {
            m.system_bus_read_data_valid = false;
        }

        if client_read_addr < data.len() {
            m.client_bus_enable = true;
            m.client_bus_addr = client_read_addr as _;
        } else {
            m.client_bus_enable = false;
        }

        m.system_bus_ready = true;

        m.prop();
        m.update_trace(time_stamp)?;

        if m.client_bus_enable && m.client_bus_ready {
            client_read_addr += 1;
        }

        system_read_addr = if m.system_bus_enable {
            Some(m.system_bus_addr)
        } else {
            None
        };

        m.prop();
        m.update_trace(time_stamp)?;

        m.posedge_clk();
        time_stamp += 1;
    }

    assert_eq!(time_stamp, 37);

    Ok(())
}

#[test]
fn read_all_with_delays() -> io::Result<()> {
    let addr_bit_width = 4;
    let num_elements = 1 << addr_bit_width;
    let data = (0..num_elements).collect::<Vec<_>>();

    let mut client_read_addr = 0;
    let mut client_read_data = Vec::new();

    let mut system_read_addr = None;

    let trace = build_trace("ReadCache__read_all_with_delays")?;

    let mut m = ReadCacheDelayedReturnPath::new(trace)?;
    let mut time_stamp = 0;

    m.reset();

    loop {
        m.prop();
        m.update_trace(time_stamp)?;

        if m.client_bus_read_data_valid {
            client_read_data.push(m.client_bus_read_data);
            if client_read_data.len() == data.len() {
                assert_eq!(client_read_data, data);
                break;
            }
        }

        if let Some(addr) = system_read_addr {
            m.system_bus_read_data = data[addr as usize];
            m.system_bus_read_data_valid = true;
        } else {
            m.system_bus_read_data_valid = false;
        }

        if client_read_addr < data.len() {
            m.client_bus_enable = true;
            m.client_bus_addr = client_read_addr as _;
        } else {
            m.client_bus_enable = false;
        }

        m.system_bus_ready = true;

        m.prop();
        m.update_trace(time_stamp)?;

        if m.client_bus_enable && m.client_bus_ready {
            client_read_addr += 1;
        }

        system_read_addr = if m.system_bus_enable {
            Some(m.system_bus_addr)
        } else {
            None
        };

        m.prop();
        m.update_trace(time_stamp)?;

        m.posedge_clk();
        time_stamp += 1;
    }

    assert_eq!(time_stamp, 164);

    Ok(())
}

#[test]
fn read_all_multiple_times_with_delays() -> io::Result<()> {
    let addr_bit_width = 4;
    let num_elements = 1 << addr_bit_width;
    let data = (0..num_elements).collect::<Vec<_>>();
    let repeat_times = 4;
    let expanded_data = data.clone().into_iter().flat_map(|x| vec![x; repeat_times]).collect::<Vec<_>>();

    let mut client_read_addr = 0;
    let mut client_read_repeat_count = 0;
    let mut client_read_data = Vec::new();

    let mut system_read_addr = None;

    let trace = build_trace("ReadCache__read_all_multiple_times_with_delays")?;

    let mut m = ReadCacheDelayedReturnPath::new(trace)?;
    let mut time_stamp = 0;

    m.reset();

    loop {
        m.prop();
        m.update_trace(time_stamp)?;

        if m.client_bus_read_data_valid {
            client_read_data.push(m.client_bus_read_data);
            if client_read_data.len() == expanded_data.len() {
                assert_eq!(client_read_data, expanded_data);
                break;
            }
        }

        if let Some(addr) = system_read_addr {
            m.system_bus_read_data = data[addr as usize];
            m.system_bus_read_data_valid = true;
        } else {
            m.system_bus_read_data_valid = false;
        }

        if client_read_addr < data.len() {
            m.client_bus_enable = true;
            m.client_bus_addr = client_read_addr as _;
        } else {
            m.client_bus_enable = false;
        }

        m.system_bus_ready = true;

        m.prop();
        m.update_trace(time_stamp)?;

        if m.client_bus_enable && m.client_bus_ready {
            client_read_repeat_count += 1;
            if client_read_repeat_count == repeat_times {
                client_read_addr += 1;
                client_read_repeat_count = 0;
            }
        }

        system_read_addr = if m.system_bus_enable {
            Some(m.system_bus_addr)
        } else {
            None
        };

        m.prop();
        m.update_trace(time_stamp)?;

        m.posedge_clk();
        time_stamp += 1;
    }

    assert_eq!(time_stamp, 212);

    Ok(())
}
