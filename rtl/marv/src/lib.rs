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
    pc(c);
    cycle_counter(c);
    instructions_retired_counter(c);
    control(c);
    instruction_fetch(c);
    decode(c);
    execute(c);
    mem(c);
    writeback(c);

    let m = c.module("marv");

    let control0 = m.instance("control0", "control");

    let pc0 = m.instance("pc0", "pc");
    let cycle_counter0 = m.instance("cycle_counter0", "cycle_counter");
    let instructions_retired_counter0 = m.instance("instructions_retired_counter0", "instructions_retired_counter");

    let bus_ready = m.input("bus_ready", 1);
    let bus_read_data = m.input("bus_read_data", 32);
    let bus_read_data_valid = m.input("bus_read_data_valid", 1);

    let instruction_fetch0 = m.instance("instruction_fetch0", "instruction_fetch");
    control0.drive_input("instruction_fetch_ready", instruction_fetch0.output("ready"));
    instruction_fetch0.drive_input("enable", control0.output("instruction_fetch_enable"));
    instruction_fetch0.drive_input("pc", pc0.output("value").bits(31, 2));
    instruction_fetch0.drive_input("bus_ready", bus_ready);

    let decode0 = m.instance("decode0", "decode");
    control0.drive_input("decode_ready", decode0.output("ready"));
    decode0.drive_input("bus_read_data", bus_read_data);
    decode0.drive_input("bus_read_data_valid", bus_read_data_valid);

    let instruction = m.reg("instruction", 32);
    instruction.drive_next(control0.output("decode_enable").mux(decode0.output("instruction"), instruction.value));
    let instruction = Instruction::new(instruction.value);

    m.output("register_file_read_addr1", instruction.rs1());
    m.output("register_file_read_addr2", instruction.rs2());

    // TODO: If we refactor the register file mem to register its outputs, these can just be inputs (unless writeback causes problems with that somehow!)
    let rs1_value = m.reg("rs1_value", 32);
    rs1_value.drive_next(control0.output("reg_wait_enable").mux(m.input("register_file_read_data1", 32), rs1_value.value));
    let rs2_value = m.reg("rs2_value", 32);
    rs2_value.drive_next(control0.output("reg_wait_enable").mux(m.input("register_file_read_data2", 32), rs2_value.value));

    let execute0 = m.instance("execute0", "execute");
    execute0.drive_input("pc", pc0.output("value"));
    execute0.drive_input("instruction", instruction.value);
    execute0.drive_input("register_file_read_data1", m.input("register_file_read_data1", 32));
    execute0.drive_input("register_file_read_data2", m.input("register_file_read_data2", 32));
    m.output("alu_op", execute0.output("alu_op"));
    m.output("alu_op_mod", execute0.output("alu_op_mod"));
    m.output("alu_lhs", execute0.output("alu_lhs"));
    m.output("alu_rhs", execute0.output("alu_rhs"));
    execute0.drive_input("alu_res", m.input("alu_res", 32));
    execute0.drive_input("cycle_counter_value", cycle_counter0.output("value"));
    execute0.drive_input("instructions_retired_counter_value", instructions_retired_counter0.output("value"));

    let mem0 = m.instance("mem0", "mem");
    control0.drive_input("mem_ready", mem0.output("ready"));
    mem0.drive_input("enable", control0.output("mem_enable"));
    mem0.drive_input("bus_ready", bus_ready);
    mem0.drive_input("bus_addr_in", execute0.output("bus_addr"));
    mem0.drive_input("bus_write_data_in", execute0.output("bus_write_data"));
    mem0.drive_input("bus_byte_enable_in", execute0.output("bus_byte_enable"));
    mem0.drive_input("bus_read_req_in", execute0.output("bus_read_req"));
    mem0.drive_input("bus_write_req_in", execute0.output("bus_write_req"));
    m.output("bus_write_data", mem0.output("bus_write_data_out"));

    let writeback0 = m.instance("writeback0", "writeback");
    control0.drive_input("writeback_ready", writeback0.output("ready"));
    writeback0.drive_input("enable", control0.output("writeback_enable"));
    writeback0.drive_input("instruction", instruction.value);
    writeback0.drive_input("bus_addr_low", mem0.output("bus_addr_out").bits(1, 0));
    writeback0.drive_input("next_pc", execute0.output("next_pc"));
    writeback0.drive_input("rd_value_write_enable", execute0.output("rd_value_write_enable"));
    writeback0.drive_input("rd_value_write_data", execute0.output("rd_value_write_data"));
    pc0.drive_input("write_data", writeback0.output("pc_write_data"));
    pc0.drive_input("write_enable", writeback0.output("pc_write_enable"));
    instructions_retired_counter0.drive_input("increment_enable", writeback0.output("instructions_retired_counter_increment_enable"));
    m.output("register_file_write_enable", writeback0.output("register_file_write_enable"));
    m.output("register_file_write_addr", writeback0.output("register_file_write_addr"));
    m.output("register_file_write_data", writeback0.output("register_file_write_data"));
    writeback0.drive_input("bus_read_data", bus_read_data);
    writeback0.drive_input("bus_read_data_valid", bus_read_data_valid);

    let mem_bus_read_req = mem0.output("bus_read_req_out");
    let mem_bus_write_req = mem0.output("bus_write_req_out");
    m.output("bus_addr", (mem_bus_read_req | mem_bus_write_req).mux(mem0.output("bus_addr_out").bits(31, 2), instruction_fetch0.output("bus_addr")));
    m.output("bus_byte_enable", (mem_bus_read_req | mem_bus_write_req).mux(mem0.output("bus_byte_enable_out"), instruction_fetch0.output("bus_byte_enable")));
    m.output("bus_read_req", mem_bus_read_req | instruction_fetch0.output("bus_read_req"));
    m.output("bus_write_req", mem_bus_write_req);

    m
}

fn pc<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("pc");

    let value = m.reg("value", 32);
    value.default_value(0x10000000u32);
    value.drive_next(m.input("write_enable", 1).mux(m.input("write_data", 32), value.value));
    m.output("value", value.value);

    m
}

fn cycle_counter<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("cycle_counter");

    let value = m.reg("value", 64);
    value.default_value(0u64);
    value.drive_next(value.value + m.lit(1u64, 64));
    m.output("value", value.value);

    m
}

fn instructions_retired_counter<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("instructions_retired_counter");

    let value = m.reg("value", 64);
    value.default_value(0u64);
    value.drive_next(m.input("increment_enable", 1).mux(value.value + m.lit(1u64, 64), value.value));
    m.output("value", value.value);

    m
}

fn control<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("control");

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
    let mut next_state = state.value;
    kaze_sugar! {
        // TODO: Enum matching sugar
        if (state.value.eq(m.lit(state_instruction_fetch, state_bit_width)) & m.input("instruction_fetch_ready", 1)) {
            next_state = m.lit(state_decode, state_bit_width);
        }
        if (state.value.eq(m.lit(state_decode, state_bit_width)) & m.input("decode_ready", 1)) {
            next_state = m.lit(state_reg_wait, state_bit_width);
        }
        if (state.value.eq(m.lit(state_reg_wait, state_bit_width))) {
            next_state = m.lit(state_execute, state_bit_width);
        }
        if (state.value.eq(m.lit(state_execute, state_bit_width))) {
            next_state = m.lit(state_mem, state_bit_width);
        }
        if (state.value.eq(m.lit(state_mem, state_bit_width)) & m.input("mem_ready", 1)) {
            next_state = m.lit(state_writeback, state_bit_width);
        }
        if (state.value.eq(m.lit(state_writeback, state_bit_width)) & m.input("writeback_ready", 1)) {
            next_state = m.lit(state_instruction_fetch, state_bit_width);
        }
    }
    state.drive_next(next_state);

    m.output("instruction_fetch_enable", state.value.eq(m.lit(state_instruction_fetch, state_bit_width)));
    m.output("decode_enable", state.value.eq(m.lit(state_decode, state_bit_width)));
    m.output("reg_wait_enable", state.value.eq(m.lit(state_reg_wait, state_bit_width)));
    m.output("mem_enable", state.value.eq(m.lit(state_mem, state_bit_width)));
    m.output("writeback_enable", state.value.eq(m.lit(state_writeback, state_bit_width)));

    m
}

fn instruction_fetch<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("instruction_fetch");

    m.output("ready", m.input("bus_ready", 1));
    m.output("bus_addr", m.input("pc", 30));
    m.output("bus_byte_enable", m.high().repeat(4));
    m.output("bus_read_req", m.input("enable", 1));

    m
}

fn decode<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("decode");

    m.output("ready", m.input("bus_read_data_valid", 1));
    m.output("instruction", m.input("bus_read_data", 32));

    m
}

fn execute<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("execute");

    let instruction = Instruction::new(m.input("instruction", 32));

    let rs1_value = m.input("register_file_read_data1", 32);
    let rs2_value = m.input("register_file_read_data2", 32);

    let mut alu_op = instruction.funct3();
    let mut alu_op_mod = m.low();
    m.output("alu_lhs", rs1_value);
    let mut alu_rhs = rs2_value;
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
    let mut bus_byte_enable = m.lit(0b1111u32, 4);

    kaze_sugar! {
        if (instruction.funct3().bits(1, 0).eq(m.lit(0b01u32, 2))) {
            // lh/lhu/sh
            bus_byte_enable = m.lit(0b0011u32, 4);
            if (bus_addr.bit(1)) {
                bus_byte_enable = m.lit(0b1100u32, 4);
            }
        }

        if (instruction.funct3().bits(1, 0).eq(m.lit(0b00u32, 2))) {
            // lb/lbu/sb
            bus_byte_enable = m.lit(0b0001u32, 4);
            if (bus_addr.bits(1, 0).eq(m.lit(0b01u32, 2))) {
                bus_byte_enable = m.lit(0b0010u32, 4);
            }
            if (bus_addr.bits(1, 0).eq(m.lit(0b10u32, 2))) {
                bus_byte_enable = m.lit(0b0100u32, 4);
            }
            if (bus_addr.bits(1, 0).eq(m.lit(0b11u32, 2))) {
                bus_byte_enable = m.lit(0b1000u32, 4);
            }
        }
    }

    m.output("bus_byte_enable", bus_byte_enable);

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
    let mut bus_write_data = rs2_value;
    let mut bus_write_req = m.low();

    kaze_sugar! {
        if (instruction.opcode().eq(m.lit(0b01000u32, 5))) {
            // sw
            alu_op = m.lit(0u32, 3);
            alu_op_mod = m.low();
            alu_rhs = instruction.store_offset();
            rd_value_write_enable = m.low();
            bus_write_req = m.high();

            if (instruction.funct3().bits(1, 0).eq(m.lit(0b01u32, 2))) {
                // sh
                if (bus_addr.bit(1)) {
                    bus_write_data = rs2_value.bits(15, 0).concat(m.lit(0u32, 16));
                }
            }

            if (instruction.funct3().bits(1, 0).eq(m.lit(0b00u32, 2))) {
                // sb
                if (bus_addr.bits(1, 0).eq(m.lit(0b01u32, 2))) {
                    bus_write_data = m.lit(0u32, 16).concat(rs2_value.bits(7, 0)).concat(m.lit(0u32, 8));
                }
                if (bus_addr.bits(1, 0).eq(m.lit(0b10u32, 2))) {
                    bus_write_data = m.lit(0u32, 8).concat(rs2_value.bits(7, 0)).concat(m.lit(0u32, 16));
                }
                if (bus_addr.bits(1, 0).eq(m.lit(0b11u32, 2))) {
                    bus_write_data = rs2_value.bits(7, 0).concat(m.lit(0u32, 24));
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
    let mut branch_taken = m.low();
    kaze_sugar! {
        // TODO: switch/case construct?
        if (instruction.funct3().bits(2, 1).eq(m.lit(0b00u32, 2))) {
            branch_taken = rs1_value.eq(rs2_value);
        }
        if (instruction.funct3().bits(2, 1).eq(m.lit(0b10u32, 2))) {
            branch_taken = rs1_value.lt_signed(rs2_value);
        }
        if (instruction.funct3().bits(2, 1).eq(m.lit(0b11u32, 2))) {
            branch_taken = rs1_value.lt(rs2_value);
        }
        if (instruction.funct3().bit(0)) {
            branch_taken = !branch_taken;
        }
        if (instruction.opcode().eq(m.lit(0b11000u32, 5))) {
            rd_value_write_enable = m.low();

            if (branch_taken) {
                next_pc = pc + instruction.branch_offset(m);
            }
        }
    }

    m.output("next_pc", next_pc);

    // Fence instructions
    kaze_sugar! {
        if (instruction.opcode().eq(m.lit(0b00011u32, 5))) {
            // Do nothing (nop)
            rd_value_write_enable = m.low();
        }
    }

    let cycle_counter_value = m.input("cycle_counter_value", 64);
    let instructions_retired_counter_value = m.input("instructions_retired_counter_value", 64);

    // System instructions
    kaze_sugar! {
        if (instruction.opcode().eq(m.lit(0b11100u32, 5))) {
            if (instruction.funct3().eq(m.lit(0b000u32, 3))) {
                // ecall/ebreak: do nothing (nop)
                rd_value_write_enable = m.low();
            }

            if (
                instruction.funct3().eq(m.lit(0b001u32, 3)) |
                instruction.funct3().eq(m.lit(0b010u32, 3)) |
                instruction.funct3().eq(m.lit(0b011u32, 3)) |
                instruction.funct3().eq(m.lit(0b101u32, 3)) |
                instruction.funct3().eq(m.lit(0b110u32, 3)) |
                instruction.funct3().eq(m.lit(0b111u32, 3))) {
                // csrrw, csrrs, csrrc, csrrwi, csrrsi, csrrci
                if (instruction.csr().bits(1, 0).eq(m.lit(0b00u32, 2)) | instruction.csr().bits(1, 0).eq(m.lit(0b01u32, 2))) {
                    // cycle, time
                    rd_value_write_data = cycle_counter_value.bits(31, 0);
                    if (instruction.csr().bit(7)) {
                        // cycleh, timeh
                        rd_value_write_data = cycle_counter_value.bits(63, 32);
                    }
                }
                if (instruction.csr().bits(1, 0).eq(m.lit(0b10u32, 2))) {
                    // instret
                    rd_value_write_data = instructions_retired_counter_value.bits(31, 0);
                    if (instruction.csr().bit(7)) {
                        // instreth
                        rd_value_write_data = instructions_retired_counter_value.bits(63, 32);
                    }
                }
            }
        }
    }

    m.output("rd_value_write_enable", rd_value_write_enable);
    m.output("rd_value_write_data", rd_value_write_data);

    m
}

fn mem<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("mem");

    let enable = m.input("enable", 1);

    let bus_ready = m.input("bus_ready", 1);
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

    let mut ready = m.high();
    kaze_sugar! {
        if (bus_read_req.value | bus_write_req.value) {
            ready = bus_ready;
        }
    }

    m.output("ready", ready);

    m
}

fn writeback<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("writeback");

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

            if (instruction.funct3().bits(1, 0).eq(m.lit(0b00u32, 2))) {
                // lb/lbu
                register_file_write_data = bus_read_data.bit(7).repeat(24).concat(bus_read_data.bits(7, 0));
                if (bus_addr_low.bits(1, 0).eq(m.lit(0b01u32, 2))) {
                    register_file_write_data = bus_read_data.bit(15).repeat(24).concat(bus_read_data.bits(15, 8));
                }
                if (bus_addr_low.bits(1, 0).eq(m.lit(0b10u32, 2))) {
                    register_file_write_data = bus_read_data.bit(23).repeat(24).concat(bus_read_data.bits(23, 16));
                }
                if (bus_addr_low.bits(1, 0).eq(m.lit(0b11u32, 2))) {
                    register_file_write_data = bus_read_data.bit(31).repeat(24).concat(bus_read_data.bits(31, 24));
                }

                if (instruction.funct3().bit(2)) {
                    register_file_write_data = m.lit(0u32, 24).concat(register_file_write_data.bits(7, 0));
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
