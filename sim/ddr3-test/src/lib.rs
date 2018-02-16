mod ddr3_test;

use ddr3_test::*;

use std::collections::VecDeque;

struct Fifo<T> {
    inner: VecDeque<T>,
    depth: usize,
}

impl<T> Fifo<T> {
    fn new(depth: usize) -> Fifo<T> {
        Fifo {
            inner: VecDeque::new(),
            depth: depth,
        }
    }

    fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    fn is_full(&self) -> bool {
        self.inner.len() == self.depth
    }

    fn push_front(&mut self, value: T) {
        if self.is_full() {
            panic!("Attempted to push_front, but the FIFO was full");
        }

        self.inner.push_front(value);
    }

    fn pop_back(&mut self) -> T {
        self.inner.pop_back().expect("Attempted to pop_back, but the FIFO was empty")
    }
}

enum Command {
    Read { addr: u32, byte_enable: u32 },
    Write { addr: u32, byte_enable: u32, data: u64 },
}

enum ActiveCommand {
    Command(Command),
    Refresh,
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

    let mut memory = vec![0; 0x1000000];
    let mut command_fifo = Fifo::new(10);
    let mut current_command: Option<(ActiveCommand, u32)> = None;
    let mut cycles_since_last_refresh = 0;

    let mut test_passed = false;

    for _ in 0..0xa000000 {
        // Check test status
        if ddr3_test.is_finished() {
            let pass = ddr3_test.pass();
            let fail = ddr3_test.fail();

            if pass && fail {
                panic!("Test module asserted both pass and fail");
            } else if !pass && !fail {
                panic!("Test module didn't assert pass or fail");
            }

            test_passed = pass;

            break;
        }

        // Rising edge
        ddr3_test.set_clk(true);
        ddr3_test.eval();

        ddr3_test.set_avl_rdata_valid(false);

        cycles_since_last_refresh += 1;

        let mut is_current_command_finished = false;

        if let Some(ref mut command) = current_command {
            // Process current command, if any
            //  Arbitrary 2-cycle latency for each command, implemented here as dummy waits
            command.1 += 1;
            if command.1 >= 2 {
                // Command finished timing-wise; actually perform command here
                if let ActiveCommand::Command(ref command) = command.0 {
                    match command {
                        &Command::Read { addr, byte_enable } => {
                            let mut rdata = ddr3_test.avl_rdata();
                            let mem_word = memory[addr as usize];

                            for i in 0..8 {
                                if (byte_enable & (1 << i)) != 0 {
                                    rdata &= !(0xff << (i * 8));
                                    rdata |= mem_word & (0xff << i * 8);
                                }
                            }

                            ddr3_test.set_avl_rdata(rdata);
                            ddr3_test.set_avl_rdata_valid(true);
                        }
                        &Command::Write { addr, byte_enable, data } => {
                            let mut mem_word = memory[addr as usize];

                            for i in 0..8 {
                                if (byte_enable & (1 << i)) != 0 {
                                    mem_word &= !(0xff << (i * 8));
                                    mem_word |= data & (0xff << i * 8);
                                }
                            }

                            memory[addr as usize] = mem_word;
                        }
                    }
                }

                is_current_command_finished = true;
            }
        } else {
            // If there's no current command, attempt to read one from the command FIFO,
            //  unless it's been a sufficient amount of time between refreshes to do that instead
            //  200 cycles between refreshes here is totally arbitrary
            if cycles_since_last_refresh >= 200 {
                current_command = Some((ActiveCommand::Refresh, 0));

                cycles_since_last_refresh = 0;
            } else if !command_fifo.is_empty() {
                current_command = Some((ActiveCommand::Command(command_fifo.pop_back()), 0));
            }
        }

        if is_current_command_finished {
            current_command = None;
        }

        // Check for read/write requests and place them in the FIFO if possible, otherwise assert not ready signal
        ddr3_test.set_avl_ready(true);

        let read_req = if ddr3_test.avl_read_req() { Some(Command::Read { addr: ddr3_test.avl_addr(), byte_enable: ddr3_test.avl_be() }) } else { None };
        let write_req = if ddr3_test.avl_write_req() { Some(Command::Write { addr: ddr3_test.avl_addr(), byte_enable: ddr3_test.avl_be(), data: ddr3_test.avl_wdata() }) } else { None };

        if read_req.is_some() && write_req.is_some() {
            panic!("Avalon master tried to assert read and write in the same cycle");
        }

        if ddr3_test.avl_burstbegin() {
            if read_req.is_none() && write_req.is_none() {
                panic!("Avalon master tried to assert burstbegin without also asserting read or write");
            }

            if ddr3_test.avl_size() != 1 {
                panic!("Avalon master tried to assert a burst read or write with a size other than 1");
            }
        }

        if command_fifo.is_full() {
            ddr3_test.set_avl_ready(false);
        } else {
            if let Some(command) = read_req {
                command_fifo.push_front(command);
            }
            if let Some(command) = write_req {
                command_fifo.push_front(command);
            }
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

    if test_passed { 0 } else { 1 }
}
