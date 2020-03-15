use kaze::*;

pub fn reg_next<'a, S: Into<String>>(name: S, next: &'a Signal<'a>, m: &'a Module<'a>) -> &'a Signal<'a> {
    let reg = m.reg(name, next.bit_width());
    reg.drive_next(next);
    reg.value
}
