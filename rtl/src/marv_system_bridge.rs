use crate::buster::*;

use kaze::*;

pub struct MarvSystemBridge<'a> {
    pub m: &'a Module<'a>,
    pub marv_port: ReplicaPort<'a>,
    pub system_port: PrimaryPort<'a>,
}

impl<'a> MarvSystemBridge<'a> {
    pub fn new(
        instance_name: impl Into<String>,
        p: &'a impl ModuleParent<'a>,
    ) -> MarvSystemBridge<'a> {
        let m = p.module(instance_name, "MarvSystemBridge");

        let marv_data_bit_width = 32;
        let marv_addr_bit_width = 30;

        let marv_bus_enable = m.input("marv_bus_enable", 1);
        let marv_bus_addr = m.input("marv_bus_addr", marv_addr_bit_width);
        let marv_bus_write = m.input("marv_bus_write", 1);
        let marv_bus_write_data = m.input("marv_bus_write_data", marv_data_bit_width);
        let marv_bus_write_byte_enable =
            m.input("marv_bus_write_byte_enable", marv_data_bit_width / 8);

        let system_bus_enable = m.output("system_bus_enable", marv_bus_enable);
        let system_bus_addr = m.output(
            "system_bus_addr",
            marv_bus_addr.bits(marv_addr_bit_width - 1, 2),
        );
        let system_bus_write = m.output("system_bus_write", marv_bus_write);
        let system_bus_write_data = marv_bus_write_data.repeat(4);

        let marv_issue_word_select = marv_bus_addr.bits(1, 0);
        let system_bus_write_byte_enable = if_(marv_issue_word_select.eq(m.lit(0b00u32, 2)), {
            m.lit(0u32, 12).concat(marv_bus_write_byte_enable)
        })
        .else_if(marv_issue_word_select.eq(m.lit(0b01u32, 2)), {
            m.lit(0u32, 8)
                .concat(marv_bus_write_byte_enable)
                .concat(m.lit(0u32, 4))
        })
        .else_if(marv_issue_word_select.eq(m.lit(0b10u32, 2)), {
            m.lit(0u32, 4)
                .concat(marv_bus_write_byte_enable)
                .concat(m.lit(0u32, 8))
        })
        .else_(marv_bus_write_byte_enable.concat(m.lit(0u32, 12)));

        let system_bus_read_data = m.input("system_bus_read_data", 128);

        let read_data_word_select = m.reg("read_data_word_select", 2);
        read_data_word_select.drive_next(
            if_(marv_bus_enable & !marv_bus_write, {
                marv_bus_addr.bits(1, 0)
            })
            .else_(read_data_word_select),
        );

        let system_bus_ready = m.input("system_bus_ready", 1);
        let marv_bus_ready = m.output("marv_bus_ready", system_bus_ready);
        let marv_bus_read_data = m.output(
            "marv_bus_read_data",
            if_(read_data_word_select.eq(m.lit(0b00u32, 2)), {
                system_bus_read_data.bits(31, 0)
            })
            .else_if(read_data_word_select.eq(m.lit(0b01u32, 2)), {
                system_bus_read_data.bits(63, 32)
            })
            .else_if(read_data_word_select.eq(m.lit(0b10u32, 2)), {
                system_bus_read_data.bits(95, 64)
            })
            .else_(system_bus_read_data.bits(127, 96)),
        );
        let system_bus_read_data_valid = m.input("system_bus_read_data_valid", 1);
        let marv_bus_read_data_valid =
            m.output("marv_bus_read_data_valid", system_bus_read_data_valid);

        MarvSystemBridge {
            m,
            marv_port: ReplicaPort {
                bus_enable: marv_bus_enable,
                bus_addr: marv_bus_addr,
                bus_write: marv_bus_write,
                bus_write_data: marv_bus_write_data,
                bus_write_byte_enable: marv_bus_write_byte_enable,
                bus_ready: marv_bus_ready,
                bus_read_data: marv_bus_read_data,
                bus_read_data_valid: marv_bus_read_data_valid,
            },
            system_port: PrimaryPort {
                bus_enable: system_bus_enable,
                bus_addr: system_bus_addr,
                bus_write: system_bus_write,
                bus_write_data: m.output("system_bus_write_data", system_bus_write_data),
                bus_write_byte_enable: m
                    .output("system_bus_write_byte_enable", system_bus_write_byte_enable),
                bus_ready: system_bus_ready,
                bus_read_data: system_bus_read_data,
                bus_read_data_valid: system_bus_read_data_valid,
            },
        }
    }
}
