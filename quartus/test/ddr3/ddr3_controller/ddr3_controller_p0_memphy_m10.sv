// (C) 2001-2017 Intel Corporation. All rights reserved.
// Your use of Intel Corporation's design tools, logic functions and other 
// software and tools, and its AMPP partner logic functions, and any output 
// files from any of the foregoing (including device programming or simulation 
// files), and any associated documentation or information are expressly subject 
// to the terms and conditions of the Intel Program License Subscription 
// Agreement, Intel FPGA IP License Agreement, or other applicable 
// license agreement, including, without limitation, that your use is for the 
// sole purpose of programming logic devices manufactured by Intel and sold by 
// Intel or its authorized distributors.  Please refer to the applicable 
// agreement for further details.


// ******************************************************************************************************************************** 
// File name: memphy.v
// This file instantiates all the main components of the PHY. 
// ******************************************************************************************************************************** 

`timescale 1 ps / 1 ps

module ddr3_controller_p0_memphy_m10(
    global_reset_n,
    soft_reset_n,
    ctl_reset_n,
    ctl_reset_export_n,
    pll_locked,
    afi_addr,
    afi_cke,
    afi_cs_n,
    afi_ba,
    afi_ras_n,
    afi_cas_n,
    afi_we_n,
    afi_odt,
    afi_rst_n,
    afi_mem_clk_disable,
    afi_dqs_burst,
    afi_wdata,
    afi_wdata_valid,
    afi_dm,
    afi_rdata,
    afi_rdata_en,
    afi_rdata_valid,
    afi_cal_debug_info,
    afi_cal_success,
    afi_cal_fail,

    mem_a,
    mem_ba,
    mem_odt,
    mem_ras_n,
    mem_cas_n,
    mem_we_n,
    mem_reset_n,
    mem_ck,
    mem_ck_n,
    mem_cke,
    mem_cs_n,
    mem_dm,
    mem_dq,
    mem_dqs,
    mem_dqs_n,
    phy_read_latency_counter,
    phy_read_increment_vfifo_hr,
    phy_cal_debug_info,
    phy_read_fifo_reset,
    phy_vfifo_rd_en_override,
    calib_skip_steps,
    pll_afi_clk,
    pll_mem_clk,
    pll_write_clk,
    pll_capture0_clk,
    pll_capture1_clk,
    phy_clk,
    phy_reset_n,
    
    pd_reset_n,
    pd_ack,
    pd_up,
    pd_down
);

//  ******************************************************************************************************************************** 
//  BEGIN PARAMETER SECTION
//  All parameters default to "" will have their values passed in from higher level wrapper with the controller and driver 
parameter DEVICE_FAMILY                         = "";

// PHY-Memory Interface
// Memory device specific parameters, they are set according to the memory spec
parameter MEM_ADDRESS_WIDTH                     = ""; 
parameter MEM_BANK_WIDTH                        = "";
parameter MEM_CLK_EN_WIDTH                      = ""; 
parameter MEM_CK_WIDTH                          = ""; 
parameter MEM_ODT_WIDTH                         = ""; 
parameter MEM_DQS_WIDTH                         = "";
parameter MEM_CHIP_SELECT_WIDTH                 = ""; 
parameter MEM_CONTROL_WIDTH                     = ""; 
parameter MEM_DM_WIDTH                          = ""; 
parameter MEM_DQ_WIDTH                          = ""; 
parameter MEM_READ_DQS_WIDTH                    = ""; 
parameter MEM_WRITE_DQS_WIDTH                   = "";

//  PHY-Controller (AFI) Interface
//  The AFI interface widths are derived from the memory interface widths based on full/half rate operations
//  The calculations are done on higher level wrapper
parameter AFI_ADDRESS_WIDTH                     = ""; 
parameter AFI_DEBUG_INFO_WIDTH                  = "";
parameter AFI_BANK_WIDTH                        = "";
parameter AFI_CHIP_SELECT_WIDTH                 = "";
parameter AFI_CLK_EN_WIDTH                      = "";
parameter AFI_ODT_WIDTH                         = "";
parameter AFI_DATA_MASK_WIDTH                   = "";
parameter AFI_CONTROL_WIDTH                     = "";
parameter AFI_DATA_WIDTH                        = "";
parameter AFI_DQS_WIDTH                         = "";
parameter AFI_RATE_RATIO                        = "";

// Write Datapath
//The sequencer uses this value to control write latency during calibration
parameter NUM_WRITE_PATH_FLOP_STAGES            = "";
parameter NUM_WRITE_FR_CYCLE_SHIFTS             = "";

parameter NUM_AC_FR_CYCLE_SHIFTS                = "";

parameter CALIB_REG_WIDTH                       = "";

//  The number of AFI Resets to generate
localparam NUM_AFI_RESET = 4;

//  Read Datapath parameters
parameter MEM_T_RL                              = "";
parameter MAX_LATENCY_COUNT_WIDTH               = "";
localparam MAX_READ_LATENCY                     = MEM_T_RL/2 + 12; 
localparam INVERT_HR_CLOCK                      = MEM_T_RL % 2;

//  END PARAMETER SECTION
//  ******************************************************************************************************************************** 

//  ******************************************************************************************************************************** 
//  BEGIN PORT SECTION

//   Reset Interface
input                                            global_reset_n;        //Resets (active-low) the whole system (all PHY logic + PLL)
input                                            soft_reset_n;        
input                                            pll_locked;        
output                                           ctl_reset_export_n;    
output                                           ctl_reset_n;        

//  PHY-Controller Interface, AFI 2.0
//  Control Interface
input   [AFI_ADDRESS_WIDTH-1:0]                 afi_addr;               
input   [AFI_CLK_EN_WIDTH-1:0]                  afi_cke;
input   [AFI_CHIP_SELECT_WIDTH-1:0]             afi_cs_n;

input   [AFI_BANK_WIDTH-1:0]                    afi_ba;
input   [AFI_CONTROL_WIDTH-1:0]                 afi_cas_n;
input   [AFI_ODT_WIDTH-1:0]                     afi_odt;
input   [AFI_CONTROL_WIDTH-1:0]                 afi_ras_n;
input   [AFI_CONTROL_WIDTH-1:0]                 afi_we_n;
input   [AFI_CONTROL_WIDTH-1:0]                 afi_rst_n;

input   [MEM_CK_WIDTH-1:0]                      afi_mem_clk_disable;
input   [AFI_DQS_WIDTH-1:0]                     afi_dqs_burst;

//  Write data interface
input   [AFI_DATA_WIDTH-1:0]                    afi_wdata;              //write data
input   [AFI_DQS_WIDTH-1:0]                     afi_wdata_valid;        

input   [AFI_DATA_MASK_WIDTH-1:0]               afi_dm;                 

//  Read data interface
output  [AFI_DATA_WIDTH-1:0]                    afi_rdata;              //read data                
input   [AFI_RATE_RATIO-1:0]                    afi_rdata_en;           
output  [AFI_RATE_RATIO-1:0]                    afi_rdata_valid;        

//  Status interface
input                                           afi_cal_success;        //calibration success
input                                           afi_cal_fail;           
output  [AFI_DEBUG_INFO_WIDTH - 1:0]            afi_cal_debug_info;


//  PHY-Memory Interface
output  [MEM_ADDRESS_WIDTH-1:0]                 mem_a;
output  [MEM_BANK_WIDTH-1:0]                    mem_ba;
output  [MEM_CONTROL_WIDTH-1:0]                 mem_ras_n;
output  [MEM_CONTROL_WIDTH-1:0]                 mem_cas_n;
output  [MEM_CONTROL_WIDTH-1:0]                 mem_we_n;
output  [MEM_ODT_WIDTH-1:0]                     mem_odt;
output                                          mem_reset_n;
inout   [MEM_CK_WIDTH-1:0]                      mem_ck;
inout   [MEM_CK_WIDTH-1:0]                      mem_ck_n;
output  [MEM_CLK_EN_WIDTH-1:0]                  mem_cke;
output  [MEM_CHIP_SELECT_WIDTH-1:0]             mem_cs_n;
output  [MEM_DM_WIDTH-1:0]                      mem_dm;
inout   [MEM_DQ_WIDTH-1:0]                      mem_dq;
inout   [MEM_DQS_WIDTH-1:0]                     mem_dqs;
inout   [MEM_DQS_WIDTH-1:0]                     mem_dqs_n;

input   [MAX_LATENCY_COUNT_WIDTH-1:0]           phy_read_latency_counter;
input   [MEM_READ_DQS_WIDTH-1:0]                phy_read_increment_vfifo_hr;
input   [AFI_DEBUG_INFO_WIDTH - 1:0]            phy_cal_debug_info;
input   [MEM_READ_DQS_WIDTH-1:0]                phy_read_fifo_reset;
input   [MEM_READ_DQS_WIDTH-1:0]                phy_vfifo_rd_en_override;

output  [CALIB_REG_WIDTH-1:0]                   calib_skip_steps;

//  PLL Interface
input                                           pll_afi_clk;        //clocks AFI interface logic
input                                           pll_mem_clk;        
input                                           pll_write_clk;        
input                                           pll_capture0_clk;
input                                           pll_capture1_clk;

//  output to sequencer
output                                          phy_clk;
output                                          phy_reset_n;

input                                           pd_reset_n;
input                                           pd_ack;
output                                          pd_up;
output                                          pd_down;

//  END PORT SECTION
//  ******************************************************************************************************************************** 

// PHY clock tree buffered clock driving to DDIO
wire                                            pll_mem_clk_ddio;           
wire                                            pll_write_clk_ddio;      
wire                                            pll_capture0_clk_ddio;
wire                                            pll_capture1_clk_ddio;

wire    [AFI_ADDRESS_WIDTH-1:0]                 phy_ddio_address;
wire    [AFI_BANK_WIDTH-1:0]                    phy_ddio_bank;
wire    [AFI_CHIP_SELECT_WIDTH-1:0]             phy_ddio_cs_n;
wire    [AFI_CLK_EN_WIDTH-1:0]                  phy_ddio_cke;
wire    [AFI_ODT_WIDTH-1:0]                     phy_ddio_odt;
wire    [AFI_CONTROL_WIDTH-1:0]                 phy_ddio_ras_n;
wire    [AFI_CONTROL_WIDTH-1:0]                 phy_ddio_cas_n;
wire    [AFI_CONTROL_WIDTH-1:0]                 phy_ddio_we_n;
wire    [AFI_CONTROL_WIDTH-1:0]                 phy_ddio_reset_n;
wire    [AFI_DATA_WIDTH-1:0]                    phy_ddio_dq;
wire    [AFI_DQS_WIDTH-1:0]                     phy_ddio_dqs_en;
wire    [AFI_DQS_WIDTH-1:0]                     phy_ddio_wrdata_en;
wire    [AFI_DATA_MASK_WIDTH-1:0]               phy_ddio_wrdata_mask;
wire    [AFI_DATA_WIDTH-1:0]                    ddio_phy_dq;
wire    [MEM_READ_DQS_WIDTH-1:0]                read_capture_clk_hr_dq;


wire    [NUM_AFI_RESET-1:0]                     reset_n_afi_clk;
wire                                            reset_n_resync_clk;
wire                                            invert_hr_clock;

localparam SKIP_CALIBRATION_STEPS       = 7'b1111111;
localparam CALIBRATION_STEPS            = SKIP_CALIBRATION_STEPS;
localparam SKIP_MEM_INIT                = 1'b1;
localparam SEQ_CALIB_INIT               = {CALIBRATION_STEPS, SKIP_MEM_INIT};

localparam NUM_OF_DQDQS                 = MEM_WRITE_DQS_WIDTH;
localparam DQDQS_DATA_WIDTH             = MEM_DQ_WIDTH / NUM_OF_DQDQS;
localparam DQDQS_DM_WIDTH               = MEM_DM_WIDTH / NUM_OF_DQDQS;
localparam DQDQS_DDIO_PHY_DQ_WIDTH      = AFI_DATA_WIDTH / NUM_OF_DQDQS;

reg [CALIB_REG_WIDTH-1:0] seq_calib_init_reg /* synthesis syn_noprune syn_preserve = 1 */;

//  Initialization of the sequencer status register. This register
//  is preserved in the netlist so that it can be forced during simulation
always @(posedge pll_afi_clk)
    `ifndef SYNTH_FOR_SIM
    `endif
    seq_calib_init_reg <= SEQ_CALIB_INIT;
    `ifndef SYNTH_FOR_SIM
    `endif
    `ifndef SYNTH_FOR_SIM
    `endif

//  ******************************************************************************************************************************** 
//  Instantiate PHYCLK buffer for fitter 
//  ******************************************************************************************************************************** 
        fiftyfivenm_phy_clkbuf uphy_clkbuf(
                .inclk ({pll_capture1_clk, pll_capture0_clk, pll_write_clk, pll_mem_clk}),
                .outclk ({pll_capture1_clk_ddio, pll_capture0_clk_ddio, pll_write_clk_ddio, pll_mem_clk_ddio})
        );

//  ******************************************************************************************************************************** 
//  The reset scheme used in the UNIPHY is asynchronous assert and synchronous de-assert
//  The reset block has 2 main functionalities:
//  1. Keep all the PHY logic in reset state until after the PLL is locked
//  2. Synchronize the reset to each clock domain 
//  ******************************************************************************************************************************** 


    ddr3_controller_p0_reset_m10    ureset(
        .pll_afi_clk            (pll_afi_clk),
        .pll_write_clk          (pll_write_clk),
        .pll_locked             (pll_locked),
        .global_reset_n         (global_reset_n),
        .soft_reset_n           (soft_reset_n),
        .ctl_reset_export_n     (ctl_reset_export_n),
        .ctl_reset_n            (ctl_reset_n),
        .reset_n_afi_clk        (reset_n_afi_clk),
        .reset_n_resync_clk     (reset_n_resync_clk)
    );

    defparam ureset.MEM_READ_DQS_WIDTH = MEM_READ_DQS_WIDTH;
    defparam ureset.NUM_AFI_RESET = NUM_AFI_RESET;

assign calib_skip_steps = seq_calib_init_reg;
assign afi_cal_debug_info = phy_cal_debug_info;

assign phy_clk = pll_afi_clk;
assign phy_reset_n = reset_n_afi_clk[0];


//  ******************************************************************************************************************************** 
//  The address and command datapath is responsible for adding any flop stages/extra logic that may be required between the AFI
//  interface and the output DDIOs.
//  ******************************************************************************************************************************** 

    ddr3_controller_p0_addr_cmd_datapath    uaddr_cmd_datapath(
        .clk                (pll_afi_clk), 
        .reset_n            (reset_n_afi_clk[1]), 
        .afi_address        (afi_addr),
        .afi_cke            (afi_cke),
        .afi_cs_n           (afi_cs_n),
        .afi_bank           (afi_ba),
        .afi_odt            (afi_odt),
        .afi_ras_n          (afi_ras_n),
        .afi_cas_n          (afi_cas_n),
        .afi_we_n           (afi_we_n),
        .phy_ddio_odt       (phy_ddio_odt),
        .phy_ddio_bank      (phy_ddio_bank),
        .phy_ddio_we_n      (phy_ddio_we_n),
        .phy_ddio_ras_n     (phy_ddio_ras_n),
        .phy_ddio_cas_n     (phy_ddio_cas_n),
        .afi_rst_n          (afi_rst_n),
        .phy_ddio_reset_n   (phy_ddio_reset_n),
        .phy_ddio_address   (phy_ddio_address),
        .phy_ddio_cke       (phy_ddio_cke),
        .phy_ddio_cs_n      (phy_ddio_cs_n)
    );
        defparam uaddr_cmd_datapath.MEM_ADDRESS_WIDTH                  = MEM_ADDRESS_WIDTH;
        defparam uaddr_cmd_datapath.MEM_BANK_WIDTH                     = MEM_BANK_WIDTH;
        defparam uaddr_cmd_datapath.MEM_CHIP_SELECT_WIDTH              = MEM_CHIP_SELECT_WIDTH;
        defparam uaddr_cmd_datapath.MEM_CLK_EN_WIDTH                   = MEM_CLK_EN_WIDTH;
        defparam uaddr_cmd_datapath.MEM_ODT_WIDTH                      = MEM_ODT_WIDTH;
        defparam uaddr_cmd_datapath.MEM_DM_WIDTH                       = MEM_DM_WIDTH;
        defparam uaddr_cmd_datapath.MEM_CONTROL_WIDTH                  = MEM_CONTROL_WIDTH;
        defparam uaddr_cmd_datapath.MEM_DQ_WIDTH                       = MEM_DQ_WIDTH;
        defparam uaddr_cmd_datapath.MEM_READ_DQS_WIDTH                 = MEM_READ_DQS_WIDTH;
        defparam uaddr_cmd_datapath.MEM_WRITE_DQS_WIDTH                = MEM_WRITE_DQS_WIDTH;
        defparam uaddr_cmd_datapath.AFI_ADDRESS_WIDTH                  = AFI_ADDRESS_WIDTH;
        defparam uaddr_cmd_datapath.AFI_BANK_WIDTH                     = AFI_BANK_WIDTH;
        defparam uaddr_cmd_datapath.AFI_CHIP_SELECT_WIDTH              = AFI_CHIP_SELECT_WIDTH;
        defparam uaddr_cmd_datapath.AFI_CLK_EN_WIDTH                   = AFI_CLK_EN_WIDTH;
        defparam uaddr_cmd_datapath.AFI_ODT_WIDTH                      = AFI_ODT_WIDTH;
        defparam uaddr_cmd_datapath.AFI_DATA_MASK_WIDTH                = AFI_DATA_MASK_WIDTH;
        defparam uaddr_cmd_datapath.AFI_CONTROL_WIDTH                  = AFI_CONTROL_WIDTH;
        defparam uaddr_cmd_datapath.AFI_DATA_WIDTH                     = AFI_DATA_WIDTH;
        defparam uaddr_cmd_datapath.NUM_AC_FR_CYCLE_SHIFTS             = NUM_AC_FR_CYCLE_SHIFTS;    



//  ******************************************************************************************************************************** 
//  The write datapath is responsible for adding any flop stages/extra logic that may be required between the AFI interface 
//  and the output DDIOs.
//  ******************************************************************************************************************************** 

    ddr3_controller_p0_write_datapath_m10    uwrite_datapath(
        .pll_afi_clk                   (pll_afi_clk),
        .reset_n                       (reset_n_afi_clk[2]),
        .afi_dqs_en                    (afi_dqs_burst),
        .afi_wdata                     (afi_wdata),
        .afi_wdata_valid               (afi_wdata_valid),
        .afi_dm                        (afi_dm),
        .phy_ddio_dq                   (phy_ddio_dq),
        .phy_ddio_dqs_en               (phy_ddio_dqs_en),
        .phy_ddio_wrdata_en            (phy_ddio_wrdata_en),
        .phy_ddio_wrdata_mask          (phy_ddio_wrdata_mask)
    );
        defparam uwrite_datapath.MEM_ADDRESS_WIDTH                  = MEM_ADDRESS_WIDTH;
        defparam uwrite_datapath.MEM_DM_WIDTH                       = MEM_DM_WIDTH;
        defparam uwrite_datapath.MEM_CONTROL_WIDTH                  = MEM_CONTROL_WIDTH;
        defparam uwrite_datapath.MEM_DQ_WIDTH                       = MEM_DQ_WIDTH;
        defparam uwrite_datapath.MEM_READ_DQS_WIDTH                 = MEM_READ_DQS_WIDTH;
        defparam uwrite_datapath.MEM_WRITE_DQS_WIDTH                = MEM_WRITE_DQS_WIDTH;
        defparam uwrite_datapath.AFI_ADDRESS_WIDTH                  = AFI_ADDRESS_WIDTH;
        defparam uwrite_datapath.AFI_DATA_MASK_WIDTH                = AFI_DATA_MASK_WIDTH;
        defparam uwrite_datapath.AFI_CONTROL_WIDTH                  = AFI_CONTROL_WIDTH;
        defparam uwrite_datapath.AFI_DATA_WIDTH                     = AFI_DATA_WIDTH;
        defparam uwrite_datapath.AFI_DQS_WIDTH                      = AFI_DQS_WIDTH;
        defparam uwrite_datapath.NUM_WRITE_PATH_FLOP_STAGES         = NUM_WRITE_PATH_FLOP_STAGES;
        defparam uwrite_datapath.NUM_WRITE_FR_CYCLE_SHIFTS          = NUM_WRITE_FR_CYCLE_SHIFTS;

     
//  ******************************************************************************************************************************** 
//  The read datapath is responsible for read data resynchronization from the memory clock domain to the AFI clock domain.
//  It contains 1 FIFO per DQS group for read valid prediction and 1 FIFO per DQS group for read data synchronization.
//  ******************************************************************************************************************************** 

    ddr3_controller_p0_read_datapath_m10    uread_datapath(
        .reset_n_afi_clk            (reset_n_afi_clk[3]),
        .pll_afi_clk                (pll_afi_clk),
        .read_capture_clk_hr_dq     (read_capture_clk_hr_dq),
        .rdata_hr                   (ddio_phy_dq),
        .seq_read_increment_vfifo   (phy_read_increment_vfifo_hr),
        .seq_read_fifo_reset        (phy_read_fifo_reset),
        .seq_read_latency_counter   (phy_read_latency_counter),
        .afi_rdata_en               (afi_rdata_en),
        .afi_rdata                  (afi_rdata),
        .afi_rdata_valid            (afi_rdata_valid)
    );
    defparam uread_datapath.DEVICE_FAMILY                          = DEVICE_FAMILY;
    defparam uread_datapath.MEM_ADDRESS_WIDTH                      = MEM_ADDRESS_WIDTH; 
    defparam uread_datapath.MEM_DM_WIDTH                           = MEM_DM_WIDTH; 
    defparam uread_datapath.MEM_CONTROL_WIDTH                      = MEM_CONTROL_WIDTH; 
    defparam uread_datapath.MEM_DQ_WIDTH                           = MEM_DQ_WIDTH; 
    defparam uread_datapath.MEM_READ_DQS_WIDTH                     = MEM_READ_DQS_WIDTH; 
    defparam uread_datapath.MEM_WRITE_DQS_WIDTH                    = MEM_WRITE_DQS_WIDTH; 
    defparam uread_datapath.AFI_ADDRESS_WIDTH                      = AFI_ADDRESS_WIDTH; 
    defparam uread_datapath.AFI_DATA_MASK_WIDTH                    = AFI_DATA_MASK_WIDTH; 
    defparam uread_datapath.AFI_CONTROL_WIDTH                      = AFI_CONTROL_WIDTH; 
    defparam uread_datapath.AFI_DATA_WIDTH                         = AFI_DATA_WIDTH; 
    defparam uread_datapath.AFI_DQS_WIDTH                          = AFI_DQS_WIDTH;
    defparam uread_datapath.AFI_RATE_RATIO                         = AFI_RATE_RATIO;
    defparam uread_datapath.NUM_OF_DQDQS                           = NUM_OF_DQDQS;
    defparam uread_datapath.DQDQS_DATA_WIDTH                       = DQDQS_DATA_WIDTH;
    defparam uread_datapath.MAX_LATENCY_COUNT_WIDTH                = MAX_LATENCY_COUNT_WIDTH;
    defparam uread_datapath.MAX_READ_LATENCY                       = MAX_READ_LATENCY;
    defparam uread_datapath.MEM_T_RL                               = MEM_T_RL;

//  ******************************************************************************************************************************** 
//  The I/O block is responsible for instantiating all the built-in I/O logic in the FPGA
//  ******************************************************************************************************************************** 

wire addr_cmd_clk;
wire mimic_clock;

        assign addr_cmd_clk = pll_mem_clk_ddio;
        localparam INVERT_OUTPUT_CLOCK = "true";

    ddr3_controller_p0_addr_cmd_pads_m10 uaddr_cmd_pads(
        .reset_n            (reset_n_afi_clk[3]),
        .pll_afi_clk        (pll_afi_clk),
        .pll_mem_clk_ddio   (pll_mem_clk_ddio),
        .pll_mem_clk        (pll_mem_clk),
        .pll_write_clk      (addr_cmd_clk),
        .enable_mem_clk     (~afi_mem_clk_disable),
        .phy_ddio_address   (phy_ddio_address),
        .phy_ddio_cs_n      (phy_ddio_cs_n),
        .phy_ddio_cke       (phy_ddio_cke),
        .phy_ddio_bank      (phy_ddio_bank),
        .phy_ddio_odt       (phy_ddio_odt),
        .phy_ddio_we_n      (phy_ddio_we_n),    
        .phy_ddio_ras_n     (phy_ddio_ras_n),
        .phy_ddio_cas_n     (phy_ddio_cas_n),
        .phy_ddio_reset_n   (phy_ddio_reset_n),

        .phy_mem_cs_n       (mem_cs_n),
        .phy_mem_cke        (mem_cke),
        .phy_mem_address    (mem_a),
        .phy_mem_bank       (mem_ba),
        .phy_mem_odt        (mem_odt),
        .phy_mem_we_n       (mem_we_n),
        .phy_mem_ras_n      (mem_ras_n),
        .phy_mem_cas_n      (mem_cas_n),
        .phy_mem_reset_n    (mem_reset_n),
        .phy_mem_ck         (mem_ck),
        .phy_mem_ck_n       (mem_ck_n),
        .mimic_clock        (mimic_clock)
    );
    defparam uaddr_cmd_pads.DEVICE_FAMILY           = DEVICE_FAMILY;
    defparam uaddr_cmd_pads.MEM_ADDRESS_WIDTH       = MEM_ADDRESS_WIDTH;
    defparam uaddr_cmd_pads.MEM_CHIP_SELECT_WIDTH   = MEM_CHIP_SELECT_WIDTH;
    defparam uaddr_cmd_pads.MEM_CLK_EN_WIDTH        = MEM_CLK_EN_WIDTH;
    defparam uaddr_cmd_pads.MEM_CONTROL_WIDTH       = MEM_CONTROL_WIDTH;
    defparam uaddr_cmd_pads.AFI_ADDRESS_WIDTH       = AFI_ADDRESS_WIDTH; 
    defparam uaddr_cmd_pads.AFI_CHIP_SELECT_WIDTH   = AFI_CHIP_SELECT_WIDTH; 
    defparam uaddr_cmd_pads.AFI_CLK_EN_WIDTH        = AFI_CLK_EN_WIDTH; 
    defparam uaddr_cmd_pads.AFI_CONTROL_WIDTH       = AFI_CONTROL_WIDTH; 
    defparam uaddr_cmd_pads.INVERT_OUTPUT_CLOCK     = INVERT_OUTPUT_CLOCK;

    defparam uaddr_cmd_pads.MEM_BANK_WIDTH          = MEM_BANK_WIDTH;
    defparam uaddr_cmd_pads.MEM_ODT_WIDTH           = MEM_ODT_WIDTH;
    defparam uaddr_cmd_pads.MEM_CK_WIDTH            = MEM_CK_WIDTH;
    defparam uaddr_cmd_pads.AFI_BANK_WIDTH          = AFI_BANK_WIDTH; 
    defparam uaddr_cmd_pads.AFI_ODT_WIDTH           = AFI_ODT_WIDTH; 

        generate
        if (INVERT_HR_CLOCK == 1)
                assign invert_hr_clock = 1'b1;
        else
                assign invert_hr_clock = 1'b0;
        endgenerate

        generate
    genvar i;
    for (i=0; i<NUM_OF_DQDQS; i=i+1)
    begin: dq_ddio
        wire dqs_busout;

        //  The phy_ddio_dq bus is the write data for all DQS groups in one
        //  AFI cycle. The bus is ordered by time slot and subordered by DQS group:
        // 
        // FR: D1_T1, D0_T1, D1_T0, D0_T0
        // HR: D1_T3, D0_T3, D1_T2, D0_T2, D1_T1, D0_T1, D1_T0, D0_T0
        //
        // Extract the write data targeting the current DQS group
        wire [DQDQS_DATA_WIDTH-1:0] phy_ddio_dq_t0 = phy_ddio_dq [DQDQS_DATA_WIDTH*(i+1+0*NUM_OF_DQDQS)-1 : DQDQS_DATA_WIDTH*(i+0*NUM_OF_DQDQS)];
        wire [DQDQS_DATA_WIDTH-1:0] phy_ddio_dq_t1 = phy_ddio_dq [DQDQS_DATA_WIDTH*(i+1+1*NUM_OF_DQDQS)-1 : DQDQS_DATA_WIDTH*(i+1*NUM_OF_DQDQS)];
        wire [DQDQS_DATA_WIDTH-1:0] phy_ddio_dq_t2 = phy_ddio_dq [DQDQS_DATA_WIDTH*(i+1+2*NUM_OF_DQDQS)-1 : DQDQS_DATA_WIDTH*(i+2*NUM_OF_DQDQS)];
        wire [DQDQS_DATA_WIDTH-1:0] phy_ddio_dq_t3 = phy_ddio_dq [DQDQS_DATA_WIDTH*(i+1+3*NUM_OF_DQDQS)-1 : DQDQS_DATA_WIDTH*(i+3*NUM_OF_DQDQS)];

        // Extract the OE signal targeting the current DQS group
        wire [DQDQS_DATA_WIDTH-1:0] phy_ddio_wrdata_en_t0 = {DQDQS_DATA_WIDTH{phy_ddio_dqs_en[i]}};
        wire [DQDQS_DATA_WIDTH-1:0] phy_ddio_wrdata_en_t1 = {DQDQS_DATA_WIDTH{phy_ddio_dqs_en[i+NUM_OF_DQDQS]}};

        // Extract the write data mask signal targeting the current DQS group
        wire [DQDQS_DM_WIDTH-1:0] phy_ddio_wrdata_mask_t0;
        wire [DQDQS_DM_WIDTH-1:0] phy_ddio_wrdata_mask_t1;
        wire [DQDQS_DM_WIDTH-1:0] phy_ddio_wrdata_mask_t2;
        wire [DQDQS_DM_WIDTH-1:0] phy_ddio_wrdata_mask_t3;
        assign phy_ddio_wrdata_mask_t0 = phy_ddio_wrdata_mask [DQDQS_DM_WIDTH*(i+1+0*NUM_OF_DQDQS)-1 : DQDQS_DM_WIDTH*(i+0*NUM_OF_DQDQS)];
        assign phy_ddio_wrdata_mask_t1 = phy_ddio_wrdata_mask [DQDQS_DM_WIDTH*(i+1+1*NUM_OF_DQDQS)-1 : DQDQS_DM_WIDTH*(i+1*NUM_OF_DQDQS)];
        assign phy_ddio_wrdata_mask_t2 = phy_ddio_wrdata_mask [DQDQS_DM_WIDTH*(i+1+2*NUM_OF_DQDQS)-1 : DQDQS_DM_WIDTH*(i+2*NUM_OF_DQDQS)];
        assign phy_ddio_wrdata_mask_t3 = phy_ddio_wrdata_mask [DQDQS_DM_WIDTH*(i+1+3*NUM_OF_DQDQS)-1 : DQDQS_DM_WIDTH*(i+3*NUM_OF_DQDQS)];

        wire pll_capture_clk;
        assign  pll_capture_clk = pll_capture0_clk_ddio;

        ddr3_controller_p0_dqdqs_pads_m10 ubidir_dq_dqs (
            .reset_n_afi_clk        (reset_n_afi_clk[3]),
            .pll_afi_clk            (pll_afi_clk),
            .pll_mem_clk_ddio       (pll_mem_clk_ddio),
            .pll_mem_clk            (pll_mem_clk),
            .pll_write_clk          (pll_write_clk_ddio),
            .enable_mem_clk         (~afi_mem_clk_disable),
            .dq_capture_clk         (pll_capture_clk), // calibrated capture clock
            .read_capture_clk_hr_dq (read_capture_clk_hr_dq[i]), // to read datapath
            .invert_hr_clock        (invert_hr_clock),

            .phy_mem_dq (mem_dq[(DQDQS_DATA_WIDTH*(i+1)-1) : DQDQS_DATA_WIDTH*i]),
            .mem_dqs                (mem_dqs[i]),
            .mem_dqs_n              (mem_dqs_n[i]),

            .extra_write_data_in    ({phy_ddio_wrdata_mask_t3, phy_ddio_wrdata_mask_t2, phy_ddio_wrdata_mask_t1, phy_ddio_wrdata_mask_t0}),
            .extra_write_data_out   (mem_dm[i]),

            .rdata_hr               (ddio_phy_dq[(DQDQS_DDIO_PHY_DQ_WIDTH*(i+1)-1) : DQDQS_DDIO_PHY_DQ_WIDTH*i]),
            .write_oe_in            ({phy_ddio_wrdata_en_t1, phy_ddio_wrdata_en_t0}),
            .write_data_in          ({phy_ddio_dq_t3, phy_ddio_dq_t2, phy_ddio_dq_t1, phy_ddio_dq_t0}),
            .output_strobe_ena      ({phy_ddio_dqs_en[i+NUM_OF_DQDQS], phy_ddio_dqs_en[i]})
        );
        defparam ubidir_dq_dqs.MEM_CK_WIDTH            = MEM_CK_WIDTH;
        
    end
    endgenerate
    
//  ******************************************************************************************************************************** 
//  Instantiate phase detector
//  ******************************************************************************************************************************** 
    fiftyfivenm_phase_detector phase_detector_inst
    (
        .reset      (~pd_reset_n),
        .phasedone  (pd_ack),
        .clk        ({mimic_clock,pll_capture1_clk,pll_afi_clk}),
        .up         (pd_up),
        .down       (pd_down)
    );

// Calculate the ceiling of log_2 of the input value
function integer ceil_log2;
    input integer value;
    begin
        value = value - 1;
        for (ceil_log2 = 0; value > 0; ceil_log2 = ceil_log2 + 1)
            value = value >> 1;
    end
endfunction

endmodule
