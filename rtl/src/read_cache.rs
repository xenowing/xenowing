use crate::buster::*;
use crate::wire::*;

use kaze::*;

pub struct ReadCache<'a> {
    pub m: &'a Module<'a>,
    pub invalidate: &'a Input<'a>,
    pub client_port: ReplicaPort<'a>,
    pub system_port: PrimaryPort<'a>,
}

impl<'a> ReadCache<'a> {
    pub fn new(
        instance_name: impl Into<String>,
        data_bit_width: u32,
        addr_bit_width: u32,
        cache_addr_bit_width: u32,
        p: &'a impl ModuleParent<'a>,
    ) -> ReadCache<'a> {
        // TODO: Ensure cache_addr_bit_width is less than addr_bit_width

        let m = p.module(instance_name, "ReadCache");

        let tag_bit_width = addr_bit_width - cache_addr_bit_width;

        let valid_mem = m.mem("valid", cache_addr_bit_width, 1);
        let tag_mem = m.mem("tag", cache_addr_bit_width, tag_bit_width);
        let data_mem = m.mem("data", cache_addr_bit_width, data_bit_width);

        let valid_mem_read_port_value_wire = Wire::new("valid_mem_read_port_value_wire", 1, m);
        let tag_mem_read_port_value_wire = Wire::new("tag_mem_read_port_value_wire", tag_bit_width, m);

        let state_bit_width = 2;
        let state_invalidate = 0u32;
        let state_active = 1u32;
        let state_miss_return = 2u32;
        let state = m.reg("state", state_bit_width);
        state.default_value(state_invalidate);

        let invalidate = m.input("invalidate", 1);
        let invalidate_queued = m.reg("invalidate_queued", 1);
        invalidate_queued.default_value(false);
        let will_invalidate = invalidate | invalidate_queued;

        let invalidate_addr = m.reg("invalidate_addr", cache_addr_bit_width);
        invalidate_addr.default_value(0u32);

        let replica_bus_enable = m.input("replica_bus_enable", 1);
        let replica_bus_addr = m.input("replica_bus_addr", addr_bit_width);
        let cache_addr = replica_bus_addr.bits(cache_addr_bit_width - 1, 0);

        let issue_buffer_occupied = m.reg("issue_buffer_occupied", 1);
        issue_buffer_occupied.default_value(false);

        let issue_buffer_addr = m.reg("issue_buffer_addr", addr_bit_width);
        let issue_buffer_tag = issue_buffer_addr.bits(addr_bit_width - 1, cache_addr_bit_width);
        let issue_buffer_cache_addr = issue_buffer_addr.bits(cache_addr_bit_width - 1, 0);

        let system_bus_ready = m.input("system_bus_ready", 1);
        let system_bus_read_data = m.input("system_bus_read_data", data_bit_width);
        let system_bus_read_data_valid = m.input("system_bus_read_data_valid", 1);

        // A mem read that occurs simultaneously with a write to the same location will return the *previous* value
        //  at that location, *not* the new one from the write.
        // This is problematic for the special case where we're currently receiving data from the system (and
        //  returning it to the client) *and* the client is issuing a read from the same location.
        // Since we're writing to the internal mems at the same location that the request will read from this cycle,
        //  the read will return stale data!
        // To work around this, we introduce a bypass mechanism which detects this specific case (exactly as described
        //  above) and overrides *both* hit detection and returned data on the following cycle. This is sufficient for
        //  all cases since the cache memory will be up-to-date on the cycle after the bypass cycle again.
        // Note that if we ignored this case, the cache would still return correct data, but only after erroneously
        //  detecting a miss and issuing a redundant read to the system and waiting for it to return again - so at
        //  a system level, this fixes a performance bug, not a logical one... though, for a cache, this is probably
        //  not a useful distinction!
        let internal_mem_bypass =
            (system_bus_read_data_valid & replica_bus_enable & replica_bus_addr.eq(issue_buffer_addr))
            .reg_next_with_default(
                "internal_mem_bypass",
                false);

        let issue_buffer_valid = (valid_mem_read_port_value_wire.o & tag_mem_read_port_value_wire.o.eq(issue_buffer_tag)) | internal_mem_bypass;

        let hit = issue_buffer_occupied & issue_buffer_valid;
        let miss = issue_buffer_occupied & !issue_buffer_valid;

        // TODO: Simplify?
        let can_accept_issue =
            (state.eq(m.lit(state_active, state_bit_width)) & (!issue_buffer_occupied | hit)) |
            (state.eq(m.lit(state_miss_return, state_bit_width)) & system_bus_read_data_valid);
        let can_accept_issue = can_accept_issue & !will_invalidate;

        let replica_bus_ready = m.output("replica_bus_ready", can_accept_issue);

        let accept_issue = can_accept_issue & replica_bus_enable;

        valid_mem_read_port_value_wire.i.drive(valid_mem.read_port(cache_addr, accept_issue));
        tag_mem_read_port_value_wire.i.drive(tag_mem.read_port(cache_addr, accept_issue));

        issue_buffer_occupied.drive_next(if_(system_bus_read_data_valid | !miss, {
            accept_issue
        }).else_({
            issue_buffer_occupied
        }));

        issue_buffer_addr.drive_next(if_(accept_issue, {
            replica_bus_addr
        }).else_({
            issue_buffer_addr
        }));

        let start_invalidate = will_invalidate & !issue_buffer_occupied;

        invalidate_queued.drive_next(if_(start_invalidate | state.eq(m.lit(state_invalidate, state_bit_width)), {
            m.low()
        }).else_if(invalidate, {
            m.high()
        }).else_({
            invalidate_queued
        }));

        invalidate_addr.drive_next(if_(start_invalidate, {
            m.lit(0u32, cache_addr_bit_width)
        }).else_({
            invalidate_addr + m.lit(1u32, cache_addr_bit_width)
        }));

        let system_bus_enable = m.output("system_bus_enable", state.eq(m.lit(state_active, state_bit_width)) & miss);
        let system_bus_addr = m.output("system_bus_addr", issue_buffer_addr);
        let replica_bus_read_data = m.output("replica_bus_read_data", if_(system_bus_read_data_valid, {
            system_bus_read_data.into()
        }).else_if(internal_mem_bypass, {
            system_bus_read_data.reg_next("internal_mem_bypass_data")
        }).else_({
            data_mem.read_port(cache_addr, accept_issue)
        }));
        let replica_bus_read_data_valid = m.output("replica_bus_read_data_valid", system_bus_read_data_valid | hit);

        state.drive_next(if_(start_invalidate, {
            m.lit(state_invalidate, state_bit_width)
        }).else_({
            if_(state.eq(m.lit(state_invalidate, state_bit_width)), {
                if_(invalidate_addr.eq(m.lit((1u32 << cache_addr_bit_width) - 1, cache_addr_bit_width)), {
                    m.lit(state_active, state_bit_width)
                }).else_({
                    state
                })
            }).else_if(state.eq(m.lit(state_active, state_bit_width)), {
                if_(miss & system_bus_ready, {
                    m.lit(state_miss_return, state_bit_width)
                }).else_({
                    state
                })
            }).else_({
                // state_miss_return
                if_(system_bus_read_data_valid, {
                    m.lit(state_active, state_bit_width)
                }).else_({
                    state
                })
            })
        }));

        valid_mem.write_port(
            if_(system_bus_read_data_valid, {
                issue_buffer_cache_addr
            }).else_({
                invalidate_addr
            }),
            system_bus_read_data_valid,
            system_bus_read_data_valid | state.eq(m.lit(state_invalidate, state_bit_width)));
        tag_mem.write_port(
            issue_buffer_cache_addr,
            issue_buffer_tag,
            system_bus_read_data_valid);
        data_mem.write_port(
            issue_buffer_cache_addr,
            system_bus_read_data,
            system_bus_read_data_valid);

        ReadCache {
            m,
            invalidate,
            client_port: ReplicaPort {
                bus_enable: replica_bus_enable,
                bus_addr: replica_bus_addr,
                bus_write: m.input("replica_bus_write", 1),
                bus_write_data: m.input("replica_bus_write_data", data_bit_width),
                bus_write_byte_enable: m.input("replica_bus_write_byte_enable", data_bit_width / 8),
                bus_ready: replica_bus_ready,
                bus_read_data: replica_bus_read_data,
                bus_read_data_valid: replica_bus_read_data_valid,
            },
            system_port: PrimaryPort {
                bus_enable: system_bus_enable,
                bus_addr: system_bus_addr,
                bus_write: m.output("system_bus_write", m.low()),
                bus_write_data: m.output("system_bus_write_data", m.lit(0u32, data_bit_width)),
                bus_write_byte_enable: m.output("system_bus_write_byte_enable", m.lit(0u32, data_bit_width / 8)),
                bus_ready: system_bus_ready,
                bus_read_data: system_bus_read_data,
                bus_read_data_valid: system_bus_read_data_valid,
            },
        }
    }
}
