`default_nettype none

module UnidirectionalDualPortBram #(
    parameter DATA = 32,
    parameter ADDR = 7,
    parameter DEPTH = 128
) (
    input wire logic write_clk,
    input wire logic write_enable,
    input wire logic [ADDR - 1:0] write_addr,
    input wire logic [DATA - 1:0] write_data,

    input wire logic read_clk,
    input wire logic read_enable,
    input wire logic [ADDR - 1:0] read_addr,
    output logic [DATA - 1:0] read_data
);

logic [DATA - 1:0] mem[0:DEPTH-1];

always_ff @(posedge write_clk) begin
    if (write_enable)
        mem[write_addr] <= write_data;
end

always_ff @(posedge read_clk) begin
    if (read_enable)
        read_data <= mem[read_addr];
end

endmodule
