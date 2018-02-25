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

pub struct Ddr3Simulator {
    memory: Vec<u64>,
    command_fifo: Fifo<Command>,
    current_command: Option<(ActiveCommand, u32)>,
    cycles_since_last_refresh: u32,

    avl_ready: bool,
    avl_burstbegin: bool,
    avl_addr: u32,
    avl_wdata: u64,
    avl_be: u32,
    avl_read_req: bool,
    avl_write_req: bool,
    avl_rdata: u64,
    avl_rdata_valid: bool,
    avl_size: u32,

    init_done: bool,
    init_cycles: u32,
}

impl Ddr3Simulator {
    pub fn new() -> Ddr3Simulator {
        Ddr3Simulator {
            memory: vec![0; 0x1000000],
            command_fifo: Fifo::new(10),
            current_command: None,
            cycles_since_last_refresh: 0,

            avl_ready: false,
            avl_burstbegin: false,
            avl_addr: 0,
            avl_wdata: 0,
            avl_be: 0,
            avl_read_req: false,
            avl_write_req: false,
            avl_rdata: 0,
            avl_rdata_valid: false,
            avl_size: 0,

            init_done: false,
            init_cycles: 0,
        }
    }

    pub fn avl_ready(&self) -> bool {
        self.avl_ready
    }

    pub fn set_avl_burstbegin(&mut self, value: bool) {
        self.avl_burstbegin = value;
    }

    pub fn set_avl_addr(&mut self, value: u32) {
        self.avl_addr = value;
    }

    pub fn set_avl_wdata(&mut self, value: u64) {
        self.avl_wdata = value;
    }

    pub fn set_avl_be(&mut self, value: u32) {
        self.avl_be = value;
    }

    pub fn set_avl_read_req(&mut self, value: bool) {
        self.avl_read_req = value;
    }

    pub fn set_avl_write_req(&mut self, value: bool) {
        self.avl_write_req = value;
    }

    pub fn avl_rdata(&self) -> u64 {
        self.avl_rdata
    }

    pub fn avl_rdata_valid(&self) -> bool {
        self.avl_rdata_valid
    }

    pub fn set_avl_size(&mut self, value: u32) {
        self.avl_size = value;
    }

    pub fn init_done(&self) -> bool {
        self.init_done
    }

    pub fn cal_success(&self) -> bool {
        self.init_done
    }

    pub fn cal_fail(&self) -> bool {
        false
    }

    pub fn eval(&mut self) {
        if !self.init_done {
            self.init_cycles += 1;
            if self.init_cycles >= 100 {
                self.init_done = true;
            }
        } else {
            self.avl_rdata_valid = false;

            self.cycles_since_last_refresh += 1;

            let mut is_current_command_finished = false;

            if let Some(ref mut command) = self.current_command {
                // Process current command, if any
                //  Arbitrary 2-cycle latency for each command, implemented here as dummy waits
                command.1 += 1;
                if command.1 >= 2 {
                    // Command finished timing-wise; actually perform command here
                    if let ActiveCommand::Command(ref command) = command.0 {
                        match command {
                            &Command::Read { addr, byte_enable } => {
                                let mut rdata = self.avl_rdata;
                                let mem_word = self.memory[addr as usize];

                                for i in 0..8 {
                                    if (byte_enable & (1 << i)) != 0 {
                                        rdata &= !(0xff << (i * 8));
                                        rdata |= mem_word & (0xff << i * 8);
                                    }
                                }

                                self.avl_rdata = rdata;
                                self.avl_rdata_valid = true;
                            }
                            &Command::Write { addr, byte_enable, data } => {
                                let mut mem_word = self.memory[addr as usize];

                                for i in 0..8 {
                                    if (byte_enable & (1 << i)) != 0 {
                                        mem_word &= !(0xff << (i * 8));
                                        mem_word |= data & (0xff << i * 8);
                                    }
                                }

                                self.memory[addr as usize] = mem_word;
                            }
                        }
                    }

                    is_current_command_finished = true;
                }
            } else {
                // If there's no current command, attempt to read one from the command FIFO,
                //  unless it's been a sufficient amount of time between refreshes to do that instead
                //  200 cycles between refreshes here is totally arbitrary
                if self.cycles_since_last_refresh >= 200 {
                    self.current_command = Some((ActiveCommand::Refresh, 0));

                    self.cycles_since_last_refresh = 0;
                } else if !self.command_fifo.is_empty() {
                    self.current_command = Some((ActiveCommand::Command(self.command_fifo.pop_back()), 0));
                }
            }

            if is_current_command_finished {
                self.current_command = None;
            }

            // Check for read/write requests and place them in the FIFO if possible, otherwise assert not ready signal
            self.avl_ready = true;

            let read_req = if self.avl_read_req { Some(Command::Read { addr: self.avl_addr, byte_enable: self.avl_be }) } else { None };
            let write_req = if self.avl_write_req { Some(Command::Write { addr: self.avl_addr, byte_enable: self.avl_be, data: self.avl_wdata }) } else { None };

            if read_req.is_some() && write_req.is_some() {
                panic!("Avalon master tried to assert read and write in the same cycle");
            }

            if self.avl_burstbegin {
                if read_req.is_none() && write_req.is_none() {
                    panic!("Avalon master tried to assert burstbegin without also asserting read or write");
                }

                if self.avl_size != 1 {
                    panic!("Avalon master tried to assert a burst read or write with a size other than 1");
                }
            }

            if self.command_fifo.is_full() {
                self.avl_ready = false;
            } else {
                if let Some(command) = read_req {
                    self.command_fifo.push_front(command);
                }
                if let Some(command) = write_req {
                    self.command_fifo.push_front(command);
                }
            }
        }
    }
}
