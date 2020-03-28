`default_nettype none

module Top(
    input wire logic reset,
    input wire logic clk,

    output wire logic tx,
    input wire logic rx,

    output wire logic [7:0] leds);

    logic reset_n;
    SyncChain #(.DEFAULT(1'b0)) reset_sync_chain(
        .reset_n(~reset),
        .clk(clk),

        .x(1'b1),

        .x_sync(reset_n));

    logic rx_sync;
    SyncChain #(.DEFAULT(1'b1)) rx_sync_chain(
        .reset_n(reset_n),
        .clk(clk),

        .x(rx),

        .x_sync(rx_sync));

    Xenowing xenowing(
        .reset_n(reset_n),
        .clk(clk),

        .tx(tx),
        .rx(rx_sync),

        .leds(leds));

endmodule
