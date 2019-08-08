`default_nettype none

module cpu(
    input reset_n,
    input clk,

    input system_bus_ready,
    output [31:0] system_bus_addr,
    output [31:0] system_bus_write_data,
    output [3:0] system_bus_byte_enable,
    output system_bus_write_req,
    output system_bus_read_req,
    input [31:0] system_bus_read_data,
    input system_bus_read_data_valid);

    logic instruction_fetch_ready;
    logic instruction_fetch_enable;
    instruction_fetch instruction_fetch0(
        .clk(clk),
        .reset_n(reset_n),

        .ready(instruction_fetch_ready),
        .enable(instruction_fetch_enable),

        .system_bus_ready(system_bus_ready),
        .system_bus_addr(system_bus_addr),
        .system_bus_byte_enable(system_bus_byte_enable),
        .system_bus_read_req(system_bus_read_req));

    logic decode_ready;
    logic [31:0] decode_instruction;
    decode decode0(
        .ready(decode_ready),

        .instruction(decode_instruction),

        .system_bus_read_data(system_bus_read_data),
        .system_bus_read_data_valid(system_bus_read_data_valid));

    // TODO: This might not be the right kind of register depending on the outputs of decode and inputs of execute
    logic [31:0] decode_execute_instruction;
    always_ff @(posedge clk) begin
        decode_execute_instruction <= decode_instruction;
    end

    control control0(
        .clk(clk),
        .reset_n(reset_n),

        .instruction_fetch_ready(instruction_fetch_ready),
        .instruction_fetch_enable(instruction_fetch_enable),

        .decode_ready(decode_ready));

endmodule
