#![allow(dead_code)]

#[repr(C)]
pub struct Env {
    get_reset_n: extern "C" fn() -> u32,
    set_reset_n: extern "C" fn(value: u32),
    get_clk: extern "C" fn() -> u32,
    set_clk: extern "C" fn(value: u32),

    get_avl_ready: extern "C" fn() -> u32,
    set_avl_ready: extern "C" fn(value: u32),
    get_avl_burstbegin: extern "C" fn() -> u32,
    set_avl_burstbegin: extern "C" fn(value: u32),
    get_avl_addr: extern "C" fn() -> u32,
    set_avl_addr: extern "C" fn(value: u32),
    get_avl_rdata_valid: extern "C" fn() -> u32,
    set_avl_rdata_valid: extern "C" fn(value: u32),
    get_avl_rdata: extern "C" fn() -> u64,
    set_avl_rdata: extern "C" fn(value: u64),
    get_avl_wdata: extern "C" fn() -> u64,
    set_avl_wdata: extern "C" fn(value: u64),
    get_avl_be: extern "C" fn() -> u32,
    set_avl_be: extern "C" fn(value: u32),
    get_avl_read_req: extern "C" fn() -> u32,
    set_avl_read_req: extern "C" fn(value: u32),
    get_avl_write_req: extern "C" fn() -> u32,
    set_avl_write_req: extern "C" fn(value: u32),
    get_avl_size: extern "C" fn() -> u32,
    set_avl_size: extern "C" fn(value: u32),

    get_ddr3_init_done: extern "C" fn() -> u32,
    set_ddr3_init_done: extern "C" fn(value: u32),
    get_ddr3_cal_success: extern "C" fn() -> u32,
    set_ddr3_cal_success: extern "C" fn(value: u32),
    get_ddr3_cal_fail: extern "C" fn() -> u32,
    set_ddr3_cal_fail: extern "C" fn(value: u32),

    get_is_finished: extern "C" fn() -> u32,
    set_is_finished: extern "C" fn(value: u32),
    get_pass: extern "C" fn() -> u32,
    set_pass: extern "C" fn(value: u32),
    get_fail: extern "C" fn() -> u32,
    set_fail: extern "C" fn(value: u32),

    eval: extern "C" fn(),
    final_: extern "C" fn(),
    trace_dump: extern "C" fn(time: u64),
}

pub struct Test {
    env: *const Env,
}

impl Test {
    pub fn new(env: *const Env) -> Test {
        Test {
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

    pub fn avl_ready(&self) -> bool {
        unsafe { ((*self.env).get_avl_ready)() != 0 }
    }

    pub fn set_avl_ready(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_ready)(if value { 1 } else { 0 });
        }
    }

    pub fn avl_burstbegin(&self) -> bool {
        unsafe { ((*self.env).get_avl_burstbegin)() != 0 }
    }

    pub fn set_avl_burstbegin(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_burstbegin)(if value { 1 } else { 0 });
        }
    }

    pub fn avl_addr(&self) -> u32 {
        unsafe { ((*self.env).get_avl_addr)() }
    }

    pub fn set_avl_addr(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_avl_addr)(value);
        }
    }

    pub fn avl_rdata_valid(&self) -> bool {
        unsafe { ((*self.env).get_avl_rdata_valid)() != 0 }
    }

    pub fn set_avl_rdata_valid(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_rdata_valid)(if value { 1 } else { 0 });
        }
    }

    pub fn avl_rdata(&self) -> u64 {
        unsafe { ((*self.env).get_avl_rdata)() }
    }

    pub fn set_avl_rdata(&mut self, value: u64) {
        unsafe {
            ((*self.env).set_avl_rdata)(value);
        }
    }

    pub fn avl_wdata(&self) -> u64 {
        unsafe { ((*self.env).get_avl_wdata)() }
    }

    pub fn set_avl_wdata(&mut self, value: u64) {
        unsafe {
            ((*self.env).set_avl_wdata)(value);
        }
    }

    pub fn avl_be(&self) -> u32 {
        unsafe { ((*self.env).get_avl_be)() }
    }

    pub fn set_avl_be(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_avl_be)(value);
        }
    }

    pub fn avl_read_req(&self) -> bool {
        unsafe { ((*self.env).get_avl_read_req)() != 0 }
    }

    pub fn set_avl_read_req(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_read_req)(if value { 1 } else { 0 });
        }
    }

    pub fn avl_write_req(&self) -> bool {
        unsafe { ((*self.env).get_avl_write_req)() != 0 }
    }

    pub fn set_avl_write_req(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_write_req)(if value { 1 } else { 0 });
        }
    }

    pub fn avl_size(&self) -> u32 {
        unsafe { ((*self.env).get_avl_size)() }
    }

    pub fn set_avl_size(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_avl_size)(value);
        }
    }

    pub fn ddr3_init_done(&self) -> bool {
        unsafe { ((*self.env).get_ddr3_init_done)() != 0 }
    }

    pub fn set_ddr3_init_done(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_ddr3_init_done)(if value { 1 } else { 0 });
        }
    }

    pub fn ddr3_cal_success(&self) -> bool {
        unsafe { ((*self.env).get_ddr3_cal_success)() != 0 }
    }

    pub fn set_ddr3_cal_success(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_ddr3_cal_success)(if value { 1 } else { 0 });
        }
    }

    pub fn ddr3_cal_fail(&self) -> bool {
        unsafe { ((*self.env).get_ddr3_cal_fail)() != 0 }
    }

    pub fn set_ddr3_cal_fail(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_ddr3_cal_fail)(if value { 1 } else { 0 });
        }
    }

    pub fn is_finished(&self) -> bool {
        unsafe { ((*self.env).get_is_finished)() != 0 }
    }

    pub fn set_is_finished(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_is_finished)(if value { 1 } else { 0 });
        }
    }

    pub fn pass(&self) -> bool {
        unsafe { ((*self.env).get_pass)() != 0 }
    }

    pub fn set_pass(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_pass)(if value { 1 } else { 0 });
        }
    }

    pub fn fail(&self) -> bool {
        unsafe { ((*self.env).get_fail)() != 0 }
    }

    pub fn set_fail(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_fail)(if value { 1 } else { 0 });
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
