use crate::xenowing::*;

pub const TILE_DIM_BITS: u32 = 5;
pub const TILE_DIM: u32 = 1 << TILE_DIM_BITS;
pub const TILE_PIXELS_BITS: u32 = TILE_DIM_BITS * 2;
pub const TILE_PIXELS: u32 = 1 << TILE_PIXELS_BITS;
pub const TILE_PIXELS_WORDS_BITS: u32 = TILE_PIXELS_BITS - 2;

// TODO: Move
pub const TEX_WORD_ADDR_BITS: u32 = SYSTEM_BUS_ADDR_BITS;

pub const EDGE_FRACT_BITS: u32 = 3;
// 8 bit component + 1 guard bit for clamping on overflow + 1 sign bit for clamping on underflow
pub const COLOR_WHOLE_BITS: u32 = 8 + 1 + 1;
// 8 whole bits (minus guard/sign bits) + 8 fractional bits
//  Equivalent to 16 fractional color bits (interpreting components as [0, 1) instead of [0, 256),
//  and ignoring guard/sign bits, which end up in the whole part of the component input)
pub const COLOR_FRACT_BITS: u32 = 8;
pub const W_INVERSE_FRACT_BITS: u32 = 30;
// Must be 16
//  TODO: Recall and document why
pub const Z_FRACT_BITS: u32 = 16;
pub const ST_FRACT_BITS: u32 = 16;
pub const ST_FILTER_FRACT_BITS: u32 = 4; // Must be less than ST_FRACT_BITS
pub const RESTORED_W_FRACT_BITS: u32 = 8; // Must be less than W_INVERSE_FRACT_BITS and ST_FRACT_BITS

// TODO: Move
pub const REG_BUS_ADDR_BITS: u32 = 20;
pub const REG_BUS_ADDR_BIT_WIDTH: u32 = 6;

pub const REG_STATUS_ADDR: u32 = 0;
pub const REG_START_ADDR: u32 = 0;

pub const REG_TEX_CACHE_INVALIDATE_ADDR: u32 = 1;

pub const REG_DEPTH_SETTINGS_ADDR: u32 = 2;
pub const REG_DEPTH_SETTINGS_BITS: u32 = 2;
pub const REG_DEPTH_TEST_ENABLE_BIT: u32 = 0;
pub const REG_DEPTH_WRITE_MASK_ENABLE_BIT: u32 = 1;

pub const REG_TEXTURE_SETTINGS_ADDR: u32 = 3;
pub const REG_TEXTURE_SETTINGS_BITS: u32 = 3;
pub const REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT_OFFSET: u32 = 0;
pub const REG_TEXTURE_SETTINGS_FILTER_SELECT_BITS: u32 = 1;
pub const REG_TEXTURE_SETTINGS_FILTER_SELECT_NEAREST: u32 = 0;
pub const REG_TEXTURE_SETTINGS_FILTER_SELECT_BILINEAR: u32 = 1;
pub const REG_TEXTURE_SETTINGS_DIM_BIT_OFFSET: u32 = REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT_OFFSET + REG_TEXTURE_SETTINGS_FILTER_SELECT_BITS;
pub const REG_TEXTURE_SETTINGS_DIM_BITS: u32 = 2;
pub const REG_TEXTURE_SETTINGS_DIM_16: u32 = 0;
pub const REG_TEXTURE_SETTINGS_DIM_32: u32 = 1;
pub const REG_TEXTURE_SETTINGS_DIM_64: u32 = 2;
pub const REG_TEXTURE_SETTINGS_DIM_128: u32 = 3;

pub const REG_TEXTURE_BASE_ADDR: u32 = 4;
// Subtract 6 bits for 2x2x2 texel coord swizzling dims
pub const REG_TEXTURE_BASE_BITS: u32 = TEX_WORD_ADDR_BITS - 6;

pub const REG_BLEND_SETTINGS_ADDR: u32 = 5;
pub const REG_BLEND_SETTINGS_BITS: u32 = 4;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET: u32 = 0;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_BITS: u32 = 2;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_ZERO: u32 = 0;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_ONE: u32 = 1;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_SRC_ALPHA: u32 = 2;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_ONE_MINUS_SRC_ALPHA: u32 = 3;
pub const REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET: u32 = REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET + REG_BLEND_SETTINGS_SRC_FACTOR_BITS;
pub const REG_BLEND_SETTINGS_DST_FACTOR_BITS: u32 = 2;
pub const REG_BLEND_SETTINGS_DST_FACTOR_ZERO: u32 = 0;
pub const REG_BLEND_SETTINGS_DST_FACTOR_ONE: u32 = 1;
pub const REG_BLEND_SETTINGS_DST_FACTOR_SRC_ALPHA: u32 = 2;
pub const REG_BLEND_SETTINGS_DST_FACTOR_ONE_MINUS_SRC_ALPHA: u32 = 3;

pub const REG_W0_MIN_ADDR: u32 = 6;
pub const REG_W0_DX_ADDR: u32 = 7;
pub const REG_W0_DY_ADDR: u32 = 8;
pub const REG_W1_MIN_ADDR: u32 = 9;
pub const REG_W1_DX_ADDR: u32 = 10;
pub const REG_W1_DY_ADDR: u32 = 11;
pub const REG_W2_MIN_ADDR: u32 = 12;
pub const REG_W2_DX_ADDR: u32 = 13;
pub const REG_W2_DY_ADDR: u32 = 14;
pub const REG_R_MIN_ADDR: u32 = 15;
pub const REG_R_DX_ADDR: u32 = 16;
pub const REG_R_DY_ADDR: u32 = 17;
pub const REG_G_MIN_ADDR: u32 = 18;
pub const REG_G_DX_ADDR: u32 = 19;
pub const REG_G_DY_ADDR: u32 = 20;
pub const REG_B_MIN_ADDR: u32 = 21;
pub const REG_B_DX_ADDR: u32 = 22;
pub const REG_B_DY_ADDR: u32 = 23;
pub const REG_A_MIN_ADDR: u32 = 24;
pub const REG_A_DX_ADDR: u32 = 25;
pub const REG_A_DY_ADDR: u32 = 26;
pub const REG_W_INVERSE_MIN_ADDR: u32 = 27;
pub const REG_W_INVERSE_DX_ADDR: u32 = 28;
pub const REG_W_INVERSE_DY_ADDR: u32 = 29;
pub const REG_Z_MIN_ADDR: u32 = 30;
pub const REG_Z_DX_ADDR: u32 = 31;
pub const REG_Z_DY_ADDR: u32 = 32;
pub const REG_S_MIN_ADDR: u32 = 33;
pub const REG_S_DX_ADDR: u32 = 34;
pub const REG_S_DY_ADDR: u32 = 35;
pub const REG_T_MIN_ADDR: u32 = 36;
pub const REG_T_DX_ADDR: u32 = 37;
pub const REG_T_DY_ADDR: u32 = 38;
