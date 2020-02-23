#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn single_read_master0() {
        let mut m = Buster::new();

        m.reset();

        m.master0_bus_enable = true;
        m.master0_bus_addr = 0xbabe;
        m.master1_bus_enable = false;
        m.master1_bus_addr = 0;
        m.slave_bus_ready = true;
        m.slave_bus_read_data = 0xffffffffffffffffffffffffffffffff;
        m.slave_bus_read_data_valid = false;

        m.prop();

        assert_eq!(m.master0_bus_ready, true);
        assert_eq!(m.master1_bus_ready, false);
        assert_eq!(m.slave_bus_enable, true);
        assert_eq!(m.slave_bus_addr, 0xbabe);

        m.posedge_clk();

        m.master0_bus_enable = false;
        m.slave_bus_read_data = 0xdeadbeeffadebabeabad1deacafebabe;
        m.slave_bus_read_data_valid = true;

        m.prop();

        while !m.master0_bus_read_data_valid {
            m.posedge_clk();
            m.prop();
        }

        assert_eq!(m.master0_bus_read_data, 0xdeadbeeffadebabeabad1deacafebabe);

        assert_eq!(m.master1_bus_read_data_valid, false);
    }

    #[test]
    fn single_read_master1() {
        let mut m = Buster::new();

        m.reset();

        m.master0_bus_enable = false;
        m.master0_bus_addr = 0;
        m.master1_bus_enable = true;
        m.master1_bus_addr = 0xbeef;
        m.slave_bus_ready = true;
        m.slave_bus_read_data = 0xffffffffffffffffffffffffffffffff;
        m.slave_bus_read_data_valid = false;

        m.prop();

        assert_eq!(m.master0_bus_ready, true);
        assert_eq!(m.master1_bus_ready, true);
        assert_eq!(m.slave_bus_enable, true);
        assert_eq!(m.slave_bus_addr, 0xbeef);

        m.posedge_clk();

        m.master1_bus_enable = false;
        m.slave_bus_read_data = 0xdeadbeeffadebabeabad1deacafebabe;
        m.slave_bus_read_data_valid = true;

        m.prop();

        while !m.master1_bus_read_data_valid {
            m.posedge_clk();
            m.prop();
        }

        assert_eq!(m.master1_bus_read_data, 0xdeadbeeffadebabeabad1deacafebabe);

        assert_eq!(m.master0_bus_read_data_valid, false);
    }

    #[test]
    fn read_all_master0() {
        let data = (0..65536).map(|x| (x + 1) * 4).collect::<Vec<_>>();

        let mut master_read_addr = 0;
        let mut master_read_data = Vec::new();

        let mut slave_read_addr = None;

        let mut m = Buster::new();

        m.reset();

        loop {
            m.prop();

            if m.master0_bus_read_data_valid {
                master_read_data.push(m.master0_bus_read_data);
                if master_read_data.len() == data.len() {
                    assert_eq!(master_read_data, data);
                    return;
                }
            }

            assert_eq!(m.master1_bus_read_data_valid, false);

            if let Some(addr) = slave_read_addr {
                m.slave_bus_read_data = data[addr as usize];
                m.slave_bus_read_data_valid = true;
            } else {
                m.slave_bus_read_data_valid = false;
            }

            if master_read_addr < data.len() {
                m.master0_bus_enable = true;
                m.master0_bus_addr = master_read_addr as _;
            } else {
                m.master0_bus_enable = false;
            }
            m.master1_bus_enable = false;
            m.master1_bus_addr = 0;

            m.slave_bus_ready = true;

            m.prop();

            if m.master0_bus_enable && m.master0_bus_ready {
                master_read_addr += 1;
            }

            slave_read_addr = if m.slave_bus_enable {
                Some(m.slave_bus_addr)
            } else {
                None
            };

            m.prop();
            m.posedge_clk();
        }
    }
}