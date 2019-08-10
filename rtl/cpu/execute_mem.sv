`default_nettype none

module execute_mem(
    output ready,
    input enable,

    input [31:0] instruction,

    input [31:0] register_file_read_data1,
    input [31:0] register_file_read_data2,

    output [31:0] alu_res);

    logic [6:0] opcode;
    logic [4:0] rs2;
    logic [2:0] funct3;
    logic [31:0] i_immediate;
    assign opcode = instruction[6:0];
    assign rs2 = instruction[24:20];
    assign funct3 = instruction[14:12];
    assign i_immediate = {{20{instruction[31]}}, instruction[31:20]};

    logic [2:0] alu_op;
    logic alu_op_mod;
    logic [31:0] alu_rhs;
    alu alu0(
        .op(alu_op),
        .op_mod(alu_op_mod),
        .lhs(register_file_read_data1),
        .rhs(alu_rhs),
        .res(alu_res));

    always_comb begin
        ready = 1; // TODO: Block if we're interacting with the bus and it's not ready

        alu_op = funct3;
        alu_op_mod = instruction[30];
        alu_rhs = register_file_read_data2;

        if (!opcode[5]) begin
            // Immediate computation
            alu_rhs = i_immediate;

            // Shifts use rs2 directly (not its register value)
            if (funct3 == 3'b001 || funct3 == 3'b101) begin
                alu_rhs = {27'h0, rs2};
            end
        end
    end

endmodule
