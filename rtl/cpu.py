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

def pc():
    mod = Module('pc')

    value = reg(32, 0x10000000)
    value.drive_next_with(mux(value, mod.input('write_data', 32), mod.input('write_enable', 1)))
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

    instruction = Instruction(mod.input('bus_read_data', 32))
    mod.output('instruction', instruction.word())
    mod.output('register_file_read_addr1', instruction.rs1())
    mod.output('register_file_read_addr2', instruction.rs2())

    return mod

def execute_mem():
    mod = Module('execute_mem')

    mod.output('ready', HIGH) # TODO: Disable if we're trying to issue a mem read/write but the bus isn't ready
    enable = mod.input('enable', 1)

    instruction = Instruction(mod.input('instruction', 32))

    mod.output('alu_op', instruction.funct3())
    alu_op_mod = LOW
    alu_lhs = mod.input('register_file_read_data1', 32)
    mod.output('alu_lhs', alu_lhs)
    alu_rhs = mod.input('register_file_read_data2', 32)
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

    mod.output('alu_op_mod', alu_op_mod)

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

    mod.output('alu_rhs', alu_rhs)

    # Branch instructions
    branch_taken = LOW
    # TODO: switch/case construct?
    with If(instruction.funct3().bits(2, 1).eq(lit(0b00, 2))):
        branch_taken = alu_lhs.eq(alu_rhs)
    with If(instruction.funct3().bits(2, 1).eq(lit(0b10, 2))):
        branch_taken = alu_lhs.lt_signed(alu_rhs)
    with If(instruction.funct3().bits(2, 1).eq(lit(0b11, 2))):
        branch_taken = alu_lhs < alu_rhs
    with If(~instruction.funct3().bit(0)):
        branch_taken = ~branch_taken
    with If(instruction.opcode().eq(lit(0b11000, 5))):
        rd_value_write_enable = LOW

        with If(branch_taken):
            next_pc = (pc + instruction.branch_offset()).bits(31, 0)

    mod.output('next_pc', next_pc)

    mod.output('rd_value_write_enable', rd_value_write_enable)
    mod.output('rd_value_write_data', rd_value_write_data)

    return mod

def writeback():
    mod = Module('writeback')

    ready = HIGH # TODO: Should be high if we didn't issue a mem read last stage, or if we did and we have data ready this cycle
    mod.output('ready', ready)

    instruction = Instruction(mod.input('instruction', 32))

    enable = mod.input('enable', 1)

    mod.output('pc_write_data', mod.input('next_pc', 32))
    mod.output('pc_write_enable', enable & ready)

    mod.output('register_file_write_addr', instruction.rd())
    mod.output('register_file_write_data', mod.input('rd_value_write_data', 32))
    mod.output('register_file_write_enable', enable & mod.input('rd_value_write_enable', 1) & instruction.rd().ne(lit(0, 5)))

    return mod
