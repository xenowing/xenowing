use crate::buster::*;
use crate::word_mem::*;

use kaze::*;

pub struct ByteRam<'a> {
    pub m: &'a Module<'a>,
    pub client_port: ReplicaPort<'a>,
}

impl<'a> ByteRam<'a> {
    pub fn new(instance_name: impl Into<String>, addr_bit_width: u32, p: &'a impl ModuleParent<'a>) -> ByteRam<'a> {
        let m = p.module(instance_name, "ByteRam");

        let bus_enable = m.input("bus_enable", 1);
        let bus_write = m.input("bus_write", 1);
        // TODO: Probably have to have parameters both for the input width and the memory depth, as these may differ
        let bus_addr = m.input("bus_addr", addr_bit_width);
        let bus_write_data = m.input("bus_write_data", 128);
        let bus_write_byte_enable = m.input("bus_write_byte_enable", 128 / 8);
        let bus_ready = m.output("bus_ready", m.high());
        let mem = WordMem::new(m, "mem", addr_bit_width, 8, 128 / 8);
        mem.write_port(bus_addr, bus_write_data, bus_enable & bus_write, bus_write_byte_enable);
        let bus_read_data = m.output("bus_read_data", mem.read_port(bus_addr, bus_enable & !bus_write));
        let bus_read_data_valid = m.output("bus_read_data_valid", (bus_enable & !bus_write).reg_next_with_default("bus_read_data_valid", false));

        ByteRam {
            m,
            client_port: ReplicaPort {
                bus_enable,
                bus_addr,
                bus_write,
                bus_write_data,
                bus_write_byte_enable,
                bus_ready,
                bus_read_data,
                bus_read_data_valid,
            },
        }
    }
}
