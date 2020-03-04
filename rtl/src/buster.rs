use crate::fifo;
use crate::peek_buffer;

use kaze::*;

pub fn generate<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, num_primaries: u32, num_replicas: u32, addr_bit_width: u32, replica_select_bit_width: u32, data_bit_width: u32, fifo_depth_bits: u32) -> &Module<'a> {
    if num_primaries == 0 {
        panic!("Cannot generate a buster module with zero primaries.");
    }
    if num_replicas == 0 {
        panic!("Cannot generate a buster module with zero replicas.");
    }

    let mod_name = mod_name.into();

    // TODO: num_primaries, num_replicas, replica_select_bit_width bounds checks
    let primary_select_bit_width = 31 - num_primaries.leading_zeros();
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
        let mut bus_addr = primaries.last().unwrap().bus_addr;

        for primary in primaries.iter().rev().skip(1) {
            let (new_bus_enable, new_bus_addr) = if_(primary.bus_enable, {
                (m.high(), primary.bus_addr)
            }).else_({
                (bus_enable, bus_addr)
            });
            bus_enable = new_bus_enable;
            bus_addr = new_bus_addr;
        }

        m.output("issue_bus_enable", bus_enable);
        m.output("issue_bus_addr", bus_addr);

        if num_primaries > 1 {
            let mut bus_primary = m.lit(num_primaries - 1, primary_select_bit_width);

            for (i, primary) in primaries.iter().enumerate().rev().skip(1) {
                 bus_primary = if_(primary.bus_enable, {
                    m.lit(i as u32, primary_select_bit_width)
                }).else_({
                    bus_primary
                });
            }

            m.output("issue_bus_primary", bus_primary);
        }

        let mut bus_ready = m.input("issue_bus_ready", 1);

        for primary in primaries.iter() {
            m.output(format!("{}_bus_ready", primary.name), bus_ready);
            bus_ready = bus_ready & !primary.bus_enable;
        }
    }

    {
        let m = c.module(format!("{}Issue", mod_name));

        let issue_arb_bus_enable = m.input("issue_arb_bus_enable", 1);
        let issue_arb_bus_addr = m.input("issue_arb_bus_addr", addr_bit_width);

        let primary_fifo_full = if num_primaries > 1 { m.input("primary_fifo_full", 1) } else { m.low() };
        let primary_fifo_write_ready = !primary_fifo_full;
        let replica_fifo_full = if num_replicas > 1 { m.input("replica_fifo_full", 1) } else { m.low() };
        let replica_fifo_write_ready = !replica_fifo_full;

        let replica_bus_enable = issue_arb_bus_enable & primary_fifo_write_ready & replica_fifo_write_ready;

        let (replica_select, replica_bus_ready) = if num_replicas > 1 {
            let replica_select = issue_arb_bus_addr.bits(addr_bit_width - 1, replica_addr_bit_width);
            let replica_bus_ready = (0..num_replicas).fold(m.low(), |acc, x| {
                acc | (m.input(format!("replica{}_bus_ready", x), 1) & replica_select.eq(m.lit(x, replica_select_bit_width)))
            });

            m.output("replica_fifo_write_enable", issue_arb_bus_enable & primary_fifo_write_ready & replica_fifo_write_ready & replica_bus_ready);
            m.output("replica_fifo_write_data", replica_select);

            (Some(replica_select), replica_bus_ready)
        } else {
            (None, m.input("replica0_bus_ready", 1))
        };

        m.output("issue_arb_bus_ready", primary_fifo_write_ready & replica_fifo_write_ready & replica_bus_ready);

        if num_primaries > 1 {
            let issue_arb_bus_primary = m.input("issue_arb_bus_primary", primary_select_bit_width);
            m.output("primary_fifo_write_enable", issue_arb_bus_enable & primary_fifo_write_ready & replica_fifo_write_ready & replica_bus_ready);
            m.output("primary_fifo_write_data", issue_arb_bus_primary);
        }

        let replica_bus_addr = issue_arb_bus_addr.bits(replica_addr_bit_width - 1, 0);
        for i in 0..num_replicas {
            m.output(format!("replica{}_bus_enable", i), replica_bus_enable & replica_select.map(|x| x.eq(m.lit(i, replica_select_bit_width))).unwrap_or(m.high()));
            m.output(format!("replica{}_bus_addr", i), replica_bus_addr);
        }
    }

    {
        let m = c.module(format!("{}ReturnArbiter", mod_name));

        let primary_fifo_empty = if num_primaries > 1 { m.input("primary_fifo_empty", 1) } else { m.low() };
        let primary_fifo_read_ready = !primary_fifo_empty;

        let replica_buffer_egress_ready = m.input("replica_buffer_egress_ready", 1);
        let replica_buffer_egress_data = m.input("replica_buffer_egress_data", replica_select_bit_width);

        let (replica_data_fifo_read_ready, replica_data) = if num_replicas > 1 {
            let replica_data_fifo_select = m.reg("replica_data_fifo_select", replica_select_bit_width);
            replica_data_fifo_select.drive_next(replica_buffer_egress_data);

            (0..num_replicas).rev().skip(1).fold((!m.input(format!("replica{}_data_fifo_empty", num_replicas - 1), 1), m.input(format!("replica{}_data_fifo_read_data", num_replicas - 1), data_bit_width)), |acc, x| {
                let replica_data_fifo_empty = m.input(format!("replica{}_data_fifo_empty", x), 1);
                let replica_data_fifo_read_ready = !replica_data_fifo_empty;
                let replica_data_fifo_read_data = m.input(format!("replica{}_data_fifo_read_data", x), data_bit_width);

                (
                    if_(replica_buffer_egress_data.eq(m.lit(x, replica_select_bit_width)), {
                        replica_data_fifo_read_ready
                    }).else_({
                        acc.0
                    }),
                    if_(replica_data_fifo_select.value.eq(m.lit(x, replica_select_bit_width)), {
                        replica_data_fifo_read_data
                    }).else_({
                        acc.1
                    }),
                )
            })
        } else {
            (!m.input("replica0_data_fifo_empty", 1), m.input("replica0_data_fifo_read_data", data_bit_width))
        };

        let fifo_read_enable = primary_fifo_read_ready & replica_buffer_egress_ready & replica_data_fifo_read_ready;
        m.output("primary_fifo_read_enable", fifo_read_enable);
        m.output("replica_buffer_egress_read_enable", fifo_read_enable);
        for i in 0..num_replicas {
            m.output(format!("replica{}_data_fifo_read_enable", i), fifo_read_enable & replica_buffer_egress_data.eq(m.lit(i, replica_select_bit_width)));
        }

        let fifo_read_data_valid = m.reg("fifo_read_data_valid", 1);
        fifo_read_data_valid.default_value(false);
        fifo_read_data_valid.drive_next(fifo_read_enable);

        let primary_fifo_read_data = if num_primaries > 1 {
            Some(m.input("primary_fifo_read_data", primary_select_bit_width))
        } else {
            None
        };
        for i in 0..num_primaries {
            m.output(format!("primary{}_bus_read_data", i), replica_data);
            let primary_fifo_read_data = primary_fifo_read_data.map(|x| x.eq(m.lit(i as u32, primary_select_bit_width))).unwrap_or(m.high());
            m.output(format!("primary{}_bus_read_data_valid", i), fifo_read_data_valid.value & primary_fifo_read_data);
        }
    }

    let m = c.module(mod_name.clone());

    let issue_arbiter = m.instance("issue_arbiter", &format!("{}IssueArbiter", mod_name));
    for i in 0..num_primaries {
        m.output(format!("primary{}_bus_ready", i), issue_arbiter.output(format!("primary{}_bus_ready", i)));
        issue_arbiter.drive_input(format!("primary{}_bus_enable", i), m.input(format!("primary{}_bus_enable", i), 1));
        issue_arbiter.drive_input(format!("primary{}_bus_addr", i), m.input(format!("primary{}_bus_addr", i), addr_bit_width));
    }

    let issue = m.instance("issue", &format!("{}Issue", mod_name));
    issue.drive_input("issue_arb_bus_enable", issue_arbiter.output("issue_bus_enable"));
    issue.drive_input("issue_arb_bus_addr", issue_arbiter.output("issue_bus_addr"));
    issue_arbiter.drive_input("issue_bus_ready", issue.output("issue_arb_bus_ready"));
    for i in 0..num_replicas {
        issue.drive_input(format!("replica{}_bus_ready", i), m.input(format!("replica{}_bus_ready", i), 1));
        m.output(format!("replica{}_bus_enable", i), issue.output(format!("replica{}_bus_enable", i)));
        m.output(format!("replica{}_bus_addr", i), issue.output(format!("replica{}_bus_addr", i)));
    }

    let return_arbiter = m.instance("return_arbiter", &format!("{}ReturnArbiter", mod_name));
    for i in 0..num_primaries {
        m.output(format!("primary{}_bus_read_data", i), return_arbiter.output(format!("primary{}_bus_read_data", i)));
        m.output(format!("primary{}_bus_read_data_valid", i), return_arbiter.output(format!("primary{}_bus_read_data_valid", i)));
    }

    if num_primaries > 1 {
        fifo::generate(&c, format!("{}PrimaryFifo", mod_name), fifo_depth_bits, primary_select_bit_width);
        let primary_fifo = m.instance("primary_fifo", &format!("{}PrimaryFifo", mod_name));
        issue.drive_input("issue_arb_bus_primary", issue_arbiter.output("issue_bus_primary"));
        issue.drive_input("primary_fifo_full", primary_fifo.output("full"));
        primary_fifo.drive_input("write_enable", issue.output("primary_fifo_write_enable"));
        primary_fifo.drive_input("write_data", issue.output("primary_fifo_write_data"));
        return_arbiter.drive_input("primary_fifo_empty", primary_fifo.output("empty"));
        return_arbiter.drive_input("primary_fifo_read_data", primary_fifo.output("read_data"));
        primary_fifo.drive_input("read_enable", return_arbiter.output("primary_fifo_read_enable"));
    }

    if num_replicas > 1 {
        fifo::generate(&c, format!("{}ReplicaFifo", mod_name), fifo_depth_bits, replica_select_bit_width);
        let replica_fifo = m.instance("replica_fifo", &format!("{}ReplicaFifo", mod_name));
        issue.drive_input("replica_fifo_full", replica_fifo.output("full"));
        replica_fifo.drive_input("write_enable", issue.output("replica_fifo_write_enable"));
        replica_fifo.drive_input("write_data", issue.output("replica_fifo_write_data"));

        peek_buffer::generate(&c, format!("ReplicaBuffer{}", mod_name), replica_select_bit_width);
        let replica_buffer = m.instance("replica_buffer", &format!("ReplicaBuffer{}", mod_name));
        replica_buffer.drive_input("ingress_data", replica_fifo.output("read_data"));
        replica_fifo.drive_input("read_enable", replica_buffer.output("ingress_read_enable"));
        let replica_fifo_read_data_valid = m.reg("replica_fifo_read_data_valid", 1);
        replica_fifo_read_data_valid.default_value(false);
        replica_fifo_read_data_valid.drive_next(!replica_fifo.output("empty") & replica_buffer.output("ingress_read_enable"));
        replica_buffer.drive_input("ingress_data_valid", replica_fifo_read_data_valid.value);
        return_arbiter.drive_input("replica_buffer_egress_ready", replica_buffer.output("egress_ready"));
        return_arbiter.drive_input("replica_buffer_egress_data", replica_buffer.output("egress_data"));
        replica_buffer.drive_input("egress_read_enable", return_arbiter.output("replica_buffer_egress_read_enable"));
    }

    fifo::generate(&c, format!("{}ReplicaDataFifo", mod_name), fifo_depth_bits, data_bit_width);
    for i in 0..num_replicas {
        let replica_data_fifo = m.instance(format!("replica{}_data_fifo", i), &format!("{}ReplicaDataFifo", mod_name));
        replica_data_fifo.drive_input("write_enable", m.input("replica_bus_read_data_valid", 1));
        replica_data_fifo.drive_input("write_data", m.input("replica_bus_read_data", data_bit_width));
        replica_data_fifo.drive_input("read_enable", return_arbiter.output(format!("replica{}_data_fifo_read_enable", i)));
        return_arbiter.drive_input(format!("replica{}_data_fifo_empty", i), replica_data_fifo.output("empty"));
        return_arbiter.drive_input(format!("replica{}_data_fifo_read_data", i), replica_data_fifo.output("read_data"));
    }

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Cannot generate a buster module with zero primaries.")]
    fn zero_primaries_error() {
        let c = Context::new();

        // Panic
        let _ = generate(&c, "BadDuder", 0, 2, 2, 1, 1, 1);
    }

    #[test]
    #[should_panic(expected = "Cannot generate a buster module with zero replicas.")]
    fn zero_replicas_error() {
        let c = Context::new();

        // Panic
        let _ = generate(&c, "BadDuder", 2, 0, 2, 1, 1, 1);
    }
}
