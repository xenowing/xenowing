use kaze::*;
use rtl::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let file = File::create(&dest_path).unwrap();

    let c = Context::new();

    sim::generate(generate_top(&c), file)
}

fn generate_top<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Top");

    xenowing::generate(c);
    let xenowing = m.instance("xenowing", "Xenowing");

    m.output("leds", xenowing.output("leds"));

    let uart_rx = m.instance("uart_rx", "UartRx");
    uart_rx.drive_input("rx", xenowing.output("tx"));
    m.output("uart_tx_data", uart_rx.output("data"));
    m.output("uart_tx_data_valid", uart_rx.output("data_valid"));

    let uart_tx = m.instance("uart_tx", "UartTx");
    xenowing.drive_input("rx", uart_tx.output("tx"));
    m.output("uart_rx_ready", uart_tx.output("ready"));
    uart_tx.drive_input("data", m.input("uart_rx_data", 8));
    uart_tx.drive_input("enable", m.input("uart_rx_enable", 1));

    /*m.output("ddr3_interface_bus_enable", xenowing.output("ddr3_interface_bus_enable"));
    m.output("ddr3_interface_bus_addr", xenowing.output("ddr3_interface_bus_addr"));
    m.output("ddr3_interface_bus_write", xenowing.output("ddr3_interface_bus_write"));
    m.output("ddr3_interface_bus_write_data", xenowing.output("ddr3_interface_bus_write_data"));
    m.output("ddr3_interface_bus_write_byte_enable", xenowing.output("ddr3_interface_bus_write_byte_enable"));
    xenowing.drive_input("ddr3_interface_bus_ready", m.input("ddr3_interface_bus_ready", 1));
    xenowing.drive_input("ddr3_interface_bus_read_data", m.input("ddr3_interface_bus_read_data", 128));
    xenowing.drive_input("ddr3_interface_bus_read_data_valid", m.input("ddr3_interface_bus_read_data_valid", 1));*/

    m
}
