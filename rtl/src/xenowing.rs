use crate::bit_pusher::*;
use crate::boot_rom::*;
use crate::buster::*;
use crate::buster_mig_ui_bridge::*;
use crate::color_thrust::*;
use crate::led_interface::*;
use crate::marv::*;
use crate::marv_system_bridge::*;
use crate::read_cache::*;
use crate::uart::*;
use crate::uart_interface::*;

use kaze::*;

pub struct Xenowing<'a> {
    pub m: &'a Module<'a>,

    pub leds: &'a Output<'a>,

    pub tx: &'a Output<'a>,
    pub rx: &'a Input<'a>,

    pub ddr3: MigUiPort<'a>,
}

impl<'a> Xenowing<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> Xenowing<'a> {
        let m = p.module(instance_name, "Xenowing");

        let inner = XenowingInner::new("inner", m);

        let leds = m.output("leds", inner.leds);

        let clock_freq = 100000000;
        let uart_baud_rate = 460800;

        let uart_tx = UartTx::new("uart_tx", clock_freq, uart_baud_rate, m);
        let tx = m.output("tx", uart_tx.tx);

        let uart_rx = UartRx::new("uart_rx", clock_freq, uart_baud_rate, m);
        let rx = m.input("rx", 1);
        uart_rx.rx.drive(rx);

        uart_tx.data.drive(inner.uart_tx_data);
        uart_tx.enable.drive(inner.uart_tx_enable);
        inner.uart_tx_ready.drive(uart_tx.ready);
        inner.uart_rx_data.drive(uart_rx.data);
        inner.uart_rx_data_valid.drive(uart_rx.data_valid);

        Xenowing {
            m,

            leds,

            tx,
            rx,

            ddr3: inner.ddr3.forward("ddr3", m),
        }
    }
}

// TODO: Better name?
pub struct XenowingInner<'a> {
    pub m: &'a Module<'a>,

    pub leds: &'a Output<'a>,

    pub uart_tx_data: &'a Output<'a>,
    pub uart_tx_enable: &'a Output<'a>,
    pub uart_tx_ready: &'a Input<'a>,
    pub uart_rx_data: &'a Input<'a>,
    pub uart_rx_data_valid: &'a Input<'a>,
    pub uart_rx_ready: &'a Output<'a>,

    pub ddr3: MigUiPort<'a>,
}

impl<'a> XenowingInner<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> XenowingInner<'a> {
        let m = p.module(instance_name, "XenowingInner");

        let marv = Marv::new("marv", m);

        let boot_rom = BootRom::new("boot_rom", m);

        let led_interface = LedInterface::new("led_interface", m);
        let leds = m.output("leds", led_interface.leds);

        let uart_interface = UartInterface::new("uart_interface", m);
        let uart_tx_data = m.output("uart_tx_data", uart_interface.tx_data);
        let uart_tx_enable = m.output("uart_tx_enable", uart_interface.tx_enable);
        let uart_tx_ready = m.input("uart_tx_ready", 1);
        uart_interface.tx_ready.drive(uart_tx_ready);
        let uart_rx_data = m.input("uart_rx_data", 8);
        uart_interface.rx_data.drive(uart_rx_data);
        let uart_rx_data_valid = m.input("uart_rx_data_valid", 1);
        uart_interface.rx_data_valid.drive(uart_rx_data_valid);
        let uart_rx_ready = m.output("uart_rx_ready", uart_interface.rx_ready);

        let color_thrust = ColorThrust::new("color_thrust", m);

        let bit_pusher = BitPusher::new("bit_pusher", m);

        let ddr3_bridge = BusterMigUiBridge::new("ddr3_bridge", 128, 24, m);

        // Interconnect
        let cpu_crossbar = Crossbar::new("cpu_crossbar", 2, 2, 28, 4, 128, 5, m);
        let marv_instruction_bridge = MarvSystemBridge::new("marv_instruction_bridge", m);
        marv.instruction_port.connect(&marv_instruction_bridge.marv_port);
        let marv_instruction_cache = ReadCache::new("marv_instruction_cache", 128, 28, 12 - 4, m);
        marv_instruction_cache.invalidate.drive(m.low()); // TODO: Expose this to the CPU somehow
        marv_instruction_bridge.system_port.connect(&marv_instruction_cache.client_port);
        marv_instruction_cache.system_port.connect(&cpu_crossbar.replica_ports[0]);
        let marv_data_bridge = MarvSystemBridge::new("marv_data_bridge", m);
        marv.data_port.connect(&marv_data_bridge.marv_port);
        marv_data_bridge.system_port.connect(&cpu_crossbar.replica_ports[1]);

        let mem_crossbar = Crossbar::new("mem_crossbar", 3, 1, 24, 0, 128, 5, m);
        cpu_crossbar.primary_ports[1].connect(&mem_crossbar.replica_ports[0]);
        color_thrust.tex_cache_system_port.connect(&mem_crossbar.replica_ports[1]);
        bit_pusher.mem_port.connect(&mem_crossbar.replica_ports[2]);
        mem_crossbar.primary_ports[0].connect(&ddr3_bridge.client_port);

        let sys_crossbar = Crossbar::new("sys_crossbar", 2, 7, 24, 4, 128, 5, m);
        cpu_crossbar.primary_ports[0].connect(&sys_crossbar.replica_ports[0]);
        bit_pusher.sys_port.connect(&sys_crossbar.replica_ports[1]);
        sys_crossbar.primary_ports[0].connect(&boot_rom.client_port);
        sys_crossbar.primary_ports[1].connect(&led_interface.client_port);
        sys_crossbar.primary_ports[2].connect(&uart_interface.client_port);
        sys_crossbar.primary_ports[3].connect(&color_thrust.reg_port);
        sys_crossbar.primary_ports[4].connect(&color_thrust.color_buffer_port);
        sys_crossbar.primary_ports[5].connect(&color_thrust.depth_buffer_port);
        sys_crossbar.primary_ports[6].connect(&bit_pusher.reg_port);

        XenowingInner {
            m,

            leds,

            uart_tx_data,
            uart_tx_enable,
            uart_tx_ready,
            uart_rx_data,
            uart_rx_data_valid,
            uart_rx_ready,

            ddr3: ddr3_bridge.ui_port.forward("ddr3", m),
        }
    }
}
