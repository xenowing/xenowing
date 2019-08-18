from kaze import *

class Instruction:
    def __init__(self, source):
        if source.num_bits() != 32:
            raise Exception('instruction must be 32 bits')
        self.source = source

    def word(self):
        return self.source

    def opcode(self):
        return self.source.bits(6, 2) # Bottom two bits are always 0b11 for RV32I, so just ignore them

    def rs1(self):
        return self.source.bits(19, 15)

    def rs2(self):
        return self.source.bits(24, 20)

    def rd(self):
        return self.source.bits(11, 7)

    def funct3(self):
        return self.source.bits(14, 12)

    def load_offset(self):
        return repeat(self.source.bit(31), 20).concat(self.source.bits(31, 20))

    def store_offset(self):
        return repeat(self.source.bit(31), 20).concat(self.source.bits(31, 25)).concat(self.source.bits(11, 7))

    def jump_offset(self):
        return repeat(self.source.bit(31), 11).concat(self.source.bit(31)).concat(self.source.bits(19, 12)).concat(self.source.bit(20)).concat(self.source.bits(30, 21)).concat(LOW)

    def branch_offset(self):
        return repeat(self.source.bit(31), 19).concat(self.source.bit(31)).concat(self.source.bit(7)).concat(self.source.bits(30, 25)).concat(self.source.bits(11, 8)).concat(LOW)

    def i_immediate(self):
        return repeat(self.source.bit(31), 20).concat(self.source.bits(31, 20))

    def u_immediate(self):
        return self.source.bits(31, 12).concat(lit(0, 12))

    def csr(self):
        return self.source.bits(31, 20)

def pc():
    mod = Module('pc')

    value = reg(32, 0x10000000)
    value.drive_next_with(mux(value, mod.input('write_data', 32), mod.input('write_enable', 1)))
    mod.output('value', value)

    return mod

def cycle_counter():
    mod = Module('cycle_counter')

    value = reg(64, 0)
    value.drive_next_with((value + lit(1, 64)).bits(63, 0))
    mod.output('value', value)

    return mod

def instructions_retired_counter():
    mod = Module('instructions_retired_counter')

    value = reg(64, 0)
    value.drive_next_with(mux(value, (value + lit(1, 64)).bits(63, 0), mod.input('increment_enable', 1)))
    mod.output('value', value)

    return mod

def control():
    mod = Module('control')

    num_state_bits = 2
    state_instruction_fetch = 0
    state_decode = 1
    state_execute_mem = 2
    state_writeback = 3
    state = reg(num_state_bits, state_instruction_fetch)
    next_state = state
    with If(state.eq(lit(state_instruction_fetch, num_state_bits)) & mod.input('instruction_fetch_ready', 1)):
        next_state = lit(state_decode, num_state_bits)
    with If(state.eq(lit(state_decode, num_state_bits)) & mod.input('decode_ready', 1)):
        next_state = lit(state_execute_mem, num_state_bits)
    with If(state.eq(lit(state_execute_mem, num_state_bits)) & mod.input('execute_mem_ready', 1)):
        next_state = lit(state_writeback, num_state_bits)
    with If(state.eq(lit(state_writeback, num_state_bits)) & mod.input('writeback_ready', 1)):
        next_state = lit(state_instruction_fetch, num_state_bits)
    state.drive_next_with(next_state)

    mod.output('instruction_fetch_enable', state.eq(lit(state_instruction_fetch, num_state_bits)))
    mod.output('decode_enable', state.eq(lit(state_decode, num_state_bits)))
    mod.output('execute_mem_enable', state.eq(lit(state_execute_mem, num_state_bits)))
    mod.output('writeback_enable', state.eq(lit(state_writeback, num_state_bits)))

    return mod

def instruction_fetch():
    mod = Module('instruction_fetch')

    mod.output('ready', mod.input('bus_ready', 1))
    mod.output('bus_addr', mod.input('pc', 30))
    mod.output('bus_byte_enable', repeat(HIGH, 4))
    mod.output('bus_read_req', mod.input('enable', 1))

    return mod

def decode():
    mod = Module('decode')

    mod.output('ready', mod.input('bus_read_data_valid', 1))
    mod.output('instruction', mod.input('bus_read_data', 32))

    return mod

def execute_mem():
    mod = Module('execute_mem')

    ready = HIGH
    enable = mod.input('enable', 1)

    instruction = Instruction(mod.input('instruction', 32))

    rs1_value = mod.input('register_file_read_data1', 32)
    rs2_value = mod.input('register_file_read_data2', 32)

    alu_op = instruction.funct3()
    alu_op_mod = LOW
    alu_lhs = rs1_value
    mod.output('alu_lhs', alu_lhs)
    alu_rhs = rs2_value
    alu_res = mod.input('alu_res', 32)

    reg_comp = instruction.opcode().bit(3)

    with If(reg_comp):
        # Register computation
        alu_op_mod = instruction.word().bit(30)

    with If(~reg_comp):
        # Immediate computation
        alu_rhs = instruction.i_immediate()

        # Shifts treat alu_op_mod the same as register computations and use rs2 directly (not its register value)
        with If(instruction.funct3().eq(lit(0b001, 3)) | instruction.funct3().eq(lit(0b101, 3))):
            alu_op_mod = instruction.word().bit(30)
            alu_rhs = lit(0, 27).concat(instruction.rs2())

    pc = mod.input('pc', 32)
    link_pc = (pc + lit(4, 32)).bits(31, 0)
    next_pc = link_pc

    rd_value_write_enable = HIGH
    rd_value_write_data = alu_res

    with If(instruction.opcode().eq(lit(0b01101, 5))):
        # lui
        rd_value_write_data = instruction.u_immediate()

    with If(instruction.opcode().eq(lit(0b00101, 5))):
        # auipc
        rd_value_write_data = (instruction.u_immediate() + pc).bits(31, 0)

    with If(instruction.opcode().eq(lit(0b11011, 5))):
        # jal
        next_pc = (pc + instruction.jump_offset()).bits(31, 0)
        rd_value_write_data = link_pc

    with If(instruction.opcode().eq(lit(0b11001, 5))):
        # jalr
        alu_rhs = instruction.i_immediate()
        next_pc = alu_res
        rd_value_write_data = link_pc

    bus_ready = mod.input('bus_ready', 1)
    bus_addr = alu_res # TODO: Consider separate adder for load/store offsets
    mod.output('bus_addr', bus_addr)
    bus_byte_enable = lit(0b1111, 4)

    with If(instruction.funct3().bits(1, 0).eq(lit(0b01, 2))):
        # lh/lhu/sh
        bus_byte_enable = lit(0b0011, 4)
        with If(bus_addr.bit(1)):
            bus_byte_enable = lit(0b1100, 4)

    with If(instruction.funct3().bits(1, 0).eq(lit(0b00, 2))):
        # lb/lbu/sb
        bus_byte_enable = lit(0b0001, 4)
        with If(bus_addr.bits(1, 0).eq(lit(0b01, 2))):
            bus_byte_enable = lit(0b0010, 4)
        with If(bus_addr.bits(1, 0).eq(lit(0b10, 2))):
            bus_byte_enable = lit(0b0100, 4)
        with If(bus_addr.bits(1, 0).eq(lit(0b11, 2))):
            bus_byte_enable = lit(0b1000, 4)

    mod.output('bus_byte_enable', bus_byte_enable)

    # Loads
    bus_read_req = LOW

    with If(instruction.opcode().eq(lit(0b00000, 5))):
        # lw
        ready = bus_ready
        alu_op = lit(0, 3)
        alu_op_mod = LOW
        alu_rhs = instruction.load_offset()
        bus_read_req = enable

    mod.output('bus_read_req', bus_read_req)

    # Stores
    bus_write_data = rs2_value
    bus_write_req = LOW

    with If(instruction.opcode().eq(lit(0b01000, 5))):
        # sw
        ready = bus_ready
        alu_op = lit(0, 3)
        alu_op_mod = LOW
        alu_rhs = instruction.store_offset()
        rd_value_write_enable = LOW
        bus_write_req = enable

        with If(instruction.funct3().bits(1, 0).eq(lit(0b01, 2))):
            # sh
            with If(bus_addr.bit(1)):
                bus_write_data = rs2_value.bits(15, 0).concat(lit(0, 16))

        with If(instruction.funct3().bits(1, 0).eq(lit(0b00, 2))):
            # sb
            with If(bus_addr.bits(1, 0).eq(lit(0b01, 2))):
                bus_write_data = lit(0, 16).concat(rs2_value.bits(7, 0)).concat(lit(0, 8))
            with If(bus_addr.bits(1, 0).eq(lit(0b10, 2))):
                bus_write_data = lit(0, 8).concat(rs2_value.bits(7, 0)).concat(lit(0, 16))
            with If(bus_addr.bits(1, 0).eq(lit(0b11, 2))):
                bus_write_data = rs2_value.bits(7, 0).concat(lit(0, 24))

    mod.output('ready', ready)

    mod.output('alu_op', alu_op)
    mod.output('alu_op_mod', alu_op_mod)
    mod.output('alu_rhs', alu_rhs)

    mod.output('bus_write_data', bus_write_data)
    mod.output('bus_write_req', bus_write_req)

    # Branch instructions
    branch_taken = LOW
    # TODO: switch/case construct?
    with If(instruction.funct3().bits(2, 1).eq(lit(0b00, 2))):
        branch_taken = rs1_value.eq(rs2_value)
    with If(instruction.funct3().bits(2, 1).eq(lit(0b10, 2))):
        branch_taken = rs1_value.lt_signed(rs2_value)
    with If(instruction.funct3().bits(2, 1).eq(lit(0b11, 2))):
        branch_taken = rs1_value < rs2_value
    with If(instruction.funct3().bit(0)):
        branch_taken = ~branch_taken
    with If(instruction.opcode().eq(lit(0b11000, 5))):
        rd_value_write_enable = LOW

        with If(branch_taken):
            next_pc = (pc + instruction.branch_offset()).bits(31, 0)

    mod.output('next_pc', next_pc)

    # Fence instructions
    with If(instruction.opcode().eq(lit(0b00011, 5))):
        # Do nothing (nop)
        rd_value_write_enable = LOW

    cycle_counter_value = mod.input('cycle_counter_value', 64)
    instructions_retired_counter_value = mod.input('instructions_retired_counter_value', 64)

    # System instructions
    with If(instruction.opcode().eq(lit(0b11100, 5))):
        with If(instruction.funct3().eq(lit(0b000, 3))):
            # ecall/ebreak: do nothing (nop)
            rd_value_write_enable = LOW

        with If(
            instruction.funct3().eq(lit(0b001, 3)) |
            instruction.funct3().eq(lit(0b010, 3)) |
            instruction.funct3().eq(lit(0b011, 3)) |
            instruction.funct3().eq(lit(0b101, 3)) |
            instruction.funct3().eq(lit(0b110, 3)) |
            instruction.funct3().eq(lit(0b111, 3))):
            # csrrw, csrrs, csrrc, csrrwi, csrrsi, csrrci
            with If(instruction.csr().bits(1, 0).eq(lit(0b00, 2)) | instruction.csr().bits(1, 0).eq(lit(0b01, 2))):
                # cycle, time
                rd_value_write_data = cycle_counter_value.bits(31, 0)
                with If(instruction.csr().bit(7)):
                    # cycleh, timeh
                    rd_value_write_data = cycle_counter_value.bits(63, 32)
            with If(instruction.csr().bits(1, 0).eq(lit(0b10, 2))):
                # instret
                rd_value_write_data = instructions_retired_counter_value.bits(31, 0)
                with If(instruction.csr().bit(7)):
                    # instreth
                    rd_value_write_data = instructions_retired_counter_value.bits(63, 32)

    mod.output('rd_value_write_enable', rd_value_write_enable)
    mod.output('rd_value_write_data', rd_value_write_data)

    return mod

def writeback():
    mod = Module('writeback')

    ready = HIGH

    instruction = Instruction(mod.input('instruction', 32))
    bus_addr_low = mod.input('bus_addr_low', 2)
    bus_read_data = mod.input('bus_read_data', 32)

    register_file_write_data = mod.input('rd_value_write_data', 32)

    # Loads
    with If(instruction.opcode().eq(lit(0b00000, 5))):
        # lw
        ready = mod.input('bus_read_data_valid', 1)
        register_file_write_data = bus_read_data

        with If(instruction.funct3().bits(1, 0).eq(lit(0b01, 2))):
            # lh/lhu
            register_file_write_data = bus_read_data.bit(15).repeat(16).concat(bus_read_data.bits(15, 0))
            with If(bus_addr_low.bit(1)):
                register_file_write_data = bus_read_data.bit(31).repeat(16).concat(bus_read_data.bits(31, 16))

            with If(instruction.funct3().bit(2)):
                register_file_write_data = lit(0, 16).concat(register_file_write_data.bits(15, 0))

        with If(instruction.funct3().bits(1, 0).eq(lit(0b00, 2))):
            # lb/lbu
            register_file_write_data = bus_read_data.bit(7).repeat(24).concat(bus_read_data.bits(7, 0))
            with If(bus_addr_low.bits(1, 0).eq(lit(0b01, 2))):
                register_file_write_data = bus_read_data.bit(15).repeat(24).concat(bus_read_data.bits(15, 8))
            with If(bus_addr_low.bits(1, 0).eq(lit(0b10, 2))):
                register_file_write_data = bus_read_data.bit(23).repeat(24).concat(bus_read_data.bits(23, 16))
            with If(bus_addr_low.bits(1, 0).eq(lit(0b11, 2))):
                register_file_write_data = bus_read_data.bit(31).repeat(24).concat(bus_read_data.bits(31, 24))

            with If(instruction.funct3().bit(2)):
                register_file_write_data = lit(0, 24).concat(register_file_write_data.bits(7, 0))

    mod.output('ready', ready)

    enable = mod.input('enable', 1)

    mod.output('pc_write_data', mod.input('next_pc', 32))
    mod.output('pc_write_enable', enable & ready)

    mod.output('instructions_retired_counter_increment_enable', enable & ready)

    mod.output('register_file_write_addr', instruction.rd())
    mod.output('register_file_write_data', register_file_write_data)
    mod.output('register_file_write_enable', enable & ready & mod.input('rd_value_write_enable', 1) & instruction.rd().ne(lit(0, 5)))

    return mod
