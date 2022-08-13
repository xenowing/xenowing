mod approx_reciprocal;
mod boot_rom;
mod byte_ram;
mod buster;
mod color_thrust;
mod fifo;
mod flow_controlled_pipe;
mod led_interface;
mod marv;
mod marv_system_bridge;
mod mimas_a7;
mod peek_buffer;
mod read_cache;
mod uart;
mod uart_interface;
mod wire;
mod word_mem;
mod xenowing;

use mimas_a7::test::lfsr::*;
use mimas_a7::test::uart::*;
use uart::*;
use xenowing::*;

use kaze::*;

use std::io::{Result, stdout};

fn main() -> Result<()> {
    let c = Context::new();

    let xenowing = Xenowing::new("xenowing", &c);
    let lfsr = Lfsr::new("lfsr", &c);
    let uart = Uart::new("uart", &c);

    let clock_freq = 100000000;
    let uart_baud_rate = 460800;
    let uart_tx = UartTx::new("uart_tx", clock_freq, uart_baud_rate, &c);

    verilog::generate(xenowing.m, stdout())?;
    verilog::generate(lfsr.m, stdout())?;
    verilog::generate(uart.m, stdout())?;
    verilog::generate(uart_tx.m, stdout())?;

    Ok(())
}
