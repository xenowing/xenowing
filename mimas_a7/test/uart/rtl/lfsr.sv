`default_nettype none

module lfsr(
    input wire logic reset_n,
    input wire logic clk,

    output wire logic [7:0] value,
    input wire logic shift_enable);

    logic [15:0] state;
    logic feedback_bit;
    assign feedback_bit = (state >> 0) ^ (state >> 2) ^ (state >> 3) ^ (state >> 5);
    logic [15:0] state_next;

    assign value = state[7:0];

    always_comb begin
        state_next = state;

        if (shift_enable) begin
            state_next = (state >> 1) | (feedback_bit << 15);
        end
    end

    always_ff @(posedge clk, negedge reset_n) begin
        if (~reset_n) begin
            state <= 16'hace1;
        end
        else begin
            state <= state_next;
        end
    end

endmodule
