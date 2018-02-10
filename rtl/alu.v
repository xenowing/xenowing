module alu(
    input rst,
    input clk,
    output reg [31:0] out);

    reg [31:0] out_next;

    always_comb
    begin
        out_next = out + 1;
    end

    always_ff @(posedge clk or negedge rst)
    begin
        if (!rst)
            out <= 0;
        else
            out <= out_next;
    end
endmodule
