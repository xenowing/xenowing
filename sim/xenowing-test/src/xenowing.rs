#![allow(dead_code)]

#[repr(C)]
pub struct Env {
    get_reset_n: extern "C" fn() -> u32,
    set_reset_n: extern "C" fn(value: u32),
    get_clk: extern "C" fn() -> u32,
    set_clk: extern "C" fn(value: u32),

    get_program_rom_addr: extern "C" fn() -> u32,
    set_program_rom_addr: extern "C" fn(value: u32),
    get_program_rom_q: extern "C" fn() -> u32,
    set_program_rom_q: extern "C" fn(value: u32),

    get_leds: extern "C" fn() -> u32,
    set_leds: extern "C" fn(value: u32),

    eval: extern "C" fn(),
    final_: extern "C" fn(),
    trace_dump: extern "C" fn(time: u64),
}

pub struct Xenowing {
    env: *const Env,
}

impl Xenowing {
    pub fn new(env: *const Env) -> Xenowing {
        Xenowing {
            env: env,
        }
    }

    pub fn reset_n(&self) -> bool {
        unsafe { ((*self.env).get_reset_n)() != 0 }
    }

    pub fn set_reset_n(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_reset_n)(if value { 1 } else { 0 });
        }
    }

    pub fn clk(&self) -> bool {
        unsafe { ((*self.env).get_clk)() != 0 }
    }

    pub fn set_clk(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_clk)(if value { 1 } else { 0 });
        }
    }

    pub fn program_rom_addr(&self) -> u32 {
        unsafe { ((*self.env).get_program_rom_addr)() }
    }

    pub fn set_program_rom_addr(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_program_rom_addr)(value);
        }
    }

    pub fn program_rom_q(&self) -> u32 {
        unsafe { ((*self.env).get_program_rom_q)() }
    }

    pub fn set_program_rom_q(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_program_rom_q)(value);
        }
    }

    pub fn leds(&self) -> u32 {
        unsafe { ((*self.env).get_leds)() }
    }

    pub fn set_leds(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_leds)(value);
        }
    }

    pub fn eval(&mut self) {
        unsafe {
            ((*self.env).eval)();
        }
    }

    pub fn final_(&mut self) {
        unsafe {
            ((*self.env).final_)();
        }
    }

    pub fn trace_dump(&mut self, time: u64) {
        unsafe {
            ((*self.env).trace_dump)(time);
        }
    }
}
