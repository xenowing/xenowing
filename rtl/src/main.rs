mod buster;
mod fifo;
mod helpers;
mod interconnect;
mod led_interface;
mod marv;
mod marv_interconnect_bridge;
mod mimas_a7;
mod peek_buffer;
mod uart;
mod uart_interface;
mod xenowing;

use kaze::*;

use std::io::{Result, stdout};

fn main() -> Result<()> {
    let c = Context::new();

    system_verilog::generate(uart::generate_rx(&c, 100000000, 460800), stdout())?;
    system_verilog::generate(uart::generate_tx(&c, 100000000, 460800), stdout())?;
    system_verilog::generate(mimas_a7::test::lfsr::generate(&c), stdout())?;
    system_verilog::generate(mimas_a7::test::uart::generate(&c), stdout())?;

    Ok(())
}
