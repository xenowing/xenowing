use crate::model_device::color_thrust::*;

use rtl_meta::bit_pusher::*;

pub struct BitPusher {
    direction: Direction,
    num_words: u32,

    sys_addr_unit: AddrUnit,
    mem_addr_unit: AddrUnit,
}

impl BitPusher {
    pub fn new() -> BitPusher {
        BitPusher {
            direction: Direction::Mem2Sys,
            num_words: 0,

            sys_addr_unit: AddrUnit::new(),
            mem_addr_unit: AddrUnit::new(),
        }
    }

    pub fn write_reg(&mut self, addr: u32, data: u32, mem: &mut [u128], color_thrust: &mut ColorThrust) {
        match addr {
            REG_START_ADDR => self.transfer(mem, color_thrust),
            REG_DIRECTION_ADDR => {
                self.direction = match data & REG_DIRECTION_BITS {
                    REG_DIRECTION_MEM2SYS => Direction::Mem2Sys,
                    REG_DIRECTION_SYS2MEM => Direction::Sys2Mem,
                    _ => unreachable!()
                }
            }
            REG_NUM_WORDS_ADDR => {
                self.num_words = data;
            }
            REG_SYS_ADDR_ADDR => {
                self.sys_addr_unit.addr = data >> 4;
            }
            REG_SYS_WORDS_PER_SPAN_ADDR => {
                self.sys_addr_unit.words_per_span = data;
            }
            REG_SYS_SPAN_STRIDE_ADDR => {
                self.sys_addr_unit.span_stride = data;
            }
            REG_MEM_ADDR_ADDR => {
                self.mem_addr_unit.addr = data >> 4;
            }
            REG_MEM_WORDS_PER_SPAN_ADDR => {
                self.mem_addr_unit.words_per_span = data;
            }
            REG_MEM_SPAN_STRIDE_ADDR => {
                self.mem_addr_unit.span_stride = data;
            }
            _ => panic!("Unrecognized addr: {}", addr)
        }
    }

    pub fn read_reg(&mut self, _addr: u32) -> u32 {
        // All regs are write-only, with the exceptoin of the status reg,
        // which is useless here, since we always perform transfers
        // immediately and "instantaneously."
        0
    }

    fn transfer(&mut self, mem: &mut [u128], color_thrust: &mut ColorThrust) {
        self.sys_addr_unit.start_transfer();
        self.mem_addr_unit.start_transfer();

        loop {
            // TODO: This is ugly and can probably be done in a more unified way with the rest of the code..
            let sys_base = (self.sys_addr_unit.addr >> 20) & 0x0f; // TODO: Proper constant(s)
            let sys_offset = self.sys_addr_unit.addr & ((1 << 20) - 1); // TODO: Proper constant(s)
            match self.direction {
                Direction::Mem2Sys => {
                    let word = mem[self.mem_addr_unit.addr as usize];

                    match sys_base {
                        0x04 => color_thrust.write_color_buffer_word(sys_offset, word), // TODO: Proper constant
                        0x05 => color_thrust.write_depth_buffer_word(sys_offset, word), // TODO: Proper constant
                        _ => panic!("Unrecognized sys addr: 0x{:08x}", self.sys_addr_unit.addr << 4)
                    }
                }
                Direction::Sys2Mem => {
                    let word = match sys_base {
                        0x04 => color_thrust.read_color_buffer_word(sys_offset), // TODO: Proper constant
                        0x05 => color_thrust.read_depth_buffer_word(sys_offset), // TODO: Proper constant
                        _ => panic!("Unrecognized sys addr: 0x{:08x}", self.sys_addr_unit.addr << 4)
                    };

                    mem[self.mem_addr_unit.addr as usize] = word;
                }
            }

            self.sys_addr_unit.step();
            self.mem_addr_unit.step();

            self.num_words -= 1;
            if self.num_words == 0 {
                break;
            }
        }
    }
}

enum Direction {
    Mem2Sys,
    Sys2Mem,
}

struct AddrUnit {
    addr: u32,
    words_per_span: u32,
    span_stride: u32,

    span_base: u32,
    span_word_counter: u32,
}

impl AddrUnit {
    fn new() -> AddrUnit {
        AddrUnit {
            addr: 0,
            words_per_span: 0,
            span_stride: 0,

            span_base: 0,
            span_word_counter: 0,
        }
    }

    fn start_transfer(&mut self) {
        self.span_base = self.addr;
        self.span_word_counter = 0;
    }

    fn step(&mut self) {
        self.addr += 1;
        self.span_word_counter += 1;
        if self.span_word_counter == self.words_per_span {
            self.span_base += self.span_stride;
            self.addr = self.span_base;
            self.span_word_counter = 0;
        }
    }
}
