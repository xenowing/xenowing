`default_nettype none

module decode(
    output ready,

    output [31:0] instruction,

    input [31:0] system_bus_read_data,
    input system_bus_read_data_valid,

    output [4:0] register_file_read_addr1,
    output [4:0] register_file_read_addr2);

    assign ready = system_bus_read_data_valid;

    assign instruction = system_bus_read_data;

    logic [4:0] rs1;
    logic [4:0] rs2;
    assign rs1 = instruction[19:15];
    assign rs2 = instruction[24:20];

    assign register_file_read_addr1 = rs1;
    assign register_file_read_addr2 = rs2;

endmodule
