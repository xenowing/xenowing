use kaze::*;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("MarvInterconnectBridge");

    let marv_data_bit_width = 32;
    let marv_addr_bit_width = 30;

    let marv_bus_enable = m.input("marv_bus_enable", 1);
    let marv_bus_addr = m.input("marv_bus_addr", marv_addr_bit_width);
    let marv_bus_write = m.input("marv_bus_write", 1);
    let marv_bus_write_data = m.input("marv_bus_write_data", marv_data_bit_width);
    let marv_bus_write_byte_enable = m.input("marv_bus_write_byte_enable", marv_data_bit_width / 8);

    m.output("interconnect_bus_enable", marv_bus_enable);
    m.output("interconnect_bus_addr", marv_bus_addr.bits(marv_addr_bit_width - 1, 2));
    m.output("interconnect_bus_write", marv_bus_write);

    let marv_issue_word_select = marv_bus_addr.bits(1, 0);
    let (interconnect_bus_write_data, interconnect_bus_write_byte_enable) = if_(marv_issue_word_select.eq(m.lit(0b00u32, 2)), {
        (m.lit(0u32, 96).concat(marv_bus_write_data), m.lit(0u32, 12).concat(marv_bus_write_byte_enable))
    }).else_if(marv_issue_word_select.eq(m.lit(0b01u32, 2)), {
        (m.lit(0u32, 64).concat(marv_bus_write_data).concat(m.lit(0u32, 32)), m.lit(0u32, 8).concat(marv_bus_write_byte_enable).concat(m.lit(0u32, 4)))
    }).else_if(marv_issue_word_select.eq(m.lit(0b10u32, 2)), {
        (m.lit(0u32, 32).concat(marv_bus_write_data).concat(m.lit(0u32, 64)), m.lit(0u32, 4).concat(marv_bus_write_byte_enable).concat(m.lit(0u32, 8)))
    }).else_({
        (marv_bus_write_data.concat(m.lit(0u32, 96)), marv_bus_write_byte_enable.concat(m.lit(0u32, 12)))
    });
    m.output("interconnect_bus_write_data", interconnect_bus_write_data);
    m.output("interconnect_bus_write_byte_enable", interconnect_bus_write_byte_enable);

    let interconnect_bus_read_data = m.input("interconnect_bus_read_data", 128);

    let read_data_word_select = m.reg("read_data_word_select", 2);
    read_data_word_select.drive_next(if_(marv_bus_enable & !marv_bus_write, {
        marv_bus_addr.bits(1, 0)
    }).else_({
        read_data_word_select.value
    }));

    m.output("marv_bus_ready", m.input("interconnect_bus_ready", 1));
    m.output("marv_bus_read_data", if_(read_data_word_select.value.eq(m.lit(0b00u32, 2)), {
        interconnect_bus_read_data.bits(31, 0)
    }).else_if(read_data_word_select.value.eq(m.lit(0b01u32, 2)), {
        interconnect_bus_read_data.bits(63, 32)
    }).else_if(read_data_word_select.value.eq(m.lit(0b10u32, 2)), {
        interconnect_bus_read_data.bits(95, 64)
    }).else_({
        interconnect_bus_read_data.bits(127, 96)
    }));
    m.output("marv_bus_read_data_valid", m.input("interconnect_bus_read_data_valid", 1));

    m
}
