use crate::boot_rom::*;
use crate::buster::*;
use crate::byte_ram::*;
use crate::color_thrust::*;
use crate::led_interface::*;
use crate::marv::*;
use crate::marv_system_bridge::*;
use crate::uart::*;
use crate::uart_interface::*;

use kaze::*;

pub struct Xenowing<'a> {
    pub m: &'a Module<'a>,
    pub leds: &'a Output<'a>,
    pub tx: &'a Output<'a>,
    pub rx: &'a Input<'a>,
}

impl<'a> Xenowing<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> Xenowing<'a> {
        let m = p.module(instance_name, "Xenowing");

        let marv = Marv::new("marv", m);
        let marv_system_bridge = MarvSystemBridge::new("marv_system_bridge", m);
        marv.system_port.connect(&marv_system_bridge.marv_port);

        let boot_rom = BootRom::new("boot_rom", m);

        let program_ram = ByteRam::new("program_ram", 13, 20, m);

        let led_interface = LedInterface::new("led_interface", m);
        let leds = m.output("leds", led_interface.leds);

        let clock_freq = 100000000;
        let uart_baud_rate = 460800;

        let uart_tx = UartTx::new("uart_tx", clock_freq, uart_baud_rate, m);
        let tx = m.output("tx", uart_tx.tx);

        let uart_rx = UartRx::new("uart_rx", clock_freq, uart_baud_rate, m);
        let rx = m.input("rx", 1);
        uart_rx.rx.drive(rx);

        let uart_interface = UartInterface::new("uart_interface", m);
        uart_tx.data.drive(uart_interface.tx_data);
        uart_tx.enable.drive(uart_interface.tx_enable);
        uart_interface.tx_ready.drive(uart_tx.ready);
        uart_interface.rx_data.drive(uart_rx.data);
        uart_interface.rx_data_valid.drive(uart_rx.data_valid);

        let color_thrust = ColorThrust::new("color_thrust", m);

        let ddr3_interface = ByteRam::new("ddr3_interface", 13, 24, m);

        // Interconnect
        let cpu_crossbar = Crossbar::new("cpu_crossbar", 1, 2, 28, 4, 128, 5, m);

        marv_system_bridge.system_port.connect(&cpu_crossbar.replica_ports[0]);

        let mem_crossbar = Crossbar::new("mem_crossbar", 2, 1, 24, 0, 128, 5, m);
        cpu_crossbar.primary_ports[1].connect(&mem_crossbar.replica_ports[0]);
        color_thrust.tex_cache_system_port.connect(&mem_crossbar.replica_ports[1]);
        mem_crossbar.primary_ports[0].connect(&ddr3_interface.client_port);

        let sys_crossbar = Crossbar::new("buster_crossbar", 1, 7, 24, 4, 128, 5, m);
        cpu_crossbar.primary_ports[0].connect(&sys_crossbar.replica_ports[0]);
        sys_crossbar.primary_ports[0].connect(&boot_rom.client_port);
        sys_crossbar.primary_ports[1].connect(&program_ram.client_port);
        sys_crossbar.primary_ports[2].connect(&led_interface.client_port);
        sys_crossbar.primary_ports[3].connect(&uart_interface.client_port);
        sys_crossbar.primary_ports[4].connect(&color_thrust.reg_port);
        sys_crossbar.primary_ports[5].connect(&color_thrust.color_buffer_port);
        sys_crossbar.primary_ports[6].connect(&color_thrust.depth_buffer_port);

        Xenowing {
            m,
            leds,
            tx,
            rx,
        }
    }
}
