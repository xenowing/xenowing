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


// *****************************************************************
// File name: simple_ddio_out_m10.sv
//
// This module can be used to double the data rate of the datain*
// bus. Outputs at the dataout. 
//
// Example 1:
//
// datain_rise = T1 at clk cycle x of hr_clk, where each Ty is a data item
// datain_fall = T0 at clk cycle x of hr_clk, where each Ty is a data item
// with width DATA_WIDTH
//
//
// dataout = {T0} at positive phase of clk cycle x clocked by fr_clk
// dataout = {T1} at negative phase of clk cycle x clocked by fr_clk
//
// *****************************************************************


`timescale 1 ps / 1 ps

module ddr3_controller_p0_simple_ddio_out_m10(
	hr_clk,
	fr_clk,
	datain_rise,
        datain_fall,
        muxsel,
	dataout
);

// *****************************************************************
// BEGIN PARAMETER SECTION

parameter DATA_WIDTH = ""; 

// END PARAMETER SECTION
// *****************************************************************

input	hr_clk;
input   fr_clk;
input	[DATA_WIDTH-1:0] datain_rise;
input   [DATA_WIDTH-1:0] datain_fall;
input   muxsel;
output	[DATA_WIDTH-1:0] dataout;

generate
genvar i, j;
	(* altera_attribute = {"-name ALLOW_SYNCH_CTRL_USAGE OFF"}*) reg [DATA_WIDTH-1:0] datain_r /* synthesis dont_merge syn_noprune syn_preserve = 1 */;
	(* altera_attribute = {"-name ALLOW_SYNCH_CTRL_USAGE OFF"}*) reg [DATA_WIDTH-1:0] datain_f /* synthesis dont_merge syn_noprune syn_preserve = 1 */;


        always_ff @ (posedge hr_clk )
	begin
		datain_r <= datain_rise;
	end

        always_ff @ (negedge hr_clk)
        begin
                datain_f <= datain_fall;
        end


	reg [DATA_WIDTH-1:0] dataout_r /* synthesis dont_merge syn_noprune syn_preserve = 1 */;
	for (i=0; i<DATA_WIDTH; i=i+1)
	begin: ddio_group

                always_ff @ (posedge fr_clk)
                begin
                        if (muxsel)
                        begin
			        dataout_r[i] <= datain_f;
                        end else begin
                                dataout_r[i] <= datain_r;
                        end
		end
	end
	
	assign dataout = dataout_r;
	
endgenerate
endmodule
