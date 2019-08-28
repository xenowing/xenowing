`default_nettype none

module ddr3_interface(
    input reset_n,
    input clk,

    output cpu_avl_ready,
    input [23:0] cpu_avl_addr,
    output cpu_avl_rdata_valid,
    output [63:0] cpu_avl_rdata,
    input [63:0] cpu_avl_wdata,
    input [7:0] cpu_avl_be,
    input cpu_avl_read_req,
    input cpu_avl_write_req,

    output display_avl_ready,
    input [23:0] display_avl_addr,
    output display_avl_rdata_valid,
    output [63:0] display_avl_rdata,
    input [7:0] display_avl_be,
    input display_avl_read_req,

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

    logic read_select_fifo_full;
    logic read_select_fifo_read_data;
    fifo #(.data_width(1), .depth_bits(4)) read_select_fifo(
        .reset_n(reset_n),
        .clk(clk),

        .full(read_select_fifo_full),

        .write_data(display_avl_read_req),
        .write_enable(avl_ready && (cpu_avl_read_req || display_avl_read_req)),

        .read_data(read_select_fifo_read_data),
        .read_enable(avl_rdata_valid));

    assign cpu_avl_rdata_valid = avl_rdata_valid && !read_select_fifo_read_data;
    assign cpu_avl_rdata = avl_rdata;

    assign display_avl_rdata_valid = avl_rdata_valid && read_select_fifo_read_data;
    assign display_avl_rdata = avl_rdata;

    assign avl_wdata = cpu_avl_wdata;
    assign avl_size = 7'h1;

    always_comb begin
        cpu_avl_ready = avl_ready && !read_select_fifo_full;

        display_avl_ready = 0;

        avl_burstbegin = cpu_avl_write_req;
        avl_addr = cpu_avl_addr;
        avl_be = cpu_avl_be;
        avl_read_req = cpu_avl_read_req;
        avl_write_req = cpu_avl_write_req;

        if (display_avl_read_req) begin
            // Display reads always take priority over CPU reads/writes
            cpu_avl_ready = 0;

            display_avl_ready = avl_ready && !read_select_fifo_full;

            avl_burstbegin = 0;
            avl_addr = display_avl_addr;
            avl_be = display_avl_be;
            avl_read_req = display_avl_read_req;
            avl_write_req = 0;
        end
    end

endmodule
