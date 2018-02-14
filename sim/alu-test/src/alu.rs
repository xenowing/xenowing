#![allow(dead_code)]

use op::*;

#[repr(C)]
pub struct Env {
    get_op: extern "C" fn() -> u32,
    set_op: extern "C" fn(value: u32),

    get_lhs: extern "C" fn() -> u32,
    set_lhs: extern "C" fn(value: u32),
    get_rhs: extern "C" fn() -> u32,
    set_rhs: extern "C" fn(value: u32),

    get_res: extern "C" fn() -> u32,
    set_res: extern "C" fn(value: u32),

    eval: extern "C" fn(),
    final_: extern "C" fn(),
}

pub struct Alu {
    env: *const Env,
}

impl Alu {
    pub fn new(env: *const Env) -> Alu {
        Alu {
            env: env,
        }
    }

    pub fn op(&self) -> Op {
        let value = unsafe { ((*self.env).get_op)() };
        match value {
            0 => Op::Add,
            1 => Op::Sub,
            2 => Op::And,
            3 => Op::Or,
            4 => Op::Xor,
            5 => Op::Sll,
            6 => Op::Srl,
            7 => Op::Sra,
            8 => Op::Eq,
            9 => Op::Ne,
            10 => Op::Lt,
            11 => Op::Ltu,
            12 => Op::Ge,
            13 => Op::Geu,
            _ => panic!("Got invalid op value: {}", value)
        }
    }

    pub fn set_op(&mut self, value: Op) {
        unsafe {
            ((*self.env).set_op)(value as _);
        }
    }

    pub fn lhs(&self) -> u32 {
        unsafe { ((*self.env).get_lhs)() }
    }

    pub fn set_lhs(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_lhs)(value);
        }
    }

    pub fn rhs(&self) -> u32 {
        unsafe { ((*self.env).get_rhs)() }
    }

    pub fn set_rhs(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_rhs)(value);
        }
    }

    pub fn res(&self) -> u32 {
        unsafe { ((*self.env).get_res)() }
    }

    pub fn set_res(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_res)(value);
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
}
