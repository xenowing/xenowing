enum Command {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Sll,
    Srl,
    Sra,
    Eq,
    Ne,
    Lt,
    Ltu,
    Ge,
    Geu,
}

#[repr(C)]
pub struct Env {
    get_command: extern "C" fn() -> u32,
    set_command: extern "C" fn(value: u32),

    get_lhs: extern "C" fn() -> u32,
    set_lhs: extern "C" fn(value: u32),
    get_rhs: extern "C" fn() -> u32,
    set_rhs: extern "C" fn(value: u32),

    get_res: extern "C" fn() -> u32,
    set_res: extern "C" fn(value: u32),

    eval: extern "C" fn(),
    final_: extern "C" fn(),
}

struct Alu {
    env: *const Env,
}

impl Alu {
    fn command(&self) -> Command {
        let value = unsafe { ((*self.env).get_command)() };
        match value {
            0 => Command::Add,
            1 => Command::Sub,
            2 => Command::And,
            3 => Command::Or,
            4 => Command::Xor,
            5 => Command::Sll,
            6 => Command::Srl,
            7 => Command::Sra,
            8 => Command::Eq,
            9 => Command::Ne,
            10 => Command::Lt,
            11 => Command::Ltu,
            12 => Command::Ge,
            13 => Command::Geu,
            _ => panic!("Got invalid command value: {}", value)
        }
    }

    fn set_command(&mut self, value: Command) {
        unsafe {
            ((*self.env).set_command)(value as _);
        }
    }

    fn lhs(&self) -> u32 {
        unsafe { ((*self.env).get_lhs)() }
    }

    fn set_lhs(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_lhs)(value);
        }
    }

    fn rhs(&self) -> u32 {
        unsafe { ((*self.env).get_rhs)() }
    }

    fn set_rhs(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_rhs)(value);
        }
    }

    fn res(&self) -> u32 {
        unsafe { ((*self.env).get_res)() }
    }

    fn set_res(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_res)(value);
        }
    }

    fn eval(&mut self) {
        unsafe {
            ((*self.env).eval)();
        }
    }

    fn final_(&mut self) {
        unsafe {
            ((*self.env).final_)();
        }
    }
}

#[no_mangle]
pub extern "C" fn run(env: *const Env) -> i32 {
    let mut alu = Alu {
        env: env,
    };

    alu.set_command(Command::Sra);
    alu.set_lhs(0xdeadbeef);
    alu.set_rhs(36);
    alu.eval();
    println!("res: 0x{:08x}", alu.res());

    alu.final_();

    0
}
