// TODO: Move
pub const REG_BUS_ADDR_BITS: u32 = 20;
pub const REG_BUS_ADDR_BIT_WIDTH: u32 = 4;

pub const REG_STATUS_ADDR: u32 = 0;
pub const REG_START_ADDR: u32 = 0;

pub const REG_DIRECTION_ADDR: u32 = 1;
pub const REG_DIRECTION_BITS: u32 = 1;
pub const REG_DIRECTION_MEM2SYS: u32 = 0;
pub const REG_DIRECTION_SYS2MEM: u32 = 1;

pub const REG_NUM_WORDS_ADDR: u32 = 2;

pub const REG_SYS_ADDR_ADDR: u32 = 3;

pub const REG_SYS_WORDS_PER_SPAN_ADDR: u32 = 4;

pub const REG_SYS_SPAN_STRIDE_ADDR: u32 = 5;

pub const REG_MEM_ADDR_ADDR: u32 = 6;

pub const REG_MEM_WORDS_PER_SPAN_ADDR: u32 = 7;

pub const REG_MEM_SPAN_STRIDE_ADDR: u32 = 8;
