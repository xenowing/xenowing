#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn reset_empty() {
        let mut m = Fifo::new();

        m.reset();
        m.prop();

        assert_eq!(m.empty, true);
    }

    #[test]
    fn reset_not_full() {
        let mut m = Fifo::new();

        m.reset();
        m.prop();

        assert_eq!(m.full, false);
    }

    #[test]
    fn read_empty_still_empty() {
        let mut m = Fifo::new();

        m.reset();
        m.prop();

        m.write_enable = false;
        m.read_enable = true;

        for _ in 0..100 {
            m.prop();
            assert_eq!(m.empty, true);
            m.posedge_clk();
        }
    }

    #[test]
    fn write_until_full() {
        let mut m = Fifo::new();

        m.reset();
        m.prop();

        m.read_enable = false;

        for _ in 0..16 {
            m.write_enable = true;
            m.write_data = 0xfadebabe;
            m.prop();
            assert_eq!(m.full, false);
            m.posedge_clk();
        }

        m.prop();
        assert_eq!(m.full, true);
    }

    #[test]
    fn write_until_full_single_read_not_full() {
        let mut m = Fifo::new();

        m.reset();
        m.prop();

        m.read_enable = false;

        for _ in 0..16 {
            m.write_enable = true;
            m.write_data = 0xfadebabe;
            m.prop();
            assert_eq!(m.full, false);
            m.posedge_clk();
        }

        for _ in 0..100 {
            m.prop();
            assert_eq!(m.full, true);
            m.posedge_clk();
        }

        m.write_enable = false;
        m.read_enable = true;
        m.prop();
        m.posedge_clk();
        m.prop();
        assert_eq!(m.full, false);
    }

    #[test]
    fn write_until_full_read_until_empty() {
        let mut m = Fifo::new();

        m.reset();
        m.prop();

        m.read_enable = false;

        for _ in 0..16 {
            m.write_enable = true;
            m.write_data = 0xfadebabe;
            m.prop();
            assert_eq!(m.full, false);
            m.posedge_clk();
        }

        m.prop();
        assert_eq!(m.full, true);

        m.write_enable = false;

        for _ in 0..16 {
            m.read_enable = true;
            m.prop();
            assert_eq!(m.empty, false);
            m.posedge_clk();
        }

        m.prop();
        assert_eq!(m.empty, true);
    }
}
