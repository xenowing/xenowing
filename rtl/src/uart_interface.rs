use crate::fifo;

use kaze::*;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("UartInterface");

    fifo::generate(c, "UartInterfaceRxFifo", 8, 8);
    let rx_fifo = m.instance("rx_fifo", "UartInterfaceRxFifo");
    rx_fifo.drive_input("write_enable", m.input("rx_data_valid", 1));
    rx_fifo.drive_input("write_data", m.input("rx_data", 8));

    let tx_ready = m.input("tx_ready", 1);

    let bus_enable = m.input("bus_enable", 1);
    let bus_addr = m.input("bus_addr", 20);
    let bus_write = m.input("bus_write", 1);
    let bus_write_data = m.input("bus_write_data", 128);
    let _bus_write_byte_enable = m.input("bus_write_byte_enable", 16);
    m.output("bus_ready", m.high());

    rx_fifo.drive_input("read_enable", bus_enable & !bus_write & bus_addr.bits(1, 0).eq(m.lit(3u32, 2)));

    let bus_read_return_addr = bus_addr.reg_next("bus_read_return_addr");
    m.output("bus_read_data", if_(bus_read_return_addr.bits(1, 0).eq(m.lit(0u32, 2)), {
        m.lit(0u32, 127).concat(tx_ready)
    }).else_if(bus_read_return_addr.bits(1, 0).eq(m.lit(1u32, 2)), {
        m.lit(0u32, 128)
    }).else_if(bus_read_return_addr.bits(1, 0).eq(m.lit(2u32, 2)), {
        m.lit(0u32, 127).concat(!rx_fifo.output("empty"))
    }).else_({
        m.lit(0u32, 120).concat(rx_fifo.output("read_data"))
    }));
    m.output("bus_read_data_valid", (bus_enable & !bus_write).reg_next_with_default("bus_read_data_valid", false));

    m.output("tx_data", bus_write_data.bits(7, 0));
    m.output("tx_enable", bus_enable & bus_write);

    m
}
