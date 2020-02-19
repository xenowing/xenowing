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
            (m.lit(true, 1), m.lit(false, 1), master0_bus_addr, m.lit(false, 1))
        }).else_({
            (master1_bus_enable, m.lit(true, 1), master1_bus_addr, issue_bus_ready)
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
        let master_fifo_write_ready = m.input("master_fifo_write_ready", 1);

        m.output("issue_arb_bus_ready", master_fifo_write_ready & slave_bus_ready);

        m.output("slave_bus_enable", issue_arb_bus_enable & master_fifo_write_ready);
        m.output("slave_bus_addr", issue_arb_bus_addr);

        m.output("master_fifo_write_enable", issue_arb_bus_enable & master_fifo_write_ready & slave_bus_ready);
        m.output("master_fifo_write_data", issue_arb_bus_master);

        sim::generate(m, io::stdout())?;
    }

    {
        let m = c.module("BusterMasterRetBuf");

        let master_fifo_read_ready = m.input("master_fifo_read_ready", 1);

        sim::generate(m, io::stdout())?;
    }

    {
        let m = c.module("BusterSlaveRetBuf");

        let ret_arb_buf_read_enable = m.input("ret_arb_buf_read_enable", 1);

        let slave_read_data = m.input("slave_read_data", data_bit_width);
        let slave_read_data_valid = m.input("slave_read_data_valid", 1);

        let data = m.reg("data", data_bit_width);
        let ready = m.reg("ready", 1);
        ready.default_value(false);

        let (next_data, next_ready) = if_(slave_read_data_valid, {
            (slave_read_data, m.lit(true, 1))
        }).else_({
            (data.value, if_(ret_arb_buf_read_enable, {
                m.lit(false, 1)
            }).else_({
                ready.value
            }))
        });
        data.drive_next(next_data);
        ready.drive_next(next_ready);

        m.output("data", data.value);
        m.output("ready", ready.value);

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

    let master_fifo_depth_bits = 4; // TODO: Adjust for max slave latency
    fifo::generate(&c, "BusterMasterFifo", master_fifo_depth_bits, 1);
    let master_fifo = m.instance("master_fifo", "BusterMasterFifo");
    issue.drive_input("master_fifo_write_ready", !master_fifo.output("full"));
    master_fifo.drive_input("write_enable", issue.output("master_fifo_write_enable"));
    master_fifo.drive_input("write_data", issue.output("master_fifo_write_data"));
    master_fifo.drive_input("read_enable", m.lit(false, 1)); // TODO

    let slave_ret_buf = m.instance("slave_ret_buf", "BusterSlaveRetBuf");
    slave_ret_buf.drive_input("ret_arb_buf_read_enable", m.lit(false, 1)); // TODO
    slave_ret_buf.drive_input("slave_read_data", m.input("slave_read_data", data_bit_width));
    slave_ret_buf.drive_input("slave_read_data_valid", m.input("slave_read_data_valid", 1));

    sim::generate(m, io::stdout())?;

    Ok(())
}
