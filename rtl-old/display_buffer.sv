`default_nettype none

module display_buffer(
    input clk,

    input [5:0] write_addr,
    input [63:0] write_data,
    input write_enable,

    input [5:0] read_addr,
    output [63:0] read_data);

    logic [63:0] mem[0:63];

    assign read_data = mem[read_addr];

    always @(posedge clk) begin
        if (write_enable) begin
            mem[write_addr] <= write_data;
        end
    end

endmodule
