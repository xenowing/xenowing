`default_nettype none

module uart_transmitter_clock_divider(
    input reset_n,
    input clk,

    output clock_edge);

    localparam CLK_FREQ = 150000000;
    localparam BAUD_RATE = 9600;
    localparam ACC_WIDTH = 16;
    localparam ACC_INC = ((BAUD_RATE << (ACC_WIDTH - 4)) + (CLK_FREQ >> 5)) / (CLK_FREQ >> 4);

    logic [ACC_WIDTH:0] acc;
    logic [ACC_WIDTH:0] acc_next;

    assign clock_edge = acc[ACC_WIDTH];

    always_comb begin
        acc_next = acc[ACC_WIDTH - 1:0] + ACC_INC;
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            acc <= 0;
        end
        else begin
            acc <= acc_next;
        end
    end

endmodule
