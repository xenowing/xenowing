use kaze::*;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("Uart");

    let has_errored = m.reg("has_errored", 1);
    has_errored.default_value(false);
    m.output("has_errored", has_errored.value);

    let write_enable = !has_errored.value;

    let uart_tx = m.instance("uart_tx", "UartTx");
    uart_tx.drive_input("enable", write_enable);
    m.output("tx", uart_tx.output("tx"));

    let tx_lfsr = m.instance("tx_lfsr", "Lfsr");
    tx_lfsr.drive_input("shift_enable", write_enable & uart_tx.output("ready"));
    uart_tx.drive_input("data", tx_lfsr.output("value"));

    let uart_rx = m.instance("uart_rx", "UartRx");
    uart_rx.drive_input("rx", m.input("rx", 1));
    let read_data = uart_rx.output("data");
    let read_data_valid = uart_rx.output("data_valid");

    let rx_lfsr = m.instance("rx_lfsr", "Lfsr");
    rx_lfsr.drive_input("shift_enable", read_data_valid);

    has_errored.drive_next(if_(read_data_valid, {
        read_data.ne(rx_lfsr.output("value"))
    }).else_({
        has_errored.value
    }));

    m
}
