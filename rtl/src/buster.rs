use crate::fifo;
use crate::peek_buffer;

use kaze::*;

pub fn generate<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, num_primaries: u32, addr_bit_width: u32, replica_select_bit_width: u32, data_bit_width: u32, fifo_depth_bits: u32) -> &Module<'a> {
    let mod_name = mod_name.into();

    // TODO: num_primaries, replica_select_bit_width bounds checks
    let primary_select_bit_width = 31 - num_primaries.leading_zeros(); // TODO: Add tests for single-primary and remove FIFO/connections in that case
    let replica_addr_bit_width = addr_bit_width - replica_select_bit_width; // TODO: Bounds checks

    {
        let m = c.module(format!("{}IssueArbiter", mod_name));

        struct Primary<'a> {
            name: String,
            bus_enable: &'a Signal<'a>,
            bus_addr: &'a Signal<'a>,
        }

        let primaries: Vec<_> = (0..num_primaries).map(|i| {
            let name = format!("primary{}", i);
            Primary {
                name: name.clone(),
                bus_enable: m.input(format!("{}_bus_enable", name), 1),
                bus_addr: m.input(format!("{}_bus_addr", name), addr_bit_width),
            }
        }).collect();

        let mut bus_enable = primaries.last().unwrap().bus_enable;
        let mut bus_primary = m.lit((primaries.len() - 1) as u32, primary_select_bit_width);
        let mut bus_addr = primaries.last().unwrap().bus_addr;

        for (i, primary) in primaries.iter().enumerate().rev().skip(1) {
            let (new_bus_enable, new_bus_primary, new_bus_addr) = if_(primary.bus_enable, {
                (m.high(), m.lit(i as u32, primary_select_bit_width), primary.bus_addr)
            }).else_({
                (bus_enable, bus_primary, bus_addr)
            });
            bus_enable = new_bus_enable;
            bus_primary = new_bus_primary;
            bus_addr = new_bus_addr;
        }

        m.output("issue_bus_enable", bus_enable);
        m.output("issue_bus_primary", bus_primary);
        m.output("issue_bus_addr", bus_addr);

        let mut bus_ready = m.input("issue_bus_ready", 1);

        for primary in primaries.iter() {
            m.output(format!("{}_bus_ready", primary.name), bus_ready);
            bus_ready = bus_ready & !primary.bus_enable;
        }
    }

    {
        let m = c.module(format!("{}Issue", mod_name));

        let issue_arb_bus_enable = m.input("issue_arb_bus_enable", 1);
        let issue_arb_bus_primary = m.input("issue_arb_bus_primary", primary_select_bit_width);
        let issue_arb_bus_addr = m.input("issue_arb_bus_addr", addr_bit_width);
        let replica_select = issue_arb_bus_addr.bits(addr_bit_width - 1, replica_addr_bit_width);

        let replica0_bus_ready = m.input("replica0_bus_ready", 1);
        let replica1_bus_ready = m.input("replica1_bus_ready", 1);
        let primary_fifo_full = m.input("primary_fifo_full", 1);
        let primary_fifo_write_ready = !primary_fifo_full;
        let replica_fifo_full = m.input("replica_fifo_full", 1);
        let replica_fifo_write_ready = !replica_fifo_full;

        let replica_bus_ready = replica_select.mux(replica1_bus_ready, replica0_bus_ready);
        m.output("issue_arb_bus_ready", primary_fifo_write_ready & replica_fifo_write_ready & replica_bus_ready);

        let replica_bus_enable = issue_arb_bus_enable & primary_fifo_write_ready & replica_fifo_write_ready;
        let replica_bus_addr = issue_arb_bus_addr.bits(replica_addr_bit_width - 1, 0);
        m.output("replica0_bus_enable", replica_bus_enable & !replica_select);
        m.output("replica0_bus_addr", replica_bus_addr);
        m.output("replica1_bus_enable", replica_bus_enable & replica_select);
        m.output("replica1_bus_addr", replica_bus_addr);

        m.output("primary_fifo_write_enable", issue_arb_bus_enable & primary_fifo_write_ready & replica_fifo_write_ready & replica_bus_ready);
        m.output("primary_fifo_write_data", issue_arb_bus_primary);
        m.output("replica_fifo_write_enable", issue_arb_bus_enable & primary_fifo_write_ready & replica_fifo_write_ready & replica_bus_ready);
        m.output("replica_fifo_write_data", replica_select);
    }

    {
        let m = c.module(format!("{}ReturnArbiter", mod_name));

        let primary_fifo_empty = m.input("primary_fifo_empty", 1);
        let primary_fifo_read_ready = !primary_fifo_empty;
        let primary_fifo_read_data = m.input("primary_fifo_read_data", primary_select_bit_width);
        let replica_buffer_egress_ready = m.input("replica_buffer_egress_ready", 1);
        let replica_buffer_egress_data = m.input("replica_buffer_egress_data", replica_select_bit_width);
        let replica0_data_fifo_empty = m.input("replica0_data_fifo_empty", 1);
        let replica0_data_fifo_read_ready = !replica0_data_fifo_empty;
        let replica0_data_fifo_read_data = m.input("replica0_data_fifo_read_data", data_bit_width);
        let replica1_data_fifo_empty = m.input("replica1_data_fifo_empty", 1);
        let replica1_data_fifo_read_ready = !replica1_data_fifo_empty;
        let replica1_data_fifo_read_data = m.input("replica1_data_fifo_read_data", data_bit_width);

        let replica_data_fifo_read_ready = replica_buffer_egress_data.mux(replica1_data_fifo_read_ready, replica0_data_fifo_read_ready);

        let fifo_read_enable = primary_fifo_read_ready & replica_buffer_egress_ready & replica_data_fifo_read_ready;
        m.output("primary_fifo_read_enable", fifo_read_enable);
        m.output("replica_buffer_egress_read_enable", fifo_read_enable);
        m.output("replica0_data_fifo_read_enable", fifo_read_enable & !replica_buffer_egress_data);
        m.output("replica1_data_fifo_read_enable", fifo_read_enable & replica_buffer_egress_data);

        let fifo_read_data_valid = m.reg("fifo_read_data_valid", 1);
        fifo_read_data_valid.default_value(false);
        fifo_read_data_valid.drive_next(fifo_read_enable);
        let replica_data_fifo_select = m.reg("replica_data_fifo_select", replica_select_bit_width);
        replica_data_fifo_select.drive_next(replica_buffer_egress_data);

        let replica_data = replica_data_fifo_select.value.mux(replica1_data_fifo_read_data, replica0_data_fifo_read_data);
        for i in 0..num_primaries {
            m.output(format!("primary{}_bus_read_data", i), replica_data);
            m.output(format!("primary{}_bus_read_data_valid", i), fifo_read_data_valid.value & primary_fifo_read_data.eq(m.lit(i as u32, primary_select_bit_width)));
        }
    }

    let m = c.module(mod_name.clone());

    let issue_arbiter = m.instance("issue_arbiter", &format!("{}IssueArbiter", mod_name));
    for i in 0..num_primaries {
        m.output(&format!("primary{}_bus_ready", i), issue_arbiter.output(&format!("primary{}_bus_ready", i)));
        issue_arbiter.drive_input(&format!("primary{}_bus_enable", i), m.input(&format!("primary{}_bus_enable", i), 1));
        issue_arbiter.drive_input(&format!("primary{}_bus_addr", i), m.input(&format!("primary{}_bus_addr", i), addr_bit_width));
    }

    let issue = m.instance("issue", &format!("{}Issue", mod_name));
    issue.drive_input("issue_arb_bus_enable", issue_arbiter.output("issue_bus_enable"));
    issue.drive_input("issue_arb_bus_primary", issue_arbiter.output("issue_bus_primary"));
    issue.drive_input("issue_arb_bus_addr", issue_arbiter.output("issue_bus_addr"));
    issue.drive_input("replica0_bus_ready", m.input("replica0_bus_ready", 1));
    issue.drive_input("replica1_bus_ready", m.input("replica1_bus_ready", 1));
    issue_arbiter.drive_input("issue_bus_ready", issue.output("issue_arb_bus_ready"));
    m.output("replica0_bus_enable", issue.output("replica0_bus_enable"));
    m.output("replica0_bus_addr", issue.output("replica0_bus_addr"));
    m.output("replica1_bus_enable", issue.output("replica1_bus_enable"));
    m.output("replica1_bus_addr", issue.output("replica1_bus_addr"));

    fifo::generate(&c, &format!("{}PrimaryFifo", mod_name), fifo_depth_bits, primary_select_bit_width);
    let primary_fifo = m.instance("primary_fifo", &format!("{}PrimaryFifo", mod_name));
    issue.drive_input("primary_fifo_full", primary_fifo.output("full"));
    primary_fifo.drive_input("write_enable", issue.output("primary_fifo_write_enable"));
    primary_fifo.drive_input("write_data", issue.output("primary_fifo_write_data"));

    fifo::generate(&c, &format!("{}ReplicaFifo", mod_name), fifo_depth_bits, replica_select_bit_width);
    let replica_fifo = m.instance("replica_fifo", &format!("{}ReplicaFifo", mod_name));
    issue.drive_input("replica_fifo_full", replica_fifo.output("full"));
    replica_fifo.drive_input("write_enable", issue.output("replica_fifo_write_enable"));
    replica_fifo.drive_input("write_data", issue.output("replica_fifo_write_data"));

    peek_buffer::generate(&c, &format!("ReplicaBuffer{}", mod_name), replica_select_bit_width);
    let replica_buffer = m.instance("replica_buffer", &format!("ReplicaBuffer{}", mod_name));
    replica_buffer.drive_input("ingress_data", replica_fifo.output("read_data"));
    replica_fifo.drive_input("read_enable", replica_buffer.output("ingress_read_enable"));
    let replica_fifo_read_data_valid = m.reg("replica_fifo_read_data_valid", 1);
    replica_fifo_read_data_valid.default_value(false);
    replica_fifo_read_data_valid.drive_next(!replica_fifo.output("empty") & replica_buffer.output("ingress_read_enable"));
    replica_buffer.drive_input("ingress_data_valid", replica_fifo_read_data_valid.value);

    fifo::generate(&c, &format!("{}ReplicaDataFifo", mod_name), fifo_depth_bits, data_bit_width);
    let replica0_data_fifo = m.instance("replica0_data_fifo", &format!("{}ReplicaDataFifo", mod_name));
    replica0_data_fifo.drive_input("write_enable", m.input("replica_bus_read_data_valid", 1));
    replica0_data_fifo.drive_input("write_data", m.input("replica_bus_read_data", data_bit_width));
    let replica1_data_fifo = m.instance("replica1_data_fifo", &format!("{}ReplicaDataFifo", mod_name));
    replica1_data_fifo.drive_input("write_enable", m.input("replica_bus_read_data_valid", 1));
    replica1_data_fifo.drive_input("write_data", m.input("replica_bus_read_data", data_bit_width));

    let return_arbiter = m.instance("return_arbiter", &format!("{}ReturnArbiter", mod_name));
    return_arbiter.drive_input("primary_fifo_empty", primary_fifo.output("empty"));
    return_arbiter.drive_input("primary_fifo_read_data", primary_fifo.output("read_data"));
    return_arbiter.drive_input("replica_buffer_egress_ready", replica_buffer.output("egress_ready"));
    return_arbiter.drive_input("replica_buffer_egress_data", replica_buffer.output("egress_data"));
    primary_fifo.drive_input("read_enable", return_arbiter.output("primary_fifo_read_enable"));
    replica_buffer.drive_input("egress_read_enable", return_arbiter.output("replica_buffer_egress_read_enable"));
    replica0_data_fifo.drive_input("read_enable", return_arbiter.output("replica0_data_fifo_read_enable"));
    replica1_data_fifo.drive_input("read_enable", return_arbiter.output("replica1_data_fifo_read_enable"));
    return_arbiter.drive_input("replica0_data_fifo_empty", replica0_data_fifo.output("empty"));
    return_arbiter.drive_input("replica0_data_fifo_read_data", replica0_data_fifo.output("read_data"));
    return_arbiter.drive_input("replica1_data_fifo_empty", replica1_data_fifo.output("empty"));
    return_arbiter.drive_input("replica1_data_fifo_read_data", replica1_data_fifo.output("read_data"));
    for i in 0..num_primaries {
        m.output(format!("primary{}_bus_read_data", i), return_arbiter.output(format!("primary{}_bus_read_data", i)));
        m.output(format!("primary{}_bus_read_data_valid", i), return_arbiter.output(format!("primary{}_bus_read_data_valid", i)));
    }

    m
}
