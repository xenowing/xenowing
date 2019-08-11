`default_nettype none

module writeback(
    output ready,
    input enable,

    input [4:0] rd,

    input [31:0] next_pc,

    input rd_value_write_enable,
    input [31:0] rd_value_write_data,

    input read_issued,

    output [31:0] pc_write_data,
    output pc_write_enable,

    output register_file_write_enable,
    output [4:0] register_file_write_addr,
    output [31:0] register_file_write_data);

    assign register_file_write_addr = rd;
    assign register_file_write_data = rd_value_write_data;

    assign pc_write_data = next_pc;

    assign ready = 1; // TODO: Wait for mem in the case we've issued a read

    always_comb begin
        pc_write_enable = 0;
        register_file_write_enable = 0;

        if (enable && ready) begin
            pc_write_enable = 1;
            register_file_write_enable = rd_value_write_enable;
        end
    end

endmodule
