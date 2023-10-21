mod approx_reciprocal;
mod bit_pusher;
mod boot_rom;
mod buster;
mod buster_mig_ui_bridge;
mod byte_ram;
mod color_thrust;
mod fifo;
mod flow_controlled_pipe;
mod led_interface;
mod marv;
mod marv_system_bridge;
mod mimas_a7;
mod peek_buffer;
mod pocket;
mod read_cache;
mod uart;
mod uart_interface;
mod wire;
mod word_mem;
mod xenowing;

use buster_mig_ui_bridge::*;
use mimas_a7::test::lfsr::*;
use mimas_a7::test::uart::*;
use pocket::video_test_pattern_generator;
use uart::*;
use xenowing::*;

use kaze::*;

use std::io::{stdout, Result};

fn main() -> Result<()> {
    let c = Context::new();

    let xenowing = Xenowing::new("xenowing", &c);
    let lfsr = Lfsr::new("lfsr", &c);
    let uart = Uart::new("uart", &c);
    let clock_freq = 100000000;
    let uart_baud_rate = 460800;
    let uart_tx = UartTx::new("uart_tx", clock_freq, uart_baud_rate, &c);
    let buster_mig_ui_bridge = BusterMigUiBridge::new("buster_mig_ui_bridge", 128, 24, &c);

    verilog::generate(xenowing.m, stdout())?;
    verilog::generate(lfsr.m, stdout())?;
    verilog::generate(uart.m, stdout())?;
    verilog::generate(uart_tx.m, stdout())?;
    verilog::generate(buster_mig_ui_bridge.m, stdout())?;
    verilog::generate(
        video_test_pattern_generator::VideoTestPatternGenerator::new(
            "video_test_pattern_generator",
            &c,
        )
        .m,
        stdout(),
    )?;

    Ok(())
}
