// (C) 2001-2019 Intel Corporation. All rights reserved.
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





`timescale 1 ps / 1 ps

module ddr3_controller_p0_dqdqs_pads_m10 (
        reset_n_afi_clk,
        pll_afi_clk,            // PLL AFI clock, 150 MHz, 0 phase        
        pll_mem_clk_ddio,       // Memory clock, 300MHz, 0 phase, PHYCLK network
        pll_mem_clk,            // Memory clock, 300MHz, 0 phase, GCLK network
        pll_write_clk,          // Write and AC clock, 300MHz, -90 phase
        enable_mem_clk,
        dq_capture_clk,        // Calibrated Read capture clock, 200MHz
        read_capture_clk_hr_dq, // Div2 HR dropback clock 
        invert_hr_clock,

        mem_dqs_n,              // memory DQS#

        rdata_hr,               // FR read data out 
        write_oe_in,            // write output enable
        write_data_in,          // write input data
        output_strobe_ena,      // DQS output enable

        extra_write_data_in,    
        extra_write_data_out,

        phy_mem_dq,             // memory DQ
        mem_dqs                // memory DQS
);

parameter       PIN_WIDTH = 8;
parameter       EXTRA_OUTPUT_WIDTH = 1;
parameter       REVERSE_READ_WORDS = "false";
parameter       MEM_CK_WIDTH = "";

localparam      rate_mult_out =  4 ;
localparam      WRITE_OE_WIDTH = PIN_WIDTH * rate_mult_out / 2;
localparam      WRITE_OUPUT_FR_WIDTH = PIN_WIDTH * rate_mult_out / 2;
localparam      fpga_width_out = PIN_WIDTH * rate_mult_out;
localparam      extra_fpga_width_out = EXTRA_OUTPUT_WIDTH * rate_mult_out;
localparam      EXTRA_OUTPUT_FR_WIDTH = EXTRA_OUTPUT_WIDTH * rate_mult_out/2;

localparam      rate_mult_in = 2;
localparam      READ_PHY_DQ_WIDTH = PIN_WIDTH * rate_mult_in;

input                           reset_n_afi_clk;
input                           pll_afi_clk;
input                           pll_mem_clk_ddio;
input                           pll_mem_clk;
input                           pll_write_clk;
input   [MEM_CK_WIDTH-1:0]      enable_mem_clk;
input                           dq_capture_clk;
output                          read_capture_clk_hr_dq;
input                           invert_hr_clock;

inout [PIN_WIDTH-1:0]           phy_mem_dq;
inout                           mem_dqs;
inout                           mem_dqs_n;
output logic [fpga_width_out-1:0]     rdata_hr;

input [WRITE_OE_WIDTH-1:0]      write_oe_in;
input [fpga_width_out-1:0]      write_data_in;
input [1:0]                     output_strobe_ena;

input [extra_fpga_width_out-1:0] extra_write_data_in;
output [EXTRA_OUTPUT_WIDTH-1:0]  extra_write_data_out;

wire [EXTRA_OUTPUT_WIDTH-1:0]   extra_wr_fr_data_hi;
wire [EXTRA_OUTPUT_WIDTH-1:0]   extra_wr_fr_data_lo;

wire [PIN_WIDTH-1:0]            rd_fr_data_hi;
wire [PIN_WIDTH-1:0]            rd_fr_data_lo;
wire [PIN_WIDTH-1:0]            wr_fr_data_hi;
wire [PIN_WIDTH-1:0]            wr_fr_data_lo;
wire [PIN_WIDTH-1:0]            fr_oe;

wire                            fr_os_hi;
wire                            fr_os_lo;
wire                            fr_os_oe;
reg                             fr_os_oe_r;

logic [READ_PHY_DQ_WIDTH-1:0]   ddio_phy_dq;
logic [READ_PHY_DQ_WIDTH-1:0]   ddio_phy_dq_r;
logic [READ_PHY_DQ_WIDTH-1:0]   ddio_phy_dq_r1;
logic [PIN_WIDTH-1:0]           read_capture_clk_fr_dq;

reg wrdata_valid_r1;
reg wrdata_valid_r2;
reg wrdata_muxsel;

reg wr_dqs_ena_r1;
reg wr_dqs_ena_r2;
reg dqs_ena_muxsel;

//############################################
// Generate the muxsel signal for DQ and DQS
// these muxsel signal generated in memclk domain
//############################################

always_ff @ (posedge pll_mem_clk)
begin
        wrdata_valid_r1 <= |write_oe_in;
        wrdata_valid_r2 <= wrdata_valid_r1;
        wr_dqs_ena_r1 <= |output_strobe_ena;
        wr_dqs_ena_r2 <= wr_dqs_ena_r1;

end

generate
begin: gen_wdata_sel
    always_ff @ (posedge pll_mem_clk)
        begin
            if (wrdata_valid_r1 & ~wrdata_valid_r2)
                wrdata_muxsel <= 1'b1;
            else
                wrdata_muxsel <= ~wrdata_muxsel;
        end
end
endgenerate

generate
begin: gen_dqs_ena_sel
    always_ff @ (posedge pll_mem_clk)
        begin
            if (wr_dqs_ena_r1 & ~wr_dqs_ena_r2)
                dqs_ena_muxsel <= 1'b1;
            else
                dqs_ena_muxsel <= ~dqs_ena_muxsel;
        end
end
endgenerate

generate
    genvar opin_num;
    for (opin_num = 0; opin_num < PIN_WIDTH; opin_num = opin_num + 1)
    begin :output_path_gen
                wire hr_data_t0 = write_data_in [opin_num + 0*PIN_WIDTH];
                wire hr_data_t1 = write_data_in [opin_num + 1*PIN_WIDTH];
                wire hr_data_t2 = write_data_in [opin_num + 2*PIN_WIDTH];
                wire hr_data_t3 = write_data_in [opin_num + 3*PIN_WIDTH];
        
                ddr3_controller_p0_simple_ddio_out_m10    # (
                    .DATA_WIDTH    (1)
                ) hr_to_fr_lo (
                    .hr_clk        (pll_afi_clk),
                    .fr_clk        (pll_mem_clk),
                    .datain_rise   (hr_data_t2),
                    .datain_fall   (hr_data_t0),
                    .muxsel        (wrdata_muxsel),
                    .dataout       (wr_fr_data_lo[opin_num])
                );
        
                ddr3_controller_p0_simple_ddio_out_m10    # (
                    .DATA_WIDTH    (1)
                ) hr_to_fr_hi (
                    .hr_clk        (pll_afi_clk),
                    .fr_clk        (pll_mem_clk),
                    .datain_rise   (hr_data_t3),
                    .datain_fall   (hr_data_t1),
                    .muxsel        (wrdata_muxsel),
                    .dataout       (wr_fr_data_hi[opin_num])
                );
        
                ddr3_controller_p0_simple_ddio_out_m10    # (
                    .DATA_WIDTH    (1)
                ) hr_to_fr_oe (
                    .hr_clk        (pll_afi_clk),
                    .fr_clk        (pll_mem_clk),
                    .datain_rise   (write_oe_in [opin_num + PIN_WIDTH]),
                    .datain_fall   (write_oe_in [opin_num + 0]),
                    .muxsel        (wrdata_muxsel),
                    .dataout       (fr_oe[opin_num])
                );

                // FR capture
                always_ff @ (posedge read_capture_clk_fr_dq[opin_num])
                    begin
                        ddio_phy_dq_r[opin_num]                 <= ddio_phy_dq[opin_num];
                        ddio_phy_dq_r[opin_num + PIN_WIDTH]     <= ddio_phy_dq[opin_num + PIN_WIDTH];
                        ddio_phy_dq_r1[opin_num]                 <= ddio_phy_dq_r[opin_num];
                        ddio_phy_dq_r1[opin_num + PIN_WIDTH]     <= ddio_phy_dq_r[opin_num + PIN_WIDTH];
                    end
                
        end
endgenerate

                altera_gpio_lite #(
                    .PIN_TYPE                  ("bidir"),
                    .SIZE                      (PIN_WIDTH),
                    .REGISTER_MODE             ("ddr"),
                    .BUFFER_TYPE               ("single-ended"),
                    .ASYNC_MODE                ("none"),
                    .SYNC_MODE                 ("none"),
                    .BUS_HOLD                  ("false"),
                    .OPEN_DRAIN_OUTPUT         ("false"),
                    .SET_REGISTER_OUTPUTS_HIGH ("false"),
                    .INVERT_OUTPUT             ("false"),
                    .INVERT_INPUT_CLOCK        ("true"),
                    .INVERT_OUTPUT_CLOCK       ("false"),
                    .INVERT_OE_INCLOCK         ("false"),
                    .USE_ONE_REG_TO_DRIVE_OE   ("false"),
                    .USE_DDIO_REG_TO_DRIVE_OE  ("true"),
                    .USE_ADVANCED_DDR_FEATURES ("true"),
                    .INVERT_CLKDIV_INPUT_CLOCK ("false"),
                    .ENABLE_HR_CLOCK           ("true"),
                    .ENABLE_OE_HALF_CYCLE_DELAY("false"),
                    .ENABLE_PHASE_INVERT_CTRL_PORT ("true"),
                    .ENABLE_NSLEEP_PORT        ("true")
                ) dq_ddio_io (
                    .sclr                      (1'b0),
                    .inclock                   (dq_capture_clk),
                    .fr_clock                  (read_capture_clk_fr_dq),
                    .hr_clock                  (read_capture_clk_hr_dq),
                    .invert_hr_clock           (invert_hr_clock), 
                    .outclock                  (pll_write_clk),
                    .phy_mem_clock             (pll_mem_clk_ddio),
                    .dout                      ({rd_fr_data_hi, rd_fr_data_lo}),
                    .din                       ({wr_fr_data_hi, wr_fr_data_lo}),
                    .pad_io                    (phy_mem_dq),
                    .oe                        (fr_oe),
                    .inclocken                 (1'b1),            
                    .outclocken                (1'b1),            
                    .pad_io_b                  (),                
                    .pad_in                    (8'b0000),         
                    .pad_in_b                  (8'b0000),         
                    .pad_out                   (),                
                    .pad_out_b                 (),                
                    .aset                      (1'b0),            
                    .aclr                      (1'b0),            
                    .nsleep                    ({8{|enable_mem_clk}})
                    );

generate
if (REVERSE_READ_WORDS == "true")
begin
    assign ddio_phy_dq [PIN_WIDTH -1 : 0] = rd_fr_data_hi;
    assign ddio_phy_dq [READ_PHY_DQ_WIDTH-1 :PIN_WIDTH] = rd_fr_data_lo;
end
else
begin
    assign ddio_phy_dq [PIN_WIDTH -1 : 0] = rd_fr_data_lo;
    assign ddio_phy_dq [READ_PHY_DQ_WIDTH-1 :PIN_WIDTH] = rd_fr_data_hi;
end
endgenerate

// FR to HR conversion

always_ff @ (posedge read_capture_clk_hr_dq) rdata_hr <= {ddio_phy_dq_r,ddio_phy_dq_r1};


always_ff @ (posedge pll_mem_clk) fr_os_oe_r <= fr_os_oe;

//######################################
// Instantiate DQS group
//######################################
generate
begin: dqs_en 
        ddr3_controller_p0_simple_ddio_out_m10    # (
            .DATA_WIDTH    (1)
        ) hr_to_fr_os_oe (
            .hr_clk        (pll_afi_clk),
            .fr_clk        (pll_mem_clk),
            .datain_rise   (output_strobe_ena[1]),
            .datain_fall   (output_strobe_ena[0]),
            .muxsel        (dqs_ena_muxsel),
            .dataout       (fr_os_oe)
        );
end
endgenerate

altera_gpio_lite #(
    .PIN_TYPE                  ("bidir"),
    .SIZE                      (1),
    .REGISTER_MODE             ("ddr"),
    .BUFFER_TYPE               ("pseudo_differential"),
    .ASYNC_MODE                ("none"),
    .SYNC_MODE                 ("none"),
    .BUS_HOLD                  ("false"),
    .OPEN_DRAIN_OUTPUT         ("false"),
    .SET_REGISTER_OUTPUTS_HIGH ("false"),
    .INVERT_OUTPUT             ("false"),
    .INVERT_INPUT_CLOCK        ("false"),
    .INVERT_OUTPUT_CLOCK       ("false"),
    .INVERT_OE_INCLOCK         ("false"),
    .USE_ONE_REG_TO_DRIVE_OE   ("false"),
    .USE_DDIO_REG_TO_DRIVE_OE  ("true"),
    .USE_ADVANCED_DDR_FEATURES ("false"),
    .INVERT_CLKDIV_INPUT_CLOCK ("false"),
    .ENABLE_HR_CLOCK           ("false"),
    .ENABLE_OE_HALF_CYCLE_DELAY("false"),
    .ENABLE_PHASE_INVERT_CTRL_PORT ("false"),
    .ENABLE_OE_PORT            ("true"),
    .ENABLE_NSLEEP_PORT        ("true")
) dqs_ddio_io (
    .inclock                   (1'b0),
    .fr_clock                  (),
    .hr_clock                  (),
    .invert_hr_clock           (1'b0),
    .dout                      (),
    .pad_io                   (mem_dqs),         
    .pad_io_b                 (mem_dqs_n),       
    .pad_out                    (),
    .pad_out_b                  (),
    .sclr                      (1'b0),
    .outclock                  (pll_mem_clk_ddio),
    .phy_mem_clock             (pll_mem_clk_ddio),
    .din                       (2'b01),

    .oe                        (fr_os_oe_r),
    .inclocken                 (1'b1),            
    .outclocken                (1'b1),            
    .pad_in                    (1'b1),            
    .pad_in_b                  (1'b1),            
    .aset                      (1'b0),            
    .aclr                      (1'b0),            
    .nsleep                    (|enable_mem_clk)
    );


//######################################
// Instantiate extra output pins
//######################################
generate
    genvar epin_num;
    for (epin_num = 0; epin_num < EXTRA_OUTPUT_WIDTH; epin_num = epin_num + 1)
    begin :extra_output_pad_gen
            wire hr_data_t0 = extra_write_data_in [epin_num + 0*EXTRA_OUTPUT_WIDTH];
            wire hr_data_t1 = extra_write_data_in [epin_num + 1*EXTRA_OUTPUT_WIDTH];
            wire hr_data_t2 = extra_write_data_in [epin_num + 2*EXTRA_OUTPUT_WIDTH];
            wire hr_data_t3 = extra_write_data_in [epin_num + 3*EXTRA_OUTPUT_WIDTH];

                ddr3_controller_p0_simple_ddio_out_m10    # (
                    .DATA_WIDTH    (1)
                ) hr_to_fr_lo (
                    .hr_clk        (pll_afi_clk),
                    .fr_clk        (pll_mem_clk),
                    .datain_rise   (hr_data_t2),
                    .datain_fall   (hr_data_t0),
                    .muxsel        (wrdata_muxsel),
                    .dataout       (extra_wr_fr_data_lo[epin_num])
                );

                ddr3_controller_p0_simple_ddio_out_m10    # (
                    .DATA_WIDTH    (1)
                ) hr_to_fr_hi (
                    .hr_clk        (pll_afi_clk),
                    .fr_clk        (pll_mem_clk),
                    .datain_rise   (hr_data_t3),
                    .datain_fall   (hr_data_t1),
                    .muxsel        (wrdata_muxsel),
                    .dataout       (extra_wr_fr_data_hi[epin_num])
                );

        end
endgenerate
                altera_gpio_lite #(
                    .PIN_TYPE                  ("output"),
                    .SIZE                      (1),
                    .REGISTER_MODE             ("ddr"),
                    .BUFFER_TYPE               ("single-ended"),
                    .ASYNC_MODE                ("none"),
                    .SYNC_MODE                 ("none"),
                    .BUS_HOLD                  ("false"),
                    .OPEN_DRAIN_OUTPUT         ("false"),
                    .SET_REGISTER_OUTPUTS_HIGH ("false"),
                    .INVERT_OUTPUT             ("false"),
                    .INVERT_INPUT_CLOCK        ("false"),
                    .INVERT_OUTPUT_CLOCK       ("false"),
                    .INVERT_OE_INCLOCK         ("false"),
                    .USE_ONE_REG_TO_DRIVE_OE   ("false"),
                    .USE_DDIO_REG_TO_DRIVE_OE  ("false"),
                    .USE_ADVANCED_DDR_FEATURES ("true"),
                    .INVERT_CLKDIV_INPUT_CLOCK ("false"),
                    .ENABLE_HR_CLOCK           ("false"),
                    .ENABLE_OE_HALF_CYCLE_DELAY("false"),
                    .ENABLE_PHASE_INVERT_CTRL_PORT ("false"),
                    .ENABLE_OE_PORT            ("true")
                ) extra_ddio_out (
                    .sclr                      (1'b0),
                    .outclock                  (pll_write_clk),                  
                    .phy_mem_clock             (pll_mem_clk_ddio),             
                    .din                       ({extra_wr_fr_data_hi, extra_wr_fr_data_lo}),                       
                    .pad_out                   (extra_write_data_out),                       
                    .oe                        (|enable_mem_clk),                   
                    .outclocken                (1'b1),          
                    .inclock                   (1'b0),          
                    .inclocken                 (1'b0),          
                    .fr_clock                  (),              
                    .hr_clock                  (),              
                    .invert_hr_clock           (1'b0),          
                    .dout                      (),              
                    .pad_io                    (),              
                    .pad_io_b                  (),              
                    .pad_in                    (1'b1),          
                    .pad_in_b                  (1'b1),          
                    .pad_out_b                 (),              
                    .aset                      (1'b0),          
                    .aclr                      (1'b0)           
                    );

endmodule
