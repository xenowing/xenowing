`default_nettype none

module uart_clock_divider#(parameter clk_freq = 100000000, baud_rate = 9600)(
    input wire reset_n,
    input wire clk,

    output logic clock_edge);

    logic clock_edge_next;

    localparam baud_counter_max = clk_freq / baud_rate - 1;
    localparam baud_counter_width = $clog2(baud_counter_max);

    logic [baud_counter_width - 1:0] baud_counter;
    logic [baud_counter_width - 1:0] baud_counter_next;

    always_comb begin
        clock_edge_next = 1'b0;
        baud_counter_next = baud_counter + 1;

        if (baud_counter == baud_counter_max) begin
            clock_edge_next = 1'b1;
            baud_counter_next = 0;
        end
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            clock_edge <= 0;
            baud_counter <= 0;
        end
        else begin
            clock_edge <= clock_edge_next;
            baud_counter <= baud_counter_next;
        end
    end

endmodule
