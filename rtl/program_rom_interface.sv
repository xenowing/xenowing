`default_nettype none

module program_rom_interface(
    input reset_n,
    input clk,

    input [13:2] addr,
    input read_req,
    output [31:0] read_data,
    output logic read_data_valid,

    output [11:0] program_rom_addr,
    input [31:0] program_rom_q);

    assign program_rom_addr = addr[13:2];
    assign read_data = program_rom_q;

    logic read_data_valid_next;

    always_comb begin
        read_data_valid_next = read_data_valid;

        read_data_valid_next = read_req;
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            read_data_valid <= 0;
        end
        else begin
            read_data_valid <= read_data_valid_next;
        end
    end
endmodule
