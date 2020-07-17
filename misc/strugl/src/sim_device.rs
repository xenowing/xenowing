use crate::device::*;
use crate::modules::*;

pub struct SimDevice {
    color_thrust: ColorThrust,
}

impl SimDevice {
    pub fn new() -> SimDevice {
        let mut color_thrust = ColorThrust::new();
        color_thrust.reset();
        color_thrust.color_buffer_bus_enable = false;
        color_thrust.reg_bus_enable = false;
        color_thrust.tex_buffer_bus_enable = false;
        color_thrust.prop();

        SimDevice {
            color_thrust,
        }
    }
}

impl Device for SimDevice {
    fn write_reg(&mut self, addr: u32, data: u32) {
        self.color_thrust.reg_bus_addr = addr;
        self.color_thrust.reg_bus_enable = true;
        self.color_thrust.reg_bus_write = true;
        self.color_thrust.reg_bus_write_data = data;
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.reg_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.reg_bus_enable = false;
        self.color_thrust.prop();
    }

    fn read_reg(&mut self, addr: u32) -> u32 {
        self.color_thrust.reg_bus_addr = addr;
        self.color_thrust.reg_bus_enable = true;
        self.color_thrust.reg_bus_write = false;
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.reg_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.reg_bus_enable = false;
        self.color_thrust.prop();
        while !self.color_thrust.reg_bus_read_data_valid {
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
        }
        self.color_thrust.reg_bus_read_data
    }

    fn write_color_buffer_word(&mut self, addr: u32, data: u32) {
        self.color_thrust.color_buffer_bus_addr = addr >> 2;
        self.color_thrust.color_buffer_bus_enable = true;
        self.color_thrust.color_buffer_bus_write = true;
        self.color_thrust.color_buffer_bus_write_byte_enable = 0xf << ((addr & 0x3) * 4);
        self.color_thrust.color_buffer_bus_write_data = (data as u128) << ((addr & 0x3) * 32);
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.color_buffer_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.color_buffer_bus_enable = false;
        self.color_thrust.prop();
    }

    fn read_color_buffer_word(&mut self, addr: u32) -> u32 {
        self.color_thrust.color_buffer_bus_addr = addr >> 2;
        self.color_thrust.color_buffer_bus_enable = true;
        self.color_thrust.color_buffer_bus_write = false;
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.color_buffer_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.color_buffer_bus_enable = false;
        self.color_thrust.prop();
        while !self.color_thrust.color_buffer_bus_read_data_valid {
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
        }
        (self.color_thrust.color_buffer_bus_read_data >> ((addr & 0x3) * 32)) as u32
    }

    fn write_tex_buffer_word(&mut self, addr: u32, data: u32) {
        self.color_thrust.tex_buffer_bus_addr = addr >> 2;
        self.color_thrust.tex_buffer_bus_enable = true;
        self.color_thrust.tex_buffer_bus_write = true;
        self.color_thrust.tex_buffer_bus_write_byte_enable = 0xf << ((addr & 0x3) * 4);
        self.color_thrust.tex_buffer_bus_write_data = (data as u128) << ((addr & 0x3) * 32);
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.tex_buffer_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.tex_buffer_bus_enable = false;
        self.color_thrust.prop();
    }

    fn read_tex_buffer_word(&mut self, addr: u32) -> u32 {
        self.color_thrust.tex_buffer_bus_addr = addr >> 2;
        self.color_thrust.tex_buffer_bus_enable = true;
        self.color_thrust.tex_buffer_bus_write = false;
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.tex_buffer_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.tex_buffer_bus_enable = false;
        self.color_thrust.prop();
        while !self.color_thrust.tex_buffer_bus_read_data_valid {
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
        }
        (self.color_thrust.tex_buffer_bus_read_data >> ((addr & 0x3) * 32)) as u32
    }
}
