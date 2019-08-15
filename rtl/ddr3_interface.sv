`default_nettype none

module ddr3_interface(
    input reset_n,
    input clk,

    output ready,
    input [26:2] addr,
    input [31:0] write_data,
    input [3:0] byte_enable,
    input write_req,
    input read_req,
    output [31:0] read_data,
    output read_data_valid,

    input avl_ready,
    output avl_burstbegin,
    output [23:0] avl_addr,
    input avl_rdata_valid,
    input [63:0] avl_rdata,
    output [63:0] avl_wdata,
    output [7:0] avl_be,
    output avl_read_req,
    output avl_write_req,
    output [6:0] avl_size);

    logic addr2_fifo_full;

    logic addr2_fifo_read_data;

    fifo #(.data_width(1), .depth_bits(4)) addr2_fifo(
        .reset_n(reset_n),
        .clk(clk),

        .full(addr2_fifo_full),

        .write_data(addr[2]),
        .write_enable(avl_ready && read_req),

        .read_data(addr2_fifo_read_data),
        .read_enable(avl_rdata_valid));

    assign ready = avl_ready && !addr2_fifo_full;
    assign read_data = avl_rdata >> {addr2_fifo_read_data, 5'h0};
    assign read_data_valid = avl_rdata_valid;

    assign avl_burstbegin = write_req; // TODO: This is probably crap. Somehow we need to detect that a write is starting, or we need to extend our interface for this.
    assign avl_addr = addr[26:3];
    assign avl_wdata = {32'h0, write_data} << {addr[2], 5'h0};
    assign avl_be = {4'h0, byte_enable} << {addr[2], 2'h0};
    assign avl_read_req = read_req;
    assign avl_write_req = write_req;
    assign avl_size = 7'h1;

endmodule
