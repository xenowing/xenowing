#!/usr/bin/env python

from kaze import *
import cpu
import uart
import display
from sys import argv

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

def system_bus():
    mod = Module('system_bus')

    addr = mod.input('addr', 30)
    write_data = mod.input('write_data', 32)
    byte_enable = mod.input('byte_enable', 4)
    write_req = mod.input('write_req', 1)
    read_req = mod.input('read_req', 1)

    mod.output('program_rom_interface_addr', addr.bits(11, 0))

    mod.output('led_interface_write_data', write_data.bits(2, 0))
    mod.output('led_interface_byte_enable', byte_enable.bit(0))

    mod.output('uart_transmitter_interface_addr', addr.bit(0))
    mod.output('uart_transmitter_interface_write_data', write_data.bits(7, 0))
    mod.output('uart_transmitter_interface_byte_enable', byte_enable.bit(0))

    mod.output('display_interface_addr', addr.bit(0))
    mod.output('display_interface_write_data', write_data.bits(1, 0))
    mod.output('display_interface_byte_enable', byte_enable.bit(0))

    mod.output('ddr3_interface_addr', addr.bits(24, 0))
    mod.output('ddr3_interface_write_data', write_data)
    mod.output('ddr3_interface_byte_enable', byte_enable)

    ready = HIGH
    read_data = mod.input('program_rom_interface_read_data', 32)
    read_data_valid = mod.input('program_rom_interface_read_data_valid', 1)

    with If(mod.input('led_interface_read_data_valid', 1)):
        read_data = lit(0, 29).concat(mod.input('led_interface_read_data', 3))
        read_data_valid = HIGH

    with If(mod.input('uart_transmitter_interface_read_data_valid', 1)):
        read_data = lit(0, 31).concat(mod.input('uart_transmitter_interface_read_data', 1))
        read_data_valid = HIGH

    with If(mod.input('display_interface_read_data_valid', 1)):
        read_data = lit(0, 30).concat(mod.input('display_interface_read_data', 2))
        read_data_valid = HIGH

    with If(mod.input('ddr3_interface_read_data_valid', 1)):
        read_data = mod.input('ddr3_interface_read_data', 32)
        read_data_valid = HIGH

    program_rom_interface_read_req = LOW

    led_interface_write_req = LOW
    led_interface_read_req = LOW

    uart_transmitter_interface_write_req = LOW
    uart_transmitter_interface_read_req = LOW

    display_interface_write_req = LOW
    display_interface_read_req = LOW

    ddr3_interface_write_req = LOW
    ddr3_interface_read_req = LOW

    # TODO: switch/case construct?
    with If(addr.bits(29, 26).eq(lit(1, 4))):
        program_rom_interface_read_req = read_req

    with If(addr.bits(29, 26).eq(lit(2, 4))):
        with If(addr.bits(23, 22).eq(lit(0, 2))):
            led_interface_write_req = write_req
            led_interface_read_req = read_req

        with If(addr.bits(23, 22).eq(lit(1, 2))):
            uart_transmitter_interface_write_req = write_req
            uart_transmitter_interface_read_req = read_req

        with If(addr.bits(23, 22).eq(lit(2, 2))):
            display_interface_write_req = write_req
            display_interface_read_req = read_req

    with If(addr.bits(29, 26).eq(lit(3, 4))):
        ready = mod.input('ddr3_interface_ready', 1)
        ddr3_interface_write_req = write_req
        ddr3_interface_read_req = read_req

    mod.output('ready', ready)
    mod.output('read_data', read_data)
    mod.output('read_data_valid', read_data_valid)

    mod.output('program_rom_interface_read_req', program_rom_interface_read_req)

    mod.output('led_interface_write_req', led_interface_write_req)
    mod.output('led_interface_read_req', led_interface_read_req)

    mod.output('uart_transmitter_interface_write_req', uart_transmitter_interface_write_req)
    mod.output('uart_transmitter_interface_read_req', uart_transmitter_interface_read_req)

    mod.output('display_interface_write_req', display_interface_write_req)
    mod.output('display_interface_read_req', display_interface_read_req)

    mod.output('ddr3_interface_write_req', ddr3_interface_write_req)
    mod.output('ddr3_interface_read_req', ddr3_interface_read_req)

    return mod

if __name__ == '__main__':
    output_file_name = argv[1]

    modules = [
        cpu.pc(),
        cpu.cycle_counter(),
        cpu.instructions_retired_counter(),
        cpu.control(),
        cpu.instruction_fetch(),
        cpu.decode(),
        cpu.execute(),
        cpu.mem(),
        cpu.writeback(),
        #fifo(),
        led_interface(),
        program_rom_interface(),
        system_bus(),
        uart.receiver(150000000, 115200),
        display.display_interface(),
        display.display(),
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
