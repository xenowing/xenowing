`default_nettype none

module writeback(
    output ready,
    input enable,

    input [31:0] pc_value,
    output [31:0] pc_write_data,
    output pc_write_enable,

    output register_file_write_enable,
    output [4:0] register_file_write_addr,
    output [31:0] register_file_write_data,

    input [31:0] instruction,

    input [31:0] alu_res);

    logic [6:0] opcode;
    logic [4:0] rd;
    logic [31:0] jump_offset;
    assign opcode = instruction[6:0];
    assign rd = instruction[11:7];
    assign jump_offset = {{11{instruction[31]}}, {instruction[31], instruction[19:12], instruction[20], instruction[30:21]}, 1'b0};

    assign register_file_write_addr = rd;

    assign ready = 1; // TODO: Wait for mem in the case we've issued a read

    always_comb begin
        pc_write_data = pc_value + 4;
        pc_write_enable = 0;

        register_file_write_enable = 0; // TODO
        register_file_write_data = alu_res; // TODO

        if (enable && ready) begin
            pc_write_enable = 1;

            if (opcode == 7'b1101111) begin
                // JAL
                pc_write_data = pc_value + jump_offset; // TODO: Calculate pc_write_data in previous stage!!!

                register_file_write_enable = 1;
                register_file_write_data = pc_value;
            end
        end
    end

endmodule
