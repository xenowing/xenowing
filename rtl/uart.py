from kaze import *

def receiver(clock_freq, baud_rate):
    mod = Module('uart_receiver')

    # TODO: Two sync FF's before the actual sampling FF is probably overkill
    sync_1 = reg(1)
    sync_1.drive_next_with(mod.input('rx', 1))
    sync_2 = reg(1)
    sync_2.drive_next_with(sync_1)
    rx = reg(1)
    rx.drive_next_with(sync_2)

    # Sample at 4x baud_rate
    #  We should technically only need 2x due to nyquist/shannon, but due to slight rate differences we want some headroom, so we go for 4 instead
    tick_rate = baud_rate * 4
    clocks_per_tick = int(clock_freq / tick_rate)
    tick_counter = reg(32)
    tick = tick_counter.eq(lit(clocks_per_tick, 32))
    tick_counter.drive_next_with(mux((tick_counter + lit(1, 32)).bits(31, 0), lit(0, 32), tick))

    num_wait_counter_bits = 2
    wait_counter = reg(num_wait_counter_bits)
    next_wait_counter = wait_counter

    data = reg(8)
    next_data = data
    data_ready = reg(1)
    next_data_ready = LOW

    bit_counter = reg(3, 0)
    next_bit_counter = bit_counter

    num_state_bits = 2
    state_idle = 0
    state_start_bit_wait = 1
    state_input_bit = 2
    state_stop_bit_wait = 3
    state = reg(num_state_bits, state_idle)
    next_state = state
    with If(tick):
        next_wait_counter = (wait_counter + lit(1, num_wait_counter_bits)).bits(num_wait_counter_bits - 1, 0)

        with If(state.eq(lit(state_idle, num_state_bits))):
            with If(~rx):
                next_state = lit(state_start_bit_wait, num_state_bits)
                next_wait_counter = lit(0, num_wait_counter_bits)

        with If(state.eq(lit(state_start_bit_wait, num_state_bits))):
            with If(wait_counter.eq(lit(1, num_wait_counter_bits))):
                next_state = lit(state_input_bit, num_state_bits)
                next_wait_counter = lit(0, num_wait_counter_bits)

            with If(rx):
                # Input is probably spurious, go back to idle state
                next_state = lit(state_idle, num_state_bits)

        with If(state.eq(lit(state_input_bit, num_state_bits))):
            with If(wait_counter.eq(lit(3, num_wait_counter_bits))):
                next_data = rx.concat(data.bits(7, 1))
                next_bit_counter = (bit_counter + lit(1, 3)).bits(2, 0)

                with If(bit_counter.eq(lit(7, 3))):
                    next_state = lit(state_stop_bit_wait, num_state_bits)
                    next_data_ready = HIGH

        with If(state.eq(lit(state_stop_bit_wait, num_state_bits))):
            with If(wait_counter.eq(lit(3, num_wait_counter_bits))):
                next_state = lit(state_idle, num_state_bits)

    wait_counter.drive_next_with(next_wait_counter)

    data.drive_next_with(next_data)
    mod.output('data', data)
    data_ready.drive_next_with(next_data_ready)
    mod.output('data_ready', data_ready)

    bit_counter.drive_next_with(next_bit_counter)

    state.drive_next_with(next_state)

    return mod
