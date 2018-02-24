`default_nettype none

module hw_top(
    input pll_ref_clk,
    input global_reset_n,

    output [12:0] mem_a,
    output [2:0] mem_ba,
    inout mem_ck,
    inout mem_ck_n,
    output mem_cke,
    output mem_cs_n,
    output [2:0] mem_dm,
    output mem_ras_n,
    output mem_cas_n,
    output mem_we_n,
    output mem_reset_n,
    inout [23:0] mem_dq,
    inout [2:0] mem_dqs,
    inout [2:0] mem_dqs_n,
    output mem_odt,

    output [2:0] leds_n);

    logic clk;

    logic avl_ready;
    logic avl_burstbegin;
    logic [23:0] avl_addr;
    logic avl_rdata_valid;
    logic [95:0] avl_rdata;
    logic [95:0] avl_wdata;
    logic [11:0] avl_be;
    logic avl_read_req;
    logic avl_write_req;
    logic [6:0] avl_size;

    logic [2:0] leds;
    logic [2:0] leds_next;
    assign leds_n = ~leds;

    logic ddr3_init_done;
    logic ddr3_cal_success;
    logic ddr3_cal_fail;

    logic test_is_finished;
    logic test_pass;
    logic test_fail;
    test test0(
        .reset_n(global_reset_n),
        .clk(clk),

        .avl_ready(avl_ready),
        .avl_burstbegin(avl_burstbegin),
        .avl_addr(avl_addr),
        .avl_rdata_valid(avl_rdata_valid),
        .avl_rdata(avl_rdata[63:0]),
        .avl_wdata(avl_wdata[63:0]),
        .avl_be(avl_be[7:0]),
        .avl_read_req(avl_read_req),
        .avl_write_req(avl_write_req),
        .avl_size(avl_size),

        .ddr3_init_done(ddr3_init_done),
        .ddr3_cal_success(ddr3_cal_success),
        .ddr3_cal_fail(ddr3_cal_fail),

        .is_finished(test_is_finished),
        .pass(test_pass),
        .fail(test_fail));

    ddr3_controller ddr3_controller0(
        .pll_ref_clk(pll_ref_clk),
        .global_reset_n(global_reset_n),
        .soft_reset_n(1),
        .afi_clk(clk),

        .mem_a(mem_a),
        .mem_ba(mem_ba),
        .mem_ck(mem_ck),
        .mem_ck_n(mem_ck_n),
        .mem_cke(mem_cke),
        .mem_cs_n(mem_cs_n),
        .mem_dm(mem_dm),
        .mem_ras_n(mem_ras_n),
        .mem_cas_n(mem_cas_n),
        .mem_we_n(mem_we_n),
        .mem_reset_n(mem_reset_n),
        .mem_dq(mem_dq),
        .mem_dqs(mem_dqs),
        .mem_dqs_n(mem_dqs_n),
        .mem_odt(mem_odt),

        .avl_ready(avl_ready),
        .avl_burstbegin(avl_burstbegin),
        .avl_addr(avl_addr),
        .avl_rdata_valid(avl_rdata_valid),
        .avl_rdata(avl_rdata),
        .avl_wdata(avl_wdata),
        .avl_be(avl_be),
        .avl_read_req(avl_read_req),
        .avl_write_req(avl_write_req),
        .avl_size(avl_size),

        .local_init_done(ddr3_init_done),
        .local_cal_success(ddr3_cal_success),
        .local_cal_fail(ddr3_cal_fail));

    logic led_clock_edge;
    led_clock_divider led_clock_divider0(
        .reset_n(global_reset_n),
        .clk(clk),
        .clock_edge(led_clock_edge));

    always_comb begin
        leds_next = leds;

        if (test_is_finished && led_clock_edge) begin
            if (test_pass) begin
                leds_next = leds + 3'h1;
            end
            else begin
                leds_next = ~leds;
            end
        end
    end

    always_ff @(posedge clk or negedge global_reset_n) begin
        if (!global_reset_n) begin
            leds <= 3'h0;
        end
        else begin
            leds <= leds_next;
        end
    end

endmodule
