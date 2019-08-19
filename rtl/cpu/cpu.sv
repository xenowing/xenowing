`default_nettype none

module cpu(
    input reset_n,
    input clk,

    input system_bus_ready,
    output [29:0] system_bus_addr,
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

    logic [63:0] cycle_counter_value;
    cycle_counter cycle_counter0(
        .clk(clk),
        .reset_n(reset_n),

        .value(cycle_counter_value));

    logic [63:0] instructions_retired_counter_value;
    logic instructions_retired_counter_increment_enable;
    instructions_retired_counter instructions_retired_counter0(
        .clk(clk),
        .reset_n(reset_n),

        .value(instructions_retired_counter_value),
        .increment_enable(instructions_retired_counter_increment_enable));

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
    logic [29:0] instruction_fetch_bus_addr;
    logic [3:0] instruction_fetch_bus_byte_enable;
    logic instruction_fetch_bus_read_req;
    instruction_fetch instruction_fetch0(
        .clk(clk),
        .reset_n(reset_n),

        .ready(instruction_fetch_ready),
        .enable(instruction_fetch_enable),

        .pc(pc_value[31:2]),

        .bus_ready(system_bus_ready),
        .bus_addr(instruction_fetch_bus_addr),
        .bus_byte_enable(instruction_fetch_bus_byte_enable),
        .bus_read_req(instruction_fetch_bus_read_req));

    logic decode_ready;
    logic [31:0] decode_instruction;
    decode decode0(
        .clk(clk),
        .reset_n(reset_n),

        .ready(decode_ready),

        .instruction(decode_instruction),

        .bus_read_data(system_bus_read_data),
        .bus_read_data_valid(system_bus_read_data_valid));

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

    assign register_file_read_addr1 = instruction[19:15]; // rs1
    assign register_file_read_addr2 = instruction[24:20]; // rs2

    logic [31:0] rs1_value;
    logic [31:0] rs1_value_next;
    always_comb begin
        rs1_value_next = rs1_value;

        if (reg_wait_enable) begin
            rs1_value_next = register_file_read_data1;
        end
    end
    always_ff @(posedge clk) begin
        rs1_value <= rs1_value_next;
    end
    logic [31:0] rs2_value;
    logic [31:0] rs2_value_next;
    always_comb begin
        rs2_value_next = rs2_value;

        if (reg_wait_enable) begin
            rs2_value_next = register_file_read_data2;
        end
    end
    always_ff @(posedge clk) begin
        rs2_value <= rs2_value_next;
    end

    logic [2:0] alu_op;
    logic alu_op_mod;
    logic [31:0] alu_lhs;
    logic [31:0] alu_rhs;
    logic [31:0] alu_res;
    alu alu0(
        .op(alu_op),
        .op_mod(alu_op_mod),
        .lhs(alu_lhs),
        .rhs(alu_rhs),
        .res(alu_res));

    logic execute_enable;
    logic [31:0] execute_next_pc;
    logic execute_rd_value_write_enable;
    logic [31:0] execute_rd_value_write_data;
    logic [31:0] execute_bus_addr;
    logic [31:0] execute_bus_write_data;
    logic [3:0] execute_bus_byte_enable;
    logic execute_bus_read_req;
    logic execute_bus_write_req;
    execute execute0(
        .enable(execute_enable),

        .pc(pc_value),

        .instruction(instruction),

        .register_file_read_data1(rs1_value),
        .register_file_read_data2(rs2_value),

        .alu_op(alu_op),
        .alu_op_mod(alu_op_mod),
        .alu_lhs(alu_lhs),
        .alu_rhs(alu_rhs),
        .alu_res(alu_res),

        .next_pc(execute_next_pc),

        .rd_value_write_enable(execute_rd_value_write_enable),
        .rd_value_write_data(execute_rd_value_write_data),

        .cycle_counter_value(cycle_counter_value),
        .instructions_retired_counter_value(instructions_retired_counter_value),

        .bus_addr(execute_bus_addr),
        .bus_write_data(execute_bus_write_data),
        .bus_byte_enable(execute_bus_byte_enable),
        .bus_read_req(execute_bus_read_req),
        .bus_write_req(execute_bus_write_req));

    logic mem_ready;
    logic mem_enable;
    logic [31:0] mem_bus_addr;
    logic [3:0] mem_bus_byte_enable;
    logic mem_bus_read_req;
    logic mem_bus_write_req;
    mem mem0(
        .clk(clk),
        .reset_n(reset_n),

        .ready(mem_ready),
        .enable(mem_enable),

        .bus_ready(system_bus_ready),

        .bus_addr_in(execute_bus_addr),
        .bus_write_data_in(execute_bus_write_data),
        .bus_byte_enable_in(execute_bus_byte_enable),
        .bus_read_req_in(execute_bus_read_req),
        .bus_write_req_in(execute_bus_write_req),

        .bus_addr_out(mem_bus_addr),
        .bus_write_data_out(system_bus_write_data),
        .bus_byte_enable_out(mem_bus_byte_enable),
        .bus_read_req_out(mem_bus_read_req),
        .bus_write_req_out(mem_bus_write_req));

    assign system_bus_addr = (mem_bus_read_req | mem_bus_write_req) ? mem_bus_addr[31:2] : instruction_fetch_bus_addr;
    assign system_bus_byte_enable = (mem_bus_read_req | mem_bus_write_req) ? mem_bus_byte_enable : instruction_fetch_bus_byte_enable;
    assign system_bus_read_req = mem_bus_read_req | instruction_fetch_bus_read_req;
    assign system_bus_write_req = mem_bus_write_req;

    logic writeback_ready;
    logic writeback_enable;
    writeback writeback0(
        .ready(writeback_ready),
        .enable(writeback_enable),

        .instruction(instruction),
        .bus_addr_low(mem_bus_addr[1:0]),

        .next_pc(execute_next_pc),

        .rd_value_write_enable(execute_rd_value_write_enable),
        .rd_value_write_data(execute_rd_value_write_data),

        .pc_write_data(pc_write_data),
        .pc_write_enable(pc_write_enable),

        .instructions_retired_counter_increment_enable(instructions_retired_counter_increment_enable),

        .register_file_write_enable(register_file_write_enable),
        .register_file_write_addr(register_file_write_addr),
        .register_file_write_data(register_file_write_data),

        .bus_read_data(system_bus_read_data),
        .bus_read_data_valid(system_bus_read_data_valid));

    logic decode_enable;
    logic reg_wait_enable;
    control control0(
        .clk(clk),
        .reset_n(reset_n),

        .instruction_fetch_ready(instruction_fetch_ready),
        .instruction_fetch_enable(instruction_fetch_enable),

        .decode_ready(decode_ready),
        .decode_enable(decode_enable),

        .reg_wait_enable(reg_wait_enable),

        .execute_enable(execute_enable),

        .mem_ready(mem_ready),
        .mem_enable(mem_enable),

        .writeback_ready(writeback_ready),
        .writeback_enable(writeback_enable));

endmodule
