#!/usr/bin/env python

from kaze import *
from sys import argv

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

x = LOW
y = ~x
z = mux(x, y, HIGH)

h, c = half_add(x, y)
f, c = full_add(x, y, z)

osc = reg(1)
osc.drive_next_with(~osc)

class Instruction:
    def __init__(self, source):
        if source.num_bits() != 32:
            raise Exception('instruction must be 32 bits')
        self.source = source

    def word(self):
        return self.source

    def rs1(self):
        return self.source.bits(19, 15)

    def rs2(self):
        return self.source.bits(24, 20)

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
    next_state = mux(
        next_state,
        lit(1 << state_decode, num_states),
        state.bit(state_instruction_fetch) & mod.input('instruction_fetch_ready', 1))
    next_state = mux(
        next_state,
        lit(1 << state_execute_mem, num_states),
        state.bit(state_decode) & mod.input('decode_ready', 1))
    next_state = mux(
        next_state,
        lit(1 << state_writeback, num_states),
        state.bit(state_execute_mem) & mod.input('execute_mem_ready', 1))
    next_state = mux(
        next_state,
        lit(1 << state_instruction_fetch, num_states),
        state.bit(state_writeback) & mod.input('writeback_ready', 1))
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

def fifo(data_width, depth_bits):
    mod = Module('fifo')

    write_data = mod.input('write_data', data_width)
    write_enable = mod.input('write_enable', 1)
    read_enable = mod.input('read_enable', 1)

    depth = 1 << depth_bits

    # TODO

    return mod

#fifo = fifo(32, 4)

def led_interface():
    mod = Module('led_interface')

    leds = reg(3)
    leds.drive_next_with(
        mux(
            leds,
            mod.input('write_data', 3),
            mod.input('write_req', 1) & mod.input('byte_enable', 1)))

    read_data_valid = reg(1)
    read_data_valid.drive_next_with(mod.input('read_req', 1))

    mod.output('read_data', leds)
    mod.output('read_data_valid', read_data_valid)

    mod.output('leds', leds)

    return mod

def program_rom_interface():
    mod = Module('program_rom_interface')

    mod.output('program_rom_addr', mod.input('addr', 12))
    mod.output('read_data', mod.input('program_rom_q', 32))

    read_data_valid = reg(1)
    read_data_valid.drive_next_with(mod.input('read_req', 1))
    mod.output('read_data_valid', read_data_valid)

    return mod

# TODO
#led_interface = led_interface_module.instantiate()

def ugly():
    mod = Module('ugly')

    x = mod.input('some_input', 1)
    y = repeat(~x | HIGH, 20)
    mod.output('some_output', y)
    mod.output('some_other_output', ~~~y)

    return mod

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

if __name__ == '__main__':
    output_file_name = argv[1]

    modules = [
        pc(),
        control(),
        instruction_fetch(),
        decode(),
        #fifo(),
        led_interface(),
        program_rom_interface(),
        #alu(),
        #ugly(),
    ]

    w = CodeWriter()

    w.append_line('/* verilator lint_off DECLFILENAME */')
    w.append_newline()

    w.append_line('`default_nettype none')
    w.append_newline()

    for module in modules:
        c = CodegenContext()

        module.gen_code(c, w)

    with open(output_file_name, 'w') as file:
        file.write(w.buffer)
