use crate::fifo;
use crate::peek_buffer;

use kaze::*;

pub fn generate<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, addr_bit_width: u32, data_bit_width: u32, fifo_depth_bits: u32) -> &Module<'a> {
    let mod_name = mod_name.into();

    let replica_select_bit_width = 1; // TODO: Derive from number of replica ports (is this a good idea? Seems like the memory map will change as we add replicas...)
    let replica_addr_bit_width = addr_bit_width - replica_select_bit_width; // TODO: Bounds checks

    {
        let m = c.module(format!("{}IssueArbiter", mod_name));

        let primary0_bus_enable = m.input("primary0_bus_enable", 1);
        let primary0_bus_addr = m.input("primary0_bus_addr", addr_bit_width);

        let primary1_bus_enable = m.input("primary1_bus_enable", 1);
        let primary1_bus_addr = m.input("primary1_bus_addr", addr_bit_width);

        let issue_bus_ready = m.input("issue_bus_ready", 1);

        let (bus_enable, bus_primary, bus_addr, primary1_bus_ready) = if_(primary0_bus_enable, {
            (m.high(), m.low(), primary0_bus_addr, m.low())
        }).else_({
            (primary1_bus_enable, m.high(), primary1_bus_addr, issue_bus_ready)
        });

        m.output("issue_bus_enable", bus_enable);
        m.output("issue_bus_primary", bus_primary);
        m.output("issue_bus_addr", bus_addr);

        m.output("primary0_bus_ready", issue_bus_ready);
        m.output("primary1_bus_ready", primary1_bus_ready);
    }

    {
        let m = c.module(format!("{}Issue", mod_name));

        let issue_arb_bus_enable = m.input("issue_arb_bus_enable", 1);
        let issue_arb_bus_primary = m.input("issue_arb_bus_primary", 1);
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
        let primary_fifo_read_data = m.input("primary_fifo_read_data", 1);
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
        m.output("primary0_bus_read_data", replica_data);
        m.output("primary0_bus_read_data_valid", fifo_read_data_valid.value & !primary_fifo_read_data);
        m.output("primary1_bus_read_data", replica_data);
        m.output("primary1_bus_read_data_valid", fifo_read_data_valid.value & primary_fifo_read_data);
    }

    let m = c.module(mod_name.clone());

    let issue_arbiter = m.instance("issue_arbiter", &format!("{}IssueArbiter", mod_name));
    m.output("primary0_bus_ready", issue_arbiter.output("primary0_bus_ready"));
    issue_arbiter.drive_input("primary0_bus_enable", m.input("primary0_bus_enable", 1));
    issue_arbiter.drive_input("primary0_bus_addr", m.input("primary0_bus_addr", addr_bit_width));
    m.output("primary1_bus_ready", issue_arbiter.output("primary1_bus_ready"));
    issue_arbiter.drive_input("primary1_bus_enable", m.input("primary1_bus_enable", 1));
    issue_arbiter.drive_input("primary1_bus_addr", m.input("primary1_bus_addr", addr_bit_width));

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

    fifo::generate(&c, &format!("{}PrimaryFifo", mod_name), fifo_depth_bits, 1);
    let primary_fifo = m.instance("primary_fifo", &format!("{}PrimaryFifo", mod_name));
    issue.drive_input("primary_fifo_full", primary_fifo.output("full"));
    primary_fifo.drive_input("write_enable", issue.output("primary_fifo_write_enable"));
    primary_fifo.drive_input("write_data", issue.output("primary_fifo_write_data"));

    fifo::generate(&c, &format!("{}ReplicaFifo", mod_name), fifo_depth_bits, 1);
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

    let return_arbiter = m.instance("return_arbiter", "BusterReturnArbiter");
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
    m.output("primary0_bus_read_data", return_arbiter.output("primary0_bus_read_data"));
    m.output("primary0_bus_read_data_valid", return_arbiter.output("primary0_bus_read_data_valid"));
    m.output("primary1_bus_read_data", return_arbiter.output("primary1_bus_read_data"));
    m.output("primary1_bus_read_data_valid", return_arbiter.output("primary1_bus_read_data_valid"));

    m
}
