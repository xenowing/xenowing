`default_nettype none

module decode(
    output ready,

    output [31:0] instruction,

    input [31:0] system_bus_read_data,
    input system_bus_read_data_valid);

    assign ready = system_bus_read_data_valid;

    assign instruction = system_bus_read_data;

endmodule
