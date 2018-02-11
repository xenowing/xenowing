`include "rtl/command_t.vh"

module alu(
    input command_t command,

    input [31:0] lhs,
    input [31:0] rhs,

    output [31:0] res);

    logic [4:0] shift_amt = rhs[4:0];

    always_comb begin
        case (command)
            ADD: res = lhs + rhs;
            SUB: res = lhs - rhs;
            AND: res = lhs & rhs;
            OR: res = lhs | rhs;
            XOR: res = lhs ^ rhs;
            SLL: res = lhs << shift_amt;
            SRL: res = lhs >> shift_amt;
            SRA: res = $signed(lhs) >>> shift_amt;
            EQ: res = lhs == rhs ? 32'h1 : 32'h0;
            NE: res = lhs != rhs ? 32'h1 : 32'h0;
            LT: res = $signed(lhs) < $signed(rhs) ? 32'h1 : 32'h0;
            LTU: res = lhs < rhs ? 32'h1 : 32'h0;
            GE: res = $signed(lhs) >= $signed(rhs) ? 32'h1 : 32'h0;
            GEU: res = lhs >= rhs ? 32'h1 : 32'h0;
            default: res = 32'h0;
        endcase
    end
endmodule
