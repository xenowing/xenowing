mod ddr3_test;

use ddr3_test::*;

enum State {
    Idle,
}

#[no_mangle]
pub extern "C" fn run(env: *const Env) -> i32 {
    let mut ddr3_test = Ddr3Test::new(env);

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
