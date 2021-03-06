use kaze::*;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("LedInterface");

    let leds = m.reg("leds", 8);
    leds.default_value(0u32);

    let bus_enable = m.input("bus_enable", 1);
    let _bus_addr = m.input("bus_addr", 20);
    let bus_write = m.input("bus_write", 1);
    let bus_write_data = m.input("bus_write_data", 128);
    let _bus_write_byte_enable = m.input("bus_write_byte_enable", 16);
    m.output("bus_ready", m.high());
    m.output("bus_read_data", m.lit(0u32, 120).concat(leds.value));
    m.output("bus_read_data_valid", (bus_enable & !bus_write).reg_next_with_default("bus_read_data_valid", false));

    leds.drive_next(if_(bus_enable & bus_write, {
        bus_write_data.bits(7, 0)
    }).else_({
        leds.value
    }));

    m.output("leds", leds.value);

    m
}
