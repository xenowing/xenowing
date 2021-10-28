use crate::buster::*;
use crate::mimas_a7::test::lfsr::*;
use crate::word_mem::*;

use kaze::*;

pub struct ByteRam<'a> {
    pub m: &'a Module<'a>,
    pub client_port: ReplicaPort<'a>,
}

impl<'a> ByteRam<'a> {
    pub fn new(instance_name: impl Into<String>, addr_bit_width: u32, port_addr_bit_width: u32, p: &'a impl ModuleParent<'a>) -> ByteRam<'a> {
        assert!(port_addr_bit_width >= addr_bit_width, "Port cannot reach all elements");

        let m = p.module(instance_name, "ByteRam");

        let bus_enable = m.input("bus_enable", 1);
        let bus_write = m.input("bus_write", 1);
        let bus_addr = m.input("bus_addr", port_addr_bit_width);
        let bus_write_data = m.input("bus_write_data", 128);
        let bus_write_byte_enable = m.input("bus_write_byte_enable", 128 / 8);
        let bus_ready = m.output("bus_ready", m.high());
        let mem = WordMem::new(m, "mem", addr_bit_width, 8, 128 / 8);
        // TODO: Consider a way to make port connection adapters to match widths instead of handling that ad-hoc for every port
        let truncated_bus_addr = bus_addr.bits(addr_bit_width - 1, 0);
        mem.write_port(truncated_bus_addr, bus_write_data, bus_enable & bus_write, bus_write_byte_enable);
        let bus_read_data = m.output("bus_read_data", mem.read_port(truncated_bus_addr, bus_enable & !bus_write));
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

pub struct CheekyByteRam<'a> {
    pub client_port: ReplicaPort<'a>,
}

impl<'a> CheekyByteRam<'a> {
    pub fn new(instance_name: impl Into<String>, addr_bit_width: u32, port_addr_bit_width: u32, p: &'a impl ModuleParent<'a>) -> CheekyByteRam<'a> {
        let m = p.module(instance_name, "CheekyByteRam");

        // Gate client port ready and enable signals with a random signal
        let lfsr = Lfsr::new("lfsr", m);
        lfsr.shift_enable.drive(m.high());
        let ready = lfsr.value.bit(0);

        let byte_ram = ByteRam::new("mem", addr_bit_width, port_addr_bit_width, m);

        let bus_enable = m.input("bus_enable", 1);
        byte_ram.client_port.bus_enable.drive(ready & bus_enable);
        let bus_write = m.input("bus_write", 1);
        byte_ram.client_port.bus_write.drive(bus_write);
        let bus_addr = m.input("bus_addr", port_addr_bit_width);
        byte_ram.client_port.bus_addr.drive(bus_addr);
        let bus_write_data = m.input("bus_write_data", 128);
        byte_ram.client_port.bus_write_data.drive(bus_write_data);
        let bus_write_byte_enable = m.input("bus_write_byte_enable", 128 / 8);
        byte_ram.client_port.bus_write_byte_enable.drive(bus_write_byte_enable);
        let bus_ready = m.output("bus_ready", ready & byte_ram.client_port.bus_ready);
        let bus_read_data = m.output("bus_read_data", byte_ram.client_port.bus_read_data);
        let bus_read_data_valid = m.output("bus_read_data_valid", byte_ram.client_port.bus_read_data_valid);

        CheekyByteRam {
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
