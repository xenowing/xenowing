`default_nettype none

module uart_clock_divider#(parameter clk_freq = 100000000, baud_rate = 9600, baud_clk_acc_width = 16)(
    input wire reset_n,
    input wire clk,

    output wire clock_edge);

    localparam BAUD_CLK_ACC_INC = ((baud_rate << (baud_clk_acc_width - 4)) + (clk_freq >> 5)) / (clk_freq >> 4);

    logic [baud_clk_acc_width:0] acc;
    logic [baud_clk_acc_width:0] acc_next;

    assign clock_edge = acc[baud_clk_acc_width];

    always_comb begin
        acc_next = acc[baud_clk_acc_width - 1:0] + BAUD_CLK_ACC_INC;
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
