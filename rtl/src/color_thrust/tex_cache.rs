use crate::buster::*;
use crate::read_cache::*;
use super::*;

use kaze::*;

use std::collections::HashMap;

pub struct TexCache<'a> {
    pub m: &'a Module<'a>,

    pub invalidate: &'a Input<'a>,
    pub in_valid: &'a Input<'a>,
    pub in_ready: &'a Output<'a>,
    pub out_valid: &'a Output<'a>,

    pub in_tex_buffer_read_addrs: Vec<&'a Input<'a>>,
    pub out_tex_buffer_read_values: Vec<&'a Output<'a>>,

    pub system_port: PrimaryPort<'a>,

    pub forward_inputs: HashMap<String, &'a Input<'a>>,
    pub forward_outputs: HashMap<String, &'a Output<'a>>,
}

impl<'a> TexCache<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> TexCache<'a> {
        let m = p.module(instance_name, "TexCache");

        let invalidate = m.input("invalidate", 1);

        let in_valid = m.input("in_valid", 1);

        let issue_buffer_occupied = m.reg("issue_buffer_occupied", 1);
        issue_buffer_occupied.default_value(false);

        let block_cache_crossbar = Crossbar::new("block_cache_crossbar", 4, 1, SYSTEM_BUS_BITS, 0, 128, 5, m);
        let system_port = block_cache_crossbar.primary_ports[0].forward("system", m);

        let mut in_tex_buffer_read_addrs = Vec::new();
        let mut out_tex_buffer_read_values = Vec::new();
        let mut acc: Option<(&dyn Signal<'a>, &dyn Signal<'a>)> = None;
        let block_caches = (0..4).map(|i| {
            let block_cache = BlockCache::new(format!("block_cache{}", i), m);
            block_cache.invalidate.drive(invalidate);
            // Inputs address individual texels, of which there are 4 per word, so we need two more bits than the system bus width (which is in words)
            let addr = m.input(format!("in_tex_buffer{}_read_addr", i), SYSTEM_BUS_BITS + 2);
            block_cache.in_addr.drive(addr);
            let return_data = block_cache.return_data;
            let value = m.output(format!("out_tex_buffer{}_read_value", i), return_data);

            in_tex_buffer_read_addrs.push(addr);
            out_tex_buffer_read_values.push(value);

            block_cache.system_port.connect(&block_cache_crossbar.replica_ports[i]);
            let in_ready = block_cache.in_ready;
            let return_data_valid = block_cache.return_data_valid;
            acc = Some(match acc {
                Some((acc_in_ready, acc_return_data_valid)) => (acc_in_ready & in_ready, acc_return_data_valid & return_data_valid),
                _ => (in_ready, return_data_valid)
            });

            block_cache
        }).collect::<Vec<_>>();
        let (caches_in_ready, caches_return_data_valid) = acc.unwrap();

        let out_valid = issue_buffer_occupied & caches_return_data_valid;

        //  Note that we exploit implementation details of `ReadCache` - namely that we
        //   know that its `system_bus_ready` output is independent of its
        //   `client_bus_ready` input, so regardless of arbitration or whatever else we
        //   connect between the caches on the primary side (which may introduce some
        //   interdepencies), we know that they can be in a state where all of them can
        //   accept reads simultaneously. This simplifies issue logic in this cache.
        let can_accept_issue = caches_in_ready & (!issue_buffer_occupied | caches_return_data_valid);
        let in_ready = m.output("in_ready", can_accept_issue);

        let accept_issue = can_accept_issue & in_valid;

        issue_buffer_occupied.drive_next(if_(accept_issue, {
            m.high()
        }).else_if(out_valid, {
            m.low()
        }).else_({
            issue_buffer_occupied
        }));

        for block_cache in block_caches {
            block_cache.issue.drive(accept_issue);
        }

        let mut forward_inputs = HashMap::new();
        let mut forward_outputs = HashMap::new();
        for &(name, bit_width) in [
            ("tile_addr", TILE_PIXELS_BITS),

            ("r", 9),
            ("g", 9),
            ("b", 9),
            ("a", 9),

            ("z", 16),

            ("s_fract", ST_FILTER_FRACT_BITS + 1),
            ("one_minus_s_fract", ST_FILTER_FRACT_BITS + 1),
            ("t_fract", ST_FILTER_FRACT_BITS + 1),
            ("one_minus_t_fract", ST_FILTER_FRACT_BITS + 1),
        ].iter() {
            let input = m.input(format!("in_{}", name), bit_width);
            let reg = m.reg(format!("{}_forward", name), bit_width);
            reg.drive_next(if_(accept_issue, {
                input
            }).else_({
                reg
            }));
            let output = m.output(format!("out_{}", name), reg);
            forward_inputs.insert(name.into(), input);
            forward_outputs.insert(name.into(), output);
        }

        TexCache {
            m,

            invalidate,
            in_valid,
            in_ready,
            out_valid: m.output("out_valid", out_valid),

            in_tex_buffer_read_addrs,
            out_tex_buffer_read_values,

            system_port,

            forward_inputs,
            forward_outputs,
        }
    }
}

pub struct BlockCache<'a> {
    pub m: &'a Module<'a>,

    pub invalidate: &'a Input<'a>,

    pub system_port: PrimaryPort<'a>,
    pub issue: &'a Input<'a>,
    pub in_ready: &'a Output<'a>,
    pub in_addr: &'a Input<'a>,
    pub return_data: &'a Output<'a>,
    pub return_data_valid: &'a Output<'a>,
}

impl<'a> BlockCache<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> BlockCache<'a> {
        let m = p.module(instance_name, "BlockCache");

        let invalidate = m.input("invalidate", 1);

        // A block cache will (via a read cache) read whole words from the system bus, but should be addressed with two additional bits, to select the correct texel (of 4) from the returned data word.
        let read_cache = ReadCache::new("read_cache", 128, SYSTEM_BUS_BITS, 8, m);
        let system_port = read_cache.system_port.forward("system", m);

        read_cache.invalidate.drive(invalidate);

        let issue = m.input("issue", 1);
        let in_ready = m.output("in_ready", read_cache.client_port.bus_ready);
        read_cache.client_port.bus_enable.drive(issue);
        let in_addr = m.input("in_addr", SYSTEM_BUS_BITS + 2);
        read_cache.client_port.bus_addr.drive(in_addr.bits(in_addr.bit_width() - 1, 2));

        read_cache.client_port.bus_write.drive(m.low());
        read_cache.client_port.bus_write_data.drive(m.lit(0u32, 128));
        read_cache.client_port.bus_write_byte_enable.drive(m.lit(0u32, 128 / 8));

        let pixel_sel = m.reg("pixel_sel", 2);
        pixel_sel.drive_next(if_(issue, {
            in_addr.bits(1, 0)
        }).else_({
            pixel_sel
        }));

        let read_data = read_cache.client_port.bus_read_data;
        let read_pixel = if_(pixel_sel.eq(m.lit(0u32, 2)), {
            read_data.bits(31, 0)
        }).else_if(pixel_sel.eq(m.lit(1u32, 2)), {
            read_data.bits(63, 32)
        }).else_if(pixel_sel.eq(m.lit(2u32, 2)), {
            read_data.bits(95, 64)
        }).else_({
            read_data.bits(127, 96)
        });

        let read_data_valid = read_cache.client_port.bus_read_data_valid;

        let return_buffer_occupied = m.reg("return_buffer_occupied", 1);
        return_buffer_occupied.default_value(false);
        return_buffer_occupied.drive_next(if_(issue, {
            m.low()
        }).else_if(read_data_valid, {
            m.high()
        }).else_({
            return_buffer_occupied
        }));

        let return_buffer_pixel = m.reg("return_buffer_pixel", 32);
        return_buffer_pixel.drive_next(if_(read_data_valid, {
            read_pixel
        }).else_({
            return_buffer_pixel
        }));

        let return_data = m.output("return_data", if_(read_data_valid, {
            read_pixel
        }).else_({
            return_buffer_pixel
        }));
        let return_data_valid = m.output("return_data_valid", read_data_valid | return_buffer_occupied);

        BlockCache {
            m,

            invalidate,

            system_port,
            issue,
            in_ready,
            in_addr,
            return_data,
            return_data_valid,
        }
    }
}
