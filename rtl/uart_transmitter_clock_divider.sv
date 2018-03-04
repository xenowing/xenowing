`default_nettype none

module uart_transmitter_clock_divider(
    input reset_n,
    input clk,

    output clock_edge);

    parameter CLK_FREQ = 150000000;
    parameter BAUD_RATE = 9600;
    parameter BAUD_CLK_ACC_WIDTH = 16;
    localparam BAUD_CLK_ACC_INC = ((BAUD_RATE << (BAUD_CLK_ACC_WIDTH - 4)) + (CLK_FREQ >> 5)) / (CLK_FREQ >> 4);

    logic [BAUD_CLK_ACC_WIDTH:0] acc;
    logic [BAUD_CLK_ACC_WIDTH:0] acc_next;

    assign clock_edge = acc[BAUD_CLK_ACC_WIDTH];

    always_comb begin
        acc_next = acc[BAUD_CLK_ACC_WIDTH - 1:0] + BAUD_CLK_ACC_INC;
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
