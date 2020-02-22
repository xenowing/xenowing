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
        m.slave_read_data = 0xffffffffffffffffffffffffffffffff;
        m.slave_read_data_valid = false;

        m.prop();

        assert_eq!(m.master0_bus_ready, true);
        assert_eq!(m.master1_bus_ready, false);
        assert_eq!(m.slave_bus_enable, true);
        assert_eq!(m.slave_bus_addr, 0xbabe);

        m.posedge_clk();

        m.master0_bus_enable = false;
        m.slave_read_data = 0xdeadbeeffadebabeabad1deacafebabe;
        m.slave_read_data_valid = true;

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
        m.slave_read_data = 0xffffffffffffffffffffffffffffffff;
        m.slave_read_data_valid = false;

        m.prop();

        assert_eq!(m.master0_bus_ready, true);
        assert_eq!(m.master1_bus_ready, true);
        assert_eq!(m.slave_bus_enable, true);
        assert_eq!(m.slave_bus_addr, 0xbeef);

        m.posedge_clk();

        m.master1_bus_enable = false;
        m.slave_read_data = 0xdeadbeeffadebabeabad1deacafebabe;
        m.slave_read_data_valid = true;

        m.prop();

        while !m.master1_bus_read_data_valid {
            m.posedge_clk();
            m.prop();
        }

        assert_eq!(m.master1_bus_read_data, 0xdeadbeeffadebabeabad1deacafebabe);

        assert_eq!(m.master0_bus_read_data_valid, false);
    }
}