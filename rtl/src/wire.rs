use kaze::*;

pub struct Wire<'a> {
    pub m: &'a Module<'a>,
    pub i: &'a Input<'a>,
    pub o: &'a Output<'a>,
}

impl<'a> Wire<'a> {
    pub fn new(instance_name: impl Into<String>, bit_width: u32, p: &'a impl ModuleParent<'a>) -> Wire<'a> {
        let m = p.module(instance_name, "Wire");

        let i = m.input("i", bit_width);
        let o = m.output("o", i);

        Wire {
            m,
            i,
            o,
        }
    }
}
