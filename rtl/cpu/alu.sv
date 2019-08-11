`default_nettype none

module alu(
    input [2:0] op,
    input op_mod,

    input [31:0] lhs,
    input [31:0] rhs,

    output [31:0] res);

    logic [4:0] shift_amt;
    assign shift_amt = rhs[4:0];

    always_comb begin
        case (op)
            3'b000: begin
                if (!op_mod) begin
                    // ADD
                    res = lhs + rhs;
                end
                else begin
                    // SUB
                    res = lhs - rhs;
                end
            end
            3'b001: res = lhs << shift_amt; // SLL
            3'b010: res = $signed(lhs) < $signed(rhs) ? 32'h1 : 32'h0; // LT
            3'b011: res = lhs < rhs ? 32'h1 : 32'h0; // LTU
            3'b100: res = lhs ^ rhs; // XOR
            3'b101: begin
                if (!op_mod) begin
                    // SRL
                    res = lhs >> shift_amt;
                end
                else begin
                    // SRA
                    res = $signed(lhs) >>> shift_amt;
                end
            end
            3'b110: res = lhs | rhs; // OR
            3'b111: res = lhs & rhs; // AND
        endcase
    end
endmodule
