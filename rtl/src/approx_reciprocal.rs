use crate::helpers::*;

use kaze::*;

// 32 bit internal resolution, 32 - `fract_bits` integral bits, `fract_bits` fractional bits, 1 + 2 * `refinement_stages` cycles latency
pub fn generate<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, fract_bits: u32, refinement_stages: u32) -> &'a Module<'a> {
    let m = c.module(mod_name);

    let x = m.input("x", 32);

    // Prep stage
    let shl = leading_zeros(x, m);
    let normalized_x = x << shl;

    let mut shr = m.lit(64 - 2 * fract_bits, 5) - shl;

    let mut e = !normalized_x;
    let mut q = e;

    // Refinement stages
    for stage in 0..refinement_stages {
        shr = reg_next(format!("refinement_stage_{}_shr", stage), shr, m);

        e = reg_next(format!("refinement_stage_{}_e", stage), e, m);
        q = reg_next(format!("refinement_stage_{}_q", stage), q, m);

        let prev_q = q;

        q = (q * e).bits(63, 32);
        e = (e * e).bits(63, 32);

        e = reg_next(format!("refinement_stage_buffer_{}_e", stage), e, m);
        q = reg_next(format!("refinement_stage_buffer_{}_q", stage), q, m);

        q = q + prev_q;
    }

    // Shift stage
    let shr = reg_next("shift_stage_shr", shr, m);

    let q = reg_next("shift_stage_q", q, m);

    let res = (q >> shr) | (m.lit(1u32, 32) << (m.lit(32u32, 6) - m.low().concat(shr)));

    // Output
    m.output("quotient", res);

    m
}

fn leading_zeros<'a>(x: &'a Signal<'a>, m: &'a Module<'a>) -> &'a Signal<'a> {
    let mut ret = m.lit(0u32, 5);

    for i in 0..31 {
        ret = if_(x.bit(i), {
            m.lit(31 - i, 5)
        }).else_({
            ret
        });
    }

    ret
}
