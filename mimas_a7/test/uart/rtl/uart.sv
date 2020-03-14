`default_nettype none

module uart(
    input wire logic reset,
    input wire logic clk,

    output wire logic tx,
    input wire logic rx,

    output wire logic led);

    logic reset_n;
    reset_synchronizer reset_synchronizer0(
        .reset_n(~reset),
        .clk(clk),
        .reset_n_sync(reset_n));

    logic [7:0] write_data;
    logic write_req;
    logic ready;
    uart_transmitter uart_transmitter0(
        .reset_n(reset_n),
        .clk(clk),

        .write_data(write_data),
        .write_req(write_req),
        .ready(ready),

        .tx(tx));

    lfsr tx_lfsr(
        .reset_n(reset_n),
        .clk(clk),

        .value(write_data),
        .shift_enable(ready));

    parameter RX_SYNC_STAGES = 2;

    (* ASYNC_REG = "TRUE" *) logic [RX_SYNC_STAGES - 1:0] rx_sync_chain;

    always_ff @(posedge clk, negedge reset_n) begin
        if (~reset_n) begin
            rx_sync_chain <= {RX_SYNC_STAGES{1'd1}};
        end
        else begin
            rx_sync_chain <= {rx_sync_chain[RX_SYNC_STAGES - 2:0], rx};
        end
    end

    logic [7:0] read_data;
    logic read_data_ready;
    UartRx uart_rx(
        .reset_n(reset_n),
        .clk(clk),

        .rx(rx_sync_chain[RX_SYNC_STAGES - 1]),

        .data(read_data),
        .data_ready(read_data_ready));

    logic [7:0] rx_lfsr_value;
    lfsr rx_lfsr(
        .reset_n(reset_n),
        .clk(clk),

        .value(rx_lfsr_value),
        .shift_enable(read_data_ready));

    logic has_errored;
    logic has_errored_next;

    always_comb begin
        has_errored_next = has_errored;

        if (read_data_ready) begin
            has_errored_next = read_data != rx_lfsr_value;
        end
    end

    always_ff @(posedge clk, negedge reset_n) begin
        if (~reset_n) begin
            has_errored <= 1'b0;
        end
        else begin
            has_errored <= has_errored_next;
        end
    end

    assign led = has_errored;

    assign write_req = ~has_errored;

endmodule
