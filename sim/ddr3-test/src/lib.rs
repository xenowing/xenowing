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

    get_leds_n: extern "C" fn() -> u32,
    set_leds_n: extern "C" fn(value: u32),

    eval: extern "C" fn(),
    final_: extern "C" fn(),
    trace_dump: extern "C" fn(time: u64),
}

struct Ddr3Test {
    env: *const Env,
}

impl Ddr3Test {
    fn reset_n(&self) -> bool {
        unsafe { ((*self.env).get_reset_n)() != 0 }
    }

    fn set_reset_n(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_reset_n)(if value { 1 } else { 0 });
        }
    }

    fn clk(&self) -> bool {
        unsafe { ((*self.env).get_clk)() != 0 }
    }

    fn set_clk(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_clk)(if value { 1 } else { 0 });
        }
    }

    fn avl_ready(&self) -> bool {
        unsafe { ((*self.env).get_avl_ready)() != 0 }
    }

    fn set_avl_ready(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_ready)(if value { 1 } else { 0 });
        }
    }

    fn avl_burstbegin(&self) -> bool {
        unsafe { ((*self.env).get_avl_burstbegin)() != 0 }
    }

    fn set_avl_burstbegin(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_burstbegin)(if value { 1 } else { 0 });
        }
    }

    fn avl_addr(&self) -> u32 {
        unsafe { ((*self.env).get_avl_addr)() }
    }

    fn set_avl_addr(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_avl_addr)(value);
        }
    }

    fn avl_rdata_valid(&self) -> bool {
        unsafe { ((*self.env).get_avl_rdata_valid)() != 0 }
    }

    fn set_avl_rdata_valid(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_rdata_valid)(if value { 1 } else { 0 });
        }
    }

    fn avl_rdata(&self) -> u64 {
        unsafe { ((*self.env).get_avl_rdata)() }
    }

    fn set_avl_rdata(&mut self, value: u64) {
        unsafe {
            ((*self.env).set_avl_rdata)(value);
        }
    }

    fn avl_wdata(&self) -> u64 {
        unsafe { ((*self.env).get_avl_wdata)() }
    }

    fn set_avl_wdata(&mut self, value: u64) {
        unsafe {
            ((*self.env).set_avl_wdata)(value);
        }
    }

    fn avl_be(&self) -> u32 {
        unsafe { ((*self.env).get_avl_be)() }
    }

    fn set_avl_be(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_avl_be)(value);
        }
    }

    fn avl_read_req(&self) -> bool {
        unsafe { ((*self.env).get_avl_read_req)() != 0 }
    }

    fn set_avl_read_req(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_read_req)(if value { 1 } else { 0 });
        }
    }

    fn avl_write_req(&self) -> bool {
        unsafe { ((*self.env).get_avl_write_req)() != 0 }
    }

    fn set_avl_write_req(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_avl_write_req)(if value { 1 } else { 0 });
        }
    }

    fn avl_size(&self) -> u32 {
        unsafe { ((*self.env).get_avl_size)() }
    }

    fn set_avl_size(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_avl_size)(value);
        }
    }

    fn ddr3_init_done(&self) -> bool {
        unsafe { ((*self.env).get_ddr3_init_done)() != 0 }
    }

    fn set_ddr3_init_done(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_ddr3_init_done)(if value { 1 } else { 0 });
        }
    }

    fn ddr3_cal_success(&self) -> bool {
        unsafe { ((*self.env).get_ddr3_cal_success)() != 0 }
    }

    fn set_ddr3_cal_success(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_ddr3_cal_success)(if value { 1 } else { 0 });
        }
    }

    fn ddr3_cal_fail(&self) -> bool {
        unsafe { ((*self.env).get_ddr3_cal_fail)() != 0 }
    }

    fn set_ddr3_cal_fail(&mut self, value: bool) {
        unsafe {
            ((*self.env).set_ddr3_cal_fail)(if value { 1 } else { 0 });
        }
    }

    fn leds_n(&self) -> u32 {
        unsafe { ((*self.env).get_leds_n)() }
    }

    fn set_leds_n(&mut self, value: u32) {
        unsafe {
            ((*self.env).set_leds_n)(value);
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

    fn trace_dump(&mut self, time: u64) {
        unsafe {
            ((*self.env).trace_dump)(time);
        }
    }
}

enum State {
    Idle,
}

#[no_mangle]
pub extern "C" fn run(env: *const Env) -> i32 {
    let mut ddr3_test = Ddr3Test {
        env: env,
    };

    let mut time = 0;

    // Reset
    ddr3_test.set_reset_n(false);
    ddr3_test.set_clk(false);
    ddr3_test.eval();

    ddr3_test.set_avl_ready(false);
    ddr3_test.set_avl_rdata_valid(false);
    ddr3_test.set_avl_rdata(0);

    ddr3_test.set_ddr3_init_done(false);
    ddr3_test.set_ddr3_cal_success(false);
    ddr3_test.set_ddr3_cal_fail(false);

    ddr3_test.trace_dump(time);
    time += 1;

    ddr3_test.set_reset_n(true);

    // Simulate init/cal
    for _ in 0..100 {
        // Rising edge
        ddr3_test.set_clk(true);
        ddr3_test.eval();

        ddr3_test.trace_dump(time);
        time += 1;

        // Falling edge
        ddr3_test.set_clk(false);
        ddr3_test.eval();

        ddr3_test.trace_dump(time);
        time += 1;
    }

    // Init/cal successful
    ddr3_test.set_ddr3_init_done(true);
    ddr3_test.set_ddr3_cal_success(true);

    let mut state = State::Idle;

    let mut leds_n = 0;

    for _ in 0..1000 {
        // Rising edge
        ddr3_test.set_clk(true);
        ddr3_test.eval();

        match state {
            State::Idle => {
                ddr3_test.set_avl_ready(true);
            }
        }

        if ddr3_test.leds_n() != leds_n {
            leds_n = ddr3_test.leds_n();

            println!(
                "LEDS changed: {}{}{}",
                if (leds_n & 0x04) == 0 { 1 } else { 0 },
                if (leds_n & 0x02) == 0 { 1 } else { 0 },
                if (leds_n & 0x01) == 0 { 1 } else { 0 });
        }

        ddr3_test.eval();

        ddr3_test.trace_dump(time);
        time += 1;

        // Falling edge
        ddr3_test.set_clk(false);
        ddr3_test.eval();

        ddr3_test.trace_dump(time);
        time += 1;
    }

    ddr3_test.final_();

    0
}
