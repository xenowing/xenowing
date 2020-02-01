use kaze::*;

struct Instruction<'a> {
    pub value: &'a Signal<'a>,
}

impl<'a> Instruction<'a> {
    fn new(value: &'a Signal<'a>) -> Instruction<'a> {
        if value.bit_width() != 32 {
            panic!("value bit width must be 32");
        }

        Instruction {
            value,
        }
    }

    fn opcode(&self) -> &'a Signal<'a> {
        self.value.bits(6, 2) // Bottom two bits are always 0b11 for RV32I, so just ignore them
    }

    fn rs1(&self) -> &'a Signal<'a> {
        self.value.bits(19, 15)
    }

    fn rs2(&self) -> &'a Signal<'a> {
        self.value.bits(24, 20)
    }

    fn rd(&self) -> &'a Signal<'a> {
        self.value.bits(11, 7)
    }

    fn funct3(&self) -> &'a Signal<'a> {
        self.value.bits(14, 12)
    }

    fn load_offset(&self) -> &'a Signal<'a> {
        self.value.bit(31).repeat(20).concat(self.value.bits(31, 20))
    }

    fn store_offset(&self) -> &'a Signal<'a> {
        self.value.bit(31).repeat(20).concat(self.value.bits(31, 25)).concat(self.value.bits(11, 7))
    }

    fn jump_offset(&self, m: &'a Module<'a>) -> &'a Signal<'a> {
        self.value.bit(31).repeat(11).concat(self.value.bit(31)).concat(self.value.bits(19, 12)).concat(self.value.bit(20)).concat(self.value.bits(30, 21)).concat(m.low())
    }

    fn branch_offset(&self, m: &'a Module<'a>) -> &'a Signal<'a> {
        self.value.bit(31).repeat(19).concat(self.value.bit(31)).concat(self.value.bit(7)).concat(self.value.bits(30, 25)).concat(self.value.bits(11, 8)).concat(m.low())
    }

    fn i_immediate(&self) -> &'a Signal<'a> {
        self.value.bit(31).repeat(20).concat(self.value.bits(31, 20))
    }

    fn u_immediate(&self, m: &'a Module<'a>) -> &'a Signal<'a> {
        self.value.bits(31, 12).concat(m.lit(0u32, 12))
    }

    fn csr(&self) -> &'a Signal<'a> {
        self.value.bits(31, 20)
    }
}

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    generate_control(c);
    generate_instruction_fetch(c);
    generate_decode(c);
    generate_execute(c);
    generate_mem(c);
    generate_writeback(c);

    let m = c.module("Marv");

    let control = m.instance("control", "Control");

    let pc = m.reg("pc", 32);
    pc.default_value(0x10000000u32);

    let cycle_counter = m.reg("cycle_counter", 64);
    cycle_counter.default_value(0u64);
    cycle_counter.drive_next(cycle_counter.value + m.lit(1u64, 64));

    let instructions_retired_counter = m.reg("instructions_retired_counter", 64);
    instructions_retired_counter.default_value(0u64);

    let bus_ready = m.input("bus_ready", 1);
    let bus_read_data = m.input("bus_read_data", 32);
    let bus_read_data_valid = m.input("bus_read_data_valid", 1);

    let instruction_fetch = m.instance("instruction_fetch", "InstructionFetch");
    control.drive_input("instruction_fetch_ready", instruction_fetch.output("ready"));
    instruction_fetch.drive_input("enable", control.output("instruction_fetch_enable"));
    instruction_fetch.drive_input("pc", pc.value.bits(31, 2));
    instruction_fetch.drive_input("bus_ready", bus_ready);

    let decode = m.instance("decode", "Decode");
    control.drive_input("decode_ready", decode.output("ready"));
    decode.drive_input("bus_read_data", bus_read_data);
    decode.drive_input("bus_read_data_valid", bus_read_data_valid);

    let instruction = m.reg("instruction", 32);
    instruction.drive_next(control.output("decode_enable").mux(decode.output("instruction"), instruction.value));
    let instruction = Instruction::new(instruction.value);

    m.output("register_file_read_addr1", instruction.rs1());
    m.output("register_file_read_addr2", instruction.rs2());
    let reg1 = m.reg("rs1", 32);
    let reg2 = m.reg("rs2", 32);
    reg1.drive_next(control.output("reg_wait_enable").mux(m.input("register_file_read_data1", 32), reg1.value));
    reg2.drive_next(control.output("reg_wait_enable").mux(m.input("register_file_read_data2", 32), reg2.value));

    let execute = m.instance("execute", "Execute");
    execute.drive_input("pc", pc.value);
    execute.drive_input("instruction", instruction.value);
    execute.drive_input("reg1", reg1.value);
    execute.drive_input("reg2", reg2.value);
    m.output("alu_op", execute.output("alu_op"));
    m.output("alu_op_mod", execute.output("alu_op_mod"));
    m.output("alu_lhs", execute.output("alu_lhs"));
    m.output("alu_rhs", execute.output("alu_rhs"));
    execute.drive_input("alu_res", m.input("alu_res", 32));
    execute.drive_input("cycle_counter_value", cycle_counter.value);
    execute.drive_input("instructions_retired_counter_value", instructions_retired_counter.value);

    let mem = m.instance("mem", "Mem");
    control.drive_input("mem_ready", mem.output("ready"));
    mem.drive_input("enable", control.output("mem_enable"));
    mem.drive_input("bus_ready", bus_ready);
    mem.drive_input("bus_addr_in", execute.output("bus_addr"));
    mem.drive_input("bus_write_data_in", execute.output("bus_write_data"));
    mem.drive_input("bus_byte_enable_in", execute.output("bus_byte_enable"));
    mem.drive_input("bus_read_req_in", execute.output("bus_read_req"));
    mem.drive_input("bus_write_req_in", execute.output("bus_write_req"));
    m.output("bus_write_data", mem.output("bus_write_data_out"));

    let writeback = m.instance("writeback", "Writeback");
    control.drive_input("writeback_ready", writeback.output("ready"));
    writeback.drive_input("enable", control.output("writeback_enable"));
    writeback.drive_input("instruction", instruction.value);
    writeback.drive_input("bus_addr_low", mem.output("bus_addr_out").bits(1, 0));
    writeback.drive_input("next_pc", execute.output("next_pc"));
    writeback.drive_input("rd_value_write_enable", execute.output("rd_value_write_enable"));
    writeback.drive_input("rd_value_write_data", execute.output("rd_value_write_data"));
    pc.drive_next(writeback.output("pc_write_enable").mux(writeback.output("pc_write_data"), pc.value));
    instructions_retired_counter.drive_next(
        writeback.output("instructions_retired_counter_increment_enable").mux(
            instructions_retired_counter.value + m.lit(1u64, 64),
            instructions_retired_counter.value));
    m.output("register_file_write_enable", writeback.output("register_file_write_enable"));
    m.output("register_file_write_addr", writeback.output("register_file_write_addr"));
    m.output("register_file_write_data", writeback.output("register_file_write_data"));
    writeback.drive_input("bus_read_data", bus_read_data);
    writeback.drive_input("bus_read_data_valid", bus_read_data_valid);

    let mem_bus_read_req = mem.output("bus_read_req_out");
    let mem_bus_write_req = mem.output("bus_write_req_out");
    m.output("bus_addr", (mem_bus_read_req | mem_bus_write_req).mux(mem.output("bus_addr_out").bits(31, 2), instruction_fetch.output("bus_addr")));
    m.output("bus_byte_enable", (mem_bus_read_req | mem_bus_write_req).mux(mem.output("bus_byte_enable_out"), instruction_fetch.output("bus_byte_enable")));
    m.output("bus_read_req", mem_bus_read_req | instruction_fetch.output("bus_read_req"));
    m.output("bus_write_req", mem_bus_write_req);

    m
}

pub struct If_<'a> {
    cond: &'a Signal<'a>,
    when_true: &'a Signal<'a>,
}

impl<'a> If_<'a> {
    fn new(cond: &'a Signal<'a>, when_true: &'a Signal<'a>) -> If_<'a> {
        If_ {
            cond,
            when_true,
        }
    }

    pub fn else_if(self, cond: &'a Signal<'a>, when_true: &'a Signal<'a>) -> ElseIf<'a> {
        ElseIf {
            parent: ElseIfParent::If_(self),
            cond,
            when_true,
        }
    }

    pub fn else_(self, when_false: &'a Signal<'a>) -> &'a Signal<'a> {
        self.cond.mux(self.when_true, when_false)
    }
}

pub fn if_<'a>(cond: &'a Signal<'a>, when_true: &'a Signal<'a>) -> If_<'a> {
    If_::new(cond, when_true)
}

enum ElseIfParent<'a> {
    If_(If_<'a>),
    ElseIf(Box<ElseIf<'a>>),
}

pub struct ElseIf<'a> {
    parent: ElseIfParent<'a>,
    cond: &'a Signal<'a>,
    when_true: &'a Signal<'a>,
}

impl<'a> ElseIf<'a> {
    pub fn else_if(self, cond: &'a Signal<'a>, when_true: &'a Signal<'a>) -> ElseIf<'a> {
        ElseIf {
            parent: ElseIfParent::ElseIf(Box::new(self)),
            cond,
            when_true,
        }
    }

    pub fn else_(self, when_false: &'a Signal<'a>) -> &'a Signal<'a> {
        let ret = self.cond.mux(self.when_true, when_false);
        match self.parent {
            ElseIfParent::If_(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

fn generate_control<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Control");

    // TODO: Figure out how to use/describe enums properly in kaze!
    let state_bit_width = 3;
    let state_instruction_fetch = 0u32;
    let state_decode = 1u32;
    let state_reg_wait = 2u32;
    let state_execute = 3u32;
    let state_mem = 4u32;
    let state_writeback = 5u32;
    let state = m.reg("state", state_bit_width);
    state.default_value(state_instruction_fetch);
    // TODO: (Enum) matching sugar
    state.drive_next(if_(state.value.eq(m.lit(state_instruction_fetch, state_bit_width)) & m.input("instruction_fetch_ready", 1), {
        m.lit(state_decode, state_bit_width)
    }).else_if(state.value.eq(m.lit(state_decode, state_bit_width)) & m.input("decode_ready", 1), {
        m.lit(state_reg_wait, state_bit_width)
    }).else_if(state.value.eq(m.lit(state_reg_wait, state_bit_width)), {
        m.lit(state_execute, state_bit_width)
    }).else_if(state.value.eq(m.lit(state_execute, state_bit_width)), {
        m.lit(state_mem, state_bit_width)
    }).else_if(state.value.eq(m.lit(state_mem, state_bit_width)) & m.input("mem_ready", 1), {
        m.lit(state_writeback, state_bit_width)
    }).else_if(state.value.eq(m.lit(state_writeback, state_bit_width)) & m.input("writeback_ready", 1), {
        m.lit(state_instruction_fetch, state_bit_width)
    }).else_({
        state.value
    }));

    m.output("instruction_fetch_enable", state.value.eq(m.lit(state_instruction_fetch, state_bit_width)));
    m.output("decode_enable", state.value.eq(m.lit(state_decode, state_bit_width)));
    m.output("reg_wait_enable", state.value.eq(m.lit(state_reg_wait, state_bit_width)));
    m.output("mem_enable", state.value.eq(m.lit(state_mem, state_bit_width)));
    m.output("writeback_enable", state.value.eq(m.lit(state_writeback, state_bit_width)));

    m
}

fn generate_instruction_fetch<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("InstructionFetch");

    m.output("ready", m.input("bus_ready", 1));
    m.output("bus_addr", m.input("pc", 30));
    m.output("bus_byte_enable", m.high().repeat(4));
    m.output("bus_read_req", m.input("enable", 1));

    m
}

fn generate_decode<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Decode");

    m.output("ready", m.input("bus_read_data_valid", 1));
    m.output("instruction", m.input("bus_read_data", 32));

    m
}

fn generate_execute<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Execute");

    let instruction = Instruction::new(m.input("instruction", 32));

    let reg1 = m.input("reg1", 32);
    let reg2 = m.input("reg2", 32);

    let mut alu_op = instruction.funct3();
    let mut alu_op_mod = m.low();
    m.output("alu_lhs", reg1);
    let mut alu_rhs = reg2;
    let alu_res = m.input("alu_res", 32);

    let reg_comp = instruction.opcode().bit(3);

    kaze_sugar! {
        if (reg_comp) {
            // Register computation
            alu_op_mod = instruction.value.bit(30);
        }

        if (!reg_comp) {
            // Immediate computation
            alu_rhs = instruction.i_immediate();

            // Shifts treat alu_op_mod the same as register computations and use rs2 directly (not its register value)
            if (instruction.funct3().eq(m.lit(0b001u32, 3)) | instruction.funct3().eq(m.lit(0b101u32, 3))) {
                alu_op_mod = instruction.value.bit(30);
                alu_rhs = m.lit(0u32, 27).concat(instruction.rs2());
            }
        }
    }

    let pc = m.input("pc", 32);
    let link_pc = pc + m.lit(4u32, 32);
    let mut next_pc = link_pc;

    let mut rd_value_write_enable = m.high();
    let mut rd_value_write_data = alu_res;

    kaze_sugar! {
        if (instruction.opcode().eq(m.lit(0b01101u32, 5))) {
            // lui
            rd_value_write_data = instruction.u_immediate(m);
        }

        if (instruction.opcode().eq(m.lit(0b00101u32, 5))) {
            // auipc
            rd_value_write_data = instruction.u_immediate(m) + pc;
        }

        if (instruction.opcode().eq(m.lit(0b11011u32, 5))) {
            // jal
            next_pc = pc + instruction.jump_offset(m);
            rd_value_write_data = link_pc;
        }

        if (instruction.opcode().eq(m.lit(0b11001u32, 5))) {
            // jalr
            alu_rhs = instruction.i_immediate();
            next_pc = alu_res;
            rd_value_write_data = link_pc;
        }
    }

    let bus_addr = alu_res; // TODO: Consider separate adder for load/store offsets
    m.output("bus_addr", bus_addr);
    m.output("bus_byte_enable", if_(instruction.funct3().bits(1, 0).eq(m.lit(0b01u32, 2)), {
        // lh/lhu/sh
        if_(!bus_addr.bit(1), {
            m.lit(0b0011u32, 4)
        }).else_({
            m.lit(0b1100u32, 4)
        })
    }).else_if(instruction.funct3().bits(1, 0).eq(m.lit(0b00u32, 2)), {
        // lb/lbu/sb
        let bus_addr_low = bus_addr.bits(1, 0);
        // TODO: Express with shift?
        if_(bus_addr_low.eq(m.lit(0b00u32, 2)), {
            m.lit(0b0001u32, 4)
        }).else_if(bus_addr_low.eq(m.lit(0b01u32, 2)), {
            m.lit(0b0010u32, 4)
        }).else_if(bus_addr_low.eq(m.lit(0b10u32, 2)), {
            m.lit(0b0100u32, 4)
        }).else_({
            m.lit(0b1000u32, 4)
        })
    }).else_({
        m.lit(0b1111u32, 4)
    }));

    // Loads
    let mut bus_read_req = m.low();

    kaze_sugar! {
        if (instruction.opcode().eq(m.lit(0b00000u32, 5))) {
            // lw
            alu_op = m.lit(0u32, 3);
            alu_op_mod = m.low();
            alu_rhs = instruction.load_offset();
            bus_read_req = m.high();
        }
    }

    m.output("bus_read_req", bus_read_req);

    // Stores
    let mut bus_write_data = reg2;
    let mut bus_write_req = m.low();

    kaze_sugar! {
        if (instruction.opcode().eq(m.lit(0b01000u32, 5))) {
            // sw
            alu_op = m.lit(0u32, 3);
            alu_op_mod = m.low();
            alu_rhs = instruction.store_offset();
            rd_value_write_enable = m.low();
            bus_write_req = m.high();

            if (instruction.funct3().bits(1, 0).eq(m.lit(0b00u32, 2))) {
                // sb
                if (bus_addr.bits(1, 0).eq(m.lit(0b01u32, 2))) {
                    bus_write_data = m.lit(0u32, 16).concat(reg2.bits(7, 0)).concat(m.lit(0u32, 8));
                }
                if (bus_addr.bits(1, 0).eq(m.lit(0b10u32, 2))) {
                    bus_write_data = m.lit(0u32, 8).concat(reg2.bits(7, 0)).concat(m.lit(0u32, 16));
                }
                if (bus_addr.bits(1, 0).eq(m.lit(0b11u32, 2))) {
                    bus_write_data = reg2.bits(7, 0).concat(m.lit(0u32, 24));
                }
            }

            if (instruction.funct3().bits(1, 0).eq(m.lit(0b01u32, 2))) {
                // sh
                if (bus_addr.bit(1)) {
                    bus_write_data = reg2.bits(15, 0).concat(m.lit(0u32, 16));
                }
            }
        }
    }

    m.output("alu_op", alu_op);
    m.output("alu_op_mod", alu_op_mod);
    m.output("alu_rhs", alu_rhs);

    m.output("bus_write_data", bus_write_data);
    m.output("bus_write_req", bus_write_req);

    // Branch instructions
    let funct3_low = instruction.funct3().bits(2, 1);
    // TODO: switch/case construct?
    let branch_taken = if_(funct3_low.eq(m.lit(0b00u32, 2)), {
        reg1.eq(reg2)
    }).else_if(funct3_low.eq(m.lit(0b01u32, 2)), {
        m.low()
    }).else_if(funct3_low.eq(m.lit(0b10u32, 2)), {
        reg1.lt_signed(reg2)
    }).else_({
        reg1.lt(reg2)
    });
    // TODO: Conditional invert construct?
    let branch_taken = if_(instruction.funct3().bit(0), !branch_taken).else_(branch_taken);
    kaze_sugar! {
        if (instruction.opcode().eq(m.lit(0b11000u32, 5))) {
            rd_value_write_enable = m.low();

            if (branch_taken) {
                next_pc = pc + instruction.branch_offset(m);
            }
        }
    }

    m.output("next_pc", next_pc);

    // Fence instructions
    let mut rd_value_write_enable = if_(instruction.opcode().eq(m.lit(0b00011u32, 5)), {
        // Do nothing (nop)
        m.low()
    }).else_({
        rd_value_write_enable
    });

    let cycle_counter_value = m.input("cycle_counter_value", 64);
    let instructions_retired_counter_value = m.input("instructions_retired_counter_value", 64);

    // System instructions
    kaze_sugar! {
        if (instruction.opcode().eq(m.lit(0b11100u32, 5))) {
            if (instruction.funct3().eq(m.lit(0b000u32, 3))) {
                // ecall/ebreak: do nothing (nop)
                rd_value_write_enable = m.low();
            }

            rd_value_write_data = if_(instruction.funct3().bits(1, 0).ne(m.lit(0b00u32, 2)), {
                // csrrw, csrrs, csrrc, csrrwi, csrrsi, csrrci
                let csr_low = instruction.csr().bits(1, 0);
                if_(csr_low.eq(m.lit(0b00u32, 2)) | csr_low.eq(m.lit(0b01u32, 2)), {
                    // cycle, time
                    if_(!instruction.csr().bit(7), {
                        cycle_counter_value.bits(31, 0)
                    }).else_({
                        // cycleh, timeh
                        cycle_counter_value.bits(63, 32)
                    })
                }).else_if(csr_low.eq(m.lit(0b10u32, 2)), {
                    // instret
                    if_(!instruction.csr().bit(7), {
                        instructions_retired_counter_value.bits(31, 0)
                    }).else_({
                        // instreth
                        instructions_retired_counter_value.bits(63, 32)
                    })
                }).else_({
                    rd_value_write_data
                })
            }).else_({
                rd_value_write_data
            });
        }
    }

    m.output("rd_value_write_enable", rd_value_write_enable);
    m.output("rd_value_write_data", rd_value_write_data);

    m
}

fn generate_mem<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Mem");

    let enable = m.input("enable", 1);

    let bus_addr = m.reg("bus_addr", 32);
    bus_addr.drive_next(m.input("bus_addr_in", 32));
    m.output("bus_addr_out", bus_addr.value);
    let bus_write_data = m.reg("bus_write_data", 32);
    bus_write_data.drive_next(m.input("bus_write_data_in", 32));
    m.output("bus_write_data_out", bus_write_data.value);
    let bus_byte_enable = m.reg("bus_byte_enable", 4);
    bus_byte_enable.drive_next(m.input("bus_byte_enable_in", 4));
    m.output("bus_byte_enable_out", bus_byte_enable.value);
    let bus_read_req = m.reg("bus_read_req", 1);
    bus_read_req.drive_next(m.input("bus_read_req_in", 1));
    m.output("bus_read_req_out", enable & bus_read_req.value);
    let bus_write_req = m.reg("bus_write_req", 1);
    bus_write_req.drive_next(m.input("bus_write_req_in", 1));
    m.output("bus_write_req_out", enable & bus_write_req.value);

    m.output("ready", (bus_read_req.value | bus_write_req.value).mux(m.input("bus_ready", 1), m.high()));

    m
}

fn generate_writeback<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Writeback");

    let mut ready = m.high();

    let instruction = Instruction::new(m.input("instruction", 32));
    let bus_addr_low = m.input("bus_addr_low", 2);
    let bus_read_data = m.input("bus_read_data", 32);

    let mut register_file_write_data = m.input("rd_value_write_data", 32);

    kaze_sugar! {
        // Loads
        if (instruction.opcode().eq(m.lit(0b00000u32, 5))) {
            // lw
            ready = m.input("bus_read_data_valid", 1);
            register_file_write_data = bus_read_data;

            if (instruction.funct3().bits(1, 0).eq(m.lit(0b00u32, 2))) {
                // lb/lbu
                register_file_write_data = bus_read_data.bit(7).repeat(24).concat(bus_read_data.bits(7, 0));
                if (bus_addr_low.eq(m.lit(0b01u32, 2))) {
                    register_file_write_data = bus_read_data.bit(15).repeat(24).concat(bus_read_data.bits(15, 8));
                }
                if (bus_addr_low.eq(m.lit(0b10u32, 2))) {
                    register_file_write_data = bus_read_data.bit(23).repeat(24).concat(bus_read_data.bits(23, 16));
                }
                if (bus_addr_low.eq(m.lit(0b11u32, 2))) {
                    register_file_write_data = bus_read_data.bit(31).repeat(24).concat(bus_read_data.bits(31, 24));
                }

                if (instruction.funct3().bit(2)) {
                    register_file_write_data = m.lit(0u32, 24).concat(register_file_write_data.bits(7, 0));
                }
            }

            if (instruction.funct3().bits(1, 0).eq(m.lit(0b01u32, 2))) {
                // lh/lhu
                register_file_write_data = bus_read_data.bit(15).repeat(16).concat(bus_read_data.bits(15, 0));
                if (bus_addr_low.bit(1)) {
                    register_file_write_data = bus_read_data.bit(31).repeat(16).concat(bus_read_data.bits(31, 16));
                }

                if (instruction.funct3().bit(2)) {
                    register_file_write_data = m.lit(0u32, 16).concat(register_file_write_data.bits(15, 0));
                }
            }
        }
    }

    m.output("ready", ready);

    let enable = m.input("enable", 1);

    m.output("pc_write_data", m.input("next_pc", 32));
    m.output("pc_write_enable", enable & ready);

    m.output("instructions_retired_counter_increment_enable", enable & ready);

    m.output("register_file_write_addr", instruction.rd());
    m.output("register_file_write_data", register_file_write_data);
    m.output("register_file_write_enable", enable & ready & m.input("rd_value_write_enable", 1) & instruction.rd().ne(m.lit(0u32, 5)));

    m
}
