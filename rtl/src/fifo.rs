use kaze::*;

pub fn generate<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, num_depth_bits: u32, num_element_bits: u32) -> &'a Module<'a> {
    let m = c.module(mod_name);

    let mem = m.mem("mem", num_depth_bits, num_element_bits);

    let count_bits = num_depth_bits + 1;
    let count = m.reg("count", count_bits);
    count.default_value(0u32);

    let next_count = count.value;

    // Writes
    let full = count.value.bit(count_bits - 1);
    m.output("full", full);

    let write_enable = m.input("write_enable", 1) & !full;

    let mem_write_addr = m.reg("mem_write_addr", num_depth_bits);
    mem_write_addr.default_value(0u32);

    mem.write_port(mem_write_addr.value, m.input("write_data", num_element_bits), write_enable);

    let (next_count, next_mem_write_addr) = if_(write_enable, {
        (next_count + m.lit(1u32, count_bits), mem_write_addr.value + m.lit(1u32, num_depth_bits))
    }).else_({
        (next_count, mem_write_addr.value)
    });

    mem_write_addr.drive_next(next_mem_write_addr);

    // Reads
    let empty = if_(count.value.eq(m.lit(0u32, count_bits)), {
        !write_enable
    }).else_({
        m.low()
    });
    m.output("empty", empty);

    let read_enable = m.input("read_enable", 1) & !empty;

    let mem_read_addr = m.reg("mem_read_addr", num_depth_bits);
    mem_read_addr.default_value(0u32);

    m.output("read_data", mem.read_port(mem_read_addr.value, read_enable));

    let (next_count, next_mem_read_addr) = if_(read_enable, {
        (next_count - m.lit(1u32, count_bits), mem_read_addr.value + m.lit(1u32, num_depth_bits))
    }).else_({
        (next_count, mem_read_addr.value)
    });

    mem_read_addr.drive_next(next_mem_read_addr);

    count.drive_next(next_count);

    m.output("debug_count", count.value);

    m
}
