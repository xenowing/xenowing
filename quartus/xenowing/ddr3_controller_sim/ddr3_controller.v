// ddr3_controller.v

// Generated using ACDS version 18.1 646

`timescale 1 ps / 1 ps
module ddr3_controller (
		input  wire        pll_ref_clk,        //      pll_ref_clk.clk
		input  wire        global_reset_n,     //     global_reset.reset_n
		input  wire        soft_reset_n,       //       soft_reset.reset_n
		output wire        afi_clk,            //          afi_clk.clk
		output wire        afi_half_clk,       //     afi_half_clk.clk
		output wire        afi_reset_n,        //        afi_reset.reset_n
		output wire        afi_reset_export_n, // afi_reset_export.reset_n
		output wire [12:0] mem_a,              //           memory.mem_a
		output wire [2:0]  mem_ba,             //                 .mem_ba
		inout  wire [0:0]  mem_ck,             //                 .mem_ck
		inout  wire [0:0]  mem_ck_n,           //                 .mem_ck_n
		output wire [0:0]  mem_cke,            //                 .mem_cke
		output wire [0:0]  mem_cs_n,           //                 .mem_cs_n
		output wire [2:0]  mem_dm,             //                 .mem_dm
		output wire [0:0]  mem_ras_n,          //                 .mem_ras_n
		output wire [0:0]  mem_cas_n,          //                 .mem_cas_n
		output wire [0:0]  mem_we_n,           //                 .mem_we_n
		output wire        mem_reset_n,        //                 .mem_reset_n
		inout  wire [23:0] mem_dq,             //                 .mem_dq
		inout  wire [2:0]  mem_dqs,            //                 .mem_dqs
		inout  wire [2:0]  mem_dqs_n,          //                 .mem_dqs_n
		output wire [0:0]  mem_odt,            //                 .mem_odt
		output wire        avl_ready,          //              avl.waitrequest_n
		input  wire        avl_burstbegin,     //                 .beginbursttransfer
		input  wire [23:0] avl_addr,           //                 .address
		output wire        avl_rdata_valid,    //                 .readdatavalid
		output wire [95:0] avl_rdata,          //                 .readdata
		input  wire [95:0] avl_wdata,          //                 .writedata
		input  wire [11:0] avl_be,             //                 .byteenable
		input  wire        avl_read_req,       //                 .read
		input  wire        avl_write_req,      //                 .write
		input  wire        avl_size,           //                 .burstcount
		output wire        local_init_done,    //           status.local_init_done
		output wire        local_cal_success,  //                 .local_cal_success
		output wire        local_cal_fail,     //                 .local_cal_fail
		output wire        pll_mem_clk,        //      pll_sharing.pll_mem_clk
		output wire        pll_write_clk,      //                 .pll_write_clk
		output wire        pll_locked,         //                 .pll_locked
		output wire        pll_capture0_clk,   //                 .pll_capture0_clk
		output wire        pll_capture1_clk    //                 .pll_capture1_clk
	);

	ddr3_controller_0002 ddr3_controller_inst (
		.pll_ref_clk        (pll_ref_clk),        //      pll_ref_clk.clk
		.global_reset_n     (global_reset_n),     //     global_reset.reset_n
		.soft_reset_n       (soft_reset_n),       //       soft_reset.reset_n
		.afi_clk            (afi_clk),            //          afi_clk.clk
		.afi_half_clk       (afi_half_clk),       //     afi_half_clk.clk
		.afi_reset_n        (afi_reset_n),        //        afi_reset.reset_n
		.afi_reset_export_n (afi_reset_export_n), // afi_reset_export.reset_n
		.mem_a              (mem_a),              //           memory.mem_a
		.mem_ba             (mem_ba),             //                 .mem_ba
		.mem_ck             (mem_ck),             //                 .mem_ck
		.mem_ck_n           (mem_ck_n),           //                 .mem_ck_n
		.mem_cke            (mem_cke),            //                 .mem_cke
		.mem_cs_n           (mem_cs_n),           //                 .mem_cs_n
		.mem_dm             (mem_dm),             //                 .mem_dm
		.mem_ras_n          (mem_ras_n),          //                 .mem_ras_n
		.mem_cas_n          (mem_cas_n),          //                 .mem_cas_n
		.mem_we_n           (mem_we_n),           //                 .mem_we_n
		.mem_reset_n        (mem_reset_n),        //                 .mem_reset_n
		.mem_dq             (mem_dq),             //                 .mem_dq
		.mem_dqs            (mem_dqs),            //                 .mem_dqs
		.mem_dqs_n          (mem_dqs_n),          //                 .mem_dqs_n
		.mem_odt            (mem_odt),            //                 .mem_odt
		.avl_ready          (avl_ready),          //              avl.waitrequest_n
		.avl_burstbegin     (avl_burstbegin),     //                 .beginbursttransfer
		.avl_addr           (avl_addr),           //                 .address
		.avl_rdata_valid    (avl_rdata_valid),    //                 .readdatavalid
		.avl_rdata          (avl_rdata),          //                 .readdata
		.avl_wdata          (avl_wdata),          //                 .writedata
		.avl_be             (avl_be),             //                 .byteenable
		.avl_read_req       (avl_read_req),       //                 .read
		.avl_write_req      (avl_write_req),      //                 .write
		.avl_size           (avl_size),           //                 .burstcount
		.local_init_done    (local_init_done),    //           status.local_init_done
		.local_cal_success  (local_cal_success),  //                 .local_cal_success
		.local_cal_fail     (local_cal_fail),     //                 .local_cal_fail
		.pll_mem_clk        (pll_mem_clk),        //      pll_sharing.pll_mem_clk
		.pll_write_clk      (pll_write_clk),      //                 .pll_write_clk
		.pll_locked         (pll_locked),         //                 .pll_locked
		.pll_capture0_clk   (pll_capture0_clk),   //                 .pll_capture0_clk
		.pll_capture1_clk   (pll_capture1_clk)    //                 .pll_capture1_clk
	);

endmodule
