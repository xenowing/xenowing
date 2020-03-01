#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn reset_empty() {
        let mut m = PeekBuffer::new();

        m.reset();
        m.prop();

        assert_eq!(m.ingress_read_enable, true);
        assert_eq!(m.egress_ready, false);
    }

    #[test]
    fn reset_item_pass_through() {
        let mut m = PeekBuffer::new();

        m.reset();

        // Ingress item
        m.ingress_data = 0xfadebabe;
        m.ingress_data_valid = true;
        m.prop();

        // Egress item on same cycle
        assert_eq!(m.egress_ready, true);
        assert_eq!(m.egress_data, 0xfadebabe);
    }

    #[test]
    fn reset_item_ingress_on_egress() {
        let mut m = PeekBuffer::new();

        m.reset();

        // Ingress initial item
        m.ingress_data = 0xfadebabe;
        m.ingress_data_valid = true;

        // Enable egress read
        m.egress_read_enable = true;
        m.prop();

        // Egress initial item on same cycle
        assert_eq!(m.egress_ready, true);
        assert_eq!(m.egress_data, 0xfadebabe);

        // Ingress read for next item on same cycle, since egress read is enabled
        assert_eq!(m.ingress_read_enable, true);

        m.posedge_clk();
        
        // Ingress next item
        m.ingress_data = 0xdeadbeef;
        m.ingress_data_valid = true;
        m.prop();

        // Egress next item on same cycle
        assert_eq!(m.egress_ready, true);
        assert_eq!(m.egress_data, 0xdeadbeef);
    }

    #[test]
    fn reset_item_ingress_hold_without_egress() {
        let mut m = PeekBuffer::new();

        m.reset();

        // Ingress initial item
        m.ingress_data = 0xfadebabe;
        m.ingress_data_valid = true;

        // Disable egress read
        m.egress_read_enable = false;
        m.prop();

        // Egress initial item on same cycle
        assert_eq!(m.egress_ready, true);
        assert_eq!(m.egress_data, 0xfadebabe);

        // Don't ingress read for next item, since we could overrun the buffer
        assert_eq!(m.ingress_read_enable, false);

        m.posedge_clk();

        // Change ingress data (just to be sure)
        m.ingress_data = 0xdeadbeef;
        m.prop();

        // Don't ingress read for next item, since we could overrun the buffer
        assert_eq!(m.ingress_read_enable, false);

        // Egress initial item on same cycle
        assert_eq!(m.egress_ready, true);
        assert_eq!(m.egress_data, 0xfadebabe);
    }

    #[test]
    fn reset_item_egress_after_hold_cycles() {
        let mut m = PeekBuffer::new();

        m.reset();

        // Ingress initial item
        m.ingress_data = 0xfadebabe;
        m.ingress_data_valid = true;

        // Disable egress read
        m.egress_read_enable = false;
        m.prop();

        // Egress initial item on same cycle
        assert_eq!(m.egress_ready, true);
        assert_eq!(m.egress_data, 0xfadebabe);

        // Don't ingress read for next item, since we could overrun the buffer
        assert_eq!(m.ingress_read_enable, false);

        m.posedge_clk();

        // Don't return any new data, since a read was not requested last cycle
        m.ingress_data = 0xdeadbeef; // This value should be ignored
        m.ingress_data_valid = false;
        m.prop();

        // Hold cycles
        for _ in 0..10 {
            m.posedge_clk();
            m.prop();

            // Ensure all hold cycles egress initial item, since it hasn't been read from the buffer
            assert_eq!(m.egress_ready, true);
            assert_eq!(m.egress_data, 0xfadebabe);
        }

        // Assert egress read enable for this cycle
        m.egress_read_enable = true;
        m.prop();

        // Ingress read for next item, since we're flushing the buffer this cycle
        assert_eq!(m.ingress_read_enable, true);

        // We should still egress initial item until next cycle
        assert_eq!(m.egress_ready, true);
        assert_eq!(m.egress_data, 0xfadebabe);

        m.posedge_clk();

        // Return second item from read in previous cycle
        m.ingress_data = 0xabad1dea;
        m.ingress_data_valid = true;
        m.prop();

        // Egress second item on same cycle
        assert_eq!(m.egress_ready, true);
        assert_eq!(m.egress_data, 0xabad1dea);
    }
}
