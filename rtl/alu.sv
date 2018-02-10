typedef enum [3:0] {
    ADD,
    SUB,
    AND,
    OR,
    XOR,
    SLL,
    SRL,
    SRA,
    EQ,
    NE,
    LT,
    LTU,
    GE,
    GEU
} command_t;

module alu(
    input command_t command,

    input [31:0] lhs,
    input [31:0] rhs,

    output [31:0] res);

    always_comb begin
        case (command)
            ADD: res = lhs + rhs;
            SUB: res = lhs - rhs;
            AND: res = lhs & rhs;
            OR: res = lhs | rhs;
            XOR: res = lhs ^ rhs;
            SLL: res = lhs << rhs;
            SRL: res = lhs >> rhs;
            SRA: res = $signed(lhs) >>> $signed(rhs);
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
