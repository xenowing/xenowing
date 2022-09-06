use abstract_device::*;

use rtl_meta::bit_pusher::*;

pub fn mem2sys(
    device: &mut impl Device,
    sys_addr: u32,
    sys_words_per_span: u32,
    sys_span_stride: u32,
    mem_addr: u32,
    mem_words_per_span: u32,
    mem_span_stride: u32,
    num_words: u32,
) {
    device.bit_pusher_write_reg(REG_DIRECTION_ADDR, REG_DIRECTION_MEM2SYS);
    device.bit_pusher_write_reg(REG_NUM_WORDS_ADDR, num_words);

    device.bit_pusher_write_reg(REG_SYS_ADDR_ADDR, sys_addr);
    device.bit_pusher_write_reg(REG_SYS_WORDS_PER_SPAN_ADDR, sys_words_per_span);
    device.bit_pusher_write_reg(REG_SYS_SPAN_STRIDE_ADDR, sys_span_stride);

    device.bit_pusher_write_reg(REG_MEM_ADDR_ADDR, mem_addr);
    device.bit_pusher_write_reg(REG_MEM_WORDS_PER_SPAN_ADDR, mem_words_per_span);
    device.bit_pusher_write_reg(REG_MEM_SPAN_STRIDE_ADDR, mem_span_stride);

    // Dispatch transfer
    device.bit_pusher_write_reg(REG_START_ADDR, 1);

    // Wait until transfer is complete
    while device.bit_pusher_read_reg(REG_STATUS_ADDR) != 0 {
        // Do nothing
    }
}

pub fn sys2mem(
    device: &mut impl Device,
    sys_addr: u32,
    sys_words_per_span: u32,
    sys_span_stride: u32,
    mem_addr: u32,
    mem_words_per_span: u32,
    mem_span_stride: u32,
    num_words: u32,
) {
    device.bit_pusher_write_reg(REG_DIRECTION_ADDR, REG_DIRECTION_SYS2MEM);
    device.bit_pusher_write_reg(REG_NUM_WORDS_ADDR, num_words);

    device.bit_pusher_write_reg(REG_SYS_ADDR_ADDR, sys_addr);
    device.bit_pusher_write_reg(REG_SYS_WORDS_PER_SPAN_ADDR, sys_words_per_span);
    device.bit_pusher_write_reg(REG_SYS_SPAN_STRIDE_ADDR, sys_span_stride);

    device.bit_pusher_write_reg(REG_MEM_ADDR_ADDR, mem_addr);
    device.bit_pusher_write_reg(REG_MEM_WORDS_PER_SPAN_ADDR, mem_words_per_span);
    device.bit_pusher_write_reg(REG_MEM_SPAN_STRIDE_ADDR, mem_span_stride);

    // Dispatch transfer
    device.bit_pusher_write_reg(REG_START_ADDR, 1);

    // Wait until transfer is complete
    while device.bit_pusher_read_reg(REG_STATUS_ADDR) != 0 {
        // Do nothing
    }
}
