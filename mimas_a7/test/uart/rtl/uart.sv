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
    logic write_req;
    logic ready;
    uart_transmitter uart_transmitter0(
        .reset_n(reset_n),
        .clk(clk),

        .write_data(write_data),
        .write_req(write_req),
        .ready(ready),

        .tx(tx));

    localparam MESSAGE_LEN = 15;
    localparam MESSAGE_BITS = MESSAGE_LEN * 8;
    logic [MESSAGE_BITS - 1:0] message;
    logic [MESSAGE_BITS - 1:0] message_next;

    always_comb begin
        message_next = message;

        write_data = message[MESSAGE_BITS - 1:MESSAGE_BITS - 8];
        write_req = write_data != 8'd0;

        if (write_req && ready) begin
            message_next = {message[MESSAGE_BITS - 8:0], 8'd0};
        end
    end
    always_ff @(posedge clk, negedge reset_n) begin
        if (~reset_n) begin
            message <= "Hello, world!\r\n";
        end
        else begin
            message <= message_next;
        end
    end

endmodule
