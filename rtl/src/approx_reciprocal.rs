use kaze::*;

// 32 bit internal resolution, 32 - `fract_bits` integral bits, `fract_bits` fractional bits, 1 + 3 * `refinement_stages` cycles latency
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
        shr = shr.reg_next(format!("refinement_stage_{}_shr", stage));

        e = e.reg_next(format!("refinement_stage_{}_e", stage));
        q = q.reg_next(format!("refinement_stage_{}_q", stage));

        let mut prev_q = q;

        q = (q * e).bits(63, 32);
        e = (e * e).bits(63, 32);

        // Buffer/pipeline regs to meet timing for multiplies
        for i in 0..2 {
            shr = shr.reg_next(format!("refinement_stage_{}_buffer_{}_shr", stage, i));

            e = e.reg_next(format!("refinement_stage_{}_buffer_{}_e", stage, i));
            q = q.reg_next(format!("refinement_stage_{}_buffer_{}_q", stage, i));

            prev_q = prev_q.reg_next(format!("refinement_stage_{}_buffer_{}_prev_q", stage, i));
        }

        q = q + prev_q;
    }

    // Shift stage
    let shr = shr.reg_next("shift_stage_shr");

    let q = q.reg_next("shift_stage_q");

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
