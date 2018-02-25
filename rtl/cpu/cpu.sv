`default_nettype none

module cpu(
    input reset_n,
    input clk,

    input mem_ready,
    output [31:0] mem_addr,
    output [31:0] mem_write_data,
    output [3:0] mem_byte_enable,
    output mem_write_req,
    output mem_read_req,
    input [31:0] mem_read_data,
    input mem_read_data_valid);

    logic store_unit_write_req;
    logic [31:0] store_unit_write_addr;
    logic [31:0] store_unit_write_data;
    logic [3:0] store_unit_write_byte_enable;

    logic store_unit_busy;

    logic [31:0] store_unit_mem_addr;
    logic [3:0] store_unit_mem_byte_enable;
    logic store_unit_mem_write_req;

    store_unit store_unit0(
        .clk(clk),
        .reset_n(reset_n),

        .write_req(store_unit_write_req),
        .write_addr(store_unit_write_addr),
        .write_data(store_unit_write_data),
        .write_byte_enable(store_unit_write_byte_enable),

        .busy(store_unit_busy),

        .mem_ready(mem_ready),
        .mem_addr(store_unit_mem_addr),
        .mem_write_data(mem_write_data),
        .mem_byte_enable(store_unit_mem_byte_enable),
        .mem_write_req(store_unit_mem_write_req));

    logic [31:0] core_mem_addr;
    logic [3:0] core_mem_byte_enable;
    logic core_mem_read_req;

    core core0(
        .reset_n(reset_n),
        .clk(clk),

        .mem_ready(mem_ready && !store_unit_mem_write_req),
        .mem_addr(core_mem_addr),
        .mem_byte_enable(core_mem_byte_enable),
        .mem_read_req(core_mem_read_req),
        .mem_read_data(mem_read_data),
        .mem_read_data_valid(mem_read_data_valid),

        .store_unit_busy(store_unit_busy),
        .store_unit_write_req(store_unit_write_req),
        .store_unit_write_addr(store_unit_write_addr),
        .store_unit_write_data(store_unit_write_data),
        .store_unit_write_byte_enable(store_unit_write_byte_enable));

    assign mem_addr = !store_unit_mem_write_req ? core_mem_addr : store_unit_mem_addr;
    assign mem_byte_enable = !store_unit_mem_write_req ? core_mem_byte_enable : store_unit_mem_byte_enable;
    assign mem_write_req = store_unit_mem_write_req;
    assign mem_read_req = core_mem_read_req && !store_unit_mem_write_req;

endmodule
