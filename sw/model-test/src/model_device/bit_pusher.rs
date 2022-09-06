use crate::model_device::color_thrust::*;

use rtl_meta::bit_pusher::*;

enum Direction {
    Mem2Sys,
    Sys2Mem,
}

pub struct BitPusher {
    direction: Direction,
    num_words: u32,

    sys_addr: u32,
    sys_words_per_span: u32,
    sys_span_stride: u32,

    mem_addr: u32,
    mem_words_per_span: u32,
    mem_span_stride: u32,
}

impl BitPusher {
    pub fn new() -> BitPusher {
        BitPusher {
            direction: Direction::Mem2Sys,
            num_words: 0,

            sys_addr: 0,
            sys_words_per_span: 0,
            sys_span_stride: 0,

            mem_addr: 0,
            mem_words_per_span: 0,
            mem_span_stride: 0,
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
                self.sys_addr = data >> 4;
            }
            REG_SYS_WORDS_PER_SPAN_ADDR => {
                self.sys_words_per_span = data;
            }
            REG_SYS_SPAN_STRIDE_ADDR => {
                self.sys_span_stride = data;
            }
            REG_MEM_ADDR_ADDR => {
                self.mem_addr = data >> 4;
            }
            REG_MEM_WORDS_PER_SPAN_ADDR => {
                self.mem_words_per_span = data;
            }
            REG_MEM_SPAN_STRIDE_ADDR => {
                self.mem_span_stride = data;
            }
            _ => panic!("Unrecognized addr: {}", addr)
        }
    }

    pub fn read_reg(&mut self, addr: u32) -> u32 {
        match addr {
            // The model always performs copies synchronously, so we'll never see a busy status here
            REG_STATUS_ADDR => 0,
            REG_DIRECTION_ADDR => match self.direction {
                Direction::Mem2Sys => REG_DIRECTION_MEM2SYS,
                Direction::Sys2Mem => REG_DIRECTION_SYS2MEM,
            }
            REG_NUM_WORDS_ADDR => self.num_words,
            REG_SYS_ADDR_ADDR => self.sys_addr << 4,
            REG_SYS_WORDS_PER_SPAN_ADDR => self.sys_words_per_span,
            REG_SYS_SPAN_STRIDE_ADDR => self.sys_span_stride,
            REG_MEM_ADDR_ADDR => self.mem_addr << 4,
            REG_MEM_WORDS_PER_SPAN_ADDR => self.mem_words_per_span,
            REG_MEM_SPAN_STRIDE_ADDR => self.mem_span_stride,
            _ => panic!("Unrecognized addr: {}", addr)
        }
    }

    fn transfer(&mut self, mem: &mut [u128], color_thrust: &mut ColorThrust) {
        let mut sys_span_base = self.sys_addr;
        let mut sys_span_word_counter = 0;

        let mut mem_span_base = self.mem_addr;
        let mut mem_span_word_counter = 0;

        loop {
            // TODO: This is ugly and can probably be done in a more unified way with the rest of the code..
            let sys_base = (self.sys_addr >> 20) & 0x0f; // TODO: Proper constant(s)
            let sys_offset = self.sys_addr & ((1 << 20) - 1); // TODO: Proper constant(s)
            match self.direction {
                Direction::Mem2Sys => {
                    let word = mem[self.mem_addr as usize];

                    match sys_base {
                        0x04 => color_thrust.write_color_buffer_word(sys_offset, word), // TODO: Proper constant
                        0x05 => color_thrust.write_depth_buffer_word(sys_offset, word), // TODO: Proper constant
                        _ => panic!("Unrecognized sys addr: 0x{:08x}", self.sys_addr << 4)
                    }
                }
                Direction::Sys2Mem => {
                    let word = match sys_base {
                        0x04 => color_thrust.read_color_buffer_word(sys_offset), // TODO: Proper constant
                        0x05 => color_thrust.read_depth_buffer_word(sys_offset), // TODO: Proper constant
                        _ => panic!("Unrecognized sys addr: 0x{:08x}", self.sys_addr << 4)
                    };

                    mem[self.mem_addr as usize] = word;
                }
            }

            self.sys_addr += 1;
            sys_span_word_counter += 1;
            if sys_span_word_counter == self.sys_words_per_span {
                sys_span_base += self.sys_span_stride;
                self.sys_addr = sys_span_base;
                sys_span_word_counter = 0;
            }

            self.mem_addr += 1;
            mem_span_word_counter += 1;
            if mem_span_word_counter == self.mem_words_per_span {
                mem_span_base += self.mem_span_stride;
                self.mem_addr = mem_span_base;
                mem_span_word_counter = 0;
            }

            self.num_words -= 1;
            if self.num_words == 0 {
                break;
            }
        }
    }
}
