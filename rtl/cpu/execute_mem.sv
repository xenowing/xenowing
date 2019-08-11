`default_nettype none

module execute_mem(
    output ready,
    input enable,

    input [31:0] pc_value,

    input [31:0] instruction,

    input [31:0] register_file_read_data1,
    input [31:0] register_file_read_data2,

    output [4:0] rd,

    output [31:0] next_pc,

    output rd_value_write_enable,
    output [31:0] rd_value_write_data,

    output read_issued,

    output [31:0] system_bus_write_data,
    output system_bus_write_req);

    logic [6:2] opcode;
    logic [4:0] rs2;
    logic [2:0] funct3;
    logic [31:0] load_offset;
    logic [31:0] store_offset;
    logic [31:0] jump_offset;
    logic [31:0] branch_offset;
    logic [31:0] i_immediate;
    logic [31:0] u_immediate;
    assign opcode = instruction[6:0];
    assign rd = instruction[11:7];
    assign rs2 = instruction[24:20];
    assign funct3 = instruction[14:12];
    assign load_offset = {{20{instruction[31]}}, instruction[31:20]};
    assign store_offset = {{20{instruction[31]}}, {instruction[31:25], instruction[11:7]}};
    assign jump_offset = {{11{instruction[31]}}, {instruction[31], instruction[19:12], instruction[20], instruction[30:21]}, 1'b0};
    assign branch_offset = {{19{instruction[31]}}, instruction[31], instruction[7], instruction[30:25], instruction[11:8], 1'b0};
    assign i_immediate = {{20{instruction[31]}}, instruction[31:20]};
    assign u_immediate = {instruction[31:12], 12'h0};

    logic [31:0] alu_lhs;
    logic [31:0] alu_rhs;
    logic [31:0] alu_res;
    alu alu0(
        .op(funct3),
        .op_mod(instruction[30]),
        .lhs(alu_lhs),
        .rhs(alu_rhs),
        .res(alu_res));
    assign alu_lhs = register_file_read_data1;

    logic [31:0] load_address;
    logic [31:0] store_address;
    assign load_address = register_file_read_data1 + load_offset;
    assign store_address = register_file_read_data1 + store_offset;

    assign system_bus_write_data = register_file_read_data2;

    logic branch_taken;

    always_comb begin
        ready = 1; // TODO: Block if we're interacting with the bus and it's not ready

        next_pc = pc_value + 32'h4;

        read_issued = 0;

        alu_rhs = register_file_read_data2;

        if (!opcode[5]) begin
            // Immediate computation
            alu_rhs = i_immediate;

            // Shifts use rs2 directly (not its register value)
            if (funct3 == 3'b001 || funct3 == 3'b101) begin
                alu_rhs = {27'h0, rs2};
            end
        end

        rd_value_write_enable = 1;
        rd_value_write_data = alu_res;

        /*case (opcode)
            5'b01101: begin
                // lui
                rd_value_write_data = u_immediate;
            end
            5'b00101: begin
                // auipc
                rd_value_write_data = pc_value + u_immediate;
            end
            5'b11011: begin
                // jal
                rd_value_write_data = next_pc;
                next_pc = pc_value + jump_offset;
            end
            5'b11001: begin
                // jalr
                alu_rhs = i_immediate;
                rd_value_write_data = next_pc;
                next_pc = alu_res;
            end
        endcase*/

        // branches
        case (funct3[2:1])
            2'b00: branch_taken = alu_lhs == alu_rhs;
            2'b10: branch_taken = $signed(alu_lhs) < $signed(alu_rhs);
            2'b11: branch_taken = alu_lhs < alu_rhs;
            default: branch_taken = 0;
        endcase
        if (funct3[0]) begin
            branch_taken = !branch_taken;
        end
        if (opcode == 5'b11000) begin
            rd_value_write_enable = 0;

            if (branch_taken) begin
                next_pc = pc_value + branch_offset;
            end
        end
    end

endmodule
