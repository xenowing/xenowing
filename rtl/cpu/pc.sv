`default_nettype none

module pc(
    input reset_n,
    input clk,

    output logic [31:0] value,

    input [31:0] write_data,
    input write_enable);

    logic [31:0] value_next;

    always_comb begin
        value_next = value;

        if (write_enable) begin
            value_next = write_data;
        end
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            value <= 32'h10000000;
        end
        else begin
            value <= value_next;
        end
    end

endmodule
