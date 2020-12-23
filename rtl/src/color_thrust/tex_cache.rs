use crate::buster;
use crate::read_cache;
use super::*;

use kaze::*;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("TexCache");

    let invalidate = m.input("invalidate", 1);

    let in_valid = m.input("in_valid", 1);

    let issue_buffer_occupied = m.reg("issue_buffer_occupied", 1);
    issue_buffer_occupied.default_value(false);

    buster::generate(c, "BlockCacheCrossbar", 4, 1, TEX_WORD_ADDR_BITS, 0, 128, 5);
    let block_cache_crossbar = m.instance("block_cache_crossbar", "BlockCacheCrossbar");
    block_cache_crossbar.drive_input("replica0_bus_ready", m.input("replica_bus_ready", 1));
    m.output("replica_bus_enable", block_cache_crossbar.output("replica0_bus_enable"));
    m.output("replica_bus_addr", block_cache_crossbar.output("replica0_bus_addr"));
    block_cache_crossbar.drive_input("replica0_bus_read_data", m.input("replica_bus_read_data", 128));
    block_cache_crossbar.drive_input("replica0_bus_read_data_valid", m.input("replica_bus_read_data_valid", 1));

    generate_block_cache(c, "BlockCache");
    let mut acc = None;
    let block_caches = (0..4).map(|i| {
        let block_cache = m.instance(format!("block_cache{}", i), "BlockCache");
        block_cache.drive_input("invalidate", invalidate);
        let addr = m.input(format!("in_tex_buffer{}_read_addr", i), TEX_PIXEL_ADDR_BITS);
        block_cache.drive_input("in_addr", addr);
        let return_data = block_cache.output("return_data");
        m.output(format!("out_tex_buffer{}_read_value", i), return_data);

        block_cache.drive_input("replica_bus_ready", block_cache_crossbar.output(format!("primary{}_bus_ready", i)));
        block_cache_crossbar.drive_input(format!("primary{}_bus_enable", i), block_cache.output("replica_bus_enable"));
        block_cache_crossbar.drive_input(format!("primary{}_bus_addr", i), block_cache.output("replica_bus_addr"));
        block_cache.drive_input("replica_bus_read_data", block_cache_crossbar.output(format!("primary{}_bus_read_data", i)));
        block_cache.drive_input("replica_bus_read_data_valid", block_cache_crossbar.output(format!("primary{}_bus_read_data_valid", i)));
        // TODO: Consider read-only/write-only crossbar ports
        block_cache_crossbar.drive_input(format!("primary{}_bus_write", i), m.low());
        block_cache_crossbar.drive_input(format!("primary{}_bus_write_data", i), m.lit(0u32, 128));
        block_cache_crossbar.drive_input(format!("primary{}_bus_write_byte_enable", i), m.lit(0u32, 16));
        let in_ready = block_cache.output("in_ready");
        let return_data_valid = block_cache.output("return_data_valid");
        acc = Some(match acc {
            Some((acc_in_ready, acc_return_data_valid)) => (acc_in_ready & in_ready, acc_return_data_valid & return_data_valid),
            _ => (in_ready, return_data_valid)
        });

        block_cache
    }).collect::<Vec<_>>();
    let (caches_in_ready, caches_return_data_valid) = acc.unwrap();

    let out_valid = issue_buffer_occupied.value & caches_return_data_valid;
    m.output("out_valid", out_valid);

    //  Note that we exploit implementation details of `ReadCache` - namely that we
    //   know that its `primary_bus_ready` output is independent of its
    //   `replica_bus_ready` input, so regardless of arbitration or whatever else we
    //   connect between the caches on the replica side (which may introduce some
    //   interdepencies), we know that they can be in a state where all of them can
    //   accept reads simultaneously. This simplifies issue logic in this cache.
    let can_accept_issue = caches_in_ready & (!issue_buffer_occupied.value | caches_return_data_valid);
    m.output("in_ready", can_accept_issue);

    let accept_issue = can_accept_issue & in_valid;

    issue_buffer_occupied.drive_next(if_(accept_issue, {
        m.high()
    }).else_if(out_valid, {
        m.low()
    }).else_({
        issue_buffer_occupied.value
    }));

    for block_cache in block_caches {
        block_cache.drive_input("issue", accept_issue);
    }

    for (name, bit_width) in [
        ("tile_addr", TILE_PIXELS_BITS),

        ("r", 9),
        ("g", 9),
        ("b", 9),
        ("a", 9),

        ("z", 16),

        ("depth_test_result", 1),

        ("s_fract", ST_FILTER_FRACT_BITS + 1),
        ("one_minus_s_fract", ST_FILTER_FRACT_BITS + 1),
        ("t_fract", ST_FILTER_FRACT_BITS + 1),
        ("one_minus_t_fract", ST_FILTER_FRACT_BITS + 1),
    ].iter() {
        let in_ = m.input(format!("in_{}", name), *bit_width);
        let reg = m.reg(format!("{}_forward", name), *bit_width);
        reg.drive_next(if_(accept_issue, {
            in_
        }).else_({
            reg.value
        }));
        m.output(format!("out_{}", name), reg.value);
    }

    m
}

fn generate_block_cache<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S) -> &Module<'a> {
    let mod_name = mod_name.into();

    let m = c.module(&mod_name);

    let invalidate = m.input("invalidate", 1);

    // TODO: Properly expose/check these parameters!
    let read_cache_mod_name = format!("{}ReadCache", mod_name);
    read_cache::generate(c, &read_cache_mod_name, 128, TEX_WORD_ADDR_BITS, TEX_WORD_ADDR_BITS - 3);
    let read_cache = m.instance("read_cache", &read_cache_mod_name);

    read_cache.drive_input("invalidate", invalidate);

    read_cache.drive_input("replica_bus_ready", m.input("replica_bus_ready", 1));
    m.output("replica_bus_enable", read_cache.output("replica_bus_enable"));
    m.output("replica_bus_addr", read_cache.output("replica_bus_addr"));
    read_cache.drive_input("replica_bus_read_data", m.input("replica_bus_read_data", 128));
    read_cache.drive_input("replica_bus_read_data_valid", m.input("replica_bus_read_data_valid", 1));

    let issue = m.input("issue", 1);
    m.output("in_ready", read_cache.output("primary_bus_ready"));
    read_cache.drive_input("primary_bus_enable", issue);
    let in_addr = m.input("in_addr", TEX_PIXEL_ADDR_BITS);
    read_cache.drive_input("primary_bus_addr", in_addr.bits(TEX_PIXEL_ADDR_BITS - 1, 2));

    let pixel_sel = m.reg("pixel_sel", 2);
    pixel_sel.drive_next(if_(issue, {
        in_addr.bits(1, 0)
    }).else_({
        pixel_sel.value
    }));

    let read_data = read_cache.output("primary_bus_read_data");
    let read_pixel = if_(pixel_sel.value.eq(m.lit(0u32, 2)), {
        read_data.bits(31, 0)
    }).else_if(pixel_sel.value.eq(m.lit(1u32, 2)), {
        read_data.bits(63, 32)
    }).else_if(pixel_sel.value.eq(m.lit(2u32, 2)), {
        read_data.bits(95, 64)
    }).else_({
        read_data.bits(127, 96)
    });

    let read_data_valid = read_cache.output("primary_bus_read_data_valid");

    let return_buffer_occupied = m.reg("return_buffer_occupied", 1);
    return_buffer_occupied.default_value(false);
    return_buffer_occupied.drive_next(if_(issue, {
        m.low()
    }).else_if(read_data_valid, {
        m.high()
    }).else_({
        return_buffer_occupied.value
    }));

    let return_buffer_pixel = m.reg("return_buffer_pixel", 32);
    return_buffer_pixel.drive_next(if_(read_data_valid, {
        read_pixel
    }).else_({
        return_buffer_pixel.value
    }));

    m.output("return_data", if_(read_data_valid, {
        read_pixel
    }).else_({
        return_buffer_pixel.value
    }));
    m.output("return_data_valid", read_data_valid | return_buffer_occupied.value);

    m
}
