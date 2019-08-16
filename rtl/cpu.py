from kaze import *

class Instruction:
    def __init__(self, source):
        if source.num_bits() != 32:
            raise Exception('instruction must be 32 bits')
        self.source = source

    def word(self):
        return self.source

    def opcode(self):
        return self.source.bits(6, 0)

    def rs1(self):
        return self.source.bits(19, 15)

    def rs2(self):
        return self.source.bits(24, 20)

    def funct3(self):
        return self.source.bits(14, 12)

    def load_offset(self):
        return repeat(20, self.source.bit(31)).concat(self.source.bits(31, 20))

def pc():
    mod = Module('pc')

    value = reg(32, 0x10000000)
    value.drive_next_with(mux(value, mod.input('write_data', 32), mod.input('write_enable', 1)))
    mod.output('value', value)

    return mod

def control():
    mod = Module('control')

    num_states = 4
    state_instruction_fetch = 0
    state_decode = 1
    state_execute_mem = 2
    state_writeback = 3
    state = reg(num_states, 1 << state_instruction_fetch)
    next_state = state
    with If(state.bit(state_instruction_fetch) & mod.input('instruction_fetch_ready', 1)):
        next_state = lit(1 << state_decode, num_states)
    with If(state.bit(state_decode) & mod.input('decode_ready', 1)):
        next_state = lit(1 << state_execute_mem, num_states)
    with If(state.bit(state_execute_mem) & mod.input('execute_mem_ready', 1)):
        next_state = lit(1 << state_writeback, num_states)
    with If(state.bit(state_writeback) & mod.input('writeback_ready', 1)):
        next_state = lit(1 << state_instruction_fetch, num_states)
    state.drive_next_with(next_state)

    mod.output('instruction_fetch_enable', state.bit(state_instruction_fetch))
    mod.output('decode_enable', state.bit(state_decode))
    mod.output('execute_mem_enable', state.bit(state_execute_mem))
    mod.output('writeback_enable', state.bit(state_writeback))

    return mod

def instruction_fetch():
    mod = Module('instruction_fetch')

    mod.output('ready', mod.input('system_bus_ready', 1))
    mod.output('system_bus_addr', mod.input('pc', 30))
    mod.output('system_bus_byte_enable', repeat(HIGH, 4))
    mod.output('system_bus_read_req', mod.input('enable', 1))

    return mod

def decode():
    mod = Module('decode')

    mod.output('ready', mod.input('system_bus_read_data_valid', 1))

    instruction = Instruction(mod.input('system_bus_read_data', 32))
    mod.output('instruction', instruction.word())
    mod.output('register_file_read_addr1', instruction.rs1())
    mod.output('register_file_read_addr2', instruction.rs2())

    return mod

def half_add(a, b):
    a.ensure_num_bits(1)
    b.ensure_num_bits(1)
    return a ^ b, a & b

def full_add(a, b, c):
    a.ensure_num_bits(1)
    b.ensure_num_bits(1)
    c.ensure_num_bits(1)
    x, y = half_add(a, b)
    s, z = half_add(x, c)
    return s, y | z

def add(a, b, carry_in = LOW):
    if a.num_bits() != b.num_bits():
        raise Exception('a and b must have the same number of bits')
    bit_sum, bit_carry_out = full_add(a.bit(0), b.bit(0), carry_in)
    acc = bit_sum, bit_carry_out
    for i in range(1, a.num_bits()):
        bit_sum, bit_carry_out = full_add(a.bit(i), b.bit(i), acc[1])
        acc = concat(bit_sum, acc[0]), bit_carry_out
    return acc

def alu():
    mod = Module('alu')

    op = mod.input('op', 3)
    op_mod = mod.input('op_mod', 1)

    lhs = mod.input('lhs', 32)
    rhs = mod.input('rhs', 32)
    shift_amt = rhs.bits(4, 0)

    # TODO
    sum, sum_carry_out = add(lhs, mux(rhs, ~rhs, op_mod), op_mod)
    mod.output('res', sum)

    return mod
