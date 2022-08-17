#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    use rtl::buster_mig_ui_bridge::{UI_CMD_WRITE, UI_CMD_READ};

    #[test]
    fn reset_not_ready_until_calibration_is_complete() {
        let mut m = BusterMigUiBridge::new();

        m.reset();

        m.init_calib_complete = false;
        m.prop();

        assert_eq!(m.bus_ready, false);

        m.init_calib_complete = true;
        m.prop();

        assert_eq!(m.bus_ready, true);
    }

    #[test]
    fn reset_empty() {
        let mut m = BusterMigUiBridge::new();

        m.reset();

        m.init_calib_complete = true;
        m.prop();

        assert_eq!(m.bus_ready, true);
        assert_eq!(m.app_en, false);
    }

    #[test]
    fn single_read() {
        let mut m = BusterMigUiBridge::new();

        m.reset();

        m.init_calib_complete = true;
        m.app_rdy = false;
        m.app_rd_data_valid = false;

        // Issue read
        m.bus_enable = true;
        m.bus_write = false;
        m.bus_addr = 0xaa;
        m.prop();
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_wren, false);

        m.posedge_clk();

        // Stop issuing read on the cycle following successful buster issue
        m.bus_enable = false;
        m.prop();

        // Read should be issued to the UI on the cycle following successful buster issue
        assert_eq!(m.bus_ready, false);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, true);
        assert_eq!(m.app_cmd, UI_CMD_READ);
        assert_eq!(m.app_addr, 0xaa);
        assert_eq!(m.app_wdf_wren, false);

        m.posedge_clk();

        // UI read was not accepted, so should still be asserted on the following cycle (where it should be accepted)
        m.app_rdy = true;
        m.prop();

        // New commands should be acceptable from buster when the UI is accepting the buffered command
        assert_eq!(m.bus_ready, true);

        // Data should not be presented to buster before it's been returned from UI
        assert_eq!(m.bus_read_data_valid, false);

        m.posedge_clk();

        // UI read should no longer be asserted on the cycle following succesful UI issue
        m.prop();
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_wren, false);

        m.posedge_clk();

        // Return data from UI
        m.app_rd_data = 0xfadebabe;
        m.app_rd_data_valid = true;
        m.prop();

        // Data returned from UI should be presented to buster immediately
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data, 0xfadebabe);
        assert_eq!(m.bus_read_data_valid, true);
    }

    #[test]
    fn single_write() {
        let mut m = BusterMigUiBridge::new();

        m.reset();

        m.init_calib_complete = true;
        m.app_rdy = false;
        m.app_rd_data_valid = false;

        // Issue write
        m.bus_enable = true;
        m.bus_write = true;
        m.bus_write_data = 0xdeadbeef;
        m.bus_write_byte_enable = 0b1010;
        m.bus_addr = 0x55;
        m.prop();
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_wren, false);

        m.posedge_clk();

        // Stop issuing write on the cycle following successful buster issue
        m.bus_enable = false;
        m.prop();

        // Write should be issued to the UI on the cycle following successful buster issue
        assert_eq!(m.bus_ready, false);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, true);
        assert_eq!(m.app_cmd, UI_CMD_WRITE);
        assert_eq!(m.app_addr, 0x55);
        assert_eq!(m.app_wdf_data, 0xdeadbeef);
        assert_eq!(m.app_wdf_wren, true);
        assert_eq!(m.app_wdf_mask, 0b0101);
        assert_eq!(m.app_wdf_end, true);

        m.posedge_clk();

        // UI write was not accepted, so should still be asserted on the following cycle (where it should be accepted)
        m.app_rdy = true;
        m.app_wdf_rdy = true;
        m.prop();

        // New commands should be acceptable from buster when the UI is accepting the buffered command
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);

        m.posedge_clk();

        // UI write should no longer be asserted on the cycle following succesful UI issue
        m.prop();
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_wren, false);
    }

    #[test]
    fn single_write_delayed_data_accept() {
        let mut m = BusterMigUiBridge::new();

        m.reset();

        m.init_calib_complete = true;
        m.app_rdy = false;
        m.app_rd_data_valid = false;

        // Issue write
        m.bus_enable = true;
        m.bus_write = true;
        m.bus_write_data = 0xdeadbeef;
        m.bus_write_byte_enable = 0b1010;
        m.bus_addr = 0x55;
        m.prop();
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_wren, false);

        m.posedge_clk();

        // Stop issuing write on the cycle following successful buster issue
        m.bus_enable = false;
        m.prop();

        // Write should be issued to the UI on the cycle following successful buster issue
        assert_eq!(m.bus_ready, false);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, true);
        assert_eq!(m.app_cmd, UI_CMD_WRITE);
        assert_eq!(m.app_addr, 0x55);
        assert_eq!(m.app_wdf_data, 0xdeadbeef);
        assert_eq!(m.app_wdf_wren, true);
        assert_eq!(m.app_wdf_mask, 0b0101);
        assert_eq!(m.app_wdf_end, true);

        m.posedge_clk();

        // UI write was not accepted, so should still be asserted on the following cycle, where we only accept the command
        m.app_rdy = true;
        m.prop();

        // New commands should not be acceptable from buster while the UI hasn't accepted both parts of the most recent command
        assert_eq!(m.bus_ready, false);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, true);
        assert_eq!(m.app_cmd, UI_CMD_WRITE);
        assert_eq!(m.app_addr, 0x55);
        assert_eq!(m.app_wdf_data, 0xdeadbeef);
        assert_eq!(m.app_wdf_wren, true);
        assert_eq!(m.app_wdf_mask, 0b0101);
        assert_eq!(m.app_wdf_end, true);

        m.posedge_clk();

        // Accept data as well
        m.app_wdf_rdy = true;
        m.prop();

        // New commands should be acceptable from buster when the UI is accepting the buffered command
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_data, 0xdeadbeef);
        assert_eq!(m.app_wdf_wren, true);
        assert_eq!(m.app_wdf_mask, 0b0101);
        assert_eq!(m.app_wdf_end, true);

        m.posedge_clk();

        // UI write should no longer be asserted on the cycle following succesful UI issue
        m.prop();
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_wren, false);
    }

    #[test]
    fn single_write_delayed_command_accept() {
        let mut m = BusterMigUiBridge::new();

        m.reset();

        m.init_calib_complete = true;
        m.app_rdy = false;
        m.app_rd_data_valid = false;

        // Issue write
        m.bus_enable = true;
        m.bus_write = true;
        m.bus_write_data = 0xdeadbeef;
        m.bus_write_byte_enable = 0b1010;
        m.bus_addr = 0x55;
        m.prop();
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_wren, false);

        m.posedge_clk();

        // Stop issuing write on the cycle following successful buster issue
        m.bus_enable = false;
        m.prop();

        // Write should be issued to the UI on the cycle following successful buster issue
        assert_eq!(m.bus_ready, false);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, true);
        assert_eq!(m.app_cmd, UI_CMD_WRITE);
        assert_eq!(m.app_addr, 0x55);
        assert_eq!(m.app_wdf_data, 0xdeadbeef);
        assert_eq!(m.app_wdf_wren, true);
        assert_eq!(m.app_wdf_mask, 0b0101);
        assert_eq!(m.app_wdf_end, true);

        m.posedge_clk();

        // UI write was not accepted, so should still be asserted on the following cycle, where we only accept the data
        m.app_wdf_rdy = true;
        m.prop();

        // New commands should not be acceptable from buster while the UI hasn't accepted both parts of the most recent command
        assert_eq!(m.bus_ready, false);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, true);
        assert_eq!(m.app_cmd, UI_CMD_WRITE);
        assert_eq!(m.app_addr, 0x55);
        assert_eq!(m.app_wdf_data, 0xdeadbeef);
        assert_eq!(m.app_wdf_wren, true);
        assert_eq!(m.app_wdf_mask, 0b0101);
        assert_eq!(m.app_wdf_end, true);

        m.posedge_clk();

        // Accept command as well
        m.app_rdy = true;
        m.prop();

        // New commands should be acceptable from buster when the UI is accepting the buffered command
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, true);
        assert_eq!(m.app_cmd, UI_CMD_WRITE);
        assert_eq!(m.app_addr, 0x55);
        assert_eq!(m.app_wdf_wren, false);

        m.posedge_clk();

        // UI write should no longer be asserted on the cycle following succesful UI issue
        m.prop();
        assert_eq!(m.bus_ready, true);
        assert_eq!(m.bus_read_data_valid, false);
        assert_eq!(m.app_en, false);
        assert_eq!(m.app_wdf_wren, false);
    }
}
