use crate::helpers::*;

use kaze::*;

// We're going to drive some mems' read ports' enable signals with logic that includes those read ports'
//  registered output values, which is not possible with the current kaze Mem API. To get around this, we'll
//  create a `wire` construct (similar to a Verilog `wire`) which will allow us to use these output values
//  symbolically before binding their actual values.
// This still results in a valid signal graph because memory elements behave as registers and thus don't
//  form combinational loops.
fn generate_wire<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, bit_width: u32) -> &Module<'a> {
    let m = c.module(mod_name);

    m.output("o", m.input("i", bit_width));

    m
}

pub fn generate<'a, S: Into<String>>(
    c: &'a Context<'a>,
    mod_name: S,
    data_bit_width: u32,
    addr_bit_width: u32,
    cache_addr_bit_width: u32) -> &Module<'a> {

    // TODO: Ensure cache_addr_bit_width is less than addr_bit_width

    // TODO: Signal to enter invalidate state, and make sure we're not accepting/returning reads when this is asserted somehow!!

    let mod_name = mod_name.into();

    let m = c.module(&mod_name);

    let tag_bit_width = addr_bit_width - cache_addr_bit_width;

    let valid_mem = m.mem("valid", cache_addr_bit_width, 1);
    let tag_mem = m.mem("tag", cache_addr_bit_width, tag_bit_width);
    let data_mem = m.mem("data", cache_addr_bit_width, data_bit_width);

    let valid_mem_read_port_value_wire_mod_name = format!("{}ValidMemReadPortWire", mod_name);
    generate_wire(c, &valid_mem_read_port_value_wire_mod_name, 1);
    let valid_mem_read_port_value_wire = m.instance("valid_mem_read_port_value_wire", &valid_mem_read_port_value_wire_mod_name);
    let valid_mem_read_port_value = valid_mem_read_port_value_wire.output("o");
    let tag_mem_read_port_value_wire_mod_name = format!("{}TagMemReadPortWire", mod_name);
    generate_wire(c, &tag_mem_read_port_value_wire_mod_name, tag_bit_width);
    let tag_mem_read_port_value_wire = m.instance("tag_mem_read_port_value_wire", &tag_mem_read_port_value_wire_mod_name);
    let tag_mem_read_port_value = tag_mem_read_port_value_wire.output("o");

    let state_bit_width = 2;
    let state_invalidate = 0u32;
    let state_active = 1u32;
    let state_miss_return = 2u32;
    let state = m.reg("state", state_bit_width);
    state.default_value(state_invalidate);

    let invalidate_addr = m.reg("invalidate_addr", cache_addr_bit_width);
    invalidate_addr.default_value(0u32);
    // TODO: Reset value when entering invalidate state
    invalidate_addr.drive_next(invalidate_addr.value + m.lit(1u32, cache_addr_bit_width));

    let primary_bus_enable = m.input("primary_bus_enable", 1);
    let primary_bus_addr = m.input("primary_bus_addr", addr_bit_width);
    let cache_addr = primary_bus_addr.bits(cache_addr_bit_width - 1, 0);

    let issue_buffer_occupied = m.reg("issue_buffer_occupied", 1);
    issue_buffer_occupied.default_value(false);

    let issue_buffer_addr = m.reg("issue_buffer_addr", addr_bit_width);
    let issue_buffer_tag = issue_buffer_addr.value.bits(addr_bit_width - 1, cache_addr_bit_width);
    let issue_buffer_cache_addr = issue_buffer_addr.value.bits(cache_addr_bit_width - 1, 0);

    let replica_bus_ready = m.input("replica_bus_ready", 1);
    let replica_bus_read_data = m.input("replica_bus_read_data", data_bit_width);
    let replica_bus_read_data_valid = m.input("replica_bus_read_data_valid", 1);

    // A mem read that occurs simultaneously with a write to the same location will return the *previous* value
    //  at that location, *not* the new one from the write.
    // This is problematic for the special case where we're currently receiving data from the replica (and
    //  returning it to the primary) *and* the primary is issuing a read from the same location.
    // Since we're writing to the internal mems at the same location that the request will read from this cycle,
    //  the read will return stale data!
    // To work around this, we introduce a bypass mechanism which detects this specific case (exactly as described
    //  above) and overrides *both* hit detection and returned data on the following cycle. This is sufficient for
    //  all cases since the cache memory will be up-to-date on the cycle after the bypass cycle again.
    // Note that if we ignored this case, the cache would still return correct data, but only after erroneously
    //  detecting a miss and issuing a redundant read to the replica and waiting for it to return again - so at
    //  a system level, this fixes a performance bug, not a logical one... though, for a cache, this is probably
    //  not a useful distinction!
    let internal_mem_bypass =
        reg_next(
            "internal_mem_bypass",
            replica_bus_read_data_valid & primary_bus_enable & primary_bus_addr.eq(issue_buffer_addr.value),
            m);

    let issue_buffer_valid = (valid_mem_read_port_value & tag_mem_read_port_value.eq(issue_buffer_tag)) | internal_mem_bypass;

    let hit = issue_buffer_occupied.value & issue_buffer_valid;
    let miss = issue_buffer_occupied.value & !issue_buffer_valid;

    // TODO: Simplify?
    let can_accept_issue =
        (state.value.eq(m.lit(state_active, state_bit_width)) & (!issue_buffer_occupied.value | hit)) |
        (state.value.eq(m.lit(state_miss_return, state_bit_width)) & replica_bus_read_data_valid);

    m.output("primary_bus_ready", can_accept_issue);

    let accept_issue = can_accept_issue & primary_bus_enable;

    valid_mem_read_port_value_wire.drive_input("i", valid_mem.read_port(cache_addr, accept_issue));
    tag_mem_read_port_value_wire.drive_input("i", tag_mem.read_port(cache_addr, accept_issue));

    issue_buffer_occupied.drive_next(if_(replica_bus_read_data_valid | !miss, {
        accept_issue
    }).else_({
        issue_buffer_occupied.value
    }));

    issue_buffer_addr.drive_next(if_(accept_issue, {
        primary_bus_addr
    }).else_({
        issue_buffer_addr.value
    }));

    m.output("replica_bus_enable", state.value.eq(m.lit(state_active, state_bit_width)) & miss);
    m.output("replica_bus_addr", issue_buffer_addr.value);
    m.output("primary_bus_read_data", if_(replica_bus_read_data_valid, {
        replica_bus_read_data
    }).else_if(internal_mem_bypass, {
        reg_next("internal_mem_bypass_data", replica_bus_read_data, m)
    }).else_({
        data_mem.read_port(cache_addr, accept_issue)
    }));
    m.output("primary_bus_read_data_valid", replica_bus_read_data_valid | hit);

    state.drive_next(if_(state.value.eq(m.lit(state_invalidate, state_bit_width)), {
        if_(invalidate_addr.value.eq(m.lit((1u32 << cache_addr_bit_width) - 1, cache_addr_bit_width)), {
            m.lit(state_active, state_bit_width)
        }).else_({
            state.value
        })
    }).else_if(state.value.eq(m.lit(state_active, state_bit_width)), {
        if_(miss & replica_bus_ready, {
            m.lit(state_miss_return, state_bit_width)
        }).else_({
            state.value
        })
    }).else_({
        // state_miss_return
        if_(replica_bus_read_data_valid, {
            m.lit(state_active, state_bit_width)
        }).else_({
            state.value
        })
    }));

    valid_mem.write_port(
        if_(replica_bus_read_data_valid, {
            issue_buffer_cache_addr
        }).else_({
            invalidate_addr.value
        }),
        replica_bus_read_data_valid,
        replica_bus_read_data_valid | state.value.eq(m.lit(state_invalidate, state_bit_width)));
    tag_mem.write_port(
        issue_buffer_cache_addr,
        issue_buffer_tag,
        replica_bus_read_data_valid);
    data_mem.write_port(
        issue_buffer_cache_addr,
        replica_bus_read_data,
        replica_bus_read_data_valid);

    m
}
