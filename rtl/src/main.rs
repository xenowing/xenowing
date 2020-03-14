mod uart;

use kaze::*;

use std::io::{self, Result};

fn main() -> Result<()> {
    let c = Context::new();

    system_verilog::generate(uart::generate_rx(&c, 100000000, 460800), io::stdout())
}
