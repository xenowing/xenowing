`default_nettype none

module ram_interface(
    input reset_n,
    input clk,

    output cpu_avl_ready,
    input [13:0] cpu_avl_addr,
    output cpu_avl_rdata_valid,
    output [63:0] cpu_avl_rdata,
    input [63:0] cpu_avl_wdata,
    input [7:0] cpu_avl_be,
    input cpu_avl_read_req,
    input cpu_avl_write_req,

    output display_avl_ready,
    input [13:0] display_avl_addr,
    output display_avl_rdata_valid,
    output [63:0] display_avl_rdata,
    input [7:0] display_avl_be,
    input display_avl_read_req,

    output [13:0] ram_address,
    output [7:0] ram_byteena,
    output [63:0] ram_data,
    output ram_wren,
    input [63:0] ram_q);

    logic avl_ready = 1;
    logic avl_read_req;

    logic avl_rdata_valid, avl_rdata_valid_buffer;
    always @(posedge clk) begin
        if (~reset_n) begin
            { avl_rdata_valid, avl_rdata_valid_buffer } <= 2'b00;
        end
        else begin
            { avl_rdata_valid, avl_rdata_valid_buffer } <= { avl_rdata_valid_buffer, avl_read_req };
        end
    end

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
    assign cpu_avl_rdata = ram_q;

    assign display_avl_rdata_valid = avl_rdata_valid && read_select_fifo_read_data;
    assign display_avl_rdata = ram_q;

    assign ram_data = cpu_avl_wdata;

    always_comb begin
        cpu_avl_ready = avl_ready && !read_select_fifo_full;

        display_avl_ready = 0;

        ram_address = cpu_avl_addr;
        ram_byteena = cpu_avl_be;
        avl_read_req = cpu_avl_read_req;
        ram_wren = cpu_avl_write_req;

        if (display_avl_read_req) begin
            // Display reads always take priority over CPU reads/writes
            cpu_avl_ready = 0;

            display_avl_ready = avl_ready && !read_select_fifo_full;

            ram_address = display_avl_addr;
            ram_byteena = display_avl_be;
            avl_read_req = display_avl_read_req;
            ram_wren = 0;
        end
    end

endmodule
