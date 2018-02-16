module test(
    input reset_n,
    input clk,

    input avl_ready,
    output avl_burstbegin,
    output [23:0] avl_addr,
    input avl_rdata_valid,
    input [63:0] avl_rdata,
    output [63:0] avl_wdata,
    output [7:0] avl_be,
    output avl_read_req,
    output avl_write_req,
    output [6:0] avl_size,

    input ddr3_init_done,
    input ddr3_cal_success,
    input ddr3_cal_fail,

    output is_finished,
    output pass,
    output fail);

    logic command_generator_is_finished;
    logic command_generator_pass;
    logic command_generator_fail;
    command_generator command_generator0(
        .reset_n(reset_n),
        .clk(clk),

        .avl_ready(avl_ready),
        .avl_burstbegin(avl_burstbegin),
        .avl_addr(avl_addr),
        .avl_wdata(avl_wdata),
        .avl_be(avl_be),
        .avl_read_req(avl_read_req),
        .avl_write_req(avl_write_req),
        .avl_size(avl_size),

        .ddr3_init_done(ddr3_init_done),
        .ddr3_cal_success(ddr3_cal_success),
        .ddr3_cal_fail(ddr3_cal_fail),

        .is_finished(command_generator_is_finished),
        .pass(command_generator_pass),
        .fail(command_generator_fail));

    logic read_checker_is_finished;
    logic read_checker_pass;
    logic read_checker_fail;
    read_checker read_checker0(
        .reset_n(reset_n),
        .clk(clk),

        .avl_rdata_valid(avl_rdata_valid),
        .avl_rdata(avl_rdata),

        .ddr3_init_done(ddr3_init_done),
        .ddr3_cal_success(ddr3_cal_success),
        .ddr3_cal_fail(ddr3_cal_fail),

        .is_finished(read_checker_is_finished),
        .pass(read_checker_pass),
        .fail(read_checker_fail));

    assign is_finished = command_generator_is_finished && read_checker_is_finished;
    assign pass = command_generator_pass && read_checker_pass;
    assign fail = command_generator_fail || read_checker_fail;

endmodule
