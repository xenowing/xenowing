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

    top: Top,
}

impl SimDevice {
    pub fn new() -> SimDevice {
        let mut top = Top::new();
        top.reset();
        top.color_buffer_bus_enable = false;
        top.depth_buffer_bus_enable = false;
        top.reg_bus_enable = false;
        top.mem_bus_enable = false;
        top.prop();

        SimDevice {
            allocations: Vec::new(),

            top,
        }
    }
}

impl Device for SimDevice {
    fn mem_alloc(&mut self, num_words: u32, align_words: u32) -> u32 {
        let size = num_words * 16;
        let align = align_words * 16;
        let mut start = 0;
        let mut end = start + size;
        let mut insert_index = 0;
        for (i, allocation) in self.allocations.iter().enumerate() {
            let allocation_end = allocation.end();
            if (start >= allocation.start && start < allocation_end) ||
                (end >= allocation.start && end < allocation_end) ||
                (allocation.start >= start && allocation.start < end) ||
                (allocation_end >= start && allocation_end < end) {
                start = (allocation_end + align - 1) / align * align;
                end = start + size;
                insert_index = i + 1;
            }
        }
        if end >= MEM_NUM_BYTES {
            panic!("Out of device memory");
        }
        self.allocations.insert(insert_index, Allocation {
            start,
            size,
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
        self.top.mem_bus_addr = addr / 16;
        self.top.mem_bus_enable = true;
        self.top.mem_bus_write = true;
        self.top.mem_bus_write_byte_enable = 0xffff;
        self.top.mem_bus_write_data = data;
        self.top.prop();
        loop {
            let ready = self.top.mem_bus_ready;
            self.top.posedge_clk();
            self.top.prop();
            if ready {
                break;
            }
        }
        self.top.mem_bus_enable = false;
        self.top.prop();
    }

    fn mem_read_word(&mut self, addr: u32) -> u128 {
        if (addr % 16) != 0 {
            panic!("Unaligned device memory access");
        }
        self.top.mem_bus_addr = addr / 16;
        self.top.mem_bus_enable = true;
        self.top.mem_bus_write = false;
        self.top.prop();
        loop {
            let ready = self.top.mem_bus_ready;
            self.top.posedge_clk();
            self.top.prop();
            if ready {
                break;
            }
        }
        self.top.mem_bus_enable = false;
        while !self.top.mem_bus_read_data_valid {
            self.top.posedge_clk();
            self.top.prop();
        }
        self.top.mem_bus_read_data
    }

    fn color_thrust_write_reg(&mut self, addr: u32, data: u32) {
        self.top.reg_bus_addr = addr;
        self.top.reg_bus_enable = true;
        self.top.reg_bus_write = true;
        self.top.reg_bus_write_data = data as _;
        self.top.prop();
        loop {
            let ready = self.top.reg_bus_ready;
            self.top.posedge_clk();
            self.top.prop();
            if ready {
                break;
            }
        }
        self.top.reg_bus_enable = false;
        self.top.prop();
    }

    fn color_thrust_read_reg(&mut self, addr: u32) -> u32 {
        self.top.reg_bus_addr = addr;
        self.top.reg_bus_enable = true;
        self.top.reg_bus_write = false;
        loop {
            let ready = self.top.reg_bus_ready;
            self.top.posedge_clk();
            self.top.prop();
            if ready {
                break;
            }
        }
        self.top.reg_bus_enable = false;
        while !self.top.reg_bus_read_data_valid {
            self.top.posedge_clk();
            self.top.prop();
        }
        self.top.reg_bus_read_data as _
    }

    fn color_thrust_write_color_buffer_word(&mut self, addr: u32, data: u128) {
        self.top.color_buffer_bus_addr = addr;
        self.top.color_buffer_bus_enable = true;
        self.top.color_buffer_bus_write = true;
        self.top.color_buffer_bus_write_byte_enable = 0xffff;
        self.top.color_buffer_bus_write_data = data;
        self.top.prop();
        loop {
            let ready = self.top.color_buffer_bus_ready;
            self.top.posedge_clk();
            self.top.prop();
            if ready {
                break;
            }
        }
        self.top.color_buffer_bus_enable = false;
        self.top.prop();
    }

    fn color_thrust_read_color_buffer_word(&mut self, addr: u32) -> u128 {
        self.top.color_buffer_bus_addr = addr;
        self.top.color_buffer_bus_enable = true;
        self.top.color_buffer_bus_write = false;
        self.top.prop();
        loop {
            let ready = self.top.color_buffer_bus_ready;
            self.top.posedge_clk();
            self.top.prop();
            if ready {
                break;
            }
        }
        self.top.color_buffer_bus_enable = false;
        self.top.prop();
        while !self.top.color_buffer_bus_read_data_valid {
            self.top.posedge_clk();
            self.top.prop();
        }
        self.top.color_buffer_bus_read_data
    }

    fn color_thrust_write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        self.top.depth_buffer_bus_addr = addr;
        self.top.depth_buffer_bus_enable = true;
        self.top.depth_buffer_bus_write = true;
        self.top.depth_buffer_bus_write_byte_enable = 0xffff;
        self.top.depth_buffer_bus_write_data = data;
        self.top.prop();
        loop {
            let ready = self.top.depth_buffer_bus_ready;
            self.top.posedge_clk();
            self.top.prop();
            if ready {
                break;
            }
        }
        self.top.depth_buffer_bus_enable = false;
        self.top.prop();
    }

    fn color_thrust_read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        self.top.depth_buffer_bus_addr = addr;
        self.top.depth_buffer_bus_enable = true;
        self.top.depth_buffer_bus_write = false;
        self.top.prop();
        loop {
            let ready = self.top.depth_buffer_bus_ready;
            self.top.posedge_clk();
            self.top.prop();
            if ready {
                break;
            }
        }
        self.top.depth_buffer_bus_enable = false;
        self.top.prop();
        while !self.top.depth_buffer_bus_read_data_valid {
            self.top.posedge_clk();
            self.top.prop();
        }
        self.top.depth_buffer_bus_read_data
    }
}
