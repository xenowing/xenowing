use crate::modules::*;

use abstract_device::*;

// TODO: Dedupe and move!
const MEM_ADDR_BITS: u32 = 24;
const MEM_NUM_WORDS: u32 = 1 << MEM_ADDR_BITS;
const MEM_NUM_BYTES: u32 = MEM_NUM_WORDS << 4;

struct Allocation {
    start: u32,
    size: u32,
}

impl Allocation {
    fn end(&self) -> u32 {
        self.start + self.size
    }
}

pub struct SimDevice {
    allocations: Vec<Allocation>,

    color_thrust: Top,
}

impl SimDevice {
    pub fn new() -> SimDevice {
        let mut color_thrust = Top::new();
        color_thrust.reset();
        color_thrust.color_buffer_bus_enable = false;
        color_thrust.depth_buffer_bus_enable = false;
        color_thrust.reg_bus_enable = false;
        color_thrust.mem_bus_enable = false;
        color_thrust.prop();

        SimDevice {
            allocations: Vec::new(),

            color_thrust,
        }
    }
}

impl Device for SimDevice {
    fn mem_alloc(&mut self, num_words: u32, align_words: u32) -> u32 {
        let align = align_words * 16;
        let start = (self.allocations.last().map_or(0, |a| a.end()) + align - 1) / align * align;
        if start >= MEM_NUM_BYTES {
            panic!("Out of device memory");
        }
        self.allocations.push(Allocation {
            start,
            size: num_words * 16,
        });
        start
    }

    fn mem_dealloc(&mut self, _addr: u32) {
        todo!()
    }

    fn mem_write_word(&mut self, addr: u32, data: u128) {
        if (addr % 16) != 0 {
            panic!("Unaligned device memory access");
        }
        self.color_thrust.mem_bus_addr = addr / 16;
        self.color_thrust.mem_bus_enable = true;
        self.color_thrust.mem_bus_write = true;
        self.color_thrust.mem_bus_write_byte_enable = 0xffff;
        self.color_thrust.mem_bus_write_data = data;
        self.color_thrust.prop();
        loop {
            let ready = self.color_thrust.mem_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.mem_bus_enable = false;
        self.color_thrust.prop();
    }

    fn color_thrust_write_reg(&mut self, addr: u32, data: u32) {
        self.color_thrust.reg_bus_addr = addr;
        self.color_thrust.reg_bus_enable = true;
        self.color_thrust.reg_bus_write = true;
        self.color_thrust.reg_bus_write_data = data as _;
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

    fn color_thrust_read_reg(&mut self, addr: u32) -> u32 {
        self.color_thrust.reg_bus_addr = addr;
        self.color_thrust.reg_bus_enable = true;
        self.color_thrust.reg_bus_write = false;
        loop {
            let ready = self.color_thrust.reg_bus_ready;
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
            if ready {
                break;
            }
        }
        self.color_thrust.reg_bus_enable = false;
        while !self.color_thrust.reg_bus_read_data_valid {
            self.color_thrust.posedge_clk();
            self.color_thrust.prop();
        }
        self.color_thrust.reg_bus_read_data as _
    }

    fn color_thrust_write_color_buffer_word(&mut self, addr: u32, data: u128) {
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

    fn color_thrust_read_color_buffer_word(&mut self, addr: u32) -> u128 {
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

    fn color_thrust_write_depth_buffer_word(&mut self, addr: u32, data: u128) {
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

    fn color_thrust_read_depth_buffer_word(&mut self, addr: u32) -> u128 {
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
}
