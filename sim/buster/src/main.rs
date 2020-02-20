use kaze::*;
use rtl::*;

use std::io;

fn main() -> io::Result<()> {
    let c = Context::new();

    let data_bit_width = 128;
    let addr_bit_width = 16; // TODO: This is not correct.. :)

    {
        let m = c.module("BusterIssueArbiter");

        let master0_bus_enable = m.input("master0_bus_enable", 1);
        let master0_bus_addr = m.input("master0_bus_addr", addr_bit_width);

        let master1_bus_enable = m.input("master1_bus_enable", 1);
        let master1_bus_addr = m.input("master1_bus_addr", addr_bit_width);

        let issue_bus_ready = m.input("issue_bus_ready", 1);

        let (bus_enable, bus_master, bus_addr, master1_bus_ready) = if_(master0_bus_enable, {
            (m.high(), m.low(), master0_bus_addr, m.low())
        }).else_({
            (master1_bus_enable, m.high(), master1_bus_addr, issue_bus_ready)
        });

        m.output("issue_bus_enable", bus_enable);
        m.output("issue_bus_master", bus_master);
        m.output("issue_bus_addr", bus_addr);

        m.output("master0_bus_ready", issue_bus_ready);
        m.output("master1_bus_ready", master1_bus_ready);

        sim::generate(m, io::stdout())?;
    }

    {
        let m = c.module("BusterIssue");

        let issue_arb_bus_enable = m.input("issue_arb_bus_enable", 1);
        let issue_arb_bus_master = m.input("issue_arb_bus_master", 1);
        let issue_arb_bus_addr = m.input("issue_arb_bus_addr", addr_bit_width);

        let slave_bus_ready = m.input("slave_bus_ready", 1);
        let master_fifo_full = m.input("master_fifo_full", 1);
        let master_fifo_write_ready = !master_fifo_full;

        m.output("issue_arb_bus_ready", master_fifo_write_ready & slave_bus_ready);

        m.output("slave_bus_enable", issue_arb_bus_enable & master_fifo_write_ready);
        m.output("slave_bus_addr", issue_arb_bus_addr);

        m.output("master_fifo_write_enable", issue_arb_bus_enable & master_fifo_write_ready & slave_bus_ready);
        m.output("master_fifo_write_data", issue_arb_bus_master);

        sim::generate(m, io::stdout())?;
    }

    {
        let m = c.module("BusterReturnArbiter");

        let master_fifo_empty = m.input("master_fifo_empty", 1);
        let master_fifo_read_ready = !master_fifo_empty;
        let master_fifo_read_data = m.input("master_fifo_read_data", 1);
        let data_fifo_empty = m.input("data_fifo_empty", 1);
        let data_fifo_read_ready = !data_fifo_empty;
        let data_fifo_read_data = m.input("data_fifo_read_data", data_bit_width);

        let fifo_read_enable = master_fifo_read_ready & data_fifo_read_ready;
        m.output("fifo_read_enable", fifo_read_enable);

        let fifo_read_data_valid = m.reg("fifo_read_data_valid", 1);
        fifo_read_data_valid.default_value(false);
        fifo_read_data_valid.drive_next(fifo_read_enable);

        m.output("master0_read_data", data_fifo_read_data);
        m.output("master0_read_data_valid", fifo_read_data_valid.value & !master_fifo_read_data);
        m.output("master1_read_data", data_fifo_read_data);
        m.output("master1_read_data_valid", fifo_read_data_valid.value & master_fifo_read_data);

        sim::generate(m, io::stdout())?;
    }

    let m = c.module("Buster");

    let issue_arbiter = m.instance("issue_arbiter", "BusterIssueArbiter");
    m.output("master0_bus_ready", issue_arbiter.output("master0_bus_ready"));
    issue_arbiter.drive_input("master0_bus_enable", m.input("master0_bus_enable", 1));
    issue_arbiter.drive_input("master0_bus_addr", m.input("master0_bus_addr", addr_bit_width));
    m.output("master1_bus_ready", issue_arbiter.output("master1_bus_ready"));
    issue_arbiter.drive_input("master1_bus_enable", m.input("master1_bus_enable", 1));
    issue_arbiter.drive_input("master1_bus_addr", m.input("master1_bus_addr", addr_bit_width));

    let issue = m.instance("issue", "BusterIssue");
    issue.drive_input("issue_arb_bus_enable", issue_arbiter.output("issue_bus_enable"));
    issue.drive_input("issue_arb_bus_master", issue_arbiter.output("issue_bus_master"));
    issue.drive_input("issue_arb_bus_addr", issue_arbiter.output("issue_bus_addr"));
    issue.drive_input("slave_bus_ready", m.input("slave_bus_ready", 1));
    issue_arbiter.drive_input("issue_bus_ready", issue.output("issue_arb_bus_ready"));
    m.output("slave_bus_enable", issue.output("slave_bus_enable"));
    m.output("slave_bus_addr", issue.output("slave_bus_addr"));

    let fifo_depth_bits = 4; // TODO: Adjust for max slave latency

    fifo::generate(&c, "BusterMasterFifo", fifo_depth_bits, 1);
    let master_fifo = m.instance("master_fifo", "BusterMasterFifo");
    issue.drive_input("master_fifo_full", master_fifo.output("full"));
    master_fifo.drive_input("write_enable", issue.output("master_fifo_write_enable"));
    master_fifo.drive_input("write_data", issue.output("master_fifo_write_data"));

    fifo::generate(&c, "BusterDataFifo", fifo_depth_bits, data_bit_width);
    let data_fifo = m.instance("data_fifo", "BusterDataFifo");
    data_fifo.drive_input("write_enable", m.input("slave_read_data_valid", 1));
    data_fifo.drive_input("write_data", m.input("slave_read_data", data_bit_width));

    let return_arbiter = m.instance("return_arbiter", "BusterReturnArbiter");
    return_arbiter.drive_input("master_fifo_empty", master_fifo.output("empty"));
    master_fifo.drive_input("read_enable", return_arbiter.output("fifo_read_enable"));
    return_arbiter.drive_input("master_fifo_read_data", master_fifo.output("read_data"));
    return_arbiter.drive_input("data_fifo_empty", data_fifo.output("empty"));
    data_fifo.drive_input("read_enable", return_arbiter.output("fifo_read_enable"));
    return_arbiter.drive_input("data_fifo_read_data", data_fifo.output("read_data"));
    m.output("master0_read_data", return_arbiter.output("master0_read_data"));
    m.output("master0_read_data_valid", return_arbiter.output("master0_read_data_valid"));
    m.output("master1_read_data", return_arbiter.output("master1_read_data"));
    m.output("master1_read_data_valid", return_arbiter.output("master1_read_data_valid"));

    sim::generate(m, io::stdout())?;

    Ok(())
}
