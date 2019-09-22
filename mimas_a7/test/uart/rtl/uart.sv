`default_nettype none

module uart(
    input wire logic reset,
    input wire logic clk,

    output wire logic tx);

    logic reset_n;
    reset_synchronizer reset_synchronizer0(
        .reset_n(~reset),
        .clk(clk),
        .reset_n_sync(reset_n));

    logic [7:0] write_data;
    logic ready;
    uart_transmitter uart_transmitter0(
        .reset_n(reset_n),
        .clk(clk),

        .write_data(write_data),
        .write_req(1'b1),
        .ready(ready),

        .tx(tx));

    lfsr lfsr0(
        .reset_n(reset_n),
        .clk(clk),
        .value(write_data),
        .shift_enable(ready));

endmodule
