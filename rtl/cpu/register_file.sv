`default_nettype none

module register_file(
    input clk,
    input reset_n,

    input [4:0] read_addr1,
    output [31:0] read_data1,

    input [4:0] read_addr2,
    output [31:0] read_data2,

    input write_enable,
    input [4:0] write_addr,
    input [31:0] write_data);

    logic [31:0] regs[0:31];

    assign read_data1 = regs[read_addr1];
    assign read_data2 = regs[read_addr2];

    initial begin
        for (int i = 0; i < 32; i++) begin
            regs[i] = 32'h0;
        end
    end

    always_ff @(posedge clk) begin
        if (!reset_n) begin
            // Do nothing
        end
        else begin
            if (write_enable && write_addr != 0) begin
                regs[write_addr] <= write_data;
            end
        end
    end

endmodule
