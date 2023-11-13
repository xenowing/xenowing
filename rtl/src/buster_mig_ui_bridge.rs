use crate::buster::*;

use kaze::*;

pub const UI_CMD_BIT_WIDTH: u32 = 3;
pub const UI_CMD_WRITE: u32 = 0b000;
pub const UI_CMD_READ: u32 = 0b001;

pub struct MigUiPort<'a> {
    pub init_calib_complete: &'a Input<'a>,

    pub app_rdy: &'a Input<'a>,
    pub app_en: &'a Output<'a>,
    pub app_cmd: &'a Output<'a>,
    pub app_addr: &'a Output<'a>,

    pub app_wdf_rdy: &'a Input<'a>,
    pub app_wdf_data: &'a Output<'a>,
    pub app_wdf_wren: &'a Output<'a>,
    pub app_wdf_mask: &'a Output<'a>,
    pub app_wdf_end: &'a Output<'a>,

    pub app_rd_data: &'a Input<'a>,
    pub app_rd_data_valid: &'a Input<'a>,
}

impl<'a> MigUiPort<'a> {
    pub fn forward(&self, name_prefix: impl Into<String>, m: &'a Module<'a>) -> MigUiPort<'a> {
        let name_prefix = name_prefix.into();

        let init_calib_complete = m.input(
            format!("{}_init_calib_complete", name_prefix),
            self.init_calib_complete.bit_width(),
        );
        self.init_calib_complete.drive(init_calib_complete);

        let app_rdy = m.input(format!("{}_app_rdy", name_prefix), self.app_rdy.bit_width());
        self.app_rdy.drive(app_rdy);

        let app_wdf_rdy = m.input(
            format!("{}_app_wdf_rdy", name_prefix),
            self.app_wdf_rdy.bit_width(),
        );
        self.app_wdf_rdy.drive(app_wdf_rdy);

        let app_rd_data = m.input(
            format!("{}_app_rd_data", name_prefix),
            self.app_rd_data.bit_width(),
        );
        self.app_rd_data.drive(app_rd_data);
        let app_rd_data_valid = m.input(
            format!("{}_app_rd_data_valid", name_prefix),
            self.app_rd_data_valid.bit_width(),
        );
        self.app_rd_data_valid.drive(app_rd_data_valid);

        MigUiPort {
            init_calib_complete,

            app_rdy,
            app_en: m.output(format!("{}_app_en", name_prefix), self.app_en),
            app_cmd: m.output(format!("{}_app_cmd", name_prefix), self.app_cmd),
            app_addr: m.output(format!("{}_app_addr", name_prefix), self.app_addr),

            app_wdf_rdy,
            app_wdf_data: m.output(format!("{}_app_wdf_data", name_prefix), self.app_wdf_data),
            app_wdf_wren: m.output(format!("{}_app_wdf_wren", name_prefix), self.app_wdf_wren),
            app_wdf_mask: m.output(format!("{}_app_wdf_mask", name_prefix), self.app_wdf_mask),
            app_wdf_end: m.output(format!("{}_app_wdf_end", name_prefix), self.app_wdf_end),

            app_rd_data,
            app_rd_data_valid,
        }
    }
}

pub struct BusterMigUiBridge<'a> {
    pub m: &'a Module<'a>,
    pub client_port: ReplicaPort<'a>,
    pub ui_port: MigUiPort<'a>,
}

impl<'a> BusterMigUiBridge<'a> {
    pub fn new(
        instance_name: impl Into<String>,
        data_bit_width: u32,
        addr_bit_width: u32,
        p: &'a impl ModuleParent<'a>,
    ) -> BusterMigUiBridge<'a> {
        let m = p.module(instance_name, "BusterMigUiBridge");

        let bus_enable = m.input("bus_enable", 1);
        let bus_addr = m.input("bus_addr", addr_bit_width);
        let bus_write = m.input("bus_write", 1);
        let bus_write_data = m.input("bus_write_data", data_bit_width);
        let bus_write_byte_enable = m.input("bus_write_byte_enable", data_bit_width / 8);

        let init_calib_complete = m.input("init_calib_complete", 1);

        let app_rdy = m.input("app_rdy", 1);

        let app_wdf_rdy = m.input("app_wdf_rdy", 1);

        let app_rd_data = m.input("app_rd_data", data_bit_width);
        let app_rd_data_valid = m.input("app_rd_data_valid", 1);

        let cmd_buf_write = m.reg("cmd_buf_write", 1);
        let app_cmd = if_(cmd_buf_write, m.lit(UI_CMD_WRITE, UI_CMD_BIT_WIDTH))
            .else_(m.lit(UI_CMD_READ, UI_CMD_BIT_WIDTH));
        let cmd_buf_addr = m.reg("cmd_buf_addr", addr_bit_width);
        let cmd_buf_data = m.reg("cmd_buf_data", data_bit_width);
        let cmd_buf_write_byte_enable = m.reg("cmd_buf_write_byte_enable", data_bit_width / 8);
        let cmd_buf_mask = !cmd_buf_write_byte_enable;
        let cmd_buf_cmd_issue = m.reg("cmd_buf_cmd_issue", 1);
        cmd_buf_cmd_issue.default_value(false);
        let cmd_buf_data_issue = m.reg("cmd_buf_data_issue", 1);
        cmd_buf_data_issue.default_value(false);

        let cmd_buf_issued = if_(cmd_buf_cmd_issue, {
            if_(cmd_buf_data_issue, app_rdy & app_wdf_rdy).else_(app_rdy)
        })
        .else_(cmd_buf_data_issue & app_wdf_rdy);

        let cmd_buf_occupied = cmd_buf_cmd_issue | cmd_buf_data_issue;
        // TODO: We might not actually need to wait for calibration to be complete
        let bus_ready = (!cmd_buf_occupied | cmd_buf_issued) & init_calib_complete;

        let in_cmd_accepted = bus_enable & bus_ready;

        cmd_buf_write.drive_next(if_(in_cmd_accepted, bus_write).else_(cmd_buf_write));
        cmd_buf_addr.drive_next(if_(in_cmd_accepted, bus_addr).else_(cmd_buf_addr));
        cmd_buf_data.drive_next(if_(in_cmd_accepted, bus_write_data).else_(cmd_buf_data));
        cmd_buf_write_byte_enable.drive_next(
            if_(in_cmd_accepted, bus_write_byte_enable).else_(cmd_buf_write_byte_enable),
        );
        cmd_buf_cmd_issue.drive_next(in_cmd_accepted | (cmd_buf_cmd_issue & !app_rdy));
        cmd_buf_data_issue
            .drive_next((in_cmd_accepted & bus_write) | (cmd_buf_data_issue & !app_wdf_rdy));

        BusterMigUiBridge {
            m,
            client_port: ReplicaPort {
                bus_enable,
                bus_addr,
                bus_write,
                bus_write_data,
                bus_write_byte_enable,
                bus_ready: m.output("bus_ready", bus_ready),
                bus_read_data: m.output("bus_read_data", app_rd_data),
                bus_read_data_valid: m.output("bus_read_data_valid", app_rd_data_valid),
            },
            ui_port: MigUiPort {
                init_calib_complete,

                app_rdy,
                app_en: m.output("app_en", cmd_buf_cmd_issue),
                app_cmd: m.output("app_cmd", app_cmd),
                app_addr: m.output("app_addr", cmd_buf_addr),

                app_wdf_rdy,
                app_wdf_data: m.output("app_wdf_data", cmd_buf_data),
                app_wdf_wren: m.output("app_wdf_wren", cmd_buf_data_issue),
                app_wdf_mask: m.output("app_wdf_mask", cmd_buf_mask),
                app_wdf_end: m.output("app_wdf_end", m.high()),

                app_rd_data,
                app_rd_data_valid,
            },
        }
    }
}
