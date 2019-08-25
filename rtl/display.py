from kaze import *

def display_interface():
    mod = Module("display_interface")

    addr = mod.input('addr', 1) # We don't actually need this currently, but we'll leave it here until we do :)
    byte_enable = mod.input('byte_enable', 1)
    write_data = mod.input('write_data', 2)
    write_req = mod.input('write_req', 1)

    i2c_clk_out_n = reg(1)
    i2c_data_out_n = reg(1)
    i2c_clk_out_n.drive_next_with(mux(i2c_clk_out_n, ~write_data.bit(0), write_req & byte_enable))
    i2c_data_out_n.drive_next_with(mux(i2c_data_out_n, ~write_data.bit(1), write_req & byte_enable))

    mod.output('i2c_clk_out_n', i2c_clk_out_n)
    mod.output('i2c_data_out_n', i2c_data_out_n)

    sync_1 = reg(1)
    sync_1.drive_next_with(mod.input('i2c_clk_in', 1))
    i2c_clk_in = reg(1)
    i2c_clk_in.drive_next_with(sync_1)
    sync_2 = reg(1)
    sync_2.drive_next_with(mod.input('i2c_data_in', 1))
    i2c_data_in = reg(1)
    i2c_data_in.drive_next_with(sync_2)

    mod.output('read_data', concat(i2c_data_in, i2c_clk_in))

    read_data_valid = reg(1)
    read_data_valid.drive_next_with(mod.input('read_req', 1))
    mod.output('read_data_valid', read_data_valid)

    return mod
