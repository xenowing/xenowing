use kaze::*;

use rtl::uart::*;
use rtl::xenowing::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let c = Context::new();

    let m = c.module("top", "Top");

    let xenowing = Xenowing::new("xenowing", m);

    m.output("leds", xenowing.leds);

    let clock_freq = 100000000;
    let uart_baud_rate = 460800;

    let uart_rx = UartRx::new("uart_rx", clock_freq, uart_baud_rate, m);
    uart_rx.rx.drive(xenowing.tx);
    m.output("uart_tx_data", uart_rx.data);
    m.output("uart_tx_data_valid", uart_rx.data_valid);

    let uart_tx = UartTx::new("uart_tx", clock_freq, uart_baud_rate, m);
    xenowing.rx.drive(uart_tx.tx);
    m.output("uart_rx_ready", uart_tx.ready);
    uart_tx.data.drive(m.input("uart_rx_data", 8));
    uart_tx.enable.drive(m.input("uart_rx_enable", 1));

    xenowing.ddr3.forward("ddr3", m);

    sim::generate(m, sim::GenerationOptions::default(), &mut file)?;

    let m = c.module("top", "TopInner");

    let inner = XenowingInner::new("inner", m);

    m.output("leds", inner.leds);

    m.output("uart_tx_data", inner.uart_tx_data);
    m.output("uart_tx_enable", inner.uart_tx_enable);
    inner.uart_tx_ready.drive(m.high());
    inner.uart_rx_data.drive(m.input("uart_rx_data", 8));
    inner.uart_rx_data_valid.drive(m.input("uart_rx_data_valid", 1));
    m.output("uart_rx_ready", inner.uart_rx_ready);

    inner.ddr3.forward("ddr3", m);

    sim::generate(m, sim::GenerationOptions::default(), file)
}
