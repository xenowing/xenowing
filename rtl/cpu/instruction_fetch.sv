`default_nettype none

module instruction_fetch(
    input reset_n,
    input clk,

    output ready,
    input enable,

    input [31:0] pc,

    input system_bus_ready,
    output [31:0] system_bus_addr,
    output [3:0] system_bus_byte_enable,
    output system_bus_read_req);

    assign ready = system_bus_ready;

    assign system_bus_addr = pc;
    assign system_bus_byte_enable = 4'b1111;
    assign system_bus_read_req = enable;

endmodule
