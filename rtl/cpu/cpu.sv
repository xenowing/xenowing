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

    logic load_unit_read_ready;
    logic load_unit_read_req;
    logic [31:0] load_unit_read_addr;
    logic [3:0] load_unit_read_byte_enable;
    logic [31:0] load_unit_read_data;
    logic load_unit_read_data_valid;

    logic load_unit_mem_ready;
    logic [31:0] load_unit_mem_addr;
    logic [3:0] load_unit_mem_byte_enable;
    logic load_unit_mem_read_req;

    load_unit load_unit0(
        .reset_n(reset_n),
        .clk(clk),

        .read_ready(load_unit_read_ready),
        .read_req(load_unit_read_req),
        .read_addr(load_unit_read_addr),
        .read_byte_enable(load_unit_read_byte_enable),
        .read_data(load_unit_read_data),
        .read_data_valid(load_unit_read_data_valid),

        .mem_ready(load_unit_mem_ready),
        .mem_addr(load_unit_mem_addr),
        .mem_byte_enable(load_unit_mem_byte_enable),
        .mem_read_req(load_unit_mem_read_req),
        .mem_read_data(mem_read_data),
        .mem_read_data_valid(mem_read_data_valid));

    logic store_unit_write_req;
    logic [31:0] store_unit_write_addr;
    logic [31:0] store_unit_write_data;
    logic [3:0] store_unit_write_byte_enable;

    logic store_unit_write_ready;

    logic store_unit_mem_ready;
    logic [31:0] store_unit_mem_addr;
    logic [3:0] store_unit_mem_byte_enable;
    logic store_unit_mem_write_req;

    store_unit store_unit0(
        .clk(clk),
        .reset_n(reset_n),

        .write_ready(store_unit_write_ready),
        .write_req(store_unit_write_req),
        .write_addr(store_unit_write_addr),
        .write_data(store_unit_write_data),
        .write_byte_enable(store_unit_write_byte_enable),

        .mem_ready(store_unit_mem_ready),
        .mem_addr(store_unit_mem_addr),
        .mem_write_data(mem_write_data),
        .mem_byte_enable(store_unit_mem_byte_enable),
        .mem_write_req(store_unit_mem_write_req));

    logic core_mem_ready;
    logic [31:0] core_mem_addr;
    logic [3:0] core_mem_byte_enable;
    logic core_mem_read_req;

    core core0(
        .reset_n(reset_n),
        .clk(clk),

        .mem_ready(core_mem_ready),
        .mem_addr(core_mem_addr),
        .mem_byte_enable(core_mem_byte_enable),
        .mem_read_req(core_mem_read_req),
        .mem_read_data(mem_read_data),
        .mem_read_data_valid(mem_read_data_valid),

        .load_unit_read_ready(load_unit_read_ready),
        .load_unit_read_req(load_unit_read_req),
        .load_unit_read_addr(load_unit_read_addr),
        .load_unit_read_byte_enable(load_unit_read_byte_enable),
        .load_unit_read_data(load_unit_read_data),
        .load_unit_read_data_valid(load_unit_read_data_valid),

        .store_unit_write_ready(store_unit_write_ready),
        .store_unit_write_req(store_unit_write_req),
        .store_unit_write_addr(store_unit_write_addr),
        .store_unit_write_data(store_unit_write_data),
        .store_unit_write_byte_enable(store_unit_write_byte_enable));

    always_comb begin
        mem_addr = 32'h0;
        mem_byte_enable = 4'h0;
        mem_write_req = 0;
        mem_read_req = 0;

        load_unit_mem_ready = mem_ready;
        store_unit_mem_ready = mem_ready;
        core_mem_ready = mem_ready;

        if (load_unit_mem_read_req) begin
            mem_addr = load_unit_mem_addr;
            mem_byte_enable = load_unit_mem_byte_enable;
            mem_read_req = load_unit_mem_read_req;

            store_unit_mem_ready = 0;
            core_mem_ready = 0;
        end
        else if (store_unit_mem_write_req) begin
            mem_addr = store_unit_mem_addr;
            mem_byte_enable = store_unit_mem_byte_enable;
            mem_write_req = store_unit_mem_write_req;

            load_unit_mem_ready = 0;
            core_mem_ready = 0;
        end
        else if (core_mem_read_req) begin
            mem_addr = core_mem_addr;
            mem_byte_enable = core_mem_byte_enable;
            mem_read_req = core_mem_read_req;

            load_unit_mem_ready = 0;
            store_unit_mem_ready = 0;
        end
    end

endmodule
