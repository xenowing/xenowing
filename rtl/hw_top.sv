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

    output [2:0] leds_n,

    output uart_tx);

    logic [13:0] xenowing_program_rom_addr;
    logic [31:0] xenowing_program_rom_q;
    logic [2:0] xenowing_leds;
    xenowing xenowing0(
        .reset_n(ddr3_controller_local_init_done),
        .clk(clk),

        .program_rom_addr(xenowing_program_rom_addr),
        .program_rom_q(xenowing_program_rom_q),

        .leds(xenowing_leds),

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

        .uart_tx(uart_tx));

    program_rom program_rom0(
        .clock(clk),
        .address(xenowing_program_rom_addr),
        .q(xenowing_program_rom_q));

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

    logic ddr3_controller_local_init_done;

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

        .local_init_done(ddr3_controller_local_init_done));

    assign leds_n = ~xenowing_leds;

endmodule
