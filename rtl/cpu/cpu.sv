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

    logic instruction_fetch_issue_ready;
    logic instruction_fetch_issue_enable;
    instruction_fetch_issue instruction_fetch_issue0(
        .clk(clk),
        .reset_n(reset_n),

        .ready(instruction_fetch_issue_ready),
        .enable(instruction_fetch_issue_enable),

        .system_bus_ready(system_bus_ready),
        .system_bus_addr(system_bus_addr),
        .system_bus_byte_enable(system_bus_byte_enable),
        .system_bus_read_req(system_bus_read_req));

    logic instruction_fetch_wait_ready;
    logic instruction_fetch_wait_enable;
    logic [31:0] instruction_fetch_wait_instruction;
    instruction_fetch_wait instruction_fetch_wait0(
        .clk(clk),
        .reset_n(reset_n),

        .ready(instruction_fetch_wait_ready),
        .enable(instruction_fetch_wait_enable),

        .instruction(instruction_fetch_wait_instruction),

        .system_bus_read_data(system_bus_read_data),
        .system_bus_read_data_valid(system_bus_read_data_valid));

    logic [31:0] instruction_fetch_wait_decode_instruction;
    always_ff @(posedge clk) begin
        instruction_fetch_wait_decode_instruction <= instruction_fetch_wait_instruction;
    end

    logic decode_enable;
    decode decode0(
        .clk(clk),
        .reset_n(reset_n),

        .enable(decode_enable),

        .instruction(instruction_fetch_wait_decode_instruction));

    control control0(
        .clk(clk),
        .reset_n(reset_n),

        .instruction_fetch_issue_ready(instruction_fetch_issue_ready),
        .instruction_fetch_issue_enable(instruction_fetch_issue_enable),

        .instruction_fetch_wait_ready(instruction_fetch_wait_ready),
        .instruction_fetch_wait_enable(instruction_fetch_wait_enable),

        .decode_enable(decode_enable));

endmodule
