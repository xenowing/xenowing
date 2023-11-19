use crate::buster::*;
use crate::word_mem::*;

use kaze::*;

pub struct Bootloader<'a> {
    pub m: &'a Module<'a>,
    pub client_port: ReplicaPort<'a>,
}

impl<'a> Bootloader<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> Bootloader<'a> {
        let m = p.module(instance_name, "Bootloader");

        // TODO: Make this smaller :)
        const CONTENTS_SIZE_BITS: u32 = 14;
        const CONTENTS_SIZE: u32 = 1 << CONTENTS_SIZE_BITS;
        let contents = {
            let mut ret = include_bytes!("../../sw/bootloader/target/bootloader.bin")
                .iter()
                .cloned()
                .collect::<Vec<u8>>();
            if ret.len() as u32 > CONTENTS_SIZE {
                panic!("Bootloader cannot be larger than {} bytes", CONTENTS_SIZE);
            }
            // Zero-pad ROM to fill whole size
            while (ret.len() as u32) < CONTENTS_SIZE {
                ret.push(0);
            }
            ret
        };

        let mem = WordMem::new(m, "mem", CONTENTS_SIZE_BITS - 2, 8, 4);
        mem.initial_contents(&contents);

        let bus_enable = m.input("bus_enable", 1);
        let bus_addr = m.input("bus_addr", 22);
        let bus_write = m.input("bus_write", 1);
        let bus_write_data = m.input("bus_write_data", 32);
        let bus_write_byte_enable = m.input("bus_write_byte_enable", 32 / 8);
        mem.write_port(
            bus_addr.bits(CONTENTS_SIZE_BITS - 3, 0),
            bus_write_data,
            bus_enable & bus_write,
            bus_write_byte_enable,
        );
        let bus_ready = m.output("bus_ready", m.high());
        let read_enable = bus_enable & !bus_write;
        let bus_read_data = m.output(
            "bus_read_data",
            mem.read_port(bus_addr.bits(CONTENTS_SIZE_BITS - 3, 0), read_enable),
        );
        let bus_read_data_valid = m.output(
            "bus_read_data_valid",
            read_enable.reg_next_with_default("bus_read_data_valid", false),
        );

        Bootloader {
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
