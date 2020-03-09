use kaze::*;

pub fn generate_rx<'a>(c: &'a Context<'a>, clock_freq: u32, baud_rate: u32) -> &Module<'a> {
    let m = c.module("UartRx");

    // Requires external sync FF's
    let rx = m.input("rx", 1);

    // Sample at 4x baud_rate
    //  We should technically only need 2x due to nyquist/shannon, but due to slight rate differences we want some headroom, so we go for 4 instead
    let tick_rate = baud_rate * 4;
    let clocks_per_tick = clock_freq / tick_rate;
    let tick_counter = m.reg("tick_counter", 32);
    tick_counter.default_value(0u32);
    let tick = tick_counter.value.eq(m.lit(clocks_per_tick, 32));
    tick_counter.drive_next(tick.mux(m.lit(0u32, 32), tick_counter.value + m.lit(1u32, 32)));

    let wait_counter_bit_width = 2;
    let wait_counter = m.reg("wait_counter", wait_counter_bit_width);
    let next_wait_counter = wait_counter.value;

    let data = m.reg("data", 8);
    let next_data = data.value;
    let data_ready = m.reg("data_ready", 1);
    data_ready.default_value(false);
    let next_data_ready = m.low();

    let bit_counter = m.reg("bit_counter", 3);
    bit_counter.default_value(0u32);
        let next_bit_counter = bit_counter.value;

    let state_bit_width = 2;
    let state_idle = 0u32;
    let state_start_bit_wait = 1u32;
    let state_input_bit = 2u32;
    let state_stop_bit_wait = 3u32;
    let state = m.reg("state", state_bit_width);
    state.default_value(state_idle);
    let next_state = state.value;
    let (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state) = if_(tick, {
        let next_wait_counter = wait_counter.value + m.lit(1u32, wait_counter_bit_width);

        let (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state) = if_(state.value.eq(m.lit(state_idle, state_bit_width)), {
            if_(!rx, {
                let next_wait_counter = m.lit(0u32, wait_counter_bit_width);
                let next_state = m.lit(state_start_bit_wait, state_bit_width);

                (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
            }).else_({
                (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
            })
        }).else_if(state.value.eq(m.lit(state_start_bit_wait, state_bit_width)), {
            if_(rx, {
                // Input is probably spurious; go back to idle state
                let next_state = m.lit(state_idle, state_bit_width);

                (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
            }).else_if(wait_counter.value.eq(m.lit(1u32, wait_counter_bit_width)), {
                let next_wait_counter = m.lit(0u32, wait_counter_bit_width);
                let next_state = m.lit(state_input_bit, state_bit_width);

                (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
            }).else_({
                (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
            })
        }).else_if(state.value.eq(m.lit(state_input_bit, state_bit_width)), {
            let (next_data, next_data_ready, next_bit_counter, next_state) = if_(wait_counter.value.eq(m.lit(3u32, wait_counter_bit_width)), {
                let next_data = rx.concat(data.value.bits(7, 1));
                let next_bit_counter = bit_counter.value + m.lit(1u32, 3);

                if_(bit_counter.value.eq(m.lit(7u32, 3)), {
                    let next_data_ready = m.high();
                    let next_state = m.lit(state_stop_bit_wait, state_bit_width);

                    (next_data, next_data_ready, next_bit_counter, next_state)
                }).else_({
                    (next_data, next_data_ready, next_bit_counter, next_state)
                })
            }).else_({
                (next_data, next_data_ready, next_bit_counter, next_state)
            });

            (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
        }).else_({
            let next_state = if_(wait_counter.value.eq(m.lit(3u32, wait_counter_bit_width)), {
                m.lit(state_idle, state_bit_width)
            }).else_({
                next_state
            });

            (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
        });

        (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
    }).else_({
        (next_wait_counter, next_data, next_data_ready, next_bit_counter, next_state)
    });

    wait_counter.drive_next(next_wait_counter);

    data.drive_next(next_data);
    m.output("data", data.value);
    data_ready.drive_next(next_data_ready);
    m.output("data_ready", data_ready.value);

    bit_counter.drive_next(next_bit_counter);

    state.drive_next(next_state);

    m
}
