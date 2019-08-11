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

    logic [31:0] pc_value;
    logic [31:0] pc_write_data;
    logic pc_write_enable;
    pc pc0(
        .clk(clk),
        .reset_n(reset_n),

        .value(pc_value),

        .write_data(pc_write_data),
        .write_enable(pc_write_enable));

    logic [4:0] register_file_read_addr1;
    logic [31:0] register_file_read_data1;
    logic [4:0] register_file_read_addr2;
    logic [31:0] register_file_read_data2;
    logic register_file_write_enable;
    logic [4:0] register_file_write_addr;
    logic [31:0] register_file_write_data;
    register_file register_file0(
        .clk(clk),
        .reset_n(reset_n),

        .read_addr1(register_file_read_addr1),
        .read_data1(register_file_read_data1),

        .read_addr2(register_file_read_addr2),
        .read_data2(register_file_read_data2),

        .write_enable(register_file_write_enable),
        .write_addr(register_file_write_addr),
        .write_data(register_file_write_data));

    logic instruction_fetch_ready;
    logic instruction_fetch_enable;
    instruction_fetch instruction_fetch0(
        .clk(clk),
        .reset_n(reset_n),

        .ready(instruction_fetch_ready),
        .enable(instruction_fetch_enable),

        .pc(pc_value),

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
        .system_bus_read_data_valid(system_bus_read_data_valid),

        .register_file_read_addr1(register_file_read_addr1),
        .register_file_read_addr2(register_file_read_addr2));

    logic [31:0] instruction;
    logic [31:0] instruction_next;
    always_comb begin
        instruction_next = instruction;

        if (decode_enable) begin
            instruction_next = decode_instruction;
        end
    end
    always_ff @(posedge clk) begin
        instruction <= instruction_next;
    end

    logic execute_mem_ready;
    logic execute_mem_enable;
    logic [4:0] execute_mem_rd;
    logic [31:0] execute_mem_next_pc;
    logic execute_mem_rd_value_write_enable;
    logic [31:0] execute_mem_rd_value_write_data;
    logic execute_mem_read_issued;
    execute_mem execute_mem0(
        .ready(execute_mem_ready),
        .enable(execute_mem_enable),

        .pc_value(pc_value),

        .instruction(instruction),

        .register_file_read_data1(register_file_read_data1),
        .register_file_read_data2(register_file_read_data2),

        .rd(execute_mem_rd),

        .next_pc(execute_mem_next_pc),

        .rd_value_write_enable(execute_mem_rd_value_write_enable),
        .rd_value_write_data(execute_mem_rd_value_write_data),

        .read_issued(execute_mem_read_issued),

        .system_bus_write_data(system_bus_write_data),
        .system_bus_write_req(system_bus_write_req));

    logic writeback_ready;
    logic writeback_enable;
    writeback writeback0(
        .ready(writeback_ready),
        .enable(writeback_enable),

        .rd(execute_mem_rd),

        .next_pc(execute_mem_next_pc),

        .rd_value_write_enable(execute_mem_rd_value_write_enable),
        .rd_value_write_data(execute_mem_rd_value_write_data),

        .read_issued(execute_mem_read_issued),

        .pc_write_data(pc_write_data),
        .pc_write_enable(pc_write_enable),

        .register_file_write_enable(register_file_write_enable),
        .register_file_write_addr(register_file_write_addr),
        .register_file_write_data(register_file_write_data));

    logic decode_enable;
    control control0(
        .clk(clk),
        .reset_n(reset_n),

        .instruction_fetch_ready(instruction_fetch_ready),
        .instruction_fetch_enable(instruction_fetch_enable),

        .decode_ready(decode_ready),
        .decode_enable(decode_enable),

        .execute_mem_ready(execute_mem_ready),
        .execute_mem_enable(execute_mem_enable),

        .writeback_ready(writeback_ready),
        .writeback_enable(writeback_enable));

endmodule
