#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn single_read_primary0_replica0() {
        let mut m = Buster2x2::new();

        m.reset();

        m.primary0_bus_enable = true;
        m.primary0_bus_addr = 0x0babe;
        m.primary1_bus_enable = false;
        m.primary1_bus_addr = 0;
        m.replica0_bus_ready = true;
        m.replica1_bus_ready = false;
        m.replica_bus_read_data = 0xffffffffffffffffffffffffffffffff;
        m.replica_bus_read_data_valid = false;

        m.prop();

        assert_eq!(m.primary0_bus_ready, true);
        assert_eq!(m.primary1_bus_ready, false);
        assert_eq!(m.replica0_bus_enable, true);
        assert_eq!(m.replica1_bus_enable, false);
        assert_eq!(m.replica0_bus_addr, 0xbabe);

        m.posedge_clk();

        m.primary0_bus_enable = false;
        m.replica_bus_read_data = 0xdeadbeeffadebabeabad1deacafebabe;
        m.replica_bus_read_data_valid = true;

        m.prop();

        while !m.primary0_bus_read_data_valid {
            m.posedge_clk();
            m.prop();
        }

        assert_eq!(m.primary0_bus_read_data, 0xdeadbeeffadebabeabad1deacafebabe);

        assert_eq!(m.primary1_bus_read_data_valid, false);
    }

    #[test]
    fn single_read_primary1_replica0() {
        let mut m = Buster2x2::new();

        m.reset();

        m.primary0_bus_enable = false;
        m.primary0_bus_addr = 0;
        m.primary1_bus_enable = true;
        m.primary1_bus_addr = 0x0beef;
        m.replica0_bus_ready = true;
        m.replica1_bus_ready = false;
        m.replica_bus_read_data = 0xffffffffffffffffffffffffffffffff;
        m.replica_bus_read_data_valid = false;

        m.prop();

        assert_eq!(m.primary0_bus_ready, true);
        assert_eq!(m.primary1_bus_ready, true);
        assert_eq!(m.replica0_bus_enable, true);
        assert_eq!(m.replica1_bus_enable, false);
        assert_eq!(m.replica0_bus_addr, 0xbeef);

        m.posedge_clk();

        m.primary1_bus_enable = false;
        m.replica_bus_read_data = 0xdeadbeeffadebabeabad1deacafebabe;
        m.replica_bus_read_data_valid = true;

        m.prop();

        while !m.primary1_bus_read_data_valid {
            m.posedge_clk();
            m.prop();
        }

        assert_eq!(m.primary1_bus_read_data, 0xdeadbeeffadebabeabad1deacafebabe);

        assert_eq!(m.primary0_bus_read_data_valid, false);
    }

    #[test]
    fn single_read_primary0_replica1() {
        let mut m = Buster2x2::new();

        m.reset();

        m.primary0_bus_enable = true;
        m.primary0_bus_addr = 0x1babe;
        m.primary1_bus_enable = false;
        m.primary1_bus_addr = 0;
        m.replica0_bus_ready = false;
        m.replica1_bus_ready = true;
        m.replica_bus_read_data = 0xffffffffffffffffffffffffffffffff;
        m.replica_bus_read_data_valid = false;

        m.prop();

        assert_eq!(m.primary0_bus_ready, true);
        assert_eq!(m.primary1_bus_ready, false);
        assert_eq!(m.replica0_bus_enable, false);
        assert_eq!(m.replica1_bus_enable, true);
        assert_eq!(m.replica1_bus_addr, 0xbabe);

        m.posedge_clk();

        m.primary0_bus_enable = false;
        m.replica_bus_read_data = 0xdeadbeeffadebabeabad1deacafebabe;
        m.replica_bus_read_data_valid = true;

        m.prop();

        while !m.primary0_bus_read_data_valid {
            m.posedge_clk();
            m.prop();
        }

        assert_eq!(m.primary0_bus_read_data, 0xdeadbeeffadebabeabad1deacafebabe);

        assert_eq!(m.primary1_bus_read_data_valid, false);
    }

    #[test]
    fn single_read_primary1_replica1() {
        let mut m = Buster2x2::new();

        m.reset();

        m.primary0_bus_enable = false;
        m.primary0_bus_addr = 0;
        m.primary1_bus_enable = true;
        m.primary1_bus_addr = 0x1beef;
        m.replica0_bus_ready = false;
        m.replica1_bus_ready = true;
        m.replica_bus_read_data = 0xffffffffffffffffffffffffffffffff;
        m.replica_bus_read_data_valid = false;

        m.prop();

        assert_eq!(m.primary0_bus_ready, true);
        assert_eq!(m.primary1_bus_ready, true);
        assert_eq!(m.replica0_bus_enable, false);
        assert_eq!(m.replica1_bus_enable, true);
        assert_eq!(m.replica1_bus_addr, 0xbeef);

        m.posedge_clk();

        m.primary1_bus_enable = false;
        m.replica_bus_read_data = 0xdeadbeeffadebabeabad1deacafebabe;
        m.replica_bus_read_data_valid = true;

        m.prop();

        while !m.primary1_bus_read_data_valid {
            m.posedge_clk();
            m.prop();
        }

        assert_eq!(m.primary1_bus_read_data, 0xdeadbeeffadebabeabad1deacafebabe);

        assert_eq!(m.primary0_bus_read_data_valid, false);
    }

    #[test]
    fn read_all_primary0_replica_0() {
        let data = (0..65536).map(|x| (x + 1) * 4).collect::<Vec<_>>();

        let mut primary_read_addr = 0;
        let mut primary_read_data = Vec::new();

        let mut replica_read_addr = None;

        let mut m = Buster2x2::new();

        m.reset();

        loop {
            m.prop();

            if m.primary0_bus_read_data_valid {
                primary_read_data.push(m.primary0_bus_read_data);
                if primary_read_data.len() == data.len() {
                    assert_eq!(primary_read_data, data);
                    return;
                }
            }

            assert_eq!(m.primary1_bus_read_data_valid, false);

            if let Some(addr) = replica_read_addr {
                m.replica_bus_read_data = data[addr as usize];
                m.replica_bus_read_data_valid = true;
            } else {
                m.replica_bus_read_data_valid = false;
            }

            if primary_read_addr < data.len() {
                m.primary0_bus_enable = true;
                m.primary0_bus_addr = primary_read_addr as _;
            } else {
                m.primary0_bus_enable = false;
            }
            m.primary1_bus_enable = false;
            m.primary1_bus_addr = 0;

            m.replica0_bus_ready = true;

            m.prop();

            if m.primary0_bus_enable && m.primary0_bus_ready {
                primary_read_addr += 1;
            }

            replica_read_addr = if m.replica0_bus_enable {
                Some(m.replica0_bus_addr)
            } else {
                None
            };

            m.prop();
            m.posedge_clk();
        }
    }

    #[test]
    fn read_all_primary0_replica_1() {
        let data = (0..65536).map(|x| (x + 1) * 4).collect::<Vec<_>>();

        let mut primary_read_addr = 0;
        let mut primary_read_data = Vec::new();

        let mut replica_read_addr = None;

        let mut m = Buster2x2::new();

        m.reset();

        loop {
            m.prop();

            if m.primary0_bus_read_data_valid {
                primary_read_data.push(m.primary0_bus_read_data);
                if primary_read_data.len() == data.len() {
                    assert_eq!(primary_read_data, data);
                    return;
                }
            }

            assert_eq!(m.primary1_bus_read_data_valid, false);

            if let Some(addr) = replica_read_addr {
                m.replica_bus_read_data = data[addr as usize];
                m.replica_bus_read_data_valid = true;
            } else {
                m.replica_bus_read_data_valid = false;
            }

            if primary_read_addr < data.len() {
                m.primary0_bus_enable = true;
                m.primary0_bus_addr = 0x10000 | primary_read_addr as u32;
            } else {
                m.primary0_bus_enable = false;
            }
            m.primary1_bus_enable = false;
            m.primary1_bus_addr = 0;

            m.replica1_bus_ready = true;

            m.prop();

            if m.primary0_bus_enable && m.primary0_bus_ready {
                primary_read_addr += 1;
            }

            replica_read_addr = if m.replica1_bus_enable {
                Some(m.replica1_bus_addr)
            } else {
                None
            };

            m.prop();
            m.posedge_clk();
        }
    }

    #[test]
    fn read_all_primary1_replica0() {
        let data = (0..65536).map(|x| (x + 3) * 12).collect::<Vec<_>>();

        let mut primary_read_addr = 0;
        let mut primary_read_data = Vec::new();

        let mut replica_read_addr = None;

        let mut m = Buster2x2::new();

        m.reset();

        loop {
            m.prop();

            if m.primary1_bus_read_data_valid {
                primary_read_data.push(m.primary1_bus_read_data);
                if primary_read_data.len() == data.len() {
                    assert_eq!(primary_read_data, data);
                    return;
                }
            }

            assert_eq!(m.primary0_bus_read_data_valid, false);

            if let Some(addr) = replica_read_addr {
                m.replica_bus_read_data = data[addr as usize];
                m.replica_bus_read_data_valid = true;
            } else {
                m.replica_bus_read_data_valid = false;
            }

            if primary_read_addr < data.len() {
                m.primary1_bus_enable = true;
                m.primary1_bus_addr = primary_read_addr as _;
            } else {
                m.primary1_bus_enable = false;
            }
            m.primary0_bus_enable = false;
            m.primary0_bus_addr = 0;

            m.replica0_bus_ready = true;

            m.prop();

            if m.primary1_bus_enable && m.primary1_bus_ready {
                primary_read_addr += 1;
            }

            replica_read_addr = if m.replica0_bus_enable {
                Some(m.replica0_bus_addr)
            } else {
                None
            };

            m.prop();
            m.posedge_clk();
        }
    }

    #[test]
    fn read_all_primary1_replica1() {
        let data = (0..65536).map(|x| (x + 3) * 12).collect::<Vec<_>>();

        let mut primary_read_addr = 0;
        let mut primary_read_data = Vec::new();

        let mut replica_read_addr = None;

        let mut m = Buster2x2::new();

        m.reset();

        loop {
            m.prop();

            if m.primary1_bus_read_data_valid {
                primary_read_data.push(m.primary1_bus_read_data);
                if primary_read_data.len() == data.len() {
                    assert_eq!(primary_read_data, data);
                    return;
                }
            }

            assert_eq!(m.primary0_bus_read_data_valid, false);

            if let Some(addr) = replica_read_addr {
                m.replica_bus_read_data = data[addr as usize];
                m.replica_bus_read_data_valid = true;
            } else {
                m.replica_bus_read_data_valid = false;
            }

            if primary_read_addr < data.len() {
                m.primary1_bus_enable = true;
                m.primary1_bus_addr = 0x10000 | primary_read_addr as u32;
            } else {
                m.primary1_bus_enable = false;
            }
            m.primary0_bus_enable = false;
            m.primary0_bus_addr = 0;

            m.replica1_bus_ready = true;

            m.prop();

            if m.primary1_bus_enable && m.primary1_bus_ready {
                primary_read_addr += 1;
            }

            replica_read_addr = if m.replica1_bus_enable {
                Some(m.replica1_bus_addr)
            } else {
                None
            };

            m.prop();
            m.posedge_clk();
        }
    }
}
