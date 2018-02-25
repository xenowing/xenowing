`default_nettype none

module led_clock_divider(
    input reset_n,
    input clk,

    output clock_edge);

    logic [24:0] clock_divide_counter;
    logic [24:0] clock_divide_counter_next;
    assign clock_edge = clock_divide_counter[24];

    always_comb begin
        clock_divide_counter_next = clock_divide_counter;

        clock_divide_counter_next = clock_divide_counter + 25'h1;
        if (clock_edge) begin
            clock_divide_counter_next = 0;
        end
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            clock_divide_counter <= 0;
        end
        else begin
            clock_divide_counter <= clock_divide_counter_next;
        end
    end

endmodule
