use kaze::*;

pub fn generate<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, data_bit_width: u32) -> &Module<'a> {
    let m = c.module(mod_name);

    let ingress_data = m.input("ingress_data", data_bit_width);
    let ingress_data_valid = m.input("ingress_data_valid", 1);

    let data_buf = m.reg("data_buf", data_bit_width);
    let data_buf_full = m.reg("data_buf_full", 1);
    data_buf_full.default_value(false);

    m.output("egress_ready", data_buf_full.value | ingress_data_valid);
    m.output("egress_data", data_buf_full.value.mux(data_buf.value, ingress_data));
    let egress_read_enable = m.input("egress_read_enable", 1);

    m.output("ingress_read_enable", egress_read_enable | (!data_buf_full.value & !ingress_data_valid));

    // TODO: Can this logic be simplified?
    let next_data_buf = data_buf.value;
    let next_data_buf_full = data_buf_full.value;

    let (next_data_buf, next_data_buf_full) = if_(ingress_data_valid & !egress_read_enable, {
        (ingress_data, m.high())
    }).else_({
        (next_data_buf, next_data_buf_full)
    });

    let next_data_buf_full = if_(egress_read_enable, {
        m.low()
    }).else_({
        next_data_buf_full
    });

    data_buf.drive_next(next_data_buf);
    data_buf_full.drive_next(next_data_buf_full);

    m
}
