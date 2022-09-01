//#![feature(core_intrinsics)]
#![no_std]

mod tab {
    include!(concat!(env!("OUT_DIR"), "/tab.rs"));
}

use tab::*;

//use core::intrinsics;

pub fn cos(x: f32) -> f32 {
    sin(x + core::f32::consts::PI / 2.0)
}

pub fn sin(x: f32) -> f32 {
    let phase_scale = 1.0 / core::f32::consts::TAU;
    let phase = x * phase_scale;
    let phase = phase - (phase as u32 as f32);//unsafe { intrinsics::floorf32(phase) };
    let phase_with_offset = phase + 1.0;
    let bits = phase_with_offset.to_bits();
    const NUM_SIGNIFICAND_BITS: usize = 23;
    let significand = bits & ((1 << NUM_SIGNIFICAND_BITS) - 1);
    let index = (significand >> (NUM_SIGNIFICAND_BITS - NUM_ENTRIES_BITS)) as usize;
    f32::from_bits(SIN_TAB[index])
}

pub fn tan(x: f32) -> f32 {
    sin(x) / cos(x)
}
