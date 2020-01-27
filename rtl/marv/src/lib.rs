use kaze::*;

struct Instruction<'a> {
    source: &'a Signal<'a>,
}

impl<'a> Instruction<'a> {
    fn new(source: &'a Signal<'a>) -> Instruction<'a> {
        if source.bit_width() != 32 {
            panic!("source bit width must be 32");
        }

        Instruction {
            source,
        }
    }

    fn word(&self) -> &'a Signal<'a> {
        self.source
    }

    fn opcode(&self) -> &'a Signal<'a> {
        self.source.bits(6, 2) // Bottom two bits are always 0b11 for RV32I, so just ignore them
    }

    fn rs1(&self) -> &'a Signal<'a> {
        self.source.bits(19, 15)
    }

    fn rs2(&self) -> &'a Signal<'a> {
        self.source.bits(24, 20)
    }

    fn rd(&self) -> &'a Signal<'a> {
        self.source.bits(11, 7)
    }

    fn funct3(&self) -> &'a Signal<'a> {
        self.source.bits(14, 12)
    }

    fn load_offset(&self) -> &'a Signal<'a> {
        self.source.bit(31).repeat(20).concat(self.source.bits(31, 20))
    }

    fn store_offset(&self) -> &'a Signal<'a> {
        self.source.bit(31).repeat(20).concat(self.source.bits(31, 25)).concat(self.source.bits(11, 7))
    }

    fn jump_offset(&self, m: &'a Module<'a>) -> &'a Signal<'a> {
        self.source.bit(31).repeat(11).concat(self.source.bit(31)).concat(self.source.bits(19, 12)).concat(self.source.bit(20)).concat(self.source.bits(30, 21)).concat(m.low())
    }

    fn branch_offset(&self, m: &'a Module<'a>) -> &'a Signal<'a> {
        self.source.bit(31).repeat(19).concat(self.source.bit(31)).concat(self.source.bit(7)).concat(self.source.bits(30, 25)).concat(self.source.bits(11, 8)).concat(m.low())
    }

    fn i_immediate(&self) -> &'a Signal<'a> {
        self.source.bit(31).repeat(20).concat(self.source.bits(31, 20))
    }

    fn u_immediate(&self, m: &'a Module<'a>) -> &'a Signal<'a> {
        self.source.bits(31, 12).concat(m.lit(0u32, 12))
    }

    fn csr(&self) -> &'a Signal<'a> {
        self.source.bits(31, 20)
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

    let m = c.module("marv");

    let pc0 = m.instance("pc0", "pc");
    let cycle_counter0 = m.instance("cycle_counter0", "cycle_counter");
    let instructions_retired_counter0 = m.instance("instructions_retired_counter0", "instructions_retired_counter");

    pc0.drive_input("write_enable", m.lit(false, 1));
    pc0.drive_input("write_data", m.lit(0xdeadbeefu32, 32));
    m.output("cycle_counter0_value_test", cycle_counter0.output("value"));
    instructions_retired_counter0.drive_input("increment_enable", m.lit(false, 1));
    m.output("instructions_retired_counter0_value_test", instructions_retired_counter0.output("value"));

    let control0 = m.instance("control0", "control");

    let instruction_fetch0 = m.instance("instruction_fetch0", "instruction_fetch");
    control0.drive_input("instruction_fetch_ready", instruction_fetch0.output("ready"));
    instruction_fetch0.drive_input("enable", control0.output("instruction_fetch_enable"));
    instruction_fetch0.drive_input("pc", pc0.output("value").bits(31, 2));
    instruction_fetch0.drive_input("bus_ready", m.input("bus_ready", 1));
    m.output("bus_addr", instruction_fetch0.output("bus_addr"));
    m.output("bus_byte_enable", instruction_fetch0.output("bus_byte_enable"));
    m.output("bus_read_req", instruction_fetch0.output("bus_read_req"));

    let decode0 = m.instance("decode0", "decode");
    control0.drive_input("decode_ready", decode0.output("ready"));
    decode0.drive_input("bus_read_data", m.input("bus_read_data", 32));
    decode0.drive_input("bus_read_data_valid", m.input("bus_read_data_valid", 1));

    let instruction = m.reg("instruction", 32);
    instruction.drive_next(control0.output("decode_enable").mux(decode0.output("instruction"), instruction.value));

    m.output("register_file_read_addr1", instruction.value.bits(19, 15)); // rs1
    m.output("register_file_read_addr2", instruction.value.bits(24, 20)); // rs2

    // TODO: If we refactor the register file mem to register its outputs, these can just be inputs (unless writeback causes problems with that somehow!)
    let rs1_value = m.reg("rs1_value", 32);
    rs1_value.drive_next(control0.output("reg_wait_enable").mux(m.input("register_file_read_data1", 32), rs1_value.value));
    let rs2_value = m.reg("rs2_value", 32);
    rs2_value.drive_next(control0.output("reg_wait_enable").mux(m.input("register_file_read_data2", 32), rs2_value.value));

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

    let alu_op = instruction.funct3();
    let mut alu_op_mod = m.low();
    m.output("alu_lhs", rs1_value);
    let mut alu_rhs = rs2_value;
    let alu_res = m.input("alh_res", 32);

    let reg_comp = instruction.opcode().bit(3);

    kaze_sugar! {
        if (reg_comp) {
            // Register computation
            alu_op_mod = instruction.word().bit(30);
        }

        if (!reg_comp) {
            // Immediate computation
            alu_rhs = instruction.i_immediate();

            // Shifts treat alu_op_mod the same as register computations and use rs2 directly (not its register value)
            if (instruction.funct3().eq(m.lit(0b001u32, 3)) | instruction.funct3().eq(m.lit(0b101u32, 3))) {
                alu_op_mod = instruction.word().bit(30);
                alu_rhs = m.lit(0u32, 27).concat(instruction.rs2());
            }
        }
    }

    let pc = m.input("pc", 32);
    let link_pc = (pc + m.lit(4u32, 32)).bits(31, 0);
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
