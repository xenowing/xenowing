use kaze::*;

pub fn reg_next<'a, S: Into<String>>(name: S, next: &'a Signal<'a>, m: &'a Module<'a>) -> &'a Signal<'a> {
    let reg = m.reg(name, next.bit_width());
    reg.drive_next(next);
    reg.value
}

pub fn reg_next_with_default<'a, S: Into<String>, C: Into<Constant>>(name: S, next: &'a Signal<'a>, default_value: C, m: &'a Module<'a>) -> &'a Signal<'a> {
    let reg = m.reg(name, next.bit_width());
    reg.drive_next(next);
    reg.default_value(default_value);
    reg.value
}
