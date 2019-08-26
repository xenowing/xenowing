from kaze import *

def display():
    mod = Module('display')

    # Pixel clock is 25mhz, which is 1/6 150mhz
    pixel_clock_counter_max = 6
    pixel_clock_counter_bits = 3
    pixel_clock_counter = reg(pixel_clock_counter_bits, 0)
    pixel_clock_counter_next = (pixel_clock_counter + lit(1, pixel_clock_counter_bits)).bits(pixel_clock_counter_bits - 1, 0)
    pixel_clock_counter_reset = pixel_clock_counter.eq(lit(pixel_clock_counter_max - 1, pixel_clock_counter_bits))
    pixel_clock_counter.drive_next_with(
        mux(
            pixel_clock_counter_next,
            lit(0, pixel_clock_counter_bits),
            pixel_clock_counter_reset))
    # Use top bit of next counter value for 50% duty cycle, shifted 180 degrees to ensure data (which should change on counter == 0) is stable before clock edge
    mod.output('pixel_clk', pixel_clock_counter_next.bit(pixel_clock_counter_bits - 1))

    pixel_counter_x = reg(10, 0)
    pixel_counter_y = reg(10, 0)
    next_pixel_counter_x = pixel_counter_x
    next_pixel_counter_y = pixel_counter_y

    frame_counter = reg(8, 0)
    next_frame_counter = frame_counter

    with If(pixel_clock_counter_reset):
        with If(pixel_counter_x.eq(lit(0, 10)) & pixel_counter_y.eq(lit(0, 10))):
            next_frame_counter = (frame_counter + lit(1, 8)).bits(7, 0)

        next_pixel_counter_x = (pixel_counter_x + lit(1, 10)).bits(9, 0)
        with If(pixel_counter_x.eq(lit(799, 10))):
            next_pixel_counter_x = lit(0, 10)
            next_pixel_counter_y = (pixel_counter_y + lit(1, 10)).bits(9, 0)
            with If(pixel_counter_y.eq(lit(524, 10))):
                next_pixel_counter_y = lit(0, 10)

    pixel_counter_x.drive_next_with(next_pixel_counter_x)
    pixel_counter_y.drive_next_with(next_pixel_counter_y)

    frame_counter.drive_next_with(next_frame_counter)

    vsync = reg(1)
    vsync.drive_next_with((pixel_counter_y >= lit(490, 10)) & (pixel_counter_y < lit(492, 10)))
    mod.output('vsync', vsync)
    hsync = reg(1)
    hsync.drive_next_with((pixel_counter_x >= lit(656, 10)) & (pixel_counter_x < lit(752, 10)))
    mod.output('hsync', hsync)
    data_enable = reg(1)
    data_enable.drive_next_with((pixel_counter_x < lit(640, 10)) & (pixel_counter_y < lit(480, 10)))
    mod.output('data_enable', data_enable)

    pixel_color = ((pixel_counter_x.bits(7, 0) ^ pixel_counter_y.bits(7, 0)) + frame_counter).bits(7, 0)
    pixel_red = reg(8)
    pixel_red.drive_next_with(pixel_color)
    pixel_green = reg(8)
    pixel_green.drive_next_with(pixel_color)
    pixel_blue = reg(8)
    pixel_blue.drive_next_with(pixel_color)
    mod.output('pixel_data', pixel_red.concat(pixel_green).concat(pixel_blue))

    return mod

def display_interface():
    mod = Module('display_interface')

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
