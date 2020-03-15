use crate::helpers::*;

use kaze::*;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("UartInterface");

    let tx_ready = m.input("tx_ready", 1);

    let bus_enable = m.input("bus_enable", 1);
    let _bus_addr = m.input("bus_addr", 20);
    let bus_write = m.input("bus_write", 1);
    let bus_write_data = m.input("bus_write_data", 128);
    let _bus_write_byte_enable = m.input("bus_write_byte_enable", 16);
    m.output("bus_ready", m.high());
    m.output("bus_read_data", m.lit(0u32, 127).concat(tx_ready));
    m.output("bus_read_data_valid", reg_next("bus_read_data_valid", bus_enable & !bus_write, m));

    m.output("tx_data", bus_write_data.bits(7, 0));
    m.output("tx_enable", bus_enable & bus_write);

    m
}
