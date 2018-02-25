extern crate ddr3_simulator;

mod test;

use ddr3_simulator::*;

use test::*;

#[no_mangle]
pub extern "C" fn run(env: *const Env) -> i32 {
    let mut test = Test::new(env);

    let mut time = 0;

    // Reset
    test.set_reset_n(false);
    test.set_clk(false);
    test.eval();

    test.trace_dump(time);
    time += 1;

    test.set_clk(true);

    let mut ddr3_simulator = Ddr3Simulator::new();

    test.set_avl_ready(ddr3_simulator.avl_ready());
    test.set_avl_rdata_valid(ddr3_simulator.avl_rdata_valid());
    test.set_avl_rdata(ddr3_simulator.avl_rdata());

    test.set_ddr3_init_done(ddr3_simulator.init_done());
    test.set_ddr3_cal_success(ddr3_simulator.cal_success());
    test.set_ddr3_cal_fail(ddr3_simulator.cal_fail());

    test.eval();

    test.trace_dump(time);
    time += 1;

    test.set_reset_n(true);
    test.set_clk(false);
    test.eval();

    test.trace_dump(time);
    time += 1;

    let mut test_passed = false;

    for _ in 0..0xa000000 {
        // Check test status
        if test.is_finished() {
            let pass = test.pass();
            let fail = test.fail();

            if pass && fail {
                panic!("Test module asserted both pass and fail");
            } else if !pass && !fail {
                panic!("Test module didn't assert pass or fail");
            }

            test_passed = pass;

            break;
        }

        // Rising edge
        test.set_clk(true);
        test.eval();

        ddr3_simulator.set_avl_burstbegin(test.avl_burstbegin());
        ddr3_simulator.set_avl_addr(test.avl_addr());
        ddr3_simulator.set_avl_wdata(test.avl_wdata());
        ddr3_simulator.set_avl_be(test.avl_be());
        ddr3_simulator.set_avl_read_req(test.avl_read_req());
        ddr3_simulator.set_avl_write_req(test.avl_write_req());
        ddr3_simulator.set_avl_size(test.avl_size());

        ddr3_simulator.eval();

        test.set_avl_ready(ddr3_simulator.avl_ready());
        test.set_avl_rdata_valid(ddr3_simulator.avl_rdata_valid());
        test.set_avl_rdata(ddr3_simulator.avl_rdata());

        test.set_ddr3_init_done(ddr3_simulator.init_done());
        test.set_ddr3_cal_success(ddr3_simulator.cal_success());
        test.set_ddr3_cal_fail(ddr3_simulator.cal_fail());

        test.eval();

        test.trace_dump(time);
        time += 1;

        // Falling edge
        test.set_clk(false);
        test.eval();

        test.trace_dump(time);
        time += 1;
    }

    test.final_();

    if test_passed { 0 } else { 1 }
}
