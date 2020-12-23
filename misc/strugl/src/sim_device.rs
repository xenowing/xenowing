use crate::device::*;
use crate::modules::*;

use rtl::color_thrust::*;

pub struct SimDevice {
    color_thrust: ColorThrust,

    tex_buffer: [u128; (1 << TEX_WORD_ADDR_BITS) as usize],
}

impl SimDevice {
    pub fn new() -> SimDevice {
        let mut color_thrust = ColorThrust::new();
        color_thrust.reset();
        color_thrust.color_buffer_bus_enable = false;
        color_thrust.depth_buffer_bus_enable = false;
        color_thrust.reg_bus_enable = false;
        // TODO: haxx
        color_thrust.replica_bus_ready = true;
        color_thrust.prop();

        SimDevice {
            color_thrust,

            tex_buffer: [0; (1 << TEX_WORD_ADDR_BITS) as usize],
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
        self.color_thrust.replica_bus_read_data = self.tex_buffer[self.color_thrust.replica_bus_addr as usize];
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.reg_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            self.color_thrust.replica_bus_read_data = self.tex_buffer[self.color_thrust.replica_bus_addr as usize];
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
        self.color_thrust.replica_bus_read_data = self.tex_buffer[self.color_thrust.replica_bus_addr as usize];
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.reg_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            self.color_thrust.replica_bus_read_data = self.tex_buffer[self.color_thrust.replica_bus_addr as usize];
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.reg_bus_enable = false;
        self.color_thrust.replica_bus_read_data = self.tex_buffer[self.color_thrust.replica_bus_addr as usize];
        self.color_thrust.prop();
        while !self.color_thrust.reg_bus_read_data_valid {
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            self.color_thrust.replica_bus_read_data = self.tex_buffer[self.color_thrust.replica_bus_addr as usize];
            self.color_thrust.prop();
        }
        self.color_thrust.reg_bus_read_data
    }

    fn write_color_buffer_word(&mut self, addr: u32, data: u128) {
        self.color_thrust.color_buffer_bus_addr = addr;
        self.color_thrust.color_buffer_bus_enable = true;
        self.color_thrust.color_buffer_bus_write = true;
        self.color_thrust.color_buffer_bus_write_byte_enable = 0xffff;
        self.color_thrust.color_buffer_bus_write_data = data;
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

    fn read_color_buffer_word(&mut self, addr: u32) -> u128 {
        self.color_thrust.color_buffer_bus_addr = addr;
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
        self.color_thrust.color_buffer_bus_read_data
    }

    fn write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        self.color_thrust.depth_buffer_bus_addr = addr;
        self.color_thrust.depth_buffer_bus_enable = true;
        self.color_thrust.depth_buffer_bus_write = true;
        self.color_thrust.depth_buffer_bus_write_byte_enable = 0xffff;
        self.color_thrust.depth_buffer_bus_write_data = data;
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.depth_buffer_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.depth_buffer_bus_enable = false;
        self.color_thrust.prop();
    }

    fn read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        self.color_thrust.depth_buffer_bus_addr = addr;
        self.color_thrust.depth_buffer_bus_enable = true;
        self.color_thrust.depth_buffer_bus_write = false;
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.depth_buffer_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.depth_buffer_bus_enable = false;
        self.color_thrust.prop();
        while !self.color_thrust.depth_buffer_bus_read_data_valid {
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
        }
        self.color_thrust.depth_buffer_bus_read_data
    }

    fn write_tex_buffer_word(&mut self, addr: u32, data: u128) {
        // TODO: Not sure it makes sense to have this as part of the `Device` trait anymore.
        //  On one hand, it's nice that a whole "system" is present so we can bootstrap a working renderer.
        //  On the other hand, this extends the coverage of the `Device` concept beyond just a ColorThrust module.
        // TODO: Non-linear swizzling for better hit rate (be sure to measure/compare first!)
        self.tex_buffer[addr as usize] = data;
    }
}
