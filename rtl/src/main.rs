mod lfsr;
mod uart;

use kaze::*;

use std::io::{Result, stdout};

fn main() -> Result<()> {
    let c = Context::new();

    system_verilog::generate(lfsr::generate(&c), stdout())?;
    system_verilog::generate(uart::generate_rx(&c, 100000000, 460800), stdout())?;

    Ok(())
}
