#!/usr/bin/env python

from kaze import *
import uart
import display
from sys import argv

if __name__ == '__main__':
    output_file_name = argv[1]

    modules = [
        uart.receiver(100000000, 460800),
        display.display(),
        display.display_load_issue(),
        display.display_load_return(),
        display.display_interface(),
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
