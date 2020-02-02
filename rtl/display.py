from kaze import *

def display():
    mod = Module('display')

    # Pixel clock is 25mhz, which is 1/4 100mhz
    pixel_clock_counter_max = 3
    pixel_clock_counter_bits = 2
    pixel_clock_counter = reg(pixel_clock_counter_bits, 0)
    pixel_clock_counter_reset = pixel_clock_counter.eq(lit(pixel_clock_counter_max, pixel_clock_counter_bits))
    pixel_clock_counter.drive_next_with(
        mux(
            (pixel_clock_counter + lit(1, pixel_clock_counter_bits)).bits(pixel_clock_counter_bits - 1, 0),
            lit(0, pixel_clock_counter_bits),
            pixel_clock_counter_reset))
    # Use top bit of counter value for 50% duty cycle, shifted 180 degrees to ensure data (which should change on counter reset to 1) is stable before clock edge
    mod.output('pixel_clk', pixel_clock_counter.bit(pixel_clock_counter_bits - 1))

    pixel_counter_x = reg(10, 0)
    pixel_counter_y = reg(10, 0)
    next_pixel_counter_x = pixel_counter_x
    next_pixel_counter_y = pixel_counter_y

    pixel_addr = reg(8)
    next_pixel_addr = pixel_addr

    mod.output('load_bus_read_addr_reset', pixel_counter_y.eq(lit(44, 10)))
    load_start = LOW

    with If(pixel_clock_counter_reset):
        # Increment pixel addr every 4th pixel
        with If((pixel_counter_x >= lit(160, 10)) & (pixel_counter_y >= lit(45, 10)) & pixel_counter_x.bits(1, 0).eq(lit(3, 2))):
            next_pixel_addr = (pixel_addr + lit(1, 8)).bits(7, 0)

        next_pixel_counter_x = (pixel_counter_x + lit(1, 10)).bits(9, 0)
        with If(pixel_counter_x.eq(lit(799, 10))):
            next_pixel_counter_x = lit(0, 10)
            next_pixel_counter_y = (pixel_counter_y + lit(1, 10)).bits(9, 0)
            with If(pixel_counter_y.eq(lit(524, 10))):
                next_pixel_counter_y = lit(0, 10)

            next_pixel_addr = lit(0, 8)

            # Dispatch new pixel loads at the beginning of every 4th scanline in active display area
            with If((pixel_counter_y >= lit(44, 10)) & pixel_counter_y.ne(lit(524, 10)) & pixel_counter_y.bits(1, 0).eq(lit(0, 2))):
                load_start = HIGH

    pixel_counter_x.drive_next_with(next_pixel_counter_x)
    pixel_counter_y.drive_next_with(next_pixel_counter_y)

    pixel_addr.drive_next_with(next_pixel_addr)
    mod.output('buffer_addr', pixel_addr.bits(7, 2))
    buffer_data = mod.input('buffer_data', 64)

    mod.output('load_start', load_start)

    mod.output('vblank', pixel_counter_y < lit(45, 10))

    vsync = reg(1)
    vsync.drive_next_with((pixel_counter_y >= lit(10, 10)) & (pixel_counter_y < lit(12, 10)))
    mod.output('vsync', vsync)
    hsync = reg(1)
    hsync.drive_next_with((pixel_counter_x >= lit(16, 10)) & (pixel_counter_x < lit(112, 10)))
    mod.output('hsync', hsync)
    data_enable = reg(1)
    data_enable.drive_next_with((pixel_counter_x >= lit(160, 10)) & (pixel_counter_y >= lit(45, 10)))
    mod.output('data_enable', data_enable)

    pixel_data = buffer_data.bits(15, 0)
    with If(pixel_addr.bits(1, 0).eq(lit(1, 2))):
        pixel_data = buffer_data.bits(31, 16)
    with If(pixel_addr.bits(1, 0).eq(lit(2, 2))):
        pixel_data = buffer_data.bits(47, 32)
    with If(pixel_addr.bits(1, 0).eq(lit(3, 2))):
        pixel_data = buffer_data.bits(63, 48)
    pixel_red = reg(8)
    pixel_red.drive_next_with(pixel_data.bits(15, 11).concat(lit(0, 3)))
    pixel_green = reg(8)
    pixel_green.drive_next_with(pixel_data.bits(10, 5).concat(lit(0, 2)))
    pixel_blue = reg(8)
    pixel_blue.drive_next_with(pixel_data.bits(4, 0).concat(lit(0, 3)))
    mod.output('pixel_data', pixel_red.concat(pixel_green).concat(pixel_blue))

    return mod

def display_load_issue():
    mod = Module('display_load_issue')

    framebuffer_base_addr = reg(14)
    framebuffer_base_addr.drive_next_with(mux(framebuffer_base_addr, mod.input('framebuffer_base_addr_data', 14), mod.input('framebuffer_base_addr_write_enable', 1)))

    bus_read_addr = reg(14)
    next_bus_read_addr = bus_read_addr
    mod.output('bus_read_addr', bus_read_addr)
    mod.output('bus_byte_enable', lit(0xff, 8))

    word_counter = reg(6)
    next_word_counter = word_counter

    with If(mod.input('start', 1)):
        next_word_counter = lit(0, 6)

    bus_read_req = LOW

    with If(word_counter < lit(40, 6)):
        bus_read_req = HIGH
        with If(mod.input('bus_ready', 1)):
            next_bus_read_addr = (bus_read_addr + lit(1, 14)).bits(13, 0)
            next_word_counter = (word_counter + lit(1, 6)).bits(5, 0)

    with If(mod.input('bus_read_addr_reset', 1)):
        next_bus_read_addr = framebuffer_base_addr

    bus_read_addr.drive_next_with(next_bus_read_addr)
    word_counter.drive_next_with(next_word_counter)

    mod.output('bus_read_req', bus_read_req)

    return mod

def display_load_return():
    mod = Module('display_load_return')

    word_counter = reg(6)
    next_word_counter = word_counter

    start = mod.input('start', 1)

    with If(start):
        next_word_counter = lit(0, 6)

    bus_read_data_valid = mod.input('bus_read_data_valid', 1)
    with If(bus_read_data_valid):
        next_word_counter = (word_counter + lit(1, 6)).bits(5, 0)

    word_counter.drive_next_with(next_word_counter)

    mod.output('load_addr', word_counter)
    mod.output('load_data', mod.input('bus_read_data', 64))
    mod.output('load_data_valid', bus_read_data_valid)

    return mod

def display_interface():
    mod = Module('display_interface')

    addr = mod.input('addr', 1)
    byte_enable = mod.input('byte_enable', 1)
    write_data = mod.input('write_data', 17)
    write_req = mod.input('write_req', 1)

    mod.output('read_data', LOW.concat(mod.input('display_vblank', 1)))

    read_data_valid = reg(1)
    read_data_valid.drive_next_with(mod.input('read_req', 1))
    mod.output('read_data_valid', read_data_valid)

    mod.output('display_load_issue_framebuffer_base_addr_data', write_data.bits(16, 3))
    mod.output('display_load_issue_framebuffer_base_addr_write_enable', write_req)

    return mod
