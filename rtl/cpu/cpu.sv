module cpu(
    input reset_n,
    input clk,

    input logic ready,
    output [31:0] addr,
    output [31:0] write_data,
    output [3:0] byte_enable,
    output write_req,
    output read_req,
    input [31:0] read_data,
    input read_data_valid);

    // TODO

endmodule
