`default_nettype none

module Top(
    input wire logic reset,
    input wire logic input_clk_100,

    inout wire logic [15:0] ddr3_dq,
    inout wire logic [1:0] ddr3_dqs_n,
    inout wire logic [1:0] ddr3_dqs_p,
    output wire logic [13:0] ddr3_addr,
    output wire logic [2:0] ddr3_ba,
    output wire logic ddr3_ras_n,
    output wire logic ddr3_cas_n,
    output wire logic ddr3_we_n,
    output wire logic ddr3_reset_n, // TODO: Add timing constraints for this pin
    output wire logic [0:0] ddr3_ck_p,
    output wire logic [0:0] ddr3_ck_n,
    output wire logic [0:0] ddr3_cke,
    output wire logic [0:0] ddr3_cs_n,
    output wire logic [1:0] ddr3_dm,
    output wire logic [0:0] ddr3_odt,

    output wire logic tx,
    input wire logic rx,

    output wire logic [7:0] leds);

    logic sys_clk_200;
    clk_mmcm clk_mmcm0(
        .clk_200(sys_clk_200),
        .reset(reset),
        .clk_in(input_clk_100));

    logic sys_reset_n;
    SyncChain #(.DEFAULT(1'b0)) reset_sync_chain(
        .reset_n(~reset),
        .clk(sys_clk_200),

        .x(1'b1),

        .x_sync(sys_reset_n));

    logic init_calib_complete;
    logic [27:0] app_addr;
    logic [2:0] app_cmd;
    logic app_en;
    logic [127:0] app_wdf_data;
    logic app_wdf_wren;
    logic app_wdf_end;
    logic [127:0] app_rd_data;
    logic app_rd_data_valid;
    logic app_rdy;
    logic app_wdf_rdy;
    logic [15:0] app_wdf_mask;
    logic clk_100;
    logic ddr3_ui_clk_sync_rst;
    ddr3 ddr3_controller(
        .sys_clk_i(sys_clk_200),
        .sys_rst(sys_reset_n),

        .ddr3_addr(ddr3_addr),
        .ddr3_ba(ddr3_ba),
        .ddr3_cas_n(ddr3_cas_n),
        .ddr3_ck_n(ddr3_ck_n),
        .ddr3_ck_p(ddr3_ck_p),
        .ddr3_cke(ddr3_cke),
        .ddr3_ras_n(ddr3_ras_n),
        .ddr3_reset_n(ddr3_reset_n),
        .ddr3_we_n(ddr3_we_n),
        .ddr3_dq(ddr3_dq),
        .ddr3_dqs_n(ddr3_dqs_n),
        .ddr3_dqs_p(ddr3_dqs_p),
        .ddr3_cs_n(ddr3_cs_n),
        .ddr3_dm(ddr3_dm),
        .ddr3_odt(ddr3_odt),

        .init_calib_complete(init_calib_complete),

        .app_addr(app_addr),
        .app_cmd(app_cmd),
        .app_en(app_en),
        .app_wdf_data(app_wdf_data),
        .app_wdf_end(app_wdf_end),
        .app_wdf_wren(app_wdf_wren),
        .app_rd_data(app_rd_data),
        .app_rd_data_valid(app_rd_data_valid),
        .app_rdy(app_rdy),
        .app_wdf_rdy(app_wdf_rdy),
        .app_sr_req(0),
        .app_ref_req(0),
        .app_zq_req(0),
        .app_wdf_mask(app_wdf_mask),

        .ui_clk(clk_100),
        .ui_clk_sync_rst(ddr3_ui_clk_sync_rst));

    logic reset_n = ~ddr3_ui_clk_sync_rst & init_calib_complete;

    logic rx_sync;
    SyncChain #(.DEFAULT(1'b1)) rx_sync_chain(
        .reset_n(reset_n),
        .clk(clk_100),

        .x(rx),

        .x_sync(rx_sync));

    logic [23:0] bridge_app_addr;
    Xenowing xenowing(
        .reset_n(reset_n),
        .clk(clk_100),

        .tx(tx),
        .rx(rx_sync),

        .leds(leds),

        .ddr3_init_calib_complete(init_calib_complete),

        .ddr3_app_rdy(app_rdy),
        .ddr3_app_en(app_en),
        .ddr3_app_cmd(app_cmd),
        .ddr3_app_addr(bridge_app_addr),

        .ddr3_app_wdf_rdy(app_wdf_rdy),
        .ddr3_app_wdf_data(app_wdf_data),
        .ddr3_app_wdf_wren(app_wdf_wren),
        .ddr3_app_wdf_mask(app_wdf_mask),
        .ddr3_app_wdf_end(app_wdf_end),

        .ddr3_app_rd_data(app_rd_data),
        .ddr3_app_rd_data_valid(app_rd_data_valid));

    // TODO: Consider at least chopping off the bottom 3 addr bits in BusterMigUiBridge
    assign app_addr = {1'h0, bridge_app_addr, 3'h0};

endmodule
