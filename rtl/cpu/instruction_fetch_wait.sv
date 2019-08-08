`default_nettype none

module instruction_fetch_wait(
    input reset_n,
    input clk,

    output ready,
    input enable,

    output [31:0] instruction,

    input [31:0] system_bus_read_data,
    input system_bus_read_data_valid);

    assign instruction = system_bus_read_data;

    assign ready = system_bus_read_data_valid;

endmodule
