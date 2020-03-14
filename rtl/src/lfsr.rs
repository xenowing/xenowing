use kaze::*;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Lfsr");

    let state = m.reg("state", 16);
    state.default_value(0xace1u32);
    m.output("value", state.value.bits(7, 0));

    state.drive_next(if_(m.input("shift_enable", 1), {
        let feedback_bit = state.value.bit(0) ^ state.value.bit(2) ^ state.value.bit(3) ^ state.value.bit(5);
        feedback_bit.concat(state.value.bits(15, 1))
    }).else_({
        state.value
    }));

    m
}
